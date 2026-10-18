//! The demo played in the demo view (implementation plan §13.4 and §13.8)
//!
//! Each viewer keeps a playback controller and worker for its demo, and a
//! thread that reads the session's map and the studio models its entities
//! use through the game's search path. Every frame the worker accepted is
//! drawn from the recorder's view. Entities with a studio model are posed
//! from their recorded sequence, frame, frame rate and animation time, and
//! a player is drawn with the model its userinfo names. The entity the
//! recorder views from is left out, as the game leaves it out.
//!
//! Brush entities, sprites, the viewmodel, interpolation between snapshots
//! and the client's player animation come with the rest of M9. Shown only
//! when a game folder is given with `--game`, until the viewer's settings
//! come (M11)

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use eframe::egui::{self, Rect, Sense, Ui};
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use eframe::wgpu;
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_playback::PresentedFrame;
use lambda_slicer_playback::controller::{Command, Controller, Reply};
use lambda_slicer_playback::time::SourceTime;
use lambda_slicer_playback::worker::Worker;
use lambda_slicer_render::Rgba;
use lambda_slicer_render::camera::{Camera, Matrix};
use lambda_slicer_render::gpu::{Blit, Brush, Instance, ModelHandle, RenderMode, Scene, View};
use lambda_slicer_render::load::{self, EntityPose, LoadError, LoadedModel};
use lambda_slicer_render::world::{self, WorldMesh};

use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};

/// `EF_NODRAW`, an entity the game does not draw
const NO_DRAW: i64 = 128;

/// The field of view when a recording names none
const DEFAULT_FOV: f32 = 90.;

/// `OBS_IN_EYE`, a spectator seeing through the eyes of the player it
/// observes
const OBS_IN_EYE: i64 = 4;

/// Time frames for this many seconds once playback runs, write what was
/// measured to `out` and close the window
pub struct Bench {
    pub seconds: f64,
    pub out: PathBuf,
}

/// Which model an entity is drawn with
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ModelRef {
    /// A model by path, such as `models/w_gauss.mdl`
    File(String),
    /// A player model by the name userinfo gives, the stock player model
    /// when it is not installed
    Player(String),
}

enum Asset {
    /// A map by name, without `maps/` or `.bsp`
    Map(String),
    Model(ModelRef),
}

enum Outcome {
    Ready(Box<LoadedModel>),
    /// A sprite under a model's name, drawn with the sprites (9d)
    Sprite,
    Failed(String),
}

/// A map's world and its sky, none when the sky could not be read
struct LoadedMap {
    world: WorldMesh,
    sky: Option<[Rgba; 6]>,
}

enum Loaded {
    Map(String, Result<Box<LoadedMap>, String>),
    Model(ModelRef, Outcome),
}

/// The thread that reads maps and models for one demo
struct Loader {
    requests: Sender<Asset>,
    loaded: Receiver<Loaded>,
}

