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
//! Sound plays on the default output device while playing forward, mixed
//! from a playback of its own that shares the worker's checkpoints and
//! heard from the recorder's view. Shown only
//! when a game folder is given with `--game`, until the viewer's settings
//! come (M11)

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use eframe::egui::{self, Sense, Ui};
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use eframe::wgpu;
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_assets::sprite::Sprite;
use lambda_slicer_assets::wad::Wad;
use lambda_slicer_assets::xenoxide::studio_pose::BoneTransform;
use lambda_slicer_audio::device::{Device, Feed};
use lambda_slicer_audio::library::Library;
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_playback::PresentedFrame;
use lambda_slicer_playback::animation::{Flash, Holder};
use lambda_slicer_playback::controller::{Command, Controller, Reply, Stamp};
use lambda_slicer_playback::scene::EntityId;
use lambda_slicer_playback::tempents::Tint;
use lambda_slicer_playback::time::SourceTime;
use lambda_slicer_playback::worker::Worker;
use lambda_slicer_render::Rgba;
use lambda_slicer_render::camera::{Camera, Matrix};
use lambda_slicer_render::gpu::{
    Blit, Brush, DrawList, Instance, ModelHandle, RenderMode, Scene, SpriteHandle, SpriteInstance,
    View,
};
use lambda_slicer_render::light::{LightTree, ModelLight};
use lambda_slicer_render::load::{self, EntityPose, GaitPose, LoadError, LoadedModel};
use lambda_slicer_render::model::ModelMesh;
use lambda_slicer_render::world::{self, WorldMesh};

use crate::decals::DecalCache;
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

/// Save the view at each of `times` as `<folder>/<seconds>.png`, `size`
/// pixels, for comparing with the game's frames, and close the window
pub struct Capture {
    pub times: Vec<f64>,
    pub folder: PathBuf,
    pub size: [u32; 2],
}

/// Choices from the command line for checking the viewer
#[derive(Default)]
pub struct Dev {
    /// Where to seek once the demo is loaded
    pub start: Option<f64>,
    pub bench: Option<Bench>,
    pub capture: Option<Capture>,
    /// Play this many seconds with sound, then close the window
    pub play: Option<f64>,
}

enum CapturePhase {
    /// Seek to the next time
    Seek,
    /// Wait for the seek's frame and everything it draws
    Wait(Stamp),
    /// The frame was drawn, read it back
    Drawn,
}

struct Capturing {
    capture: Capture,
    next: usize,
    phase: CapturePhase,
}

/// Which model an entity is drawn with
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ModelRef {
    /// A model by path, such as `models/w_gauss.mdl`
    File(String),
    /// A player model by the name userinfo gives, the stock player model
    /// when it is not installed, in the player's top and bottom colours
    Player(String, [u8; 2]),
}

enum Asset {
    /// A map by name, without `maps/` or `.bsp`
    Map(String),
    Model(ModelRef),
}

