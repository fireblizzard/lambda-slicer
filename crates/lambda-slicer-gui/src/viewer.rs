//! The M8 backend spike's view of a demo's map, with a player model running
//! at the first spawn point (docs/viewer-backend.md)
//!
//! The map and the model are read on a thread of their own, then uploaded to
//! the window's own GPU device. Each frame poses and skins the model on the
//! CPU, draws the scene offscreen at the view's size in pixels, and egui
//! shows the image through a paint callback. Dragging turns the camera
//! around the spawn point and scrolling moves it closer. Shown only when a
//! game folder is given with `--game`, as nothing here is translated yet

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use eframe::egui::{self, Rect, Sense, Ui};
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use eframe::wgpu;
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_core::container::read_header;
use lambda_slicer_render::camera::{Camera, Matrix};
use lambda_slicer_render::gpu::{Blit, Scene};
use lambda_slicer_render::load::{self, PlayerModel};
use lambda_slicer_render::model::ModelVertex;
use lambda_slicer_render::world::{self, WorldMesh};

/// The game's 90 degrees across a 4 by 3 view, as a height
const FOV_Y: f32 = 73.74;

/// Time frames for this many seconds once the scene is in, write what was
/// measured to `out` and close the window
pub struct Bench {
    pub seconds: f64,
    pub out: PathBuf,
}

struct Prepared {
    world: WorldMesh,
    player: Option<PlayerModel>,
}

struct Ready {
    player: Option<PlayerModel>,
    sequence: usize,
    spawn: [f32; 3],
    yaw: f32,
}

enum State {
    Loading(Receiver<Result<Prepared, String>>),
    Ready(Box<Ready>),
    Failed,
}

/// What the window's renderer keeps for the viewer between frames
struct Resources {
    scene: Scene,
    blit: Blit,
    blit_group: Option<wgpu::BindGroup>,
}

pub struct Viewer {
    render_state: RenderState,
    state: State,
    started: Instant,
    /// The camera's turn around the spawn point in degrees and its distance
    turn: f32,
    distance: f32,
    bench: Option<(Bench, Option<Instant>, Vec<f64>)>,
    last_frame: Option<Instant>,
}

/// The demo's game folder and map name, from its header
fn header(demo: &std::path::Path) -> Result<(String, String), String> {
    let mut file = std::fs::File::open(demo).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let h = read_header(&mut file, len).map_err(|e| e.to_string())?;
    let map = h.map_name();
    let map = map
        .strip_prefix("maps/")
        .unwrap_or(&map)
        .trim_end_matches(".bsp")
        .to_owned();
    Ok((h.game_dir(), map))
}

fn prepare(install: PathBuf, demo: PathBuf) -> Result<Prepared, String> {
    let (game, map) = header(&demo)?;
    let search = SearchPath::new(
        &install,
        &game,
        &SearchOptions::default(),
        AssetLimits::default(),
    )
    .map_err(|e| e.to_string())?;
    let (world, unread) =
        load::map(&search, &map, AssetLimits::default()).map_err(|e| e.to_string())?;
    if !unread.is_empty() {
        eprintln!("viewer: WADs not read {unread:?}");
    }
    if !world.missing_textures.is_empty() {
        eprintln!("viewer: textures not found {:?}", world.missing_textures);
    }
    let player = load::player(&search, "gordon")
        .inspect_err(|e| eprintln!("viewer: no player model, {e}"))
        .ok();
    Ok(Prepared { world, player })
}

impl Viewer {
    pub fn open(
        render_state: RenderState,
        install: PathBuf,
        demo: PathBuf,
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
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("viewer load".into())
            .spawn(move || {
                let _ = tx.send(prepare(install, demo));
                ctx.request_repaint();
            })
            .expect("spawn viewer load");
        Self {
            render_state,
            state: State::Loading(rx),
            started: Instant::now(),
            turn: 0.,
            distance: 140.,
            bench: bench.map(|b| (b, None, Vec::new())),
            last_frame: None,
        }
    }