impl Loader {
    fn start(install: PathBuf, game: String, ctx: egui::Context) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<Asset>();
        let (loaded_tx, loaded_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("viewer assets".into())
            .spawn(move || {
                let limits = AssetLimits::default();
                let search = SearchPath::new(&install, &game, &SearchOptions::default(), limits)
                    .map_err(|e| e.to_string());
                for request in request_rx {
                    let loaded = match (&search, request) {
                        (Ok(search), Asset::Map(name)) => {
                            let world = load::map(search, &name, limits).map(|(world, _)| {
                                let sky = load::sky(search, &world.sky, limits)
                                    .inspect_err(|e| eprintln!("viewer: sky {}, {e}", world.sky))
                                    .ok();
                                Box::new(LoadedMap { world, sky })
                            });
                            Loaded::Map(name, world.map_err(|e| e.to_string()))
                        }
                        (Ok(search), Asset::Model(model)) => {
                            let loaded = match &model {
                                ModelRef::File(path) => load::model(search, path),
                                ModelRef::Player(name) => load::player(search, name),
                            };
                            let outcome = match loaded {
                                Ok(m) => Outcome::Ready(Box::new(m)),
                                Err(LoadError::Sprite { .. }) => Outcome::Sprite,
                                Err(e) => Outcome::Failed(e.to_string()),
                            };
                            Loaded::Model(model, outcome)
                        }
                        (Err(e), Asset::Map(name)) => Loaded::Map(name, Err(e.clone())),
                        (Err(e), Asset::Model(model)) => {
                            Loaded::Model(model, Outcome::Failed(e.clone()))
                        }
                    };
                    if loaded_tx.send(loaded).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
            })
            .expect("spawn viewer assets");

        Self {
            requests: request_tx,
            loaded: loaded_rx,
        }
    }
}

enum Slot {
    Loading,
    Ready(Arc<LoadedModel>, ModelHandle),
    /// A sprite, or a model that could not be read
    NotDrawn,
}

/// What the window's renderer keeps for the viewer between frames
struct Resources {
    scene: Scene,
    blit: Blit,
    blit_group: Option<wgpu::BindGroup>,
}

pub struct Viewer {
    render_state: RenderState,
    controller: Controller,
    worker: Worker,
    loader: Option<Loader>,
    frame: Option<Box<PresentedFrame>>,
    /// The map whose world is drawn, and the one being read
    map: Option<String>,
    map_loading: Option<String>,
    models: HashMap<ModelRef, Slot>,
    clock: Instant,
    /// The demo's length, when the library knows it
    duration: f64,
    bench: Option<(Bench, Option<Instant>, Vec<f64>)>,
    last_frame: Option<Instant>,
}

/// The demo's game folder, from its header
fn game_dir(demo: &std::path::Path) -> Option<String> {
    let mut file = std::fs::File::open(demo).ok()?;
    let len = file.metadata().ok()?.len();
    Some(read_header(&mut file, len).ok()?.game_dir())
}

/// A map name from a server info's `maps/<name>.bsp`
fn map_name(map: &str) -> String {
    map.strip_prefix("maps/")
        .unwrap_or(map)
        .trim_end_matches(".bsp")
        .to_owned()
}

impl Viewer {
    pub fn open(
        render_state: RenderState,
        install: PathBuf,
        demo: PathBuf,
        duration: f32,
        start: Option<f64>,
        bench: Option<Bench>,
        ctx: &egui::Context,
    ) -> Self {
        {
            let device = &render_state.device;
            let resources = Resources {
                scene: Scene::new(device),
                blit: Blit::new(device, render_state.target_format),
                blit_group: None,
            };
            render_state
                .renderer
                .write()
                .callback_resources
                .insert(resources);
        }

        let repaint = ctx.clone();
        let worker = Worker::spawn(Limits::default(), move || repaint.request_repaint());
        let loader = game_dir(&demo).map(|game| Loader::start(install, game, ctx.clone()));
        let mut controller = Controller::default();
        if let Some(request) = controller.command(Command::Load(demo), 0.) {
            worker.send(request);
        }
        if let Some(request) =
            start.and_then(|t| controller.command(Command::SeekTo(SourceTime(t)), 0.))
        {
            worker.send(request);
        }

        Self {
            render_state,
            controller,
            worker,
            loader,
            frame: None,
            map: None,
            map_loading: None,
            models: HashMap::new(),
            clock: Instant::now(),
            duration: f64::from(duration),
            bench: bench.map(|b| (b, None, Vec::new())),
            last_frame: None,
        }
    }

    fn wall(&self) -> f64 {
        self.clock.elapsed().as_secs_f64()
    }

    fn command(&mut self, command: Command) {
        let wall = self.wall();
        if let Some(request) = self.controller.command(command, wall) {
            self.worker.send(request);
        }
    }

