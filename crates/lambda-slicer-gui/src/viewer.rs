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
use lambda_slicer_playback::controller::{Command, Controller, LoopRange, Reply, Stamp};
use lambda_slicer_playback::hud::{Countdown, WAITING};
use lambda_slicer_playback::scene::{Entity, EntityId};
use lambda_slicer_playback::tempents::Tint;
use lambda_slicer_playback::time::{Rate, SourceTime};
use lambda_slicer_playback::trace::Hull;
use lambda_slicer_playback::worker::Worker;
use lambda_slicer_render::Rgba;
use lambda_slicer_render::camera::{Camera, Matrix};
use lambda_slicer_render::gpu::{
    Blit, Brush, DrawList, Instance, ModelHandle, RenderMode, Scene, SpriteHandle, SpriteInstance,
    Streak, StreakPoint, View,
};
use lambda_slicer_render::light::{LightTree, ModelLight};
use lambda_slicer_render::load::{self, EntityPose, GaitPose, LoadError, LoadedModel};
use lambda_slicer_render::model::{Eye, ModelMesh, ModelVertex};
use lambda_slicer_render::world::{self, WorldMesh};

use fluent_bundle::FluentValue;
use fluent_bundle::types::{FluentNumber, FluentNumberOptions};

use crate::decals::DecalCache;
use crate::i18n::Catalog;
use crate::layout;
use crate::settings::ViewerSettings;
use crate::text::{self, Look};

/// `EF_NODRAW`, an entity the game does not draw
const NO_DRAW: i64 = 128;

/// The field of view when a recording names none
const DEFAULT_FOV: f32 = 90.;

/// `OBS_IN_EYE`, a spectator seeing through the eyes of the player it
/// observes
const OBS_IN_EYE: i64 = 4;

/// Time frames for this many seconds once playback runs, write what was
/// measured to `out` and close the window. The view is drawn `size` pixels
/// when given, whatever room the window has for it
pub struct Bench {
    pub seconds: f64,
    pub out: PathBuf,
    pub size: Option<[u32; 2]>,
}

/// A bench run under way
struct Benching {
    bench: Bench,
    /// When the viewer opened the demo
    opened: Instant,
    /// The source time of the first frame drawn, a second of playback
    /// after which timing begins
    settle_from: Option<f64>,
    /// When timing began
    begin: Option<Instant>,
    /// Milliseconds between frames
    times: Vec<f64>,
    /// Milliseconds after opening until the first picture, until every
    /// model it wanted was in, and until the checkpoints covered the demo
    picture: Option<f64>,
    complete: Option<f64>,
    indexed: Option<f64>,
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
    /// when it is not installed, in the player's top and bottom colours,
    /// body and skin
    Player(String, [u8; 2], u32, usize),
    /// A studio model in another body and skin than its first, the body
    /// reduced to the submodels it selects
    Variant(String, u32, usize),
}

enum Asset {
    /// A map by name, without `maps/` or `.bsp`
    Map(String),
    Model(ModelRef),
    /// A sky by name, from `gfx/env`
    Sky(String),
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
    /// The map's own sky, or why it could not be read
    sky: Result<[Rgba; 6], String>,
}