enum Outcome {
    Ready(Box<LoadedModel>),
    /// A `.spr`, or one of AG's sprites under a model's name
    Sprite(Box<Sprite>),
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
                                ModelRef::File(path) if path.ends_with(".spr") => {
                                    Err(LoadError::Sprite { name: path.clone() })
                                }
                                ModelRef::File(path) => load::model(search, path),
                                ModelRef::Player(name, colours) => {
                                    load::player(search, name).map(|mut m| {
                                        m.mesh = ModelMesh::build_coloured(
                                            &m.asset,
                                            0,
                                            0,
                                            Some(*colours),
                                        );
                                        m
                                    })
                                }
                            };
                            let sprite = |path: &str| match load::sprite(search, path, limits) {
                                Ok(s) => Outcome::Sprite(Box::new(s)),
                                Err(e) => Outcome::Failed(e.to_string()),
                            };
                            let outcome = match (&model, loaded) {
                                (ModelRef::File(path), _) if path.ends_with(".spr") => sprite(path),
                                (_, Ok(m)) => Outcome::Ready(Box::new(m)),
                                (ModelRef::File(path), Err(LoadError::Sprite { .. })) => {
                                    sprite(path)
                                }
                                (_, Err(e)) => Outcome::Failed(e.to_string()),
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
    Sprite(SpriteHandle),
    /// A model that could not be read
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
    /// Sound output, none without a device or in a bench or capture run
    sound: Option<Device>,
    loader: Option<Loader>,
    frame: Option<Box<PresentedFrame>>,
    /// The map whose world is drawn, and the one being read
    map: Option<String>,
    map_loading: Option<String>,
    models: HashMap<ModelRef, Slot>,
    /// The map's light under models
    light: Option<Arc<LightTree>>,
    clock: Instant,
    /// The demo's length, when the library knows it
    duration: f64,
    bench: Option<(Bench, Option<Instant>, Vec<f64>)>,
    capturing: Option<Capturing>,
    /// Seconds to play for, and when playing began
    play: Option<(f64, Option<Instant>)>,
    last_frame: Option<Instant>,
    /// The source time of the frame last drawn, so a muzzle flash fired
    /// since then is drawn at least once
    flashes_to: f64,
    /// The game's palette, for blood's colours
    palette: Vec<[u8; 3]>,
    decals: DecalCache,
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
        dev: Dev,
        ctx: &egui::Context,
    ) -> Self {
        let Dev {
            start,
            bench,
            capture,
            play,
        } = dev;
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
        let game = game_dir(&demo);
        let library = game.as_ref().and_then(|game| {
            let search = SearchPath::new(
                &install,
                game,
                &SearchOptions::default(),
                AssetLimits::default(),
            )
            .ok()?;
            Some(Arc::new(Library::new(search, AssetLimits::default())))
        });
        let worker = Worker::spawn_with(
            Limits::default(),
            library.as_ref().map(|l| l.game_files()),
            move || repaint.request_repaint(),
        );
        let palette = library.as_ref().map(|l| l.palette()).unwrap_or_default();
        let decal_wad = library
            .as_ref()
            .and_then(|l| l.read("decals.wad"))
            .and_then(|bytes| Wad::parse(bytes, AssetLimits::default()).ok());
        let quiet = bench.is_some() || capture.is_some();
        let sound = library.filter(|_| !quiet).and_then(|l| {
            let tap = std::env::var_os("LAMBDA_SLICER_SOUND_TAP").map(PathBuf::from);
            Device::open(l, tap)
                .inspect_err(|e| eprintln!("sound: {e}"))
                .ok()
        });
        let loader = game.map(|game| Loader::start(install, game, ctx.clone()));
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
            sound,

            loader,
            frame: None,
            map: None,
            map_loading: None,
            models: HashMap::new(),
            light: None,
            clock: Instant::now(),
            duration: f64::from(duration),
            bench: bench.map(|b| (b, None, Vec::new())),
            play: play.map(|seconds| (seconds, None)),
            capturing: capture.map(|capture| Capturing {
                capture,
                next: 0,
                phase: CapturePhase::Seek,
            }),
            last_frame: None,
            flashes_to: f64::NEG_INFINITY,
            palette,
            decals: DecalCache::new(decal_wad),
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
                    let LoadedMap { mut world, sky } = *map;
                    self.light = Some(Arc::new(std::mem::take(&mut world.light)));
                    resources.scene.set_world(device, queue, &world);
                    self.decals.set_world(Arc::new(world));
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
                Loaded::Model(model, Outcome::Sprite(sprite)) => {
                    let handle = resources.scene.add_sprite(device, queue, &sprite);
                    self.models.insert(model, Slot::Sprite(handle));
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

        let viewmodel = viewmodel_pose(frame).map(|(path, _)| ModelRef::File(path));
        let drawn = drawn(frame)
            .into_iter()
            .flat_map(|d| std::iter::once(d.model).chain(d.weapon));
        let flashes = frame.flashes.iter().map(|f| flash_sprite(f.sprite));
        let temp = frame
            .temp_entities
            .iter()
            .filter_map(|e| e.model.as_deref().map(|m| ModelRef::File(m.to_owned())));
        for model in drawn.chain(viewmodel).chain(flashes).chain(temp) {
            if let Entry::Vacant(slot) = self.models.entry(model) {
                let _ = loader.requests.send(Asset::Model(slot.key().clone()));
                slot.insert(Slot::Loading);
            }
        }
    }

    /// Everything drawn this frame, models posed and skinned
    fn draw_list(&self) -> DrawList {
        let Some(frame) = &self.frame else {
            return DrawList::default();
        };
        let styles = light_styles(frame);
        let light_at = |point: [f32; 3]| {
            self.light
                .as_ref()
                .and_then(|tree| tree.at(point, &styles))
                .map_or(ModelLight::UNLIT, ModelLight::from_point)
        };
        let mut instances = Vec::new();
        let mut sprites = Vec::new();
        let since = if self.flashes_to > frame.at.0 {
            f64::NEG_INFINITY
        } else {
            self.flashes_to
        };
        let flashes: Vec<&Flash> = frame
            .flashes
            .iter()
            .filter(|f| f.time.0 > since || frame.at.0 - f.time.0 < FLASH_LIFE)
            .collect();
        // Where each flash is, from its holder's pose
        let mut flashed: Vec<(&Flash, [f32; 3])> = Vec::new();
        let mut place = |holder: Holder, model: &LoadedModel, bones: &[BoneTransform]| {
            for f in flashes.iter().filter(|f| f.holder == holder) {
                if let Some(at) = model.attachment(bones, usize::from(f.attachment)) {
                    flashed.push((f, at));
                }
            }
        };
        for drawn in drawn(frame) {
            if let Some(Slot::Sprite(handle)) = self.models.get(&drawn.model) {
                sprites.push(SpriteInstance {
                    sprite: *handle,
                    frame: drawn.frame.max(0.) as usize,
                    origin: drawn.pose.origin,
                    angles: drawn.pose.angles,
                    scale: drawn.scale,
                    mode: drawn.mode,
                    amount: drawn.amount,
                    color: drawn.color,
                    steady: false,
                });
                continue;
            }
            let Some(Slot::Ready(model, handle)) = self.models.get(&drawn.model) else {
                continue;
            };
            let Some(bones) = model.pose(&drawn.pose) else {
                continue;
            };
            let light = light_at(drawn.pose.origin);
            instances.push(Instance {
                model: *handle,
                vertices: model.mesh.skin(&bones, &light),
            });
            place(Holder::Entity(drawn.id), model, &bones);

            if let Some(Slot::Ready(weapon, weapon_handle)) =
                drawn.weapon.as_ref().and_then(|w| self.models.get(w))
                && let Some(weapon_bones) = weapon.merged_pose(&drawn.pose, model, &bones)
            {
                instances.push(Instance {
                    model: *weapon_handle,
                    vertices: weapon.mesh.skin(&weapon_bones, &light),
                });
            }
        }

        // The client's own sprites and models, the models in their first
        // frame
        for m in &frame.temp_entities {
            let Some(path) = m.model.as_deref() else {
                continue;
            };
            let slot = self.models.get(&ModelRef::File(path.to_owned()));
            if let Some(Slot::Sprite(handle)) = slot {
                sprites.push(SpriteInstance {
                    sprite: *handle,
                    frame: m.frame.max(0.) as usize,
                    origin: m.origin,
                    angles: m.angles,
                    scale: m.scale,
                    mode: RenderMode::from_game(m.render_mode),
                    amount: (m.render_amount / 255.).clamp(0., 1.),
                    color: self.tint(m.tint),
                    steady: m.steady,
                });
                continue;
            }
            let Some(Slot::Ready(model, handle)) = slot else {
                continue;
            };
            let pose = EntityPose {
                sequence: 0,
                cycle: 0.,
                elapsed: 0.,
                framerate: 0.,
                controllers: [127.; 4],
                blending: [0.; 2],
                origin: m.origin,
                angles: m.angles,
                player: false,
                gait: None,
            };
            if let Some(bones) = model.pose(&pose) {
                instances.push(Instance {
                    model: *handle,
                    vertices: model.mesh.skin(&bones, &light_at(m.origin)),
                });
            }
        }

        let mut viewmodel = Vec::new();
        if let Some((path, pose)) = viewmodel_pose(frame)
            && let Some(Slot::Ready(model, handle)) = self.models.get(&ModelRef::File(path))
            && let Some(bones) = model.pose(&pose)
        {
            let light = light_at(pose.origin);
            viewmodel.push(Instance {
                model: *handle,
                vertices: model.mesh.skin(&bones, &light),
            });
            place(Holder::Viewmodel, model, &bones);
        }
        for (f, origin) in flashed {
            if let Some(Slot::Sprite(handle)) = self.models.get(&flash_sprite(f.sprite)) {
                sprites.push(SpriteInstance {
                    sprite: *handle,
                    frame: f.frame as usize,
                    origin,
                    angles: [0., 0., f.roll],
                    scale: f.scale,
                    mode: RenderMode::from_game(5),
                    amount: 1.,
                    color: [0.; 3],
                    steady: false,
                });
            }
        }

        DrawList {
            brushes: brushes(frame),
            instances,
            sprites,
            viewmodel,
            decals: self.decals.mesh(),
        }
    }

    /// A temporary sprite's colour, black for its own
    fn tint(&self, tint: Tint) -> [f32; 3] {
        let rgb = match tint {
            Tint::None => return [0.; 3],
            Tint::Rgb(rgb) => rgb,
            Tint::Palette(i) => match self.palette.get(usize::from(i)) {
                Some(rgb) => *rgb,
                None => return [0.; 3],
            },
        };
        rgb.map(|c| f32::from(c) / 255.)
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

    /// Whether the frame shown answers `stamp`, and its map and every model
    /// it draws are in
    fn ready(&self, stamp: Stamp) -> bool {
        let Some(frame) = &self.frame else {
            return false;
        };
        let map = frame.map.as_deref().map(map_name);
        let viewmodel = viewmodel_pose(frame).map(|(path, _)| ModelRef::File(path));
        let loading = drawn(frame)
            .into_iter()
            .flat_map(|d| std::iter::once(d.model).chain(d.weapon))
            .chain(viewmodel)
            .any(|m| matches!(self.models.get(&m), None | Some(Slot::Loading)));
        frame.stamp == stamp && map.is_some() && self.map == map && !loading
    }

    /// Step a capture run, and the size to draw at while it lasts. True in
    /// the second place when the run is over
    fn step_capture(&mut self) -> (Option<[u32; 2]>, bool) {
        let Some(capturing) = &self.capturing else {
            return (None, false);
        };
        let size = capturing.capture.size;
        match capturing.phase {
            CapturePhase::Seek => {
                let Some(&time) = capturing.capture.times.get(capturing.next) else {
                    return (Some(size), true);
                };
                self.command(Command::SeekTo(SourceTime(time)));
                let stamp = self.controller.stamp();
                if let Some(c) = &mut self.capturing {
                    c.phase = CapturePhase::Wait(stamp);
                }
            }
            CapturePhase::Wait(stamp) => {
                if self.ready(stamp)
                    && let Some(c) = &mut self.capturing
                {
                    c.phase = CapturePhase::Drawn;
                }
            }
            CapturePhase::Drawn => {
                let image = {
                    let renderer = self.render_state.renderer.read();
                    renderer
                        .callback_resources
                        .get::<Resources>()
                        .and_then(|r| {
                            r.scene
                                .read_color(&self.render_state.device, &self.render_state.queue)
                        })
                };
                let time = capturing.capture.times[capturing.next];
                let path = capturing.capture.folder.join(format!("{time:09.3}.png"));
                if let Some(image) = image
                    && let Err(e) = image::save_buffer(
                        &path,
                        &image.pixels,
                        image.width,
                        image.height,
                        image::ColorType::Rgba8,
                    )
                {
                    eprintln!("cannot save {}: {e}", path.display());
                }
                if let Some(c) = &mut self.capturing {
                    c.next += 1;
                    c.phase = CapturePhase::Seek;
                }
            }
        }
        (Some(size), false)
    }

    /// The view in `height` points with its controls under it, or all the
    /// space left in a bench run
    pub fn ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, height: f32) {
        self.poll();
        self.request_assets();
        if let Some(frame) = &self.frame {
            self.decals.update(&frame.decals, frame.decal_revision);
        }
        let ctx = ui.ctx().clone();
        let (capture_size, captured) = self.step_capture();
        if captured {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if capture_size.is_some() {
            ctx.request_repaint();
        }
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
        let mut drawn_at = None;
        match (&self.frame, &self.map) {
            (Some(frame), Some(_)) => {
                let pixels = capture_size.unwrap_or([
                    (rect.width() * ui.pixels_per_point()).round().max(1.) as u32,
                    (rect.height() * ui.pixels_per_point()).round().max(1.) as u32,
                ]);
                let draw = self.draw_list();
                drawn_at = Some(frame.at.0);
                drawn = draw.instances.len() + draw.sprites.len() + draw.viewmodel.len();
                let callback = Frame {
                    pixels,
                    view: view(frame, pixels[0] as f32 / pixels[1] as f32),
                    draw,
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
        if let Some(at) = drawn_at {
            self.flashes_to = at;
        }

        self.feed_sound();
        if self.step_play() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.controls(ui, catalog, rtl);
        if self.controller.playing() || drawn > 0 && self.bench.is_some() {
            ctx.request_repaint();
        }
    }

    /// Start a timed play once the first frame is in, and true when it is over
    fn step_play(&mut self) -> bool {
        let Some((seconds, began)) = self.play else {
            return false;
        };
        match began {
            None if self.frame.is_some() && self.map.is_some() => {
                self.command(Command::Play);
                self.play = Some((seconds, Some(Instant::now())));
                false
            }
            Some(began) => began.elapsed().as_secs_f64() >= seconds,
            None => false,
        }
    }

    /// Tell the sound mixer what the frame shown plays
    fn feed_sound(&self) {
        let (Some(sound), Some(frame), Some(demo)) =
            (&self.sound, &self.frame, self.worker.loaded())
        else {
            return;
        };
        sound.feed(Feed {
            stamp: self.controller.stamp(),
            demo,
            clock: self.controller.clock(),
            clock_base: self.clock,
            audible: frame.audible,
        });
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

            if let Some(sound) = &self.sound {
                let label =
                    text::widget_text(&ctx, &catalog.text("viewer-volume"), rtl, None, Look::Plain);
                ui.label(label);
                let mut volume = sound.volume();
                ui.spacing_mut().slider_width = 80.;
                if ui
                    .add(egui::Slider::new(&mut volume, 0. ..=1.).show_value(false))
                    .changed()
                {
                    sound.set_volume(volume);
                }
            }

            let mut at = now;
            ui.spacing_mut().slider_width = ui.available_width().max(40.);
            let slider = egui::Slider::new(&mut at, 0. ..=end.max(0.001)).show_value(false);
            if ui.add(slider).changed() {
                self.command(Command::SeekTo(SourceTime(at)));
            }
        });
    }
}

/// One entity to draw, its model, what it holds and its pose
struct Drawn {
    id: EntityId,
    model: ModelRef,
    weapon: Option<ModelRef>,
    pose: EntityPose,
    /// What a sprite takes from its entity
    frame: f32,
    scale: f32,
    mode: RenderMode,
    amount: f32,
    color: [f32; 3],
}

/// Which model each drawn entity uses and how it is posed. Players use the
/// model their userinfo names, upright as the game draws them. The entity
/// the recorder views from is left out, and so is the player a spectator
/// sees through the eyes of
fn drawn(frame: &PresentedFrame) -> Vec<Drawn> {
    let viewer = frame.recorder.view_entity.unwrap_or(frame.local_entity);
    let observed =
        (frame.recorder.observer_mode == OBS_IN_EYE).then_some(frame.recorder.observer_target);
    let client_time = frame.client_time;

    let mut out = Vec::new();
    for e in frame.entities.iter() {
        let number = e.id.number;
        if number == viewer || observed == Some(i64::from(number)) || e.effects & NO_DRAW != 0 {
            continue;
        }

        let player = frame.players.iter().find(|p| p.entity == number);
        let model = match (player.and_then(|p| p.model.as_deref()), e.model.as_deref()) {
            (Some(name), _) => {
                let colours = player.map_or([0; 2], |p| {
                    [p.top_color.unwrap_or(0), p.bottom_color.unwrap_or(0)]
                });
                ModelRef::Player(name.to_owned(), colours)
            }
            (None, Some(path)) if path.ends_with(".mdl") || path.ends_with(".spr") => {
                ModelRef::File(path.to_owned())
            }
            _ => continue,
        };
        let is_player = player.is_some();
        let gait = e.gait.filter(|_| is_player).map(|g| GaitPose {
            sequence: e.gait_sequence.max(0) as usize,
            yaw: g.yaw,
            torso: g.torso,
            distance: g.distance,
            elapsed: g.elapsed,
        });
        let weapon = e
            .weapon
            .as_deref()
            .filter(|w| is_player && w.ends_with(".mdl"))
            .map(|w| ModelRef::File(w.to_owned()));
        let mode = RenderMode::from_game(e.render_mode);
        out.push(Drawn {
            id: e.id,
            model,
            weapon,
            frame: e.frame,
            scale: e.scale,
            mode,
            amount: match mode {
                RenderMode::Normal => 1.,
                _ => e.render_amount.clamp(0, 255) as f32 / 255.,
            },
            color: e.render_color.map(|c| f32::from(c) / 255.),
            pose: EntityPose {
                sequence: e.sequence.max(0) as usize,
                cycle: f64::from(e.frame),
                elapsed: client_time - f64::from(e.animtime),
                framerate: f64::from(e.framerate),
                controllers: e.controllers.map(f64::from),
                blending: e.blending.map(f64::from),
                origin: e.origin,
                angles: e.angles,
                player: is_player,
                gait,
            },
        });
    }
    out
}

/// How long a muzzle flash lasts, as the game's temporary entity does
/// (`R_MuzzleFlash`), though it is drawn at least once
const FLASH_LIFE: f64 = 0.01;

/// One of the game's three muzzle flash sprites
fn flash_sprite(sprite: u8) -> ModelRef {
    ModelRef::File(format!("sprites/muzzleflash{}.spr", sprite + 1))
}

/// The recorder's first-person weapon and its pose, the last weapon
/// animation played from when it started, at the view's origin and angles,
/// the pitch turned the other way as the game gives it to a viewmodel. None
/// while dead, at intermission, or spectating other than through a player's
/// eyes
fn viewmodel_pose(frame: &PresentedFrame) -> Option<(String, EntityPose)> {
    let camera = &frame.recorder;
    let path = camera
        .viewmodel
        .as_deref()
        .filter(|p| p.ends_with(".mdl"))?;
    let observing = camera.observer_mode != 0 && camera.observer_mode != OBS_IN_EYE;
    if camera.health <= 0 || camera.intermission || observing {
        return None;
    }
    let animation = frame.viewmodel?;
    let pose = EntityPose {
        sequence: animation.sequence.max(0) as usize,
        cycle: 0.,
        elapsed: frame.at.0 - animation.started.0,
        framerate: 1.,
        controllers: [127.; 4],
        blending: [0.; 2],
        origin: camera.origin,
        angles: [-camera.angles[0], camera.angles[1], camera.angles[2]],
        player: false,
        gait: None,
    };
    Some((path.to_owned(), pose))
}

/// Every light style's brightness at the frame's client time
fn light_styles(frame: &PresentedFrame) -> [f32; 64] {
    let mut styles = [1.; 64];
    for (index, pattern) in &frame.light_styles {
        if let Some(slot) = styles.get_mut(usize::from(*index)) {
            *slot = world::style_brightness(pattern, frame.client_time);
        }
    }
    styles
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
fn view(frame: &PresentedFrame, aspect: f32) -> View {
    View {
        view_projection: view_projection(frame, aspect),
        eye: frame.recorder.origin,
        angles: frame.recorder.angles,
        time: frame.client_time as f32,
        light_styles: light_styles(frame),
    }
}

/// The recorder's view. The recorded field of view is across the screen's
/// width, as the game's is
fn view_projection(frame: &PresentedFrame, aspect: f32) -> Matrix {
    let aspect = aspect.max(0.01);
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
    draw: DrawList,
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
        let (resized, commands) =
            r.scene
                .render(device, queue, self.pixels, &self.view, &self.draw);
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