    /// Take the worker's answers and the loaded assets, and ask for the
    /// next frame while playing
    fn poll(&mut self) {
        let wall = self.wall();
        for reply in self.worker.replies() {
            if self.controller.accept(&reply, wall)
                && let Reply::Frame(frame) = reply
            {
                self.frame = Some(frame);
            }
        }
        if let Some(request) = self.controller.tick(wall) {
            self.worker.send(request);
        }

        let Some(loader) = &self.loader else {
            return;
        };
        let loaded: Vec<Loaded> = loader.loaded.try_iter().collect();
        let RenderState {
            device,
            queue,
            renderer,
            ..
        } = &self.render_state;
        let mut renderer = renderer.write();
        let Some(resources) = renderer.callback_resources.get_mut::<Resources>() else {
            return;
        };
        for item in loaded {
            match item {
                Loaded::Map(name, Ok(map)) if self.map_loading.as_ref() == Some(&name) => {
                    let LoadedMap { world, sky } = *map;
                    resources.scene.set_world(device, queue, &world);
                    resources.scene.set_sky(device, queue, sky.as_ref());
                    self.map = Some(name);
                    self.map_loading = None;
                }
                Loaded::Map(name, Err(e)) => {
                    eprintln!("viewer: map {name}, {e}");
                    self.map = Some(name);
                    self.map_loading = None;
                }
                Loaded::Map(..) => {}
                Loaded::Model(model, Outcome::Ready(loaded)) => {
                    let handle = resources.scene.add_model(device, queue, &loaded.mesh);
                    self.models
                        .insert(model, Slot::Ready(Arc::from(loaded), handle));
                }
                Loaded::Model(model, Outcome::Sprite) => {
                    self.models.insert(model, Slot::NotDrawn);
                }
                Loaded::Model(model, Outcome::Failed(e)) => {
                    eprintln!("viewer: model {model:?}, {e}");
                    self.models.insert(model, Slot::NotDrawn);
                }
            }
        }
    }

    /// Ask for the frame's map and models that are not loaded yet
    fn request_assets(&mut self) {
        let (Some(frame), Some(loader)) = (&self.frame, &self.loader) else {
            return;
        };
        if let Some(map) = frame.map.as_deref().map(map_name)
            && self.map.as_ref() != Some(&map)
            && self.map_loading.as_ref() != Some(&map)
        {
            let _ = loader.requests.send(Asset::Map(map.clone()));
            self.map_loading = Some(map);
        }

        for (model, _) in drawn(frame) {
            if let Entry::Vacant(slot) = self.models.entry(model) {
                let _ = loader.requests.send(Asset::Model(slot.key().clone()));
                slot.insert(Slot::Loading);
            }
        }
    }

    /// Every entity drawn this frame, posed and skinned
    fn instances(&self) -> Vec<Instance> {
        let Some(frame) = &self.frame else {
            return Vec::new();
        };
        let mut instances = Vec::new();
        for (model, pose) in drawn(frame) {
            if let Some(Slot::Ready(model, handle)) = self.models.get(&model)
                && let Some(bones) = model.pose(&pose)
            {
                instances.push(Instance {
                    model: *handle,
                    vertices: model.mesh.skin(&bones),
                });
            }
        }
        instances
    }

    /// Bytes the scene holds on the GPU
    fn gpu_bytes(&self) -> u64 {
        self.render_state
            .renderer
            .read()
            .callback_resources
            .get::<Resources>()
            .map_or(0, |r| r.scene.gpu_bytes())
    }

    /// Frame times for a bench run, and the report once it is over. True
    /// when the window should close
    fn time_frame(&mut self, pixels: [u32; 2], drawn: usize) -> bool {
        let now = Instant::now();
        let dt = self.last_frame.map(|l| (now - l).as_secs_f64() * 1000.);
        self.last_frame = Some(now);
        if self.bench.is_none() {
            return false;
        }
        if !self.controller.playing() {
            self.command(Command::Play);
        }
        let gpu_bytes = self.gpu_bytes();
        let adapter = self.render_state.adapter.get_info();
        let Some((bench, begin, times)) = &mut self.bench else {
            return false;
        };

        // A second of playback to settle before timing
        if self.frame.as_ref().is_none_or(|f| f.at.0 < 1.) {
            return false;
        }
        let begin = *begin.get_or_insert(now);
        if let Some(dt) = dt {
            times.push(dt);
        }
        if (now - begin).as_secs_f64() < bench.seconds || times.is_empty() {
            return false;
        }

        let mut sorted = times.clone();
        sorted.sort_by(f64::total_cmp);
        let at = |p: f64| sorted[((sorted.len() - 1) as f64 * p).round() as usize];
        let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let report = format!(
            "{} ({:?}), view {}x{} pixels, {drawn} models drawn, {} frames, mean {mean:.3} ms ({:.0} fps), p50 {:.3}, p95 {:.3}, p99 {:.3}, max {:.3} ms, scene {:.1} MiB on the GPU\n",
            adapter.name,
            adapter.backend,
            pixels[0],
            pixels[1],
            sorted.len(),
            1000. / mean,
            at(0.5),
            at(0.95),
            at(0.99),
            sorted[sorted.len() - 1],
            gpu_bytes as f64 / 1048576.,
        );
        if let Err(e) = std::fs::write(&bench.out, report) {
            eprintln!("viewer: {e}");
        }
        true
    }