enum Loaded {
    Map(String, Result<Box<LoadedMap>, String>),
    Model(ModelRef, Outcome),
    /// A sky by name, or why it could not be read
    Sky(String, Result<Box<[Rgba; 6]>, String>),
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
                                    .map_err(|e| e.to_string());
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
                                ModelRef::Variant(path, body, skin) => load::model(search, path)
                                    .map(|mut m| {
                                        m.mesh = ModelMesh::build(&m.asset, *body, *skin);
                                        m
                                    }),
                                ModelRef::Player(name, colours, body, skin) => {
                                    load::player(search, name).map(|mut m| {
                                        m.mesh = ModelMesh::build_coloured(
                                            &m.asset,
                                            *body,
                                            *skin,
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
                        (Ok(search), Asset::Sky(name)) => {
                            let sky = load::sky(search, &name, limits)
                                .map(Box::new)
                                .map_err(|e| e.to_string());
                            Loaded::Sky(name, sky)
                        }
                        (Err(e), Asset::Sky(name)) => Loaded::Sky(name, Err(e.clone())),
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
    /// The settings it shows and plays with, the app's each frame
    pub settings: ViewerSettings,
    render_state: RenderState,
    controller: Controller,
    worker: Worker,
    /// Sound output, none without a device or in a bench or capture run
    sound: Option<Device>,
    /// A scrub Escape cancelled, whose drag goes on until it is let go
    scrub_cancelled: bool,
    /// Why there is no sound, when the device could not be opened
    sound_error: Option<String>,
    /// While measuring A/V skew, where to write how far each picture
    /// painted trails the playhead, and those lags
    video_lag: Option<(PathBuf, Vec<(f64, f64)>)>,
    loader: Option<Loader>,
    frame: Option<Box<PresentedFrame>>,
    /// The map whose world is drawn, and the one being read
    map: Option<String>,
    map_loading: Option<String>,
    /// A map that could not be loaded and why, shown in place of the view
    /// and not asked for again
    map_failed: Option<(String, String)>,
    /// The sky drawn and the one being read
    sky: Option<String>,
    sky_loading: Option<String>,
    models: HashMap<ModelRef, Slot>,
    /// The map's light under models
    light: Option<Arc<LightTree>>,
    clock: Instant,
    /// The demo's length, when the library knows it
    duration: f64,
    bench: Option<Benching>,
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
    /// The game's files, for the map's hull that hides glows
    library: Option<Arc<Library>>,
    /// Maps, models, sprites and skies that could not be read, and why
    missing: std::collections::BTreeMap<String, String>,
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
        // Why the sound device could not be opened, shown under the view
        let sound_error = std::cell::RefCell::new(None);
        let glow_library = library.clone();
        let sound = library.filter(|_| !quiet).and_then(|l| {
            let tap = std::env::var_os("LAMBDA_SLICER_SOUND_TAP").map(PathBuf::from);
            Device::open(l, tap, skew_path())
                .inspect_err(|e| {
                    eprintln!("sound: {e}");
                    *sound_error.borrow_mut() = Some(e.clone());
                })
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
            settings: ViewerSettings::default(),
            render_state,
            controller,
            worker,
            sound,
            scrub_cancelled: false,
            sound_error: sound_error.into_inner(),
            video_lag: skew_path().map(|p| (p.with_extension("video.txt"), Vec::new())),

            loader,
            frame: None,
            map: None,
            map_loading: None,
            missing: std::collections::BTreeMap::new(),
            map_failed: None,
            sky: None,
            sky_loading: None,
            models: HashMap::new(),
            light: None,
            clock: Instant::now(),
            duration: f64::from(duration),
            bench: bench.map(|bench| Benching {
                bench,
                opened: Instant::now(),
                settle_from: None,
                begin: None,
                times: Vec::new(),
                picture: None,
                complete: None,
                indexed: None,
            }),
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
            library: glow_library,
        }
    }

    fn wall(&self) -> f64 {
        self.clock.elapsed().as_secs_f64()
    }

    /// Move the playhead to `time`, playing on if it was playing
    pub fn seek(&mut self, time: f32) {
        self.command(Command::SeekTo(SourceTime(f64::from(time.max(0.)))));
    }

    /// Play from `from` and pause at `to`
    pub fn play_range(&mut self, from: f32, to: f32) {
        self.command(Command::PlayRange {
            from: SourceTime(f64::from(from.max(0.))),
            to: SourceTime(f64::from(to.max(0.))),
        });
    }

    /// Play `from` to `to` over and over, or no longer loop with none,
    /// sending nothing when the loop is already that
    pub fn set_loop(&mut self, range: Option<(f32, f32)>) {
        let range = range.map(|(from, to)| LoopRange {
            start: SourceTime(f64::from(from.max(0.))),
            end: SourceTime(f64::from(to.max(0.))),
        });
        if self.controller.loop_range() != range {
            self.command(Command::SetLoop(range));
        }
    }

    /// Loop `from` to `to`, playing from `from`
    pub fn play_loop(&mut self, from: f32, to: f32) {
        self.set_loop(Some((from, to)));
        self.seek(from);
        self.command(Command::Play);
    }

    /// Show the moment at `time`, paused, as editing a clip's start or end
    /// does
    pub fn preview(&mut self, time: f32) {
        self.command(Command::Pause);
        self.seek(time);
    }

    /// Drag the playhead to `time`, playback held still and silent until
    /// the drag ends. A drag Escape cancelled moves nothing
    pub fn scrub(&mut self, time: f32) {
        if !self.scrub_cancelled {
            self.command(Command::Scrub(SourceTime(f64::from(time.max(0.)))));
        }
    }

    /// The drag is let go, playback going on from where it was dragged to
    pub fn end_scrub(&mut self) {
        if !std::mem::take(&mut self.scrub_cancelled) {
            self.command(Command::EndScrub { cancel: false });
        }
    }

    /// The playhead, in seconds of playback
    pub fn time(&self) -> f32 {
        self.controller.time(self.wall()).0 as f32
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
                    let sky = sky
                        .inspect_err(|e| {
                            self.missing.insert(sky_file(&world.sky), e.clone());
                        })
                        .ok();
                    self.light = Some(Arc::new(std::mem::take(&mut world.light)));
                    resources.scene.set_world(device, queue, &world);
                    self.sky = Some(world.sky.clone());
                    self.decals.set_world(Arc::new(world));
                    resources.scene.set_sky(device, queue, sky.as_ref());
                    self.map = Some(name);
                    self.map_loading = None;
                }
                Loaded::Map(name, Err(e)) if self.map_loading.as_ref() == Some(&name) => {
                    eprintln!("viewer: map {name}, {e}");
                    self.missing.insert(format!("maps/{name}.bsp"), e.clone());
                    // Nothing of the map drawn before is kept for this one
                    self.map = None;
                    self.light = None;
                    self.map_loading = None;
                    self.map_failed = Some((name, e));
                }
                Loaded::Map(..) => {}
                Loaded::Sky(name, sky) => {
                    // A sky that cannot be read leaves the one drawn
                    match sky {
                        Ok(sky) => resources.scene.set_sky(device, queue, Some(&*sky)),
                        Err(e) => {
                            self.missing.insert(sky_file(&name), e);
                        }
                    }
                    if self.sky_loading.as_ref() == Some(&name) {
                        self.sky_loading = None;
                    }
                    self.sky = Some(name);
                }
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
                    self.missing.insert(model_file(&model), e);
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
            && self
                .map_failed
                .as_ref()
                .is_none_or(|(name, _)| *name != map)
        {
            let _ = loader.requests.send(Asset::Map(map.clone()));
            self.map_loading = Some(map);
        }

        // The sky the server names, once the map's own is drawn
        if let Some(sky) = frame.sky.as_deref()
            && self.map_loading.is_none()
            && self
                .sky
                .as_deref()
                .is_some_and(|drawn| !drawn.eq_ignore_ascii_case(sky))
            && self.sky_loading.as_deref() != Some(sky)
        {
            let _ = loader.requests.send(Asset::Sky(sky.to_owned()));
            self.sky_loading = Some(sky.to_owned());
        }

        for model in wanted(frame, &self.models) {
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
        // Chrome reflects the view the scene is drawn from
        let eye = Eye::new(frame.recorder.origin, frame.recorder.angles);
        let mut instances = Vec::new();
        let mut silhouettes = Vec::new();
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
        // Where each flash is, and the gun of each beam drawn from one, from
        // the holder's pose
        let gun_holders: Vec<Holder> = frame
            .beams
            .beams
            .iter()
            .filter_map(|b| b.from_gun.and_then(|n| gun_holder(frame, n)))
            .collect();
        let mut flashed: Vec<(&Flash, [f32; 3])> = Vec::new();
        let mut guns: HashMap<Holder, [f32; 3]> = HashMap::new();
        let mut place = |holder: Holder, model: &LoadedModel, bones: &[BoneTransform]| {
            for f in flashes.iter().filter(|f| f.holder == holder) {
                if let Some(at) = model.attachment(bones, usize::from(f.attachment)) {
                    flashed.push((f, at));
                }
            }
            if gun_holders.contains(&holder)
                && let Some(at) = model.attachment(bones, 0)
            {
                guns.insert(holder, at);
            }
        };
        for drawn in drawn(frame, &self.models) {
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
                    // `kRenderFxNoDissipation`
                    steady: drawn.fx == 14,
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
            let first = instances.len();
            instances.push(Instance {
                model: *handle,
                vertices: model.mesh.skin(&bones, &light, &eye),
                mode: drawn.mode,
                amount: drawn.amount,
            });
            place(Holder::Entity(drawn.id), model, &bones);
            // Shown again where walls hide it, a player in its top colour
            let tint = match drawn.pose.player {
                true if self.settings.see_through => Some(team_tint(drawn.top_color)),
                false if self.settings.see_through_items => Some(ITEM_TINT),
                _ => None,
            };

            // The weapon is drawn as its holder is
            if let Some(Slot::Ready(weapon, weapon_handle)) =
                drawn.weapon.as_ref().and_then(|w| self.models.get(w))
                && let Some(weapon_bones) = weapon.merged_pose(&drawn.pose, model, &bones)
            {
                instances.push(Instance {
                    model: *weapon_handle,
                    vertices: weapon.mesh.skin(&weapon_bones, &light, &eye),
                    mode: drawn.mode,
                    amount: drawn.amount,
                });
            }
            if let Some(tint) = tint {
                for instance in &instances[first..] {
                    silhouettes.push(silhouette(instance, tint));
                }
            }
        }

        // The client's own sprites and models, the models in their first
        // frame
        for m in &frame.temp_entities {
            let Some(path) = m.model.as_deref() else {
                continue;
            };
            let slot = self
                .models
                .get(&studio_ref(&self.models, path, i64::from(m.body), 0));
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
                    vertices: model.mesh.skin(&bones, &light_at(m.origin), &eye),
                    mode: RenderMode::from_game(m.render_mode),
                    amount: (m.render_amount / 255.).clamp(0., 1.),
                });
            }
        }

        let mut viewmodel = Vec::new();
        if self.settings.viewmodel
            && let Some((_, pose)) = viewmodel_pose(frame)
            && let Some(Slot::Ready(model, handle)) = viewmodel_ref(frame, &self.models)
                .as_ref()
                .and_then(|m| self.models.get(m))
            && let Some(bones) = model.pose(&pose)
        {
            let light = light_at(pose.origin);
            viewmodel.push(Instance {
                model: *handle,
                vertices: model.mesh.skin(&bones, &light, &eye),
                mode: RenderMode::Normal,
                amount: 1.,
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

        // A glow the eye cannot see is not drawn (`R_SpriteGlowBlend`)
        let world = frame
            .map
            .as_deref()
            .and_then(|map| self.library.as_ref()?.hull(map));
        if let Some(world) = world {
            let eye = frame.recorder.origin;
            sprites.retain(|s| s.mode != RenderMode::Glow || !hidden_glow(&world, eye, s.origin));
        }

        DrawList {
            brushes: brushes(frame),
            instances,
            sprites,
            viewmodel,
            silhouettes,
            decals: self.decals.mesh(),
            streaks: self.streaks(frame, &guns),
        }
    }

    /// The frame's beams, trails and tracers as strips to draw
    fn streaks(&self, frame: &PresentedFrame, guns: &HashMap<Holder, [f32; 3]>) -> Vec<Streak> {
        let at = frame.at;
        let sprite =
            |path: &str, frame: usize| match self.models.get(&ModelRef::File(path.to_owned())) {
                Some(Slot::Sprite(handle)) => Some((*handle, frame)),
                _ => None,
            };
        let mut out = Vec::new();
        for trail in &frame.beams.trails {
            let Some(sprite) = sprite(&trail.sprite, 0) else {
                continue;
            };
            // From the entity back through each point, fading as the points
            // age and gone at the oldest
            let mut points: Vec<StreakPoint> = trail
                .head
                .map(|position| StreakPoint {
                    position,
                    v: 0.,
                    color: trail.color,
                })
                .into_iter()
                .collect();
            let last = trail.points.len().saturating_sub(1);
            for (i, (position, die)) in trail.points.iter().enumerate() {
                let fraction = if i == last {
                    0.
                } else {
                    ((die.0 - at.0) as f32 / trail.life.max(1e-3)).clamp(0., 1.)
                };
                points.push(StreakPoint {
                    position: *position,
                    v: points.len() as f32,
                    color: trail.color.map(|c| c * fraction),
                });
            }
            out.push(Streak {
                sprite: Some(sprite),
                points,
                width: trail.width,
            });
        }
        for beam in &frame.beams.beams {
            let elapsed = (at.0 - beam.born.0).max(0.) as f32;
            let Some(sprite) = sprite(
                &beam.sprite,
                (beam.start_frame + beam.framerate * elapsed) as usize,
            ) else {
                continue;
            };
            // From the gun when the beam has one to start at
            let start = beam
                .from_gun
                .and_then(|n| gun_holder(frame, n))
                .and_then(|holder| guns.get(&holder).copied())
                .unwrap_or(beam.start);
            let d = [0, 1, 2].map(|k| beam.end[k] - start[k]);
            let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let v = (elapsed * beam.scroll).fract();
            out.push(Streak {
                sprite: Some(sprite),
                points: vec![
                    StreakPoint {
                        position: start,
                        v,
                        color: beam.color,
                    },
                    StreakPoint {
                        position: beam.end,
                        v: v + length * 0.01,
                        color: beam.color,
                    },
                ],
                width: beam.width,
            });
        }
        for e in frame.entities.iter().filter(|e| e.beam) {
            let model = e.model.as_deref().unwrap_or_default();
            if let Some(sprite) = sprite(model, e.frame.max(0.) as usize) {
                out.push(entity_beam(frame, e, sprite));
            }
        }
        for tracer in &frame.beams.tracers {
            let (tail, head) = tracer.streak(at);
            let point = |position, v| StreakPoint {
                position,
                v,
                color: TRACER_COLOR,
            };
            out.push(Streak {
                sprite: None,
                points: vec![point(tail, 0.), point(head, 1.)],
                width: TRACER_WIDTH,
            });
        }
        out
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
        let complete = self.ready(self.controller.stamp());
        let indexed = self.frame.as_ref().is_some_and(|f| f.indexed.1);
        let Some(run) = &mut self.bench else {
            return false;
        };
        let since = (now - run.opened).as_secs_f64() * 1000.;
        run.picture.get_or_insert(since);
        if complete {
            run.complete.get_or_insert(since);
        }
        if indexed {
            run.indexed.get_or_insert(since);
        }

        // A second of playback to settle before timing, from wherever the
        // bench starts in the demo
        let Some(at) = self.frame.as_ref().map(|f| f.at.0) else {
            return false;
        };
        let settle_from = *run.settle_from.get_or_insert(at);
        if at - settle_from < 1. {
            return false;
        }
        let begin = *run.begin.get_or_insert(now);
        if let Some(dt) = dt {
            run.times.push(dt);
        }
        if (now - begin).as_secs_f64() < run.bench.seconds || run.times.is_empty() {
            return false;
        }

        let mut sorted = run.times.clone();
        sorted.sort_by(f64::total_cmp);
        let at = |p: f64| sorted[((sorted.len() - 1) as f64 * p).round() as usize];
        let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let after = |ms: Option<f64>| match ms {
            Some(ms) => format!("{ms:.0} ms"),
            None => "not yet".to_owned(),
        };
        let report = format!(
            "{} ({:?}), view {}x{} pixels, {drawn} models drawn, {} frames, mean {mean:.3} ms ({:.0} fps), p50 {:.3}, p95 {:.3}, p99 {:.3}, max {:.3} ms, scene {:.1} MiB on the GPU, first picture after {}, every model in after {}, checkpoints for the whole demo after {}\n",
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
            after(run.picture),
            after(run.complete),
            after(run.indexed),
        );
        if let Err(e) = std::fs::write(&run.bench.out, report) {
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
        let loading = wanted(frame, &self.models)
            .into_iter()
            .any(|m| matches!(self.models.get(&m), None | Some(Slot::Loading)));
        // The sky the server names, once it is in or could not be read
        let sky = frame.sky.as_deref().is_none_or(|sky| {
            self.sky
                .as_deref()
                .is_some_and(|drawn| drawn.eq_ignore_ascii_case(sky))
        });
        frame.stamp == stamp && map.is_some() && self.map == map && !loading && sky
    }

    /// The files the view could not read, with the reasons, folded away
    /// under their count
    fn missing_files(&self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let sounds = self
            .library
            .as_ref()
            .map(|l| l.missing_sounds())
            .unwrap_or_default();
        let count = self.missing.len() + sounds.len();
        if count == 0 {
            return;
        }
        let title = catalog.text_with("viewer-missing", &[("count", (count as i64).into())]);
        let title = text::widget_text(ui.ctx(), &title, rtl, None, Look::Plain);
        egui::CollapsingHeader::new(title)
            .id_salt("viewer-missing")
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("viewer-missing-list")
                    .max_height(120.)
                    .show(ui, |ui| {
                        let files = self.missing.iter().map(|(n, r)| (n.as_str(), r.as_str()));
                        let sounds = sounds.iter().map(|(n, r)| (n.as_str(), r.as_str()));
                        for (name, reason) in files.chain(sounds) {
                            // File names and reasons read left to right
                            let line = format!("{name} · {reason}");
                            ui.label(text::widget_text(ui.ctx(), &line, false, None, Look::Weak));
                        }
                    });
            });
    }

    /// The map the frame is on, and why, when it could not be loaded
    fn failed_map(&self) -> Option<&(String, String)> {
        let map = self.frame.as_ref()?.map.as_deref().map(map_name)?;
        self.map_failed.as_ref().filter(|(name, _)| *name == map)
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
                if let Some((map, _)) = self.failed_map() {
                    eprintln!("capture: the map {map} could not be loaded");
                    return (Some(size), true);
                }
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
                // Nothing drawn yet, as before the first frame is painted,
                // is read again next frame rather than skipped
                let Some(image) = image else {
                    return (Some(size), false);
                };
                let time = capturing.capture.times[capturing.next];
                let path = capturing.capture.folder.join(format!("{time:09.3}.png"));
                if let Err(e) = image::save_buffer(
                    &path,
                    &image.pixels,
                    image.width,
                    image.height,
                    image::ColorType::Rgba8,
                ) {
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
        if let Some(sound) = &self.sound
            && sound.volume() != self.settings.volume
        {
            sound.set_volume(self.settings.volume);
        }
        self.shortcuts(ui.ctx());
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

        // The buttons and the time bar
        let controls_height = 2. * (ui.spacing().interact_size.y + ui.spacing().item_spacing.y);
        let height = match self.bench {
            Some(_) => ui.available_height() - controls_height,
            None => height,
        };
        let size = egui::vec2(ui.available_width(), height.max(1.));
        let (rect, clicked) = ui.allocate_exact_size(size, Sense::click());
        // A click on the view pauses, or plays on the way it went last,
        // rewinding again after a pause while rewinding, unless the rewind
        // reached the start
        if clicked.clicked() {
            let clock = self.controller.clock();
            let at_start = self.controller.time(self.wall()).0 <= 0.;
            let command = if clock.playing() {
                Command::Pause
            } else if clock.backward() && !at_start {
                Command::PlayBackward
            } else {
                Command::Play
            };
            self.command(command);
            ctx.request_repaint();
        }
        let mut drawn = 0;
        let mut drawn_at = None;
        if let Some((map, error)) = self.failed_map() {
            let message = catalog.text_with("viewer-map-failed", &[("map", map.clone().into())]);
            ui.put(rect, |ui: &mut egui::Ui| {
                ui.vertical_centered(|ui| {
                    text::label(ui, &message, rtl);
                    text::label(ui, error, rtl);
                })
                .response
            });
        } else {
            match (&self.frame, &self.map) {
                (Some(frame), Some(_)) => {
                    let bench_size = self.bench.as_ref().and_then(|b| b.bench.size);
                    let pixels = capture_size.or(bench_size).unwrap_or([
                        (rect.width() * ui.pixels_per_point()).round().max(1.) as u32,
                        (rect.height() * ui.pixels_per_point()).round().max(1.) as u32,
                    ]);
                    let draw = self.draw_list();
                    drawn_at = Some(frame.at.0);
                    drawn = draw.instances.len() + draw.sprites.len() + draw.viewmodel.len();
                    let callback = Frame {
                        pixels,
                        view: view(frame, pixels[0] as f32 / pixels[1] as f32, &self.settings),
                        draw,
                    };
                    ui.painter()
                        .add(egui_wgpu::Callback::new_paint_callback(rect, callback));
                    if let Some(countdown) = &frame.countdown {
                        draw_countdown(ui.painter(), rect, countdown);
                    }
                    if self.time_frame(pixels, drawn) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                _ => {
                    ui.put(rect, egui::Spinner::new());
                }
            }
        }
        self.missing_files(ui, catalog, rtl);
        if let Some(stopped) = self.frame.as_ref().and_then(|f| f.stopped.as_ref()) {
            let reason = catalog.message(stopped);
            text::label(
                ui,
                &catalog.text_with("viewer-stopped", &[("reason", reason.into())]),
                rtl,
            );
        }
        if let Some(at) = drawn_at {
            self.flashes_to = at;
            // How far the picture painted trails the playhead
            let now = self.controller.time(self.wall()).0;
            if let Some((_, lags)) = &mut self.video_lag {
                lags.push((at, (now - at) * 1000.));
            }
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
            Some(began) => {
                let over = began.elapsed().as_secs_f64() >= seconds;
                if over {
                    // Said once, as the window closes a frame later
                    self.play = None;
                    if let Some(sound) = &self.sound {
                        let underruns = sound.underruns();
                        eprintln!("sound: {underruns} underruns in {seconds} s of playing");
                    }
                }
                if over && let Some((path, lags)) = &self.video_lag {
                    let lines: Vec<String> = lags
                        .iter()
                        .map(|(at, lag)| format!("{at:.4} {lag:.2}"))
                        .collect();
                    let text = format!("# source_s picture_lag_ms\n{}\n", lines.join("\n"));
                    if let Err(e) = std::fs::write(path, text) {
                        eprintln!("viewer: {}: {e}", path.display());
                    }
                }
                over
            }
            None => false,
        }
    }

    /// Tell the sound mixer what the frame shown plays
    /// The keys that drive playback while no text is being typed. Space and
    /// L play forward or pause, J plays backward or pauses, K pauses, comma
    /// and period step one recorded group back or forward, and the left and
    /// right arrows seek five seconds. While a control has the keyboard's
    /// focus, space and the arrows are its own
    fn shortcuts(&mut self, ctx: &egui::Context) {
        // Typing goes to the text, other controls keep only space and the
        // arrows for themselves
        if ctx.text_edit_focused() {
            return;
        }
        let focused = ctx.memory(|m| m.focused().is_some());
        let clock = self.controller.clock();
        let forward = clock.playing() && !clock.backward();
        let backward = clock.playing() && clock.backward();
        let pressed = |key: egui::Key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
        let now = self.controller.time(self.wall()).0;
        let command = if self.controller.scrubbing() && pressed(egui::Key::Escape) {
            // Back to where the drag began
            self.scrub_cancelled = true;
            Some(Command::EndScrub { cancel: true })
        } else if (!focused && pressed(egui::Key::Space)) || pressed(egui::Key::L) {
            Some(if forward {
                Command::Pause
            } else {
                Command::Play
            })
        } else if pressed(egui::Key::J) {
            Some(if backward {
                Command::Pause
            } else {
                Command::PlayBackward
            })
        } else if pressed(egui::Key::K) {
            Some(Command::Pause)
        } else if pressed(egui::Key::Comma) {
            Some(Command::StepGroup(-1))
        } else if pressed(egui::Key::Period) {
            Some(Command::StepGroup(1))
        } else if !focused && pressed(egui::Key::ArrowLeft) {
            Some(Command::SeekTo(SourceTime((now - SEEK_STEP).max(0.))))
        } else if !focused && pressed(egui::Key::ArrowRight) {
            Some(Command::SeekTo(SourceTime(now + SEEK_STEP)))
        } else {
            None
        };
        if let Some(command) = command {
            self.command(command);
            ctx.request_repaint();
        }
    }

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
            // Backward playback is heard as nothing
            audible: frame.audible && !self.controller.clock().backward(),
            until: self.controller.until(),
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

        let clock_state = self.controller.clock();
        let forward = clock_state.playing() && !clock_state.backward();
        let backward = clock_state.playing() && clock_state.backward();
        let shortcut = |key: &str| {
            catalog.text_with(
                "viewer-shortcut",
                &[("key", FluentValue::from(key.to_owned()))],
            )
        };
        let space = catalog.text("key-space");
        let rate_label = |rate: Rate| {
            let digits = match rate {
                Rate::Quarter => 2,
                Rate::Half => 1,
                _ => 0,
            };
            let options = FluentNumberOptions {
                minimum_fraction_digits: Some(digits),
                ..Default::default()
            };
            let value = FluentValue::Number(FluentNumber::new(rate.factor(), options));
            catalog.text_with("viewer-rate", &[("rate", value)])
        };

        // The buttons wrap, and the time bar takes a row of its own
        layout::wrapping_row(ui, rtl, |ui| {
            let button = |ui: &mut Ui, id: &str, tip: String| {
                let label = text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
                ui.button(label).on_hover_text(tip).clicked()
            };
            let back = if backward {
                "viewer-pause"
            } else {
                "viewer-play-backward"
            };
            if button(ui, back, shortcut("J")) {
                self.command(if backward {
                    Command::Pause
                } else {
                    Command::PlayBackward
                });
            }
            if button(ui, "viewer-step-back", shortcut(",")) {
                self.command(Command::StepGroup(-1));
            }
            let play = if forward {
                "viewer-pause"
            } else {
                "viewer-play"
            };
            if button(ui, play, shortcut(&format!("{space}, L"))) {
                self.command(if forward {
                    Command::Pause
                } else {
                    Command::Play
                });
            }
            if button(ui, "viewer-step-forward", shortcut(".")) {
                self.command(Command::StepGroup(1));
            }
            let current = clock_state.rate();
            let mut chosen = current;
            egui::ComboBox::from_id_salt("viewer-rate")
                .width(64.)
                .selected_text(rate_label(current))
                .show_ui(ui, |ui| {
                    for rate in [
                        Rate::Quarter,
                        Rate::Half,
                        Rate::Normal,
                        Rate::Double,
                        Rate::Quadruple,
                    ] {
                        ui.selectable_value(&mut chosen, rate, rate_label(rate));
                    }
                });
            if chosen != current {
                self.command(Command::SetRate(chosen));
            }

            let shown = format!("{} / {}", clock(now), clock(end));
            ui.label(text::widget_text(&ctx, &shown, false, None, Look::Plain));

            // Players through walls, a setting the app saves
            let label = text::widget_text(
                &ctx,
                &catalog.text("viewer-see-through"),
                rtl,
                None,
                Look::Plain,
            );
            ui.toggle_value(&mut self.settings.see_through, label)
                .on_hover_text(catalog.text("settings-see-through-hint"));

            // Without a device, why there is no sound
            if let Some(e) = &self.sound_error {
                let label = text::widget_text(
                    &ctx,
                    &catalog.text("viewer-no-sound"),
                    rtl,
                    None,
                    Look::Weak,
                );
                ui.label(label).on_hover_text(e);
            }

            // The volume is a setting, which the app saves
            if self.sound.is_some() {
                let label =
                    text::widget_text(&ctx, &catalog.text("viewer-volume"), rtl, None, Look::Plain);
                ui.label(label);
                ui.spacing_mut().slider_width = 80.;
                ui.add(egui::Slider::new(&mut self.settings.volume, 0. ..=1.).show_value(false));
            }
        });
        let mut at = now;
        ui.spacing_mut().slider_width = ui.available_width().max(40.);
        let slider = egui::Slider::new(&mut at, 0. ..=end.max(0.001)).show_value(false);
        let r = ui.add(slider);
        // Dragging scrubs, a click seeks
        if r.changed() {
            if r.dragged() {
                self.scrub(at as f32);
            } else {
                self.command(Command::SeekTo(SourceTime(at)));
            }
        }
        if r.drag_stopped() {
            self.end_scrub();
        }
    }
}

/// One entity to draw, its model, what it holds and its pose
struct Drawn {
    id: EntityId,
    model: ModelRef,
    weapon: Option<ModelRef>,
    pose: EntityPose,
    /// A player's top colour, which teamplay sets by team
    top_color: Option<u8>,
    /// The render effect
    fx: i64,
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
fn drawn(frame: &PresentedFrame, models: &HashMap<ModelRef, Slot>) -> Vec<Drawn> {
    let viewer = frame.recorder.view_entity.unwrap_or(frame.local_entity);
    let observed =
        (frame.recorder.observer_mode == OBS_IN_EYE).then_some(frame.recorder.observer_target);
    let client_time = frame.client_time;

    let mut out = Vec::new();
    for e in frame.entities.iter() {
        let number = e.id.number;
        // Beams are drawn as strips, not as their sprite
        if e.beam
            || number == viewer
            || observed == Some(i64::from(number))
            || e.effects & NO_DRAW != 0
        {
            continue;
        }

        let player = frame.players.iter().find(|p| p.entity == number);
        let model = match (player.and_then(|p| p.model.as_deref()), e.model.as_deref()) {
            (Some(name), _) => {
                let colours = player.map_or([0; 2], |p| {
                    [p.top_color.unwrap_or(0), p.bottom_color.unwrap_or(0)]
                });
                let first = ModelRef::Player(name.to_owned(), colours, 0, 0);
                variant(models, first, e.body, e.skin)
            }
            (None, Some(path)) if path.ends_with(".mdl") || path.ends_with(".spr") => {
                studio_ref(models, path, e.body, e.skin)
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
        // A translucent one at amount 0 is not drawn, as the game skips it
        if mode != RenderMode::Normal && e.render_amount <= 0 {
            continue;
        }
        out.push(Drawn {
            top_color: player.and_then(|p| p.top_color),
            fx: e.render_fx,
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

/// Every model and sprite `frame` draws, entities, held weapons, the
/// viewmodel, muzzle flashes, temporary entities and beams, as the models
/// in `models` say their variations
fn wanted(frame: &PresentedFrame, models: &HashMap<ModelRef, Slot>) -> Vec<ModelRef> {
    let drawn = drawn(frame, models)
        .into_iter()
        .flat_map(|d| std::iter::once(d.model).chain(d.weapon));
    let flashes = frame.flashes.iter().map(|f| flash_sprite(f.sprite));
    let temp = frame.temp_entities.iter().filter_map(|e| {
        let path = e.model.as_deref()?;
        Some(studio_ref(models, path, i64::from(e.body), 0))
    });
    let beams = &frame.beams;
    let entity_beams = frame
        .entities
        .iter()
        .filter(|e| e.beam)
        .filter_map(|e| e.model.as_deref());
    let beam_sprites = beams
        .trails
        .iter()
        .map(|t| &*t.sprite)
        .chain(beams.beams.iter().map(|b| &*b.sprite))
        .chain(entity_beams)
        .map(|s| ModelRef::File(s.to_owned()));
    drawn
        .chain(viewmodel_ref(frame, models))
        .chain(flashes)
        .chain(temp)
        .chain(beam_sprites)
        .collect()
}

/// The viewmodel to draw, in the body its last animation gave
fn viewmodel_ref(frame: &PresentedFrame, models: &HashMap<ModelRef, Slot>) -> Option<ModelRef> {
    let (path, _) = viewmodel_pose(frame)?;
    let body = frame.viewmodel.map_or(0, |a| i64::from(a.body));
    Some(studio_ref(models, &path, body, 0))
}

/// The model to draw for a studio model in `body` and `skin`, the first
/// body and skin's own until the model is in and its variations known, as
/// a body or skin past them is the first's. Any other model is its file
fn studio_ref(models: &HashMap<ModelRef, Slot>, path: &str, body: i64, skin: i64) -> ModelRef {
    let file = ModelRef::File(path.to_owned());
    if !path.ends_with(".mdl") {
        return file;
    }
    variant(models, file, body, skin)
}

/// `first`, a studio model or player model in its first body and skin, in
/// `body` and `skin` instead once it is in. The body is reduced to the
/// submodels it selects and a skin past the model's families is the
/// first, so each look is read once. A player model that is not installed
/// takes them from the stock model it falls back to
fn variant(models: &HashMap<ModelRef, Slot>, first: ModelRef, body: i64, skin: i64) -> ModelRef {
    if body == 0 && skin == 0 {
        return first;
    }
    let Some(Slot::Ready(model, _)) = models.get(&first) else {
        return first;
    };
    let asset = &model.asset;
    let value = u32::try_from(body).unwrap_or(0);
    let body = asset
        .body_parts()
        .iter()
        .enumerate()
        .filter(|(_, part)| !part.models.is_empty())
        .map(|(p, part)| asset.submodel(p, value) * part.base)
        .sum::<usize>();
    let body = u32::try_from(body).unwrap_or(0);
    let skin = usize::try_from(skin)
        .ok()
        .filter(|&s| s < asset.skin_families().len())
        .unwrap_or(0);
    match (first, body, skin) {
        (first, 0, 0) => first,
        (ModelRef::File(path), body, skin) => ModelRef::Variant(path, body, skin),
        (ModelRef::Player(name, colours, ..), body, skin) => {
            ModelRef::Player(name, colours, body, skin)
        }
        (other, ..) => other,
    }
}

/// A beam the server draws as an entity (`R_BeamDrawCustomEntity`), from
/// its origin, or the entity its sequence names, to the point its angles
/// hold, or the entity its skin names, as its render mode's low four bits
/// say. Its width is its scale, its colour its render colour by its render
/// amount, its sprite frame its frame, and it scrolls at its animation
/// time's rate. With a body its points are pushed aside by noise made
/// again each frame, as lightning flickers (`R_DrawSegs`)
fn entity_beam(frame: &PresentedFrame, e: &Entity, sprite: (SpriteHandle, usize)) -> Streak {
    // An entity's place by the number in a sequence or skin's low 12 bits
    let place = |field: i64| {
        let number = u16::try_from(field & 0x0fff).ok()?;
        frame
            .entities
            .iter()
            .find(|o| o.id.number == number)
            .map(|o| o.origin)
    };
    let (start_entity, end_entity) = match e.render_mode & 0x0f {
        // An entity to a point, either end an entity when it names one
        1 => (e.sequence != 0, e.skin != 0),
        2 => (true, true),
        _ => (false, false),
    };
    let start = start_entity
        .then(|| place(e.sequence))
        .flatten()
        .unwrap_or(e.origin);
    let end = end_entity
        .then(|| place(e.skin))
        .flatten()
        .unwrap_or(e.angles);
    let brightness = e.render_amount.clamp(0, 255) as f32 / 255.;
    let color = e.render_color.map(|c| f32::from(c) / 255. * brightness);
    let v = (frame.client_time as f32 * e.animtime).fract();
    let seed = frame.client_time.to_bits() ^ u64::from(e.id.number).rotate_left(32);
    let noise = BeamNoise {
        amplitude: e.body as f32 / 100.,
        sine: e.render_mode & 0x10 != 0,
        seed,
        time: frame.client_time as f32,
    };
    let points = beam_points(start, end, e.scale, v, &noise, frame.recorder.angles)
        .into_iter()
        .map(|(position, v)| StreakPoint { position, v, color })
        .collect();
    Streak {
        sprite: Some(sprite),
        points,
        width: e.scale,
    }
}

/// How a beam's points are pushed aside
struct BeamNoise {
    /// The body over 100, none when 0
    amplitude: f32,
    /// A sine wave turned round the beam, not fractal noise
    sine: bool,
    seed: u64,
    time: f32,
}

/// The engine's noise table's divisions
const NOISE_DIVISIONS: usize = 64;

/// A beam's points from `start` to `end` with their texture coordinate
/// along it, from `v`, a straight beam its two ends. A noisy one has a
/// point every 4 units, or every 16 under half an amplitude, no closer than
/// its width allows and 64 at most, each pushed aside by the noise
/// (`R_DrawSegs`, the fractal noise across the view, the sine noise round
/// the beam)
fn beam_points(
    start: [f32; 3],
    end: [f32; 3],
    width: f32,
    v: f32,
    noise: &BeamNoise,
    view_angles: [f32; 3],
) -> Vec<([f32; 3], f32)> {
    let delta = [0, 1, 2].map(|k| end[k] - start[k]);
    let length = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
    if noise.amplitude == 0. || length < 0.1 {
        return vec![(start, v), (end, v + length * 0.01)];
    }
    let mut segments = if noise.amplitude >= 0.5 {
        length * 0.25 + 3.
    } else {
        length * 0.075 + 3.
    } as usize;
    let widest = width * 0.5 * 1.414;
    if length / ((segments.max(2) - 1) as f32) < widest {
        segments = ((length / widest) as usize + 1).max(2);
    }
    segments = segments.clamp(2, NOISE_DIVISIONS);
    if noise.sine {
        segments = segments.max(16);
    }
    let div = 1. / (segments - 1) as f32;
    let v_step = length * 0.01 * div;

    // The table, ends at 0, made again each frame
    let mut table = [0f32; NOISE_DIVISIONS + 1];
    if noise.sine {
        let step = std::f32::consts::PI / NOISE_DIVISIONS as f32;
        for (i, n) in table.iter_mut().take(NOISE_DIVISIONS).enumerate() {
            *n = (i as f32 * step).sin();
        }
    } else {
        let mut random = noise.seed | 1;
        fractal_noise(&mut table, &mut random);
    }
    let (scale, wave) = if noise.sine {
        (noise.amplitude * 100., segments as f32 * 0.1)
    } else {
        (noise.amplitude * length * 0.01 * 2., 0.)
    };

    let (forward, right, up) = lambda_slicer_render::camera::angle_vectors(view_angles);
    let dir = delta.map(|c| c / length);
    let cross = [
        forward[1] * dir[2] - forward[2] * dir[1],
        forward[2] * dir[0] - forward[0] * dir[2],
        forward[0] * dir[1] - forward[1] * dir[0],
    ];
    let size = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2])
        .sqrt()
        .max(1e-6);
    let across = cross.map(|c| c / size);

    (0..segments)
        .map(|i| {
            let fraction = i as f32 * div;
            let index = (i * (NOISE_DIVISIONS - 1)) / (segments - 1);
            let factor = table[index] * scale;
            let mut p = [0, 1, 2].map(|k| start[k] + fraction * delta[k]);
            if noise.sine {
                let (s, c) = (fraction * std::f32::consts::PI * wave + noise.time).sin_cos();
                p = [0, 1, 2].map(|k| p[k] + factor * s * up[k] + factor * c * right[k]);
            } else {
                p = [0, 1, 2].map(|k| p[k] + factor * across[k]);
            }
            (p, v + i as f32 * v_step)
        })
        .collect()
}

/// Midpoint noise between a table's two ends, half the span a step at
/// most, normalised to about one (`FracNoise`)
fn fractal_noise(table: &mut [f32], random: &mut u64) {
    let divs = table.len() - 1;
    if divs < 2 {
        return;
    }
    let half = divs / 2;
    // xorshift, any uniform numbers will do
    *random ^= *random << 13;
    *random ^= *random >> 7;
    *random ^= *random << 17;
    let unit = (*random >> 11) as f32 / (1u64 << 53) as f32;
    table[half] = (table[0] + table[divs]) * 0.5 + divs as f32 * (unit * 0.25 - 0.125);
    if half > 1 {
        fractal_noise(&mut table[half..], random);
        fractal_noise(&mut table[..=half], random);
    }
}

/// The file a sky is read from, as it is named in a list of missing files
fn sky_file(name: &str) -> String {
    format!("gfx/env/{name}*.tga")
}

/// The file a model is read from, as it is named in a list of missing
/// files
fn model_file(model: &ModelRef) -> String {
    match model {
        ModelRef::File(path) | ModelRef::Variant(path, ..) => path.clone(),
        ModelRef::Player(name, ..) => format!("models/player/{name}/{name}.mdl"),
    }
}

/// How much of a silhouette shows over what hides it
const SILHOUETTE_ALPHA: f32 = 0.6;

/// The colour other models than players show through walls in
const ITEM_TINT: [f32; 3] = [1., 0.9, 0.4];

/// A player's colour through walls, its top colour's hue at full
/// brightness, white without one
fn team_tint(top_color: Option<u8>) -> [f32; 3] {
    let Some(top) = top_color else {
        return [1.; 3];
    };
    // The hue around the wheel as the game's colours go, 0 to 255
    let h = f32::from(top) / 255. * 6.;
    let x = 1. - (h % 2. - 1.).abs();
    let (r, g, b) = match h as u32 {
        0 => (1., x, 0.),
        1 => (x, 1., 0.),
        2 => (0., 1., x),
        3 => (0., x, 1.),
        4 => (x, 0., 1.),
        _ => (1., 0., x),
    };
    // Kept a little pale, so it reads over dark and bright walls alike
    [r, g, b].map(|c| 0.25 + 0.75 * c)
}

/// A drawn model's silhouette, every vertex in `tint`
fn silhouette(instance: &Instance, tint: [f32; 3]) -> Instance {
    Instance {
        vertices: instance
            .vertices
            .iter()
            .map(|v| ModelVertex {
                light: tint,
                alpha: SILHOUETTE_ALPHA,
                ..*v
            })
            .collect(),
        ..instance.clone()
    }
}

/// Where `LAMBDA_SLICER_SOUND_SKEW` asks the sound's skew to be written, and
/// beside it the pictures' lag, in a timed play
fn skew_path() -> Option<PathBuf> {
    std::env::var_os("LAMBDA_SLICER_SOUND_SKEW").map(PathBuf::from)
}

/// Seconds the arrow keys move the playhead
const SEEK_STEP: f64 = 5.;

/// A tracer's colour, `tracerred`, `tracergreen` and `tracerblue` at their
/// defaults by `traceralpha`'s, and its width either side of its line
const TRACER_COLOR: [f32; 3] = [0.8 * 0.5, 0.8 * 0.5, 0.4 * 0.5];
const TRACER_WIDTH: f32 = 1.5;

/// How long a muzzle flash lasts, as the game's temporary entity does
/// (`R_MuzzleFlash`), though it is drawn at least once
const FLASH_LIFE: f64 = 0.01;

/// Whether the world hides a glow at `at` from the eye at `eye`, the line
/// between them stopping more than 8 units short of it. Brush entities and
/// models do not hide one here
fn hidden_glow(world: &Hull, eye: [f32; 3], at: [f32; 3]) -> bool {
    let trace = world.trace(eye, at);
    let d = [0, 1, 2].map(|k| at[k] - eye[k]);
    let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    (1. - trace.fraction) * distance > 8.
}

/// The model whose first attachment a beam from entity `number`'s gun
/// starts at, the viewmodel when the view is through that entity
/// (`R_BeamEntPoint` with an attachment)
fn gun_holder(frame: &PresentedFrame, number: u16) -> Option<Holder> {
    let camera = &frame.recorder;
    let viewer = camera.view_entity.unwrap_or(frame.local_entity);
    let through = if camera.observer_mode == OBS_IN_EYE {
        camera.observer_target == i64::from(number)
    } else {
        camera.observer_mode == 0 && viewer == number
    };
    if through {
        return Some(Holder::Viewmodel);
    }
    let e = frame.entities.iter().find(|e| e.id.number == number)?;
    Some(Holder::Entity(e.id))
}

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
fn view(frame: &PresentedFrame, aspect: f32, settings: &ViewerSettings) -> View {
    View {
        view_projection: view_projection(frame, aspect, settings.fov),
        eye: frame.recorder.origin,
        angles: frame.recorder.angles,
        time: frame.client_time as f32,
        light_styles: light_styles(frame),
        gamma: settings.gamma(),
    }
}

/// The recorder's view. The recorded field of view is across the screen's
/// width, as the game's is
fn view_projection(frame: &PresentedFrame, aspect: f32, fov: Option<f32>) -> Matrix {
    let aspect = aspect.max(0.01);
    // A field of view chosen in Settings, else the recorded one
    let fov_x = fov
        .or(frame.recorder.fov)
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

/// HL's default HUD colour, which AG's countdown is drawn in
const HUD_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 160, 0);

/// AG's countdown over the view where its client draws it, a line of text
/// seven of the game's text lines from the top and the number three lines
/// below, sized as on a 640 by 480 screen, where a line is a 32nd of the
/// height and the HUD's digits a 20th. The text is the game's own, in
/// English as AG writes it
fn draw_countdown(painter: &egui::Painter, rect: egui::Rect, countdown: &Countdown) {
    let line = rect.height() / 32.;
    let [first, second] = countdown.names.each_ref().map(|n| without_colours(n));
    let text = if countdown.seconds == WAITING {
        if first.is_empty() {
            "Waiting for players to get ready...".to_owned()
        } else {
            format!("The last round was won by {first}!")
        }
    } else if !first.is_empty() && !second.is_empty() {
        format!("{first} vs. {second}")
    } else {
        "The match is about to start.".to_owned()
    };

    let centre = rect.center().x;
    painter.text(
        egui::pos2(centre, rect.top() + 7. * line),
        egui::Align2::CENTER_TOP,
        text,
        egui::FontId::proportional(line),
        HUD_COLOR,
    );
    if countdown.seconds != WAITING {
        // A digit stands about seven tenths of its font's size
        let digits = rect.height() / 20. / 0.7;
        painter.text(
            egui::pos2(centre, rect.top() + 10. * line),
            egui::Align2::CENTER_TOP,
            countdown.seconds.to_string(),
            egui::FontId::proportional(digits),
            HUD_COLOR,
        );
    }
}

/// A player's name without the colour codes AG writes in names, a `^` and
/// a digit
fn without_colours(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_codes_leave_names_but_other_carets_stay() {
        assert_eq!(without_colours("^1red^0 ^7naz"), "red naz");
        assert_eq!(without_colours("a^b ^"), "a^b ^");
    }

    #[test]
    fn noisy_beams_keep_their_length_and_stay_near_their_line() {
        let (start, end) = ([0.; 3], [200., 0., 0.]);
        let still = BeamNoise {
            amplitude: 0.,
            sine: false,
            seed: 7,
            time: 0.,
        };
        assert_eq!(beam_points(start, end, 3., 0., &still, [0.; 3]).len(), 2);

        // Looking down at a beam along x, the noise pushes it along y
        let noisy = BeamNoise {
            amplitude: 0.6,
            ..still
        };
        let points = beam_points(start, end, 3., 0., &noisy, [90., 0., 0.]);
        // One every 4 units, 64 at most
        assert_eq!(points.len(), 53);
        // The start stays, the far end only moves aside, as the engine
        // reads the table's second last entry for it
        assert_eq!(points[0].0, start);
        assert!((points[52].0[0] - 200.).abs() < 1e-3);
        // Pushed no further than the amplitude times twice the length in
        // hundreds, times the table's reach, 64 over 8 for its middle and
        // as much again from its halves and quarters
        let reach = 0.6 * 200. * 0.02 * 64. / 4.;
        assert!(
            points
                .iter()
                .all(|(p, _)| p[1].abs() <= reach && p[2].abs() < 1e-3)
        );
        assert!(points.iter().any(|(p, _)| p[1].abs() > 0.01));
    }
}