    fn poll(&mut self) {
        let State::Loading(rx) = &self.state else {
            return;
        };
        let prepared = match rx.try_recv() {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => {
                eprintln!("viewer: {e}");
                self.state = State::Failed;
                return;
            }
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.state = State::Failed;
                return;
            }
        };
        let (spawn, yaw) = world::spawn(&prepared.world.entities);
        let RenderState {
            device,
            queue,
            renderer,
            ..
        } = &self.render_state;
        if let Some(r) = renderer.write().callback_resources.get_mut::<Resources>() {
            r.scene.set_world(device, queue, &prepared.world);
            if let Some(p) = &prepared.player {
                r.scene.set_model(device, queue, &p.mesh);
            }
        }
        let sequence = prepared
            .player
            .as_ref()
            .and_then(|p| p.sequence("run"))
            .unwrap_or(0);
        self.state = State::Ready(Box::new(Ready {
            player: prepared.player,
            sequence,
            spawn,
            yaw,
        }));
        self.started = Instant::now();
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
    fn time_frame(&mut self, pixels: [u32; 2]) -> bool {
        let now = Instant::now();
        let dt = self.last_frame.map(|l| (now - l).as_secs_f64() * 1000.);
        self.last_frame = Some(now);
        if self.bench.is_none() {
            return false;
        }
        let gpu_bytes = self.gpu_bytes();
        let adapter = self.render_state.adapter.get_info();
        let Some((bench, begin, times)) = &mut self.bench else {
            return false;
        };
        // A second to settle before timing
        if self.started.elapsed().as_secs_f64() < 1. {
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
            "{} ({:?}), view {}x{} pixels, {} frames, mean {mean:.3} ms ({:.0} fps), p50 {:.3}, p95 {:.3}, p99 {:.3}, max {:.3} ms, scene {:.1} MiB on the GPU\n",
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

    /// The view in `height` points, or all the space left in a bench run
    pub fn ui(&mut self, ui: &mut Ui, height: f32) {
        self.poll();
        let height = match self.bench {
            Some(_) => ui.available_height(),
            None => height,
        };
        let size = egui::vec2(ui.available_width(), height);
        match &self.state {
            State::Failed => return,
            State::Loading(_) => {
                ui.allocate_ui(size, |ui| {
                    ui.centered_and_justified(|ui| ui.spinner());
                });
                return;
            }
            State::Ready(_) => {}
        }
        let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
        self.turn -= response.drag_delta().x * 0.4;
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            self.distance = (self.distance * (1. - scroll * 0.002)).clamp(32., 2048.);
        }
        let pixels = [
            (rect.width() * ui.pixels_per_point()).round().max(1.) as u32,
            (rect.height() * ui.pixels_per_point()).round().max(1.) as u32,
        ];
        let frame = self.frame(rect, pixels);
        ui.painter()
            .add(egui_wgpu::Callback::new_paint_callback(rect, frame));
        if self.time_frame(pixels) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.ctx().request_repaint();
    }

    fn frame(&self, rect: Rect, pixels: [u32; 2]) -> Frame {
        let State::Ready(ready) = &self.state else {
            unreachable!("frames are drawn once the scene is in")
        };
        let t = self.started.elapsed().as_secs_f64();
        let angle = (t as f32 * 0.4 + self.turn.to_radians()) % std::f32::consts::TAU;
        let s = ready.spawn;
        let eye = [
            s[0] + angle.cos() * self.distance,
            s[1] + angle.sin() * self.distance,
            s[2] + 40.,
        ];
        let camera = Camera::looking_at(eye, s, FOV_Y);
        let vertices = ready
            .player
            .as_ref()
            .and_then(|p| Some(p.mesh.skin(&p.bones_at(ready.sequence, t, s, ready.yaw)?)))
            .unwrap_or_default();
        Frame {
            pixels,
            view_projection: camera.view_projection(rect.aspect_ratio(), 4., 8192.),
            vertices,
        }
    }
}

/// One frame of the scene, drawn offscreen in `prepare` and shown in `paint`
struct Frame {
    pixels: [u32; 2],
    view_projection: Matrix,
    vertices: Vec<ModelVertex>,
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
            self.view_projection,
            &self.vertices,
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