    /// The view in `height` points with its controls under it, or all the
    /// space left in a bench run
    pub fn ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, height: f32) {
        self.poll();
        self.request_assets();
        let ctx = ui.ctx().clone();
        if let Some(problem) = self.controller.failed() {
            text::label(ui, &catalog.message(problem), rtl);
            return;
        }

        let controls_height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
        let height = match self.bench {
            Some(_) => ui.available_height() - controls_height,
            None => height,
        };
        let size = egui::vec2(ui.available_width(), height.max(1.));
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let mut drawn = 0;
        match (&self.frame, &self.map) {
            (Some(frame), Some(_)) => {
                let pixels = [
                    (rect.width() * ui.pixels_per_point()).round().max(1.) as u32,
                    (rect.height() * ui.pixels_per_point()).round().max(1.) as u32,
                ];
                let instances = self.instances();
                drawn = instances.len();
                let callback = Frame {
                    pixels,
                    view: view(frame, rect),
                    brushes: brushes(frame),
                    instances,
                };
                ui.painter()
                    .add(egui_wgpu::Callback::new_paint_callback(rect, callback));
                if self.time_frame(pixels, drawn) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            _ => {
                ui.put(rect, egui::Spinner::new());
            }
        }

        self.controls(ui, catalog, rtl);
        if self.controller.playing() || drawn > 0 && self.bench.is_some() {
            ctx.request_repaint();
        }
    }

    fn controls(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let wall = self.wall();
        let now = self.controller.time(wall).0;
        let end = self
            .frame
            .as_ref()
            .map_or(0., |f| f.indexed.0.0)
            .max(self.duration)
            .max(now);
        let clock = |t: f64| {
            DemoTime::from_f32(t as f32)
                .map(|t| t.to_string())
                .unwrap_or_default()
        };

        layout::row(ui, rtl, |ui| {
            let (id, command) = if self.controller.playing() {
                ("viewer-pause", Command::Pause)
            } else {
                ("viewer-play", Command::Play)
            };
            let label = text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
            if ui.button(label).clicked() {
                self.command(command);
            }

            let shown = format!("{} / {}", clock(now), clock(end));
            ui.label(text::widget_text(&ctx, &shown, false, None, Look::Plain));

            let mut at = now;
            ui.spacing_mut().slider_width = ui.available_width().max(40.);
            let slider = egui::Slider::new(&mut at, 0. ..=end.max(0.001)).show_value(false);
            if ui.add(slider).changed() {
                self.command(Command::SeekTo(SourceTime(at)));
            }
        });
    }
}

/// Which model each drawn entity uses and how it is posed. Players use the
/// model their userinfo names, upright as the game draws them. The entity
/// the recorder views from is left out, and so is the player a spectator
/// sees through the eyes of
fn drawn(frame: &PresentedFrame) -> Vec<(ModelRef, EntityPose)> {
    let viewer = frame.recorder.view_entity.unwrap_or(frame.local_entity);
    let observed =
        (frame.recorder.observer_mode == OBS_IN_EYE).then_some(frame.recorder.observer_target);
    let client_time = client_time(frame);

    let mut out = Vec::new();
    for e in frame.entities.iter() {
        let number = e.id.number;
        if number == viewer || observed == Some(i64::from(number)) || e.effects & NO_DRAW != 0 {
            continue;
        }

        let player = frame.players.iter().find(|p| p.entity == number);
        let model = match (player.and_then(|p| p.model.as_deref()), e.model.as_deref()) {
            (Some(name), _) => ModelRef::Player(name.to_owned()),
            (None, Some(path)) if path.ends_with(".mdl") => ModelRef::File(path.to_owned()),
            _ => continue,
        };
        let mut angles = e.angles;
        if player.is_some() {
            angles[0] = 0.;
        }
        out.push((
            model,
            EntityPose {
                sequence: e.sequence.max(0) as usize,
                cycle: f64::from(e.frame),
                elapsed: client_time - f64::from(e.animtime),
                framerate: f64::from(e.framerate),
                controllers: e.controllers.map(f64::from),
                blending: e.blending.map(f64::from),
                origin: e.origin,
                angles,
            },
        ));
    }
    out
}

/// The client's time for a frame, the last `svc_time` plus the source time
/// since that packet
fn client_time(frame: &PresentedFrame) -> f64 {
    let since_snapshot = frame.position.map_or(0., |p| frame.at.0 - p.time.0);
    f64::from(frame.server_time.0) + since_snapshot
}

/// The world, then every brush entity (`*N`) as its render mode draws it.
/// A translucent one at amount 0 is not drawn, as the game skips it
fn brushes(frame: &PresentedFrame) -> Vec<Brush> {
    let mut out = vec![Brush::world()];
    for e in frame.entities.iter() {
        let Some(model) = e
            .model
            .as_deref()
            .and_then(|m| m.strip_prefix('*'))
            .and_then(|n| n.parse::<usize>().ok())
        else {
            continue;
        };
        let mode = RenderMode::from_game(e.render_mode);
        let amount = match mode {
            RenderMode::Normal => 1.,
            _ => e.render_amount.clamp(0, 255) as f32 / 255.,
        };
        if e.effects & NO_DRAW != 0 || (mode != RenderMode::Normal && amount == 0.) {
            continue;
        }
        out.push(Brush {
            model,
            origin: e.origin,
            angles: e.angles,
            mode,
            amount,
            color: e.render_color.map(|c| f32::from(c) / 255.),
            alternate: e.frame != 0.,
        });
    }
    out
}

/// What the frame is seen from and when, with every light style's
/// brightness at the client's time
fn view(frame: &PresentedFrame, rect: Rect) -> View {
    let time = client_time(frame);
    let mut light_styles = [1.; 64];
    for (index, pattern) in &frame.light_styles {
        if let Some(slot) = light_styles.get_mut(usize::from(*index)) {
            *slot = world::style_brightness(pattern, time);
        }
    }
    View {
        view_projection: view_projection(frame, rect),
        eye: frame.recorder.origin,
        time: time as f32,
        light_styles,
    }
}

/// The recorder's view. The recorded field of view is across the screen's
/// width, as the game's is
fn view_projection(frame: &PresentedFrame, rect: Rect) -> Matrix {
    let aspect = rect.aspect_ratio().max(0.01);
    let fov_x = frame
        .recorder
        .fov
        .filter(|f| (1. ..179.).contains(f))
        .unwrap_or(DEFAULT_FOV);
    let fov_y = 2. * ((fov_x.to_radians() / 2.).tan() / aspect).atan();
    let camera = Camera {
        origin: frame.recorder.origin,
        angles: frame.recorder.angles,
        fov_y: fov_y.to_degrees(),
    };
    camera.view_projection(aspect, 4., 8192.)
}

/// One frame of the scene, drawn offscreen in `prepare` and shown in `paint`
struct Frame {
    pixels: [u32; 2],
    view: View,
    brushes: Vec<Brush>,
    instances: Vec<Instance>,
}

impl CallbackTrait for Frame {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(r) = resources.get_mut::<Resources>() else {
            return Vec::new();
        };
        let (resized, commands) = r.scene.render(
            device,
            queue,
            self.pixels,
            &self.view,
            &self.brushes,
            &self.instances,
        );
        if (resized || r.blit_group.is_none())
            && let Some(view) = r.scene.color_view()
        {
            r.blit_group = Some(r.blit.bind_group(device, view));
        }
        vec![commands]
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(r) = resources.get::<Resources>()
            && let Some(group) = &r.blit_group
        {
            r.blit.paint(pass, group);
        }
    }
}
