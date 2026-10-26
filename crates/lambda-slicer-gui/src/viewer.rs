//! The demo played in the demo view
//!
//! Each viewer keeps a playback controller and worker for its demo, and a
//! thread that reads the session's map and the studio models its entities
//! use through the game's search path. Every frame the worker accepted is
//! drawn from the recorder's view. Entities with a studio model are posed
//! from their recorded sequence, frame, frame rate and animation time, and
//! a player is drawn with the model its userinfo names. The entity the
//! recorder views from is left out, as the game leaves it out. Another
//! player can be followed instead, seen from their eyes while the demo
//! shows them and from the recorder's view while it does not.
//!
//! Sound plays on the default output device while playing forward, mixed
//! from a playback of its own that shares the worker's checkpoints and
//! heard from the recorder's view. Shown only once the Half-Life folder is
//! known, from the settings or from `--game`
//!
//! The seek bar shows a small picture of the moment under the pointer,
//! played to by a thread of its own that shares the worker's checkpoints
//! too, and drawn as the view would show it there. A moment on another
//! map is drawn once that map is read too

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use eframe::egui::{self, Sense, Ui};
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use eframe::wgpu;
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::SearchPath;
use lambda_slicer_assets::sprite::Sprite;
use lambda_slicer_assets::wad::Wad;
use lambda_slicer_assets::xenoxide::studio_pose::BoneTransform;
use lambda_slicer_audio::device::{Device, Feed};
use lambda_slicer_audio::library::Library;
use lambda_slicer_core::analysis::context::SessionMap;
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_playback::PresentedFrame;
use lambda_slicer_playback::animation::{Flash, Holder};
use lambda_slicer_playback::controller::{Command, Controller, LoopRange, Reply, Stamp};
use lambda_slicer_playback::hud::{Countdown, LeftOut, WAITING};
use lambda_slicer_playback::particles::{STREAK_COLOR, STREAK_WIDTH};
use lambda_slicer_playback::scene::{
    self, Entity, EntityId, HIGHEST_BOB, LOWEST_BOB, PlayerView, view_model_for, weapon_model_for,
};
use lambda_slicer_playback::tempents::{DynamicLight, Tint};
use lambda_slicer_playback::time::{BOOST, RATES, Rate, SourceTime};
use lambda_slicer_playback::trace::{Hull, Trace};
use lambda_slicer_playback::worker::Worker;
use lambda_slicer_render::Rgba;
use lambda_slicer_render::camera::{Camera, Matrix};
use lambda_slicer_render::gpu::{
    Blit, Brush, Dot, DrawList, Instance, MOST_LIGHTS, ModelHandle, RenderMode, Scene, Shell,
    SpriteHandle, SpriteInstance, Streak, StreakPoint, View, WorldLight,
};
use lambda_slicer_render::light::{LightTree, ModelLight};
use lambda_slicer_render::load::{
    self, EntityPose, GaitPose, LoadError, LoadedModel, PreviousPose,
};
use lambda_slicer_render::model::{Eye, ModelMesh, ModelVertex, shell_distance};
use lambda_slicer_render::world::{self, WorldMesh};

use fluent_bundle::FluentValue;
use fluent_bundle::types::{FluentNumber, FluentNumberOptions};

use crate::announcements;
use crate::chat::Chat;
use crate::content::GameFolders;
use crate::crosshair::Crosshairs;
use crate::decals::DecalCache;
use crate::free_camera::{FreeCamera, Held};
use crate::hud_sprites::HudSprites;
use crate::i18n::Catalog;
use crate::kill_feed::KillFeed;
use crate::layout;
use crate::preview::Previews;
use crate::settings::ViewerSettings;
use crate::shortcuts::{self, Action, Binding, Group, Shortcuts};
use crate::text::{self, Look};
use crate::theme;
use crate::timeline::Ranked;
use crate::vitals::HudLook;
use crate::widgets::{Icon, IconButton, PlayButton};

/// `EF_NODRAW`, an entity the game does not draw
const NO_DRAW: i64 = 128;

/// `kRenderFxGlowShell`, a model drawn again a little bigger in its
/// render colour, as AG's CTF flags and their carriers are
const GLOW_SHELL: i64 = 19;

/// The texture a glow shell reflects
const SHELL_SPRITE: &str = "sprites/shellchrome.spr";

/// The field of view when a recording names none
const DEFAULT_FOV: f32 = 90.;

/// `OBS_IN_EYE`, a spectator seeing through the eyes of the player it
/// observes
const OBS_IN_EYE: i64 = 4;

/// `OBS_ROAMING` and `OBS_MAP_FREE`, a spectator moving freely, following
/// no player
const OBS_ROAMING: i64 = 3;
const OBS_MAP_FREE: i64 = 5;

/// How far over a player's origin their eyes are, standing and crouched
/// (`DEFAULT_VIEWHEIGHT` and `VEC_DUCK_VIEW`)
const EYE_STANDING: f32 = 28.;
const EYE_CROUCHED: f32 = 12.;

/// How wide the list of players to follow is
const POV_MENU_WIDTH: f32 = 150.;

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

/// What playback was doing, carried to a viewer opened again for the same
/// demo, as when the game's files are found while it plays
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Playback {
    time: f64,
    rate: Rate,
    backward: bool,
    playing: bool,
    /// Where a played range pauses, apart from a loop's end
    stop_at: Option<f64>,
    loop_range: Option<LoopRange>,
}

impl Playback {
    /// The command that plays on as playback did, none when it was paused
    fn resume(&self) -> Option<Command> {
        if !self.playing {
            return None;
        }

        let command = match (self.stop_at, self.backward) {
            (_, true) => Command::PlayBackward,
            (Some(to), false) => Command::PlayRange {
                from: SourceTime(self.time),
                to: SourceTime(to),
            },
            (None, false) => Command::Play,
        };
        Some(command)
    }
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
    fn start(installs: GameFolders, game: String, ctx: egui::Context) -> Self {
        let (request_tx, request_rx) = mpsc::channel::<Asset>();
        let (loaded_tx, loaded_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("viewer assets".into())
            .spawn(move || {
                let limits = AssetLimits::default();
                let search =
                    SearchPath::across(&installs, &game, limits).map_err(|e| e.to_string());
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
    /// The seek bar's preview, once one was drawn
    preview_group: Option<wgpu::BindGroup>,
}

/// The seek bar's preview of the moment under the pointer
#[derive(Default)]
struct Preview {
    /// How many moments were asked for, numbering each ask
    asks: u64,
    /// The moment of the last ask
    asked: Option<f64>,
    /// The newest frame answered
    frame: Option<Box<PresentedFrame>>,
    /// Whether the view's next frame draws it again, as it is new or a
    /// model it shows has loaded since
    redraw: bool,
    /// The size the picture was last drawn at, in pixels, while it shows a
    /// moment of this hover
    drawn: Option<[u32; 2]>,
}

/// The map the seek bar's preview draws a moment of another map with, kept
/// from one hover to the next
#[derive(Default)]
struct PreviewMap {
    /// The map on the GPU for it, and its light for the models
    loaded: Option<(String, Arc<LightTree>)>,
    loading: Option<String>,
    /// The last map that could not be read for it
    failed: Option<String>,
}

impl PreviewMap {
    /// Whether `map` is still to be asked for
    fn wants(&self, map: &str) -> bool {
        let named = |name: &Option<String>| name.as_deref() == Some(map);
        let loaded = self.loaded.as_ref().is_some_and(|(name, _)| name == map);
        !loaded && !named(&self.loading) && !named(&self.failed)
    }
}

pub struct Viewer {
    /// The settings it shows and plays with, and the keys that drive it,
    /// the app's each frame
    pub settings: ViewerSettings,
    pub shortcuts: Shortcuts,
    /// Each session's map and when it began, from the analysis once it is
    /// in, to tell which map an intermission changes to
    pub sessions: Vec<SessionMap>,
    /// Each session's players, best scored at its end first, with when
    /// they play there, from the analysis once it is in, for the players
    /// to follow
    pub standings: BTreeMap<u32, Vec<Ranked>>,
    /// The game's files are there to draw and hear the demo with. Without
    /// them the demo still plays, so the timeline and clips work
    assets: bool,
    /// How playback goes on once the first frame of a viewer opened again
    /// is in
    resume: Option<Command>,
    /// The volume before muting, for unmuting to, kept for the session
    unmuted_volume: Option<f32>,
    /// Zoomed in to the settings' zoomed field of view, for the session
    zoomed: bool,
    /// Everything fully lit, to see into dark places, for the session
    pub fullbright: bool,
    /// A camera of the viewer's own, flown apart from the recording
    /// player's view, for the session
    free: Option<FreeCamera>,
    /// The player followed, by entity, seen from their eyes while the demo
    /// shows them. None for the recorder's own view
    pov: Option<u16>,
    /// The players an event asked to follow, the first the demo shows in
    /// the first frame answering the stamp, else the recorder
    pov_wanted: Option<(Stamp, [Option<u16>; 2])>,
    /// An event being played, followed through whom it names as they come
    /// into view, until it ends
    pov_played: Option<PlayedEvent>,
    /// The view chosen by hand before events chose for it, none for the
    /// recorder's, gone back to once a played event ends. Kept while events
    /// choose, and forgotten when a view is chosen by hand again
    own_pov: Option<Option<u16>>,
    /// Units behind the recording player's view a free camera starts once
    /// the first frame is in, turned this many degrees round them, for
    /// checking the picture from the command line
    pub free_behind: Option<(f32, f32)>,
    /// Where a free camera starts once the first frame is in, and the way
    /// it looks, for seeing again a view someone saw
    pub free_at: Option<([f32; 3], [f32; 3])>,
    /// The map's best player to follow once the demo's players are known,
    /// the first at 1, for seeing a player's view from the command line
    pub follow_at: Option<usize>,
    /// The view fills the whole screen, which the app follows
    pub fullscreen: bool,
    /// When the pointer last moved over the whole screen view, in egui's
    /// seconds, for its controls to hide a while after
    pointer_moved: f64,
    /// Where the playhead was at the last single click on the view, for a
    /// double click to go back to
    first_click: Option<f64>,
    /// The left button held down on the view
    press: Option<Press>,
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
    /// The size a capture run draws at this frame, set as the frame begins
    capture_size: Option<[u32; 2]>,
    /// What the view painted this frame, for the frame's end
    drawn: usize,
    drawn_at: Option<f64>,
    /// Seconds to play for, and when playing began
    play: Option<(f64, Option<Instant>)>,
    last_frame: Option<Instant>,
    /// The source time of the frame last drawn, so a muzzle flash fired
    /// since then is drawn at least once
    flashes_to: f64,
    /// The game's palette, for blood's colours
    palette: Vec<[u8; 3]>,
    decals: DecalCache,
    /// The game's files, for the map's hull that hides glows, the kill
    /// feed's icons and the crosshairs
    library: Option<Arc<Library>>,
    /// The kill feed's icons, the crosshairs and the chat's places, read
    /// from the game's files as needed
    kill_feed: KillFeed,
    crosshairs: Crosshairs,
    chat: Chat,
    /// The health, armour and ammo's sprites
    vitals: HudSprites,
    /// How wide the transport's parts were last drawn, for centring the
    /// play button under the picture
    transport_widths: TransportWidths,
    /// Maps, models, sprites and skies that could not be read, and why
    missing: std::collections::BTreeMap<String, String>,
    /// The thread that plays to the seek bar's previews, none without the
    /// game's files
    previews: Option<Previews>,
    preview: Preview,
    preview_map: PreviewMap,
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
    /// A viewer for `demo`, reading the game's files from the Half-Life
    /// folders in `install` in turn when there are any, and playing on as
    /// `carry` says a viewer before it did
    pub fn open(
        render_state: RenderState,
        install: Option<GameFolders>,
        demo: PathBuf,
        duration: f32,
        dev: Dev,
        carry: Option<Playback>,
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
                preview_group: None,
            };
            render_state
                .renderer
                .write()
                .callback_resources
                .insert(resources);
        }

        let repaint = ctx.clone();
        let game = game_dir(&demo);
        let assets = install.is_some();
        let library = install
            .as_ref()
            .zip(game.as_ref())
            .and_then(|(install, game)| {
                let search = SearchPath::across(install, game, AssetLimits::default()).ok()?;
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
        let previews = library.as_ref().map(|l| {
            let repaint = ctx.clone();
            Previews::spawn(l.game_files(), move || repaint.request_repaint())
        });
        let sound = library.filter(|_| !quiet).and_then(|l| {
            let tap = std::env::var_os("LAMBDA_SLICER_SOUND_TAP").map(PathBuf::from);
            Device::open(l, tap, skew_path())
                .inspect_err(|e| {
                    eprintln!("sound: {e}");
                    *sound_error.borrow_mut() = Some(e.clone());
                })
                .ok()
        });
        let loader = install
            .zip(game)
            .map(|(install, game)| Loader::start(install, game, ctx.clone()));
        let mut controller = Controller::default();
        if let Some(request) = controller.command(Command::Load(demo), 0.) {
            worker.send(request);
        }
        if let Some(request) =
            start.and_then(|t| controller.command(Command::SeekTo(SourceTime(t)), 0.))
        {
            worker.send(request);
        }

        // Where the viewer before stood, playing on once a frame is in
        let mut resume = None;
        if let Some(carry) = carry {
            let mut commands = vec![
                Command::SetRate(carry.rate),
                Command::SetLoop(carry.loop_range),
                Command::SeekTo(SourceTime(carry.time)),
            ];

            // Paused while playing backward, it still faces backward, so
            // playing on goes the same way. No time passes between the two
            if carry.backward && !carry.playing {
                commands.push(Command::PlayBackward);
                commands.push(Command::Pause);
            }

            for command in commands {
                if let Some(request) = controller.command(command, 0.) {
                    worker.send(request);
                }
            }
            resume = carry.resume();
        }

        Self {
            settings: ViewerSettings::default(),
            shortcuts: Shortcuts::default(),
            sessions: Vec::new(),
            standings: BTreeMap::new(),
            assets,
            resume,
            unmuted_volume: None,
            zoomed: false,
            fullbright: false,
            free: None,
            pov: None,
            pov_wanted: None,
            pov_played: None,
            own_pov: None,
            free_behind: None,
            free_at: None,
            follow_at: None,
            fullscreen: false,
            pointer_moved: 0.,
            first_click: None,
            press: None,
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
            capture_size: None,
            drawn: 0,
            drawn_at: None,
            last_frame: None,
            flashes_to: f64::NEG_INFINITY,
            palette,
            decals: DecalCache::new(decal_wad),
            library: glow_library,
            kill_feed: KillFeed::default(),
            crosshairs: Crosshairs::default(),
            chat: Chat::default(),
            vitals: HudSprites::default(),
            transport_widths: TransportWidths::default(),
            previews,
            preview: Preview::default(),
            preview_map: PreviewMap::default(),
        }
    }

    fn wall(&self) -> f64 {
        self.clock.elapsed().as_secs_f64()
    }

    /// Whether the game's files are there to draw and hear the demo with
    pub fn has_assets(&self) -> bool {
        self.assets
    }

    /// Each map's world from the game's files, none without them
    pub fn map_hulls(&self) -> Option<lambda_slicer_playback::MapHulls> {
        self.library.as_ref().map(|l| l.game_files().hulls)
    }

    /// What playback is doing, for a viewer opened again to go on from
    pub fn playback(&self) -> Playback {
        let clock = self.controller.clock();
        let loop_range = self.controller.loop_range();

        // An end to playing that is not the loop's is a played range's
        let loop_end = loop_range.map(|r| r.end);
        let stop_at = self
            .controller
            .until()
            .filter(|until| Some(*until) != loop_end)
            .map(|until| until.0);

        Playback {
            time: self.controller.time(self.wall()).0,
            rate: clock.rate(),
            backward: clock.backward(),
            playing: clock.playing(),
            stop_at,
            loop_range,
        }
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

    /// Whether playback is going on, forward or backward
    pub fn is_playing(&self) -> bool {
        self.controller.clock().playing()
    }

    /// Whether playback runs backward, playing or paused
    pub fn is_backward(&self) -> bool {
        self.controller.clock().backward()
    }

    /// Run `command`. Anything asked of playback after a viewer opened again
    /// replaces the playing on still waiting for its first frame, so it
    /// never undoes a later pause, step or seek
    fn command(&mut self, command: Command) {
        self.resume = None;

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
        self.settle_pov();
        self.follow_played();

        // Only the answer to the last ask is drawn
        if let Some(previews) = &self.previews {
            for answer in previews.answers() {
                if answer.number == self.preview.asks && self.preview.asked.is_some() {
                    self.preview.frame = Some(answer.frame);
                    self.preview.redraw = true;
                }
            }
        }

        // The player the command line asked to follow, once the analysis
        // has ranked the players
        if let Some(place) = self.follow_at
            && let Some(frame) = &self.frame
            && !self.followable(frame).is_empty()
        {
            self.follow_place(place.saturating_sub(1));
            self.follow_at = None;
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

                    // The preview's own map is the view's now, or soon will be
                    let previewed = self.preview_map.loaded.as_ref().map(|(map, _)| map);
                    if previewed == Some(&name) {
                        resources.scene.set_preview_map(device, queue, None);
                        self.preview_map.loaded = None;
                    }
                    if self.preview_map.loading.as_ref() == Some(&name) {
                        self.preview_map.loading = None;
                    }

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
                Loaded::Map(name, Ok(map)) if self.preview_map.loading.as_ref() == Some(&name) => {
                    let LoadedMap { mut world, sky } = *map;
                    let light = Arc::new(std::mem::take(&mut world.light));
                    let map = Some((&world, sky.as_ref().ok()));
                    resources.scene.set_preview_map(device, queue, map);
                    self.preview_map = PreviewMap {
                        loaded: Some((name, light)),
                        ..PreviewMap::default()
                    };
                    self.preview.redraw |= self.preview.frame.is_some();
                }
                Loaded::Map(name, Err(e)) if self.preview_map.loading.as_ref() == Some(&name) => {
                    eprintln!("viewer: preview's map {name}, {e}");
                    self.preview_map.loading = None;
                    self.preview_map.failed = Some(name);
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
                    self.preview.redraw |= self.preview.frame.is_some();
                }
                Loaded::Model(model, Outcome::Sprite(sprite)) => {
                    let handle = resources.scene.add_sprite(device, queue, &sprite);
                    self.models.insert(model, Slot::Sprite(handle));
                    self.preview.redraw |= self.preview.frame.is_some();
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

        // The map of the moment the seek bar previews, when it is another
        let previewed = self.preview.frame.as_ref().and_then(|f| f.map.as_deref());
        if let Some(map) = previewed.map(map_name)
            && self.map.as_ref() != Some(&map)
            && self.map_loading.as_ref() != Some(&map)
            && self.preview_map.wants(&map)
        {
            let _ = loader.requests.send(Asset::Map(map.clone()));
            self.preview_map.loading = Some(map);
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

        // The models of the moment the seek bar previews too
        let mut models = wanted(frame, &self.models, self.eyes(frame));
        if let Some(preview) = &self.preview.frame {
            models.extend(wanted(preview, &self.models, self.eyes(preview)));
        }
        for model in models {
            if let Entry::Vacant(slot) = self.models.entry(model) {
                let _ = loader.requests.send(Asset::Model(slot.key().clone()));
                slot.insert(Slot::Loading);
            }
        }
    }

    /// Whether the picture is seen from the free camera
    pub fn free_camera(&self) -> bool {
        self.free.is_some()
    }

    /// Leave the view shown for a free camera where that view is, or go
    /// back to it
    fn toggle_free(&mut self) {
        self.free = match (&self.free, &self.frame) {
            (None, Some(frame)) => {
                let (origin, angles) = self.eye(frame);
                Some(FreeCamera::new(origin, angles))
            }
            _ => None,
        };
    }

    /// Whose view the picture is, the free camera's, the followed player's
    /// while the demo shows them, or else the recorder's
    fn eyes(&self, frame: &PresentedFrame) -> Eyes {
        if self.free.is_some() {
            return Eyes::Free;
        }

        match self.pov.and_then(|number| followed(frame, number)) {
            Some(player) => Eyes::Player(player.id),
            None => Eyes::Recorder,
        }
    }

    /// Where the picture is seen from and the way it looks, the free
    /// camera's, the followed player's or the recorder's
    fn eye(&self, frame: &PresentedFrame) -> ([f32; 3], [f32; 3]) {
        if let Some(free) = &self.free {
            return (free.origin, free.angles);
        }

        match self.pov.and_then(|number| followed(frame, number)) {
            Some(player) => player_eye(player),
            None => (frame.recorder.origin, frame.recorder.angles),
        }
    }

    /// Follow the player at entity `number`, or go back to the recorder
    /// with none, leaving a free camera for their view
    fn follow(&mut self, number: Option<u16>) {
        self.pov = number;
        self.pov_wanted = None;
        self.pov_played = None;
        self.own_pov = None;
        self.free = None;
    }

    /// The slot of the player followed, none for the recorder's view
    pub fn followed_slot(&self) -> Option<u8> {
        let number = self.pov?.checked_sub(1)?;
        u8::try_from(number).ok()
    }

    /// Follow the player in `slot`, chosen beside the viewer, or go back to
    /// the recorder with none
    pub fn follow_slot(&mut self, slot: Option<u8>) {
        self.follow(slot.map(|s| u16::from(s) + 1));
    }

    /// Follow the first of `slots` the demo shows once the playhead has
    /// gone where it was just sent, else the recorder. A free camera stays,
    /// and leaving it goes to that view, unless the settings leave it now
    pub fn follow_first_seen(&mut self, slots: [Option<u8>; 2]) {
        self.keep_own_pov();
        let entities = slots.map(|slot| slot.map(|s| u16::from(s) + 1));
        self.pov_wanted = Some((self.controller.stamp(), entities));
        self.pov_played = None;
        if self.settings.events_leave_free {
            self.free = None;
        }
    }

    /// Follow the first of `slots` as they come into view while the event
    /// plays from `from` to `to`, the recorder's view until one does, and
    /// go back to the view chosen by hand before once it ends. Called once
    /// the event is sent playing
    pub fn follow_while_played(&mut self, slots: [Option<u8>; 2], (from, to): (f32, f32)) {
        self.keep_own_pov();
        self.pov_wanted = None;
        self.pov = None;
        self.pov_played = Some(PlayedEvent {
            stamp: self.controller.stamp(),
            wanted: slots.map(|slot| slot.map(|s| u16::from(s) + 1)),
            from: f64::from(from),
            to: f64::from(to),
            chosen: None,
        });
        if self.settings.events_leave_free {
            self.free = None;
        }
    }

    /// Keep the view chosen by hand before an event chooses one, unless an
    /// event already chose
    fn keep_own_pov(&mut self) {
        if self.own_pov.is_none() {
            self.own_pov = Some(self.pov);
        }
    }

    /// While an event plays, follow the first it names the demo shows once
    /// they come into view, never leaving one for one named after it, and
    /// go back to the view chosen by hand before once the playhead passes
    /// its end or leaves it
    fn follow_played(&mut self) {
        let (Some(played), Some(frame)) = (&mut self.pov_played, &self.frame) else {
            return;
        };
        if frame.stamp < played.stamp {
            return;
        }

        let at = frame.at.0;
        if at >= played.to - PLAYED_END || at < played.from - PLAYED_END {
            self.pov = self.own_pov.take().flatten();
            self.pov_played = None;
            return;
        }

        // The recorder among them is their own view, and always in it
        let before = played.chosen.unwrap_or(played.wanted.len());
        for (place, number) in played.wanted.iter().enumerate().take(before) {
            let Some(number) = *number else {
                continue;
            };
            if number == frame.local_entity {
                played.chosen = Some(place);
                self.pov = None;
                break;
            }
            if followed(frame, number).is_some() {
                played.chosen = Some(place);
                self.pov = Some(number);
                break;
            }
        }
    }

    /// Follow whom an event asked for, once the frame answering it is in.
    /// The recorder among them is their own view
    fn settle_pov(&mut self) {
        let (Some((stamp, wanted)), Some(frame)) = (self.pov_wanted, &self.frame) else {
            return;
        };
        if frame.stamp < stamp {
            return;
        }

        self.pov = None;
        for number in wanted.into_iter().flatten() {
            if number == frame.local_entity {
                break;
            }
            if followed(frame, number).is_some() {
                self.pov = Some(number);
                break;
            }
        }
        self.pov_wanted = None;
    }

    /// The players of the session shown, by the entity they play as now and
    /// by name, the best scored at its end first and the recorder left out.
    /// One not playing now, not there yet, gone or spectating, has no
    /// entity and cannot be followed. The names are as the demo has them
    /// now, or as the analysis last had them
    fn followable(&self, frame: &PresentedFrame) -> Vec<(Option<u16>, String)> {
        let segment = frame.position.map(|p| p.segment);
        let Some(ranked) = segment.and_then(|s| self.standings.get(&s)) else {
            return Vec::new();
        };
        let local = frame
            .players
            .iter()
            .find(|p| p.entity == frame.local_entity);
        let recorder = local.map(|p| p.name.as_str());

        let mut out = Vec::new();
        for player in ranked {
            let number = player.slot_at(frame.at.0 as f32).map(|s| u16::from(s) + 1);
            let own = match number {
                Some(number) => number == frame.local_entity,
                None => recorder == Some(player.name.as_str()),
            };
            if own {
                continue;
            }

            let now = number.and_then(|n| frame.players.iter().find(|p| p.entity == n));
            let name = now.map_or(player.name.as_str(), |p| p.name.as_str());
            out.push((number, text::without_colours(name)));
        }
        out
    }

    /// Follow the player `step` places after the one followed among the
    /// recorder and then the players playing now in score order, going
    /// round
    fn step_pov(&mut self, step: i32) {
        let Some(frame) = &self.frame else {
            return;
        };

        let mut order = vec![None];
        for (number, _) in self.followable(frame) {
            if number.is_some() {
                order.push(number);
            }
        }
        let at = order.iter().position(|&n| n == self.pov).unwrap_or(0);
        let count = order.len() as i32;
        let next = (at as i32 + step).rem_euclid(count);
        self.follow(order[next as usize]);
    }

    /// Follow the player at `place` in score order, the first at 0, when
    /// there is one and they play now
    fn follow_place(&mut self, place: usize) {
        let Some(frame) = &self.frame else {
            return;
        };

        if let Some(&(Some(number), _)) = self.followable(frame).get(place) {
            self.follow(Some(number));
        }
    }

    /// Fly the free camera by the keys held, and turn it by a drag with the
    /// right mouse button on the `view`, asking for the next frame while it
    /// moves. Keys being typed into a text field are left to it
    fn fly(&mut self, ctx: &egui::Context, view: &egui::Response) {
        if let Some((units, degrees)) = self.free_behind
            && let Some(frame) = &self.frame
        {
            let camera = &frame.recorder;
            let [pitch, yaw, _] = camera.angles;
            let mut free = FreeCamera::new(camera.origin, [pitch, yaw + degrees, 0.]);
            free.back_off(units);
            self.free = Some(free);
            self.free_behind = None;
        }
        if let Some((origin, angles)) = self.free_at.take() {
            self.free = Some(FreeCamera::new(origin, angles));
        }
        let Some(free) = &mut self.free else {
            return;
        };

        let typing = ctx.text_edit_focused();
        let shortcuts = &self.shortcuts;
        let (held, seconds, dragged) = ctx.input(|i| {
            let key = |action| !typing && shortcuts.held(i, action);
            let held = Held {
                forward: key(Action::FlyForward),
                back: key(Action::FlyBack),
                left: key(Action::FlyLeft),
                right: key(Action::FlyRight),
                up: key(Action::FlyUp),
                down: key(Action::FlyDown),
                fast: key(Action::FlyFast),
            };
            (held, i.stable_dt.min(LONGEST_FLIGHT), i.pointer.delta())
        });
        free.fly(held, seconds);

        let turning = view.dragged_by(egui::PointerButton::Secondary);
        if turning {
            free.turn([dragged.x, dragged.y]);
        }
        if held.moving() || turning {
            ctx.request_repaint();
        }
    }

    /// Each player drawn in the picture with its name over its head, as
    /// the picture's camera sees it, a free camera's too. A player the map
    /// hides wholly, its walls or its solid brush entities, has no name
    /// unless players are seen through walls, and nearer names are drawn
    /// over farther ones
    fn draw_names(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        frame: &PresentedFrame,
        aspect: f32,
    ) {
        let (eye, angles) = self.eye(frame);
        let matrix = view_projection(frame, (eye, angles), aspect, self.fov());
        let world = frame
            .map
            .as_deref()
            .and_then(|map| self.library.as_ref()?.hull(map));
        let (_, right, _) = lambda_slicer_render::camera::angle_vectors(angles);
        let solid = solid_brushes(frame);

        let mut names = Vec::new();
        for drawn in drawn(frame, &self.models, self.eyes(frame)) {
            if !drawn.pose.player {
                continue;
            }
            let Some(player) = frame.players.iter().find(|p| p.entity == drawn.id.number) else {
                continue;
            };
            let [x, y, z] = drawn.pose.origin;
            let head = [x, y, z + NAME_ABOVE];
            let hidden = world.as_ref().is_some_and(|w| {
                let body = body_points(drawn.pose.origin, right);
                !body.iter().any(|&p| seen(w, &solid, eye, p))
            });
            if hidden && !self.settings.see_through {
                continue;
            }
            let Some(at) = on_screen(&matrix, head, rect) else {
                continue;
            };
            let d = [0, 1, 2].map(|k| head[k] - eye[k]);
            let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            names.push((distance, at, player));
        }

        // The farthest first, so nearer names cover them
        names.sort_by(|a, b| b.0.total_cmp(&a.0));
        let size = rect.height() / NAME_LINES;
        for (_, at, player) in names {
            draw_name_over(painter, at, size, player);
        }
    }

    /// Everything drawn of `frame`, models posed and skinned and lit by the
    /// map's `light`. The muzzle flashes fired after `flashes_since` show
    /// however short, those before only while they last
    fn draw_list(
        &self,
        frame: &PresentedFrame,
        flashes_since: f64,
        light: Option<&LightTree>,
    ) -> DrawList {
        let styles = light_styles(frame);
        // What is under a model, brightened by the dynamic lights near it
        let lights = dynamic_lights(frame);
        let light_at = |point: [f32; 3]| {
            let under = light.and_then(|tree| tree.at(point, &styles));
            match under {
                Some(colour) => ModelLight::from_point(lit_by(colour, point, &lights)),
                None => ModelLight::UNLIT,
            }
        };
        // Chrome reflects the view the scene is drawn from
        let (eye_origin, eye_angles) = self.eye(frame);
        let eye = Eye::new(eye_origin, eye_angles);
        let eyes = self.eyes(frame);
        // A glow shell reflects a point turning around the world instead
        let shell_eye = Eye::shell(frame.client_time, eye.right);
        let shell_sprite = match self.models.get(&ModelRef::File(SHELL_SPRITE.to_owned())) {
            Some(Slot::Sprite(handle)) => Some((*handle, 0)),
            _ => None,
        };
        let shell = |look: Option<ShellLook>, mesh: &ModelMesh, bones: &[BoneTransform]| {
            let (look, sprite) = (look?, shell_sprite?);
            Some(Shell {
                sprite,
                color: look.color,
                vertices: mesh.shell(bones, &shell_eye, look.distance),
            })
        };
        let mut instances = Vec::new();
        let mut silhouettes = Vec::new();
        let mut shells = Vec::new();
        let mut sprites = Vec::new();
        let since = if flashes_since > frame.at.0 {
            f64::NEG_INFINITY
        } else {
            flashes_since
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
        for drawn in drawn(frame, &self.models, eyes) {
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
            shells.extend(shell(drawn.shell, &model.mesh, &bones));
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
                shells.extend(shell(drawn.shell, &weapon.mesh, &weapon_bones));
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
                previous: None,
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

        // The weapon in view is the recorder's or the followed player's,
        // whose muzzle flashes and beams then come from it. A free camera
        // shows none
        let mut viewmodel = Vec::new();
        let mut viewmodel_shells = Vec::new();
        if self.settings.viewmodel
            && let Some((_, pose)) = viewmodel_pose(frame, eyes, self.settings.weapon_bob)
            && let Some(Slot::Ready(model, handle)) = viewmodel_ref(frame, &self.models, eyes)
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
            let look = viewmodel_shell(frame, eyes);
            viewmodel_shells.extend(shell(look, &model.mesh, &bones));
            let holder = match eyes {
                Eyes::Player(id) => Holder::Entity(id),
                _ => Holder::Viewmodel,
            };
            place(holder, model, &bones);
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
            let eye = eye_origin;
            sprites.retain(|s| s.mode != RenderMode::Glow || !hidden_glow(&world, eye, s.origin));
        }

        DrawList {
            brushes: brushes(frame),
            instances,
            sprites,
            viewmodel,
            silhouettes,
            decals: self.decals.mesh(),
            dots: self.dots(frame),
            streaks: self.streaks(frame, &guns),
            shells,
            viewmodel_shells,
        }
    }

    /// The preview's newest moment to draw with the view's next frame, in
    /// the view's shape. None when there is no new one, or its map is not
    /// read yet, which leaves the picture without one until it is
    fn preview_drawing(&mut self, aspect: f32, pixels_per_point: f32) -> Option<PreviewDrawing> {
        if !self.preview.redraw {
            return None;
        }
        let frame = self.preview.frame.as_deref()?;
        self.preview.redraw = false;
        let map = frame.map.as_deref().map(map_name);
        let (light, other_map) = if map.is_some() && map == self.map {
            (self.light.clone(), false)
        } else {
            match &self.preview_map.loaded {
                Some((name, light)) if map.as_ref() == Some(name) => (Some(light.clone()), true),
                _ => {
                    self.preview.drawn = None;
                    return None;
                }
            }
        };

        let width = (PREVIEW_WIDTH * pixels_per_point).round().max(1.);
        let pixels = [width as u32, (width / aspect).round().max(1.) as u32];
        let view = View {
            fullbright: self.fullbright,
            ..view(frame, self.eye(frame), aspect, self.fov(), &self.settings)
        };
        // Its muzzle flashes only while they last, as no frame came before
        let mut draw = self.draw_list(frame, frame.at.0, light.as_deref());
        // The marks on the walls are the view's map's
        if other_map {
            draw.decals = None;
        }
        self.preview.drawn = Some(pixels);
        Some(PreviewDrawing {
            pixels,
            view,
            draw,
            other_map,
        })
    }

    /// Ask for the preview of the moment `time`, unless it was the last
    /// asked for
    fn ask_preview(&mut self, time: f64) {
        let (Some(previews), Some(demo)) = (&self.previews, self.worker.loaded()) else {
            return;
        };
        if self.preview.asked == Some(time) {
            return;
        }

        self.preview.asks += 1;
        self.preview.asked = Some(time);
        previews.ask(self.preview.asks, demo, SourceTime(time));
    }

    /// The picture of the moment under the pointer above the seek bar, with
    /// the moment's time under it, at `x` along the bar and kept within
    /// `track`. Only the time until the picture has been drawn
    fn preview_ui(&self, ui: &Ui, x: f32, track: egui::Rect, time: &str) {
        let width = PREVIEW_WIDTH + 2. * PREVIEW_MARGIN;
        let x = x.clamp(
            track.left() + width / 2.,
            (track.right() - width / 2.).max(track.left()),
        );
        let pixels_per_point = ui.pixels_per_point();
        egui::Area::new(egui::Id::unique("seek preview"))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .fixed_pos(egui::pos2(x, track.top() - PREVIEW_GAP))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(PREVIEW_MARGIN)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            if let Some([w, h]) = self.preview.drawn {
                                let size = egui::vec2(w as f32, h as f32) / pixels_per_point;
                                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                    rect,
                                    PreviewPicture,
                                ));
                            }
                            ui.monospace(time);
                        });
                    });
            });
    }

    /// The frame's particles as dots in the palette's colours
    fn dots(&self, frame: &PresentedFrame) -> Vec<Dot> {
        let mut out = Vec::new();
        if !self.settings.particles {
            return out;
        }
        for p in &frame.particles.dots {
            let Some(alpha) = p.alpha(frame.at) else {
                continue;
            };
            let [r, g, b] = self.tint(Tint::Palette(p.color));
            out.push(Dot {
                origin: p.origin,
                color: [r, g, b, alpha],
            });
        }
        out
    }

    /// The frame's beams, trails, tracers and sparks as strips to draw
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
                out.push(entity_beam(frame, e, sprite, self.eye(frame).1));
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
        if !self.settings.particles {
            return out;
        }
        let spark = STREAK_COLOR.map(|c| f32::from(c) / 255.);
        for streak in &frame.particles.streaks {
            let (tail, head) = streak.line(at);
            let color = spark.map(|c| c * streak.alpha);
            let point = |position, v| StreakPoint { position, v, color };
            out.push(Streak {
                sprite: None,
                points: vec![point(tail, 0.), point(head, 1.)],
                width: STREAK_WIDTH,
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
        let loading = wanted(frame, &self.models, self.eyes(frame))
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

    /// Why playback stopped early and how many files the view could not
    /// read, as notes over the view's bottom corner, so they take no room
    /// from the transport under it. The count opens the files and the
    /// reasons when clicked
    fn diagnostics(&self, ui: &mut Ui, rect: egui::Rect, catalog: &Catalog, rtl: bool) {
        let sounds = self
            .library
            .as_ref()
            .map(|l| l.missing_sounds())
            .unwrap_or_default();
        let count = self.missing.len() + sounds.len();
        let stopped = self.frame.as_ref().and_then(|f| f.stopped.as_ref());
        if count == 0 && stopped.is_none() {
            return;
        }

        let area = rect.shrink(theme::GAP_S);
        let corner = if rtl {
            egui::Align::Max
        } else {
            egui::Align::Min
        };
        let layout = egui::Layout::bottom_up(corner);
        let mut notes = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(layout));
        notes.set_clip_rect(rect);

        let note = egui::Frame::new()
            .fill(egui::Color32::from_black_alpha(NOTE_SHADE))
            .corner_radius(egui::CornerRadius::same(theme::RADIUS_SMALL))
            .inner_margin(egui::Margin::symmetric(theme::GAP_S as i8, NOTE_MARGIN_Y));
        let warning = Look::Color(theme::WARNING);

        if count > 0 {
            let title = catalog.text_with("viewer-missing", &[("count", (count as i64).into())]);
            let title = text::widget_text(notes.ctx(), &title, rtl, None, warning);
            let shown = note.show(&mut notes, |ui| {
                ui.add(egui::Label::new(title).sense(Sense::click()))
            });
            egui::Popup::from_toggle_button_response(&shown.inner).show(|ui| {
                egui::ScrollArea::vertical()
                    .id_salt("viewer-missing-list")
                    .max_height(MISSING_LIST_HEIGHT)
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

        if let Some(stopped) = stopped {
            let reason = catalog.message(stopped);
            let line = catalog.text_with("viewer-stopped", &[("reason", reason.into())]);
            let line = text::widget_text(notes.ctx(), &line, rtl, Some(area.width()), warning);
            note.show(&mut notes, |ui| ui.add(egui::Label::new(line).wrap()));
        }
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

    /// The name of the place nearest `at` on `map`, once the game's files
    /// are read, none before or on a map without places
    pub fn place_name(&mut self, map: &str, at: [f32; 3]) -> Option<String> {
        let library = self.library.clone()?;
        self.chat.place_name(&library, map, at)
    }

    /// Whether the game's files are read, so places can be named
    pub fn reads_game_files(&self) -> bool {
        self.library.is_some()
    }

    /// Take the worker's answers and the loaded assets, and step a capture
    /// run. Called once a frame before the view or its controls draw, so
    /// they all see the same playback state
    pub fn begin_frame(&mut self, ctx: &egui::Context) {
        if let Some(sound) = &self.sound
            && sound.volume() != self.settings.volume
        {
            sound.set_volume(self.settings.volume);
        }

        self.poll();
        self.request_assets();
        if let Some(frame) = &self.frame {
            let most = self.settings.decals;
            self.decals
                .update(&frame.decals, frame.decal_revision, most);
        }

        // A viewer opened again plays on once its demo has a frame
        if self.frame.is_some()
            && let Some(command) = self.resume.take()
        {
            self.command(command);
        }

        let (capture_size, captured) = self.step_capture();
        if captured {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if capture_size.is_some() {
            ctx.request_repaint();
        }
        self.capture_size = capture_size;

        // Nothing is painted yet this frame
        self.drawn = 0;
        self.drawn_at = None;
    }

    /// Paused where the demo ends, as playing to its end leaves it
    fn paused_at_end(&self) -> bool {
        if self.controller.clock().playing() {
            return false;
        }
        let Some(frame) = &self.frame else {
            return false;
        };

        let now = self.controller.time(self.wall());
        frame.end && now >= frame.at
    }

    /// Playing on the way it went last, rewinding again after a pause while
    /// rewinding, unless the rewind reached the start
    fn play_on(&self) -> Command {
        let at_start = self.controller.time(self.wall()).0 <= 0.;
        if self.controller.clock().backward() && !at_start {
            Command::PlayBackward
        } else {
            Command::Play
        }
    }

    /// Holding the left button on `view` plays twice as fast until it is
    /// let go, playing a paused demo for as long. Returns true as a hold is
    /// let go, which is then no click
    fn hold(&mut self, ctx: &egui::Context, view: &egui::Response) -> bool {
        let down = view.is_pointer_button_down_on() && ctx.input(|i| i.pointer.primary_down());
        let time = ctx.input(|i| i.time);

        let Some(press) = self.press else {
            if down {
                self.press = Some(Press {
                    since: time,
                    boost: None,
                });
                ctx.request_repaint_after_secs(HOLD_TO_BOOST as f32);
            }
            return false;
        };

        // Let go
        if !down {
            self.press = None;
            let Some(boost) = press.boost else {
                return false;
            };
            self.command(Command::Boost(false));
            if boost.resumed {
                self.command(Command::Pause);
            }
            ctx.request_repaint();
            return true;
        }

        // Held long enough
        let held = time - press.since;
        if press.boost.is_none() && held >= HOLD_TO_BOOST {
            let resumed = !self.controller.clock().playing();
            if resumed {
                self.command(self.play_on());
            }
            self.command(Command::Boost(true));
            self.press = Some(Press {
                boost: Some(Boost { resumed }),
                ..press
            });
            ctx.request_repaint();
        } else if press.boost.is_none() {
            ctx.request_repaint_after_secs((HOLD_TO_BOOST - held) as f32);
        }
        false
    }

    /// The view in `height` points, or in a bench run all the space left
    /// above the transport, with what it could not read and why playback
    /// stopped over its corner
    pub fn viewport_ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, height: f32) {
        let ctx = ui.ctx().clone();

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
        // Dragged with the right button, the view turns a free camera
        let (rect, clicked) = ui.allocate_exact_size(size, Sense::click_and_drag());
        self.fly(&ctx, &clicked);

        // The wheel's shortcuts count over the picture
        if clicked.hovered() {
            shortcuts::under_pointer(&ctx);
        }

        let let_go = self.hold(&ctx, &clicked);

        // A click on the view pauses, or plays on the way it went last,
        // unless it let go of a hold. Paused at the end, it plays again
        // from the start
        if clicked.clicked() && !let_go {
            let now = self.controller.time(self.wall()).0;
            if self.paused_at_end() {
                self.command(Command::SeekTo(SourceTime::ZERO));
            }
            let command = if self.controller.clock().playing() {
                Command::Pause
            } else {
                self.play_on()
            };
            let pause = matches!(command, Command::Pause);
            self.command(command);
            ctx.request_repaint();

            // A double click fills the screen or leaves it, its second click
            // undoing the first's pause or play as video players do. A
            // pause it ends on goes back to where the first click found the
            // playhead, as the moment between them played on
            if clicked.double_clicked() {
                self.fullscreen = !self.fullscreen;
                if pause && let Some(time) = self.first_click {
                    self.command(Command::SeekTo(SourceTime(time)));
                }
            } else {
                self.first_click = Some(now);
            }
        }

        let capture_size = self.capture_size;
        // The seek bar's preview in the view's shape
        let aspect = rect.width() / rect.height().max(1.);
        let preview = self.preview_drawing(aspect, ui.pixels_per_point());
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
                    let draw = self.draw_list(frame, self.flashes_to, self.light.as_deref());
                    drawn_at = Some(frame.at.0);
                    drawn = draw.instances.len() + draw.sprites.len() + draw.viewmodel.len();
                    let aspect = pixels[0] as f32 / pixels[1] as f32;
                    let callback = Frame {
                        pixels,
                        view: View {
                            fullbright: self.fullbright,
                            ..view(frame, self.eye(frame), aspect, self.fov(), &self.settings)
                        },
                        draw,
                        preview,
                    };
                    ui.painter()
                        .add(egui_wgpu::Callback::new_paint_callback(rect, callback));
                    if self.settings.player_names {
                        let aspect = pixels[0] as f32 / pixels[1] as f32;
                        self.draw_names(ui.painter(), rect, frame, aspect);
                    }
                    if self.settings.show_position {
                        let seen_from = self.eye(frame);
                        let colour = rgb(self.settings.text_color);
                        draw_position(ui.painter(), rect, catalog, seen_from, colour);
                    }
                    let hud = rgb(self.settings.hud_color);
                    if self.settings.recorder_fps
                        && let Some(fps) = frame.recorder.fps
                    {
                        draw_fps(ui.painter(), rect, catalog, fps, hud);
                    }
                    if let Some(countdown) = &frame.countdown {
                        draw_countdown(ui.painter(), rect, countdown, hud);
                    }
                    let announced = &frame.announced;
                    let settings = self
                        .settings
                        .server_settings
                        .then_some(self.settings.text_size);
                    announcements::draw(ui.painter(), rect, catalog, announced, settings, hud);
                    if let Some(next) = changing_to(&self.sessions, frame.at.0) {
                        let map = FluentValue::from(next.name.clone());
                        let line = catalog.text_with("viewer-map-change", &[("map", map)]);
                        draw_map_change(ui.painter(), rect, &line, hud);
                    }
                    let eyes = self.eyes(frame);
                    if let Some(library) = &self.library {
                        let weapon = match eyes {
                            Eyes::Recorder => weapon_in_view(frame).map(str::to_owned),
                            _ => viewmodel_pose(frame, eyes, Some(0.)).map(|(path, _)| path),
                        };
                        if self.settings.crosshair
                            && let Some(weapon) = weapon
                        {
                            let fov = self.fov().or(frame.recorder.fov);
                            self.crosshairs.draw(ui, rect, &weapon, fov, library);
                        }
                        // The game shows no health, armour, ammo, pickups,
                        // damage or weapon menu to a spectator or at
                        // intermission. A free camera has none of its own,
                        // unless the settings show the recorder's anyway. A
                        // followed player has none, as the demo only has
                        // the recorder's
                        let camera = &frame.recorder;
                        let free_shows = match eyes {
                            Eyes::Recorder => true,
                            Eyes::Free => self.settings.free_vitals,
                            Eyes::Player(_) => false,
                        };
                        let shown = !camera.spectating && !camera.intermission && free_shows;
                        if self.settings.vitals && shown {
                            let (vitals, status) = (&mut self.vitals, &frame.status);
                            let look = HudLook {
                                size: self.settings.vitals_size,
                                colour: hud,
                            };
                            let at = frame.at;
                            crate::vitals::draw(ui, vitals, library, rect, look, status, at);
                            let pickups = &frame.pickups;
                            crate::vitals::draw_pickups(
                                ui, vitals, library, rect, look, pickups, at,
                            );
                            let damage = &frame.damage;
                            let health = status.health.map(|h| h.value);
                            crate::vitals::draw_damage(
                                ui, vitals, library, rect, look, damage, health,
                            );
                            if let Some(menu) = &frame.weapon_menu {
                                crate::weapon_menu::draw(ui, vitals, library, rect, look, menu);
                            }
                        }
                        let size = self.settings.text_size;
                        let notices = &frame.death_notices;
                        self.kill_feed.draw(ui, rect, notices, library, size);
                        if self.settings.scores {
                            let most = self.settings.score_rows;
                            crate::mini_scores::draw(ui, rect, &frame.scores, most);
                        }
                        if self.settings.hud_messages {
                            crate::hud_messages::draw(ui, rect, &frame.hud_messages);
                            if let Some(title) = &frame.game_title {
                                let sprites = &mut self.vitals;
                                crate::hud_messages::draw_title(ui, sprites, library, rect, title);
                            }
                        }
                        if self.settings.chat {
                            let map = frame.map.as_deref();
                            let size = self.settings.text_size;
                            let colour = rgb(self.settings.text_color);
                            let lines = frame.chat(LeftOut {
                                places: !self.settings.chat_places,
                                long_jump: !self.settings.chat_long_jump,
                            });
                            self.chat.draw(ui, rect, lines, map, library, size, colour);
                        }
                    }
                    // The recorder's speeds as recorded while they play,
                    // and a followed player's as worked out from how they
                    // move. A free camera has none
                    let camera = &frame.recorder;
                    let playing = !camera.spectating && !camera.intermission;
                    let speeds = match eyes {
                        Eyes::Recorder if playing => Some(Speeds {
                            velocity: Some(camera.velocity),
                            last_jump: camera.last_jump,
                            estimated: false,
                        }),
                        Eyes::Player(id) => {
                            let movement = frame.movement.get(&id.number);
                            movement.map(|m| Speeds {
                                velocity: m.velocity,
                                last_jump: m.last_jump,
                                estimated: true,
                            })
                        }
                        _ => None,
                    };
                    if let Some(speeds) = &speeds {
                        draw_speeds(ui.painter(), rect, catalog, speeds, &self.settings);
                    }

                    // A free camera watches nobody, its names over the players
                    let players = &frame.players;
                    let shown = match eyes {
                        Eyes::Recorder => watched(&frame.recorder, frame.local_entity, players),
                        Eyes::Player(id) => players.iter().find(|p| p.entity == id.number),
                        Eyes::Free => None,
                    };
                    if let Some(player) = shown {
                        draw_watched(ui.painter(), rect, player);
                    }

                    // A player followed out of the demo's view is seen from
                    // the recorder's until they are back
                    if eyes == Eyes::Recorder
                        && let Some(number) = self.pov
                    {
                        let player = players.iter().find(|p| p.entity == number);
                        let name = player.map_or(String::new(), |p| text::without_colours(&p.name));
                        let name = FluentValue::from(name);
                        let line = catalog.text_with("viewer-pov-unseen", &[("name", name)]);
                        draw_unseen(ui.painter(), rect, &line, hud);
                    }

                    // The game shows the full scoreboard while its key is
                    // held and at intermission, over everything else. Only
                    // the recorder's keys are known, and a spectating
                    // recorder's can be left out
                    let held = eyes == Eyes::Recorder && frame.recorder.scores_held;
                    let spectating = frame.recorder.spectating && self.settings.scoreboard_playing;
                    let shows = self.settings.scoreboard.shows(self.fullscreen) && !spectating;
                    if shows && (held || frame.recorder.intermission) {
                        crate::scoreboard::draw(ui, rect, catalog, &frame.board);
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
        self.drawn = drawn;
        self.drawn_at = drawn_at;

        self.diagnostics(ui, rect, catalog, rtl);
    }

    /// The transport under the view, the play button under the picture's
    /// middle among the other playback buttons and the speed, the time
    /// under it, and the view and sound at the far end. It runs left to right in every language, as time and the
    /// play controls' directions do. Nothing while playback has failed.
    /// Returns true when the picture settings were asked for
    pub fn transport_ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> bool {
        if self.controller.failed().is_some() {
            return false;
        }

        // Placed by how wide everything was last drawn. The view controls
        // stay beside the buttons while the play button can still stand in
        // the middle, then join the time, and then take a row of their own
        let last = self.transport_widths;
        let mut now = last;
        let mut picture = false;
        let full = ui.available_width();
        let gap = ui.spacing().item_spacing.x;
        let view_start = full - last.view - gap;
        let centred = lead(full / 2. - last.play_middle, last.buttons, full);

        // The players to follow at the row's start, keeping the buttons
        // clear of them
        let buttons_lead = match self.assets {
            true => centred.max(last.pov + gap),
            false => centred,
        };
        let view_first = buttons_lead + last.buttons <= view_start;
        let view_second = !view_first && last.time <= view_start;

        // The buttons, the play button's middle under the picture's
        layout::row(ui, false, |ui| {
            now.pov = 0.;
            if self.assets {
                now.pov = self.pov_menu(ui, catalog, rtl);
                ui.add_space(buttons_lead - now.pov - gap);
            } else {
                ui.add_space(buttons_lead);
            }
            let group = ui.scope(|ui| self.playback_controls(ui, catalog, rtl));
            let group_left = group.response.rect.left();
            now.buttons = group.response.rect.width();
            now.play_middle = group.inner - group_left;
            if view_first {
                (picture, now.view) = self.view_group(ui, catalog, rtl);
            }
        });

        // The time under the play button, or short of the view controls
        // when they are in the way
        let end = if view_second { view_start } else { full };
        let play_middle = buttons_lead + last.play_middle;
        let time_lead = lead(play_middle - last.time / 2., last.time, end);
        layout::row(ui, false, |ui| {
            ui.add_space(time_lead);
            now.time = self.time_readout(ui);
            if view_second {
                (picture, now.view) = self.view_group(ui, catalog, rtl);
            }
        });

        if !view_first && !view_second {
            layout::row(ui, false, |ui| {
                (picture, now.view) = self.view_group(ui, catalog, rtl);
            });
        }

        // A frame drawn with other widths than these is drawn again before
        // it shows, so the controls never jump
        if !now.settled(&last) {
            self.transport_widths = now;
            ui.ctx().request_discard("transport widths");
        }

        picture
    }

    /// The list of whose view to watch, the recorder first and then the
    /// players of the session shown in score order, each with the number
    /// key that picks them. Choosing one leaves a free camera. Returns how
    /// wide it is
    fn pov_menu(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> f32 {
        let (players, recorder) = match &self.frame {
            Some(frame) => {
                let own = frame
                    .players
                    .iter()
                    .find(|p| p.entity == frame.local_entity);
                let name = own.map(|p| text::without_colours(&p.name));
                (self.followable(frame), name)
            }
            None => (Vec::new(), None),
        };
        let recorder = match recorder.filter(|n| !n.trim().is_empty()) {
            Some(name) => {
                let name = FluentValue::from(name);
                catalog.text_with("viewer-pov-recorder", &[("name", name)])
            }
            None => catalog.text("viewer-pov-recorder-unnamed"),
        };

        let followed = self
            .pov
            .and_then(|n| players.iter().find(|(p, _)| *p == Some(n)));
        let current = match followed {
            Some((_, name)) => name.clone(),
            None => recorder.clone(),
        };
        let mut chosen = self.pov;
        let selected = text::widget_text(ui.ctx(), &current, rtl, None, Look::Plain);
        let menu = egui::ComboBox::from_id_salt("viewer-pov")
            .width(POV_MENU_WIDTH)
            .truncate()
            .selected_text(selected)
            .show_ui(ui, |ui| {
                // Players not playing now keep their place and key, greyed
                let keyed = |action: Option<Action>, name: &str| {
                    let first = action.and_then(|a| self.shortcuts.of(a)[0]);
                    match first {
                        Some(binding) => format!("{}  {name}", binding.shown(catalog)),
                        None => format!("    {name}"),
                    }
                };
                let line = keyed(Some(Action::Recorder), &recorder);
                ui.selectable_value(&mut chosen, None, line);
                for (place, (number, name)) in players.iter().enumerate() {
                    let action = u8::try_from(place)
                        .ok()
                        .filter(|&p| p < shortcuts::FOLLOWED)
                        .map(Action::Follow);
                    let line = keyed(action, name);
                    match number {
                        Some(number) => {
                            ui.selectable_value(&mut chosen, Some(*number), line);
                        }
                        None => {
                            ui.add_enabled(false, egui::Button::selectable(false, line));
                        }
                    }
                }
            });
        let keys = |action| FluentValue::from(self.shortcuts.keys(action, catalog));
        let help = catalog.text_with(
            "viewer-pov-help",
            &[
                ("previous", keys(Action::PreviousPlayer)),
                ("next", keys(Action::NextPlayer)),
            ],
        );
        let width = menu.response.on_hover_text(help).rect.width();

        if chosen != self.pov {
            self.follow(chosen);
        }
        width
    }

    /// The view controls at the row's trailing end. Returns whether the
    /// picture settings were asked for, and how wide they are
    fn view_group(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> (bool, f32) {
        layout::trailing(ui, false, |ui| {
            let picture = self.view_controls(ui, catalog, rtl);
            (picture, ui.min_rect().width())
        })
    }

    /// An icon button that runs `command`, naming the keys of `action`
    fn command_button(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        icon: Icon,
        id: &str,
        action: Action,
        command: Command,
    ) {
        let label = self.shortcuts.with_keys(&catalog.text(id), action, catalog);
        if ui.add(IconButton::new(icon, &label)).clicked() {
            self.command(command);
        }
    }

    /// Back to the start, reverse, ten seconds back, a group back, play, a
    /// group forward, ten seconds forward and the speed. Returns where the
    /// play button's middle is across the row
    fn playback_controls(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> f32 {
        let clock = self.controller.clock();
        let forward = clock.playing() && !clock.backward();
        let backward = clock.playing() && clock.backward();
        let now = self.controller.time(self.wall()).0;

        self.command_button(
            ui,
            catalog,
            Icon::SkipStart,
            "viewer-start",
            Action::Start,
            Command::SeekTo(SourceTime::ZERO),
        );

        let (icon, id, command) = if backward {
            (Icon::Pause, "viewer-pause", Command::Pause)
        } else {
            (
                Icon::PlayBackward,
                "viewer-play-backward",
                Command::PlayBackward,
            )
        };
        self.command_button(ui, catalog, icon, id, Action::Rewind, command);

        self.command_button(
            ui,
            catalog,
            Icon::SkipBack,
            "viewer-skip-back",
            Action::SkipBack,
            Command::SeekTo(SourceTime((now - SKIP_STEP).max(0.))),
        );
        self.command_button(
            ui,
            catalog,
            Icon::StepBack,
            "viewer-step-back",
            Action::FrameBack,
            Command::StepGroup(-1),
        );

        // The main play button, filled in the accent
        let (id, command) = if forward {
            ("viewer-pause", Command::Pause)
        } else {
            ("viewer-play", Command::Play)
        };
        let label = self
            .shortcuts
            .with_keys(&catalog.text(id), Action::PlayPause, catalog);
        let play = ui.add(PlayButton::new(forward, &label));
        if play.clicked() {
            self.command(command);
        }

        self.command_button(
            ui,
            catalog,
            Icon::StepForward,
            "viewer-step-forward",
            Action::FrameForward,
            Command::StepGroup(1),
        );
        self.command_button(
            ui,
            catalog,
            Icon::SkipForward,
            "viewer-skip-forward",
            Action::SkipForward,
            Command::SeekTo(SourceTime(now + SKIP_STEP)),
        );

        self.rate_stepper(ui, catalog, rtl);
        play.rect.center().x
    }

    /// Learn the demo's length in seconds from its analysis, for a demo the
    /// library gave none, such as one opened from the file explorer
    pub fn learn_length(&mut self, seconds: f64) {
        self.duration = self.duration.max(seconds);
    }

    /// The demo's length as known so far, the library's figure or how far
    /// the worker has read, and never short of the playhead
    fn length(&self, now: f64) -> f64 {
        let indexed = self.frame.as_ref().map_or(0., |f| f.indexed.0.0);
        indexed.max(self.duration).max(now)
    }

    /// A bar along the bottom of the picture, as video players have. A
    /// click moves the playhead there and a drag scrubs, the same playhead
    /// the timeline shows. It reaches `reach` past the room it is given on
    /// each side, to the picture's edges, and runs left to right in every
    /// language, as time does
    pub fn seek_bar_ui(&mut self, ui: &mut Ui, catalog: &Catalog, reach: f32) {
        if self.controller.failed().is_some() {
            return;
        }

        let now = self.controller.time(self.wall()).0;
        let end = self.length(now).max(f64::EPSILON);
        let size = egui::vec2(ui.available_width(), SEEK_BAR_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let track = rect.expand2(egui::vec2(reach, 0.));
        let time_at = |x: f32| f64::from(((x - track.left()) / track.width()).clamp(0., 1.)) * end;
        response
            .widget_info(|| egui::WidgetInfo::slider(true, now, catalog.text("viewer-position")));

        // A drag scrubs, a click seeks
        if let Some(pointer) = response.interact_pointer_pos() {
            let at = time_at(pointer.x) as f32;
            if response.dragged() {
                self.scrub(at);
            } else if response.clicked() {
                self.seek(at);
            }
        }
        if response.drag_stopped() {
            self.end_scrub();
        }

        // Thicker with a knob while the pointer is on it, along the top so
        // it touches the picture
        let active = response.hovered() || response.dragged();
        let thickness = if active {
            SEEK_TRACK_ACTIVE
        } else {
            SEEK_TRACK
        };
        let bar = egui::Rect::from_min_size(track.min, egui::vec2(track.width(), thickness));
        let played_x = track.left() + track.width() * (now / end).clamp(0., 1.) as f32;
        let played = egui::Rect::from_min_max(bar.min, egui::pos2(played_x, bar.bottom()));
        let painter = ui.painter_at(track);
        painter.rect_filled(bar, 0., theme::RAISED);
        painter.rect_filled(played, 0., theme::ACCENT);
        if active {
            painter.circle_filled(
                egui::pos2(played_x, bar.center().y),
                SEEK_KNOB,
                theme::ACCENT,
            );
        }

        // The moment under the pointer and its picture, but while a drag
        // already shows it in the view
        match response.hover_pos() {
            Some(pointer) if !response.dragged() => {
                let at = time_at(pointer.x);
                self.ask_preview(at);
                self.preview_ui(ui, pointer.x, track, &clock(at, end > HOUR));
            }
            _ => {
                self.preview = Preview {
                    asks: self.preview.asks,
                    ..Preview::default()
                };
            }
        }
    }

    /// The playhead and the demo's length to the millisecond, the length
    /// fainter, with hours only for a demo over an hour. Returns how wide
    /// it is
    fn time_readout(&self, ui: &mut Ui) -> f32 {
        let now = self.controller.time(self.wall()).0;
        let end = self.length(now);
        let hours = end > HOUR;

        let mut job = egui::text::LayoutJob::default();
        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let format = |color| egui::TextFormat::simple(font.clone(), color);
        job.append(&clock(now, hours), 0., format(theme::FOREGROUND));
        let length = format!(" / {}", clock(end, hours));
        job.append(&length, 0., format(theme::FAINT));
        ui.label(job).rect.width()
    }

    /// The speed between a slower and a faster button, close together, the
    /// wheel over it stepping too and a click on it going back to the
    /// demo's own speed. While a hold plays faster, it shows that speed in
    /// the accent
    fn rate_stepper(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let clock = self.controller.clock();
        let rate = clock.rate();
        let with_key =
            |id: &str, action| self.shortcuts.with_keys(&catalog.text(id), action, catalog);

        let chosen = ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 0.;
            ui.spacing_mut().button_padding.x = 0.;
            let mut chosen = None;

            let slower = with_key("viewer-slower", Action::Slower);
            let button = IconButton::new(Icon::Minus, &slower).narrow();
            if ui.add_enabled(rate.slower().is_some(), button).clicked() {
                chosen = rate.slower();
            }

            let look = if clock.boosted() {
                Look::Color(theme::ACCENT)
            } else {
                Look::Plain
            };
            let shown = rate_label(catalog, clock.speed());
            let shown = text::widget_text(ui.ctx(), &shown, rtl, None, look);
            let size = egui::vec2(rate_width(ui, catalog), ui.spacing().interact_size.y);
            let speed = ui
                .add_sized(size, egui::Button::new(shown).frame(false))
                .on_hover_text(catalog.text("viewer-rate-normal"));
            if speed.clicked() {
                chosen = Some(Rate::NORMAL);
            }
            if speed.hovered() {
                let turned = wheel_turned(ui.ctx());
                if turned > 0. {
                    chosen = rate.faster();
                } else if turned < 0. {
                    chosen = rate.slower();
                }
            }

            let faster = with_key("viewer-faster", Action::Faster);
            let button = IconButton::new(Icon::Plus, &faster).narrow();
            if ui.add_enabled(rate.faster().is_some(), button).clicked() {
                chosen = rate.faster();
            }
            chosen
        });

        if let Some(chosen) = chosen.inner
            && chosen != rate
        {
            self.command(Command::SetRate(chosen));
        }
    }

    /// The whole screen, the picture settings, mute and the volume, see
    /// through walls and the zoom, placed from the row's trailing end. Sound
    /// and seeing through walls need the sound device and the game's files,
    /// and say so when they are not there. Returns true when the picture
    /// settings were asked for
    fn view_controls(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> bool {
        // The whole screen at the far end, as video players have it
        let (icon, id) = if self.fullscreen {
            (Icon::Shrink, "viewer-leave-fullscreen")
        } else {
            (Icon::Fit, "viewer-fullscreen")
        };
        let label = self
            .shortcuts
            .with_keys(&catalog.text(id), Action::Fullscreen, catalog);
        if ui.add(IconButton::new(icon, &label)).clicked() {
            self.fullscreen = !self.fullscreen;
        }

        let picture_label = catalog.text("viewer-picture");
        let picture = ui
            .add(IconButton::new(Icon::Picture, &picture_label))
            .clicked();

        // The volume is a setting, which the app saves, and muting keeps
        // the volume before it for unmuting to
        let why_silent = if !self.assets {
            Some(catalog.text("viewer-needs-files"))
        } else {
            self.sound_error.as_ref().map(|e| {
                let reason = FluentValue::from(e.clone());
                catalog.text_with("viewer-no-sound-why", &[("reason", reason)])
            })
        };
        let audible = why_silent.is_none();
        ui.add_enabled_ui(audible, |ui| {
            ui.spacing_mut().slider_width = VOLUME_SLIDER_WIDTH;
            let volume = egui::Slider::new(&mut self.settings.volume, 0. ..=1.).show_value(false);
            let response = ui.add(volume);
            response.widget_info(|| {
                egui::WidgetInfo::slider(
                    true,
                    f64::from(self.settings.volume),
                    catalog.text("viewer-volume"),
                )
            });

            let muted = self.settings.volume <= 0.;
            let (icon, id) = if muted {
                (Icon::Mute, "viewer-unmute")
            } else {
                (Icon::Volume, "viewer-mute")
            };
            let label = catalog.text(id);
            let mute = ui.add(IconButton::new(icon, &label));
            let mute = match &why_silent {
                Some(why) => mute.on_disabled_hover_text(why),
                None => mute,
            };
            if mute.clicked() {
                self.toggle_mute();
            }
        });

        // Players through walls, a setting the app saves
        let label = self.shortcuts.with_keys(
            &catalog.text("viewer-see-through"),
            Action::SeeThrough,
            catalog,
        );
        let button = IconButton::new(Icon::Eye, &label).selected(self.settings.see_through);
        let response = ui.add_enabled(self.assets, button);
        let response = response
            .on_hover_text(text::help(
                ui.ctx(),
                &catalog.text("settings-see-through-hint"),
                rtl,
            ))
            .on_disabled_hover_text(catalog.text("viewer-needs-files"));
        if response.clicked() {
            self.settings.see_through = !self.settings.see_through;
        }

        // Players' names over them, a setting the app saves
        let label =
            self.shortcuts
                .with_keys(&catalog.text("viewer-player-names"), Action::Names, catalog);
        let button = IconButton::new(Icon::Tag, &label).selected(self.settings.player_names);
        let response = ui.add_enabled(self.assets, button);
        let response = response
            .on_hover_text(text::help(
                ui.ctx(),
                &catalog.text("settings-player-names-hint"),
                rtl,
            ))
            .on_disabled_hover_text(catalog.text("viewer-needs-files"));
        if response.clicked() {
            self.settings.player_names = !self.settings.player_names;
        }

        // Everything fully lit, for this session only
        let label = self.shortcuts.with_keys(
            &catalog.text("viewer-fullbright"),
            Action::Fullbright,
            catalog,
        );
        let button = IconButton::new(Icon::Sun, &label).selected(self.fullbright);
        let response = ui.add_enabled(self.assets, button);
        let response = response
            .on_hover_text(text::help(
                ui.ctx(),
                &catalog.text("viewer-fullbright-help"),
                rtl,
            ))
            .on_disabled_hover_text(catalog.text("viewer-needs-files"));
        if response.clicked() {
            self.fullbright = !self.fullbright;
        }

        // A closer look, for this session only
        let label = self
            .shortcuts
            .with_keys(&catalog.text("viewer-zoom"), Action::Zoom, catalog);
        let zoom = IconButton::new(Icon::ZoomIn, &label).selected(self.zoomed);
        if ui.add(zoom).clicked() {
            self.zoomed = !self.zoomed;
        }

        // A camera of its own, flown about the map
        let label = self.shortcuts.with_keys(
            &catalog.text("viewer-free-camera"),
            Action::FreeCamera,
            catalog,
        );
        let free = IconButton::new(Icon::Move, &label).selected(self.free.is_some());
        let keys = |action| FluentValue::from(self.shortcuts.keys(action, catalog));
        let help = catalog.text_with(
            "viewer-free-camera-help",
            &[
                ("forward", keys(Action::FlyForward)),
                ("left", keys(Action::FlyLeft)),
                ("back", keys(Action::FlyBack)),
                ("right", keys(Action::FlyRight)),
                ("up", keys(Action::FlyUp)),
                ("down", keys(Action::FlyDown)),
                ("fast", keys(Action::FlyFast)),
            ],
        );
        let response = ui.add(free).on_hover_text(text::help(ui.ctx(), &help, rtl));
        if response.clicked() {
            self.toggle_free();
        }

        picture
    }

    /// Whether the controls over the whole screen view show, as they do
    /// while paused, under the pointer or a while after it moved. The
    /// pointer hides with them
    pub fn fullscreen_controls_shown(&mut self, ctx: &egui::Context, over: egui::Rect) -> bool {
        let (now, moved, pointer) = ctx.input(|i| {
            let moved = i.pointer.delta() != egui::Vec2::ZERO || i.pointer.any_pressed();
            (i.time, moved, i.pointer.hover_pos())
        });
        if moved {
            self.pointer_moved = now;
        }

        let paused = !self.controller.clock().playing();
        let over_them = pointer.is_some_and(|p| over.contains(p));
        let since = now - self.pointer_moved;
        if paused || over_them {
            return true;
        }

        // Drawn again when they are due to hide, as nothing else may ask
        if since < CONTROLS_LINGER {
            ctx.request_repaint_after_secs((CONTROLS_LINGER - since) as f32);
            return true;
        }

        ctx.set_cursor_icon(egui::CursorIcon::None);
        false
    }

    /// The field of view to draw with, zoomed in, chosen in Settings or
    /// none for the recorded one. A free camera never takes the recorded
    /// one, which narrows as the recorder zooms
    fn fov(&self) -> Option<f32> {
        if self.zoomed {
            return Some(self.settings.zoom_fov);
        }

        // A free camera or another player's view looks through nobody's
        // scope, so the recorder's zoom stays theirs
        let own_view = match &self.frame {
            Some(frame) => self.eyes(frame) == Eyes::Recorder,
            None => self.free.is_none(),
        };
        if !own_view {
            return Some(self.settings.fov.unwrap_or(DEFAULT_FOV));
        }
        self.settings.fov
    }

    /// Mute, keeping the volume for unmuting to, or unmute to the volume
    /// kept, the default volume when none was
    fn toggle_mute(&mut self) {
        if self.settings.volume > 0. {
            self.unmuted_volume = Some(self.settings.volume);
            self.settings.volume = 0.;
            return;
        }

        let volume = self
            .unmuted_volume
            .unwrap_or(ViewerSettings::default().volume);
        self.settings.volume = volume;
    }

    /// Feed the sound from the playback state every control has had its say
    /// on, and ask for the next frame while playing. Called once a frame
    /// after everything has drawn, whether or not the view was painted
    pub fn finish_frame(&mut self, ctx: &egui::Context) {
        if self.controller.failed().is_some() {
            return;
        }

        if let Some(at) = self.drawn_at {
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

        if self.controller.playing() || self.drawn > 0 && self.bench.is_some() {
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

    /// The keys that drive playback, the view and whose eyes it is, as the
    /// settings bind them, while no text is being typed. While a control
    /// has the keyboard's focus, Space, Enter, the arrows, Home and End are
    /// its own, and while the free camera flies its keys are. Escape gives
    /// up a scrub where it began, or else leaves the whole screen
    pub fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.text_edit_focused() {
            return;
        }
        let focused = ctx.memory(|m| m.focused().is_some());

        let escape = |ctx: &egui::Context| {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        };
        if self.controller.scrubbing() {
            if escape(ctx) {
                self.scrub_cancelled = true;
                self.command(Command::EndScrub { cancel: true });
                ctx.request_repaint();
            }
        } else if self.fullscreen && escape(ctx) {
            self.fullscreen = false;
            ctx.request_repaint();
        }

        // Seeing through walls, lighting everything, names and other
        // players' eyes need the game's files
        let mut actions = Vec::new();
        for action in Action::ALL {
            let ours = matches!(
                action.group(),
                Group::Playback | Group::View | Group::Players
            );
            if ours && (self.assets || !needs_files(action)) {
                actions.push(action);
            }
        }

        let flying = if self.free.is_some() {
            self.shortcuts.flying_inputs()
        } else {
            Vec::new()
        };
        let skip = |b: Binding| (focused && shortcuts::navigates(b)) || flying.contains(&b.input);
        for action in self.shortcuts.take(ctx, &actions, skip) {
            self.run_shortcut(action);
            ctx.request_repaint();
        }
    }

    /// Do what a shortcut's `action` asks
    fn run_shortcut(&mut self, action: Action) {
        let clock = self.controller.clock();
        let forward = clock.playing() && !clock.backward();
        let backward = clock.playing() && clock.backward();
        let rate = clock.rate();
        let now = self.controller.time(self.wall()).0;
        match action {
            Action::PlayPause if forward => self.command(Command::Pause),
            Action::PlayPause => self.command(Command::Play),
            Action::Rewind if backward => self.command(Command::Pause),
            Action::Rewind => self.command(Command::PlayBackward),
            Action::Pause => self.command(Command::Pause),
            Action::Start => self.command(Command::SeekTo(SourceTime::ZERO)),
            Action::SeekBack => {
                self.command(Command::SeekTo(SourceTime((now - SEEK_STEP).max(0.))))
            }
            Action::SeekForward => self.command(Command::SeekTo(SourceTime(now + SEEK_STEP))),
            Action::SkipBack => {
                self.command(Command::SeekTo(SourceTime((now - SKIP_STEP).max(0.))))
            }
            Action::SkipForward => self.command(Command::SeekTo(SourceTime(now + SKIP_STEP))),
            Action::FrameBack => self.command(Command::StepGroup(-1)),
            Action::FrameForward => self.command(Command::StepGroup(1)),
            Action::Slower => {
                if let Some(rate) = rate.slower() {
                    self.command(Command::SetRate(rate));
                }
            }
            Action::Faster => {
                if let Some(rate) = rate.faster() {
                    self.command(Command::SetRate(rate));
                }
            }
            Action::Fullscreen => self.fullscreen = !self.fullscreen,
            Action::Zoom => self.zoomed = !self.zoomed,
            Action::SeeThrough => self.settings.see_through = !self.settings.see_through,
            Action::Fullbright => self.fullbright = !self.fullbright,
            Action::Names => self.settings.player_names = !self.settings.player_names,
            Action::FreeCamera => self.toggle_free(),
            Action::Recorder => self.follow(None),
            Action::Follow(place) => self.follow_place(usize::from(place)),
            Action::PreviousPlayer => self.step_pov(-1),
            Action::NextPlayer => self.step_pov(1),
            _ => {}
        }
    }

    /// Tell the sound mixer what the frame shown plays, heard from the
    /// view shown
    fn feed_sound(&self) {
        let (Some(sound), Some(frame), Some(demo)) =
            (&self.sound, &self.frame, self.worker.loaded())
        else {
            return;
        };

        let ears = match self.eyes(frame) {
            Eyes::Recorder => None,
            _ => Some(self.eye(frame)),
        };
        sound.feed(Feed {
            stamp: self.controller.stamp(),
            demo,
            clock: self.controller.clock(),
            clock_base: self.clock,
            // Backward playback is heard as nothing
            audible: frame.audible && !self.controller.clock().backward(),
            until: self.controller.until(),
            ears,
        });
    }
}

/// A speed as the player shows it, with only the decimals it has
fn rate_label(catalog: &Catalog, factor: f64) -> String {
    let mut digits = 0;
    let mut scaled = factor;
    while digits < 2 && (scaled - scaled.round()).abs() > 1e-9 {
        digits += 1;
        scaled *= 10.;
    }

    let options = FluentNumberOptions {
        minimum_fraction_digits: Some(digits),
        ..Default::default()
    };
    let value = FluentValue::Number(FluentNumber::new(factor, options));
    catalog.text_with("viewer-rate", &[("rate", value)])
}

/// Room for the widest speed the player shows, a held one included, so
/// the buttons beside it stay put as it changes
fn rate_width(ui: &Ui, catalog: &Catalog) -> f32 {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let mut widest: f32 = 0.;
    for factor in RATES {
        for speed in [factor, factor * BOOST] {
            let label = rate_label(catalog, speed);
            let galley =
                ui.fonts_mut(|f| f.layout_no_wrap(label, font.clone(), egui::Color32::PLACEHOLDER));
            widest = widest.max(galley.size().x);
        }
    }
    widest + 2. * RATE_PADDING
}

/// How far the mouse wheel turned this frame, up positive, taken so it
/// scrolls nothing else
fn wheel_turned(ctx: &egui::Context) -> f32 {
    ctx.input_mut(|i| {
        let mut turned = 0.;
        for event in &i.events {
            if let egui::Event::MouseWheel { delta, .. } = event {
                turned += delta.y;
            }
        }
        if turned != 0. {
            i.smooth_scroll_delta = egui::Vec2::ZERO;
        }
        turned
    })
}

/// The left button held down on the view, since when in egui's seconds,
/// and the boost it started once held long enough
#[derive(Clone, Copy, Debug)]
struct Press {
    since: f64,
    boost: Option<Boost>,
}

/// A hold playing faster, and whether it set a paused demo playing, to
/// pause again when let go
#[derive(Clone, Copy, Debug)]
struct Boost {
    resumed: bool,
}

/// Whether a shortcut's `action` needs the game's files, to see through
/// walls, light everything, show names or follow other players
fn needs_files(action: Action) -> bool {
    matches!(
        action,
        Action::SeeThrough | Action::Fullbright | Action::Names
    ) || action.group() == Group::Players
}

/// How near its end a played event counts as over, as playing stops at
/// the last recorded moment before it
const PLAYED_END: f64 = 0.05;

/// An event being played and followed through whom it names
#[derive(Clone, Copy, Debug)]
struct PlayedEvent {
    /// The command that sent it playing, as frames before it are of
    /// another moment
    stamp: Stamp,
    /// Whom it names by entity, the first the best to follow
    wanted: [Option<u16>; 2],
    /// Where it plays from and to
    from: f64,
    to: f64,
    /// Which of `wanted` is followed, none before any came into view
    chosen: Option<usize>,
}

/// Whose view the picture is, which says whose model is left out and
/// whose weapon is in view
#[derive(Clone, Copy, Debug, PartialEq)]
enum Eyes {
    /// The recorder's, or the player they watch through the eyes of
    Recorder,
    /// A player followed, while the demo shows them
    Player(EntityId),
    /// A free camera's, which leaves nobody out and holds no weapon
    Free,
}

/// The player at entity `number` when the demo shows them playing, not
/// spectating, in the frame
fn followed(frame: &PresentedFrame, number: u16) -> Option<&Entity> {
    let player = frame.entities.iter().find(|e| e.id.number == number)?;
    let shown = player.effects & NO_DRAW == 0 && player.iuser1 == 0;
    let is_player = frame.players.iter().any(|p| p.entity == number);
    (shown && is_player).then_some(player)
}

/// Where a player's eyes are and the way they look, over their origin by
/// the standing or crouched view height. Their model leans a third of
/// their view's pitch the other way, which is turned back
fn player_eye(player: &Entity) -> ([f32; 3], [f32; 3]) {
    let height = if player.use_hull == 1 {
        EYE_CROUCHED
    } else {
        EYE_STANDING
    };
    let [x, y, z] = player.origin;
    let [pitch, yaw, _] = player.angles;
    ([x, y, z + height], [-3. * pitch, yaw, 0.])
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
    shell: Option<ShellLook>,
}

/// How an entity's glow shell looks, its colour 0 to 1 and how far it
/// stands out
#[derive(Clone, Copy, Debug, PartialEq)]
struct ShellLook {
    color: [f32; 3],
    distance: f32,
}

/// The glow shell `e` has, if any
fn shell_look(e: &Entity) -> Option<ShellLook> {
    if e.render_fx != GLOW_SHELL {
        return None;
    }
    Some(ShellLook {
        color: e.render_color.map(|c| f32::from(c) / 255.),
        distance: shell_distance(e.render_amount.clamp(0, 255) as f32),
    })
}

/// The glow shell of the weapon in view through `eyes`, the one the player
/// whose eyes they are has, as AG's client gives it (`StudioDrawModel`)
fn viewmodel_shell(frame: &PresentedFrame, eyes: Eyes) -> Option<ShellLook> {
    let number = match eyes {
        Eyes::Recorder => {
            let viewer = frame.recorder.view_entity.unwrap_or(frame.local_entity);
            if frame.recorder.observer_mode == OBS_IN_EYE {
                u16::try_from(frame.recorder.observer_target).ok()?
            } else {
                viewer
            }
        }
        Eyes::Player(id) => id.number,
        Eyes::Free => return None,
    };
    let holder = frame.entities.iter().find(|e| e.id.number == number)?;
    shell_look(holder)
}

/// Which model each drawn entity uses and how it is posed. Players use the
/// model their userinfo names, upright as the game draws them. The player
/// whose `eyes` the view is through is left out, the entity the recorder
/// views from or the one they spectate through the eyes of, or a player
/// followed. A free camera leaves nobody out. The server leaves the
/// recorder's own entity at nought, so it stands where the recorder's own
/// frames say they are
fn drawn(frame: &PresentedFrame, models: &HashMap<ModelRef, Slot>, eyes: Eyes) -> Vec<Drawn> {
    let viewer = frame.recorder.view_entity.unwrap_or(frame.local_entity);
    let observed =
        (frame.recorder.observer_mode == OBS_IN_EYE).then_some(frame.recorder.observer_target);
    let client_time = frame.client_time;

    let mut out = Vec::new();
    for e in frame.entities.iter() {
        let number = e.id.number;
        // Beams are drawn as strips, not as their sprite
        let seen_through = match eyes {
            Eyes::Recorder => number == viewer || observed == Some(i64::from(number)),
            Eyes::Player(id) => number == id.number,
            Eyes::Free => false,
        };
        if e.beam || seen_through || e.effects & NO_DRAW != 0 {
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
        // The recorder's own entity names no weapon, so theirs is the one
        // in their hand that their weapon in view stands for
        let own = if number == viewer {
            frame
                .recorder
                .viewmodel
                .as_deref()
                .and_then(weapon_model_for)
        } else {
            None
        };
        let weapon = e
            .weapon
            .as_deref()
            .or(own)
            .filter(|w| is_player && w.ends_with(".mdl"))
            .map(|w| ModelRef::File(w.to_owned()));
        // The sequence it changed from, unless the change came at no time
        let previous = e.change.filter(|c| c.at != 0.).map(|c| PreviousPose {
            sequence: c.sequence.max(0) as usize,
            cycle: f64::from(c.frame),
            elapsed: c.held_at - f64::from(c.animtime),
            framerate: f64::from(c.framerate),
            blending: c.blending.map(f64::from),
            since: client_time - f64::from(c.at),
        });
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
            shell: shell_look(e),
            pose: EntityPose {
                sequence: e.sequence.max(0) as usize,
                cycle: f64::from(e.frame),
                elapsed: client_time - f64::from(e.animtime),
                framerate: f64::from(e.framerate),
                controllers: e.controllers.map(f64::from),
                blending: e.blending.map(f64::from),
                origin: if number == viewer {
                    frame.recorder.player_origin
                } else {
                    e.origin
                },
                angles: e.angles,
                player: is_player,
                gait,
                previous,
            },
        });
    }
    out
}

/// Every model and sprite `frame` draws, entities, held weapons, the
/// viewmodel, muzzle flashes, temporary entities and beams, as the models
/// in `models` say their variations
fn wanted(frame: &PresentedFrame, models: &HashMap<ModelRef, Slot>, eyes: Eyes) -> Vec<ModelRef> {
    let drawn = drawn(frame, models, eyes);
    let shelled = drawn.iter().any(|d| d.shell.is_some()) || viewmodel_shell(frame, eyes).is_some();
    let shell = shelled.then(|| ModelRef::File(SHELL_SPRITE.to_owned()));
    let drawn = drawn
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
        .chain(viewmodel_ref(frame, models, eyes))
        .chain(shell)
        .chain(flashes)
        .chain(temp)
        .chain(beam_sprites)
        .collect()
}

/// The viewmodel to draw through `eyes`, the recorder's in the body its
/// last animation gave
fn viewmodel_ref(
    frame: &PresentedFrame,
    models: &HashMap<ModelRef, Slot>,
    eyes: Eyes,
) -> Option<ModelRef> {
    let (path, _) = viewmodel_pose(frame, eyes, Some(0.))?;
    let body = match eyes {
        Eyes::Recorder => frame.viewmodel.map_or(0, |a| i64::from(a.body)),
        _ => 0,
    };
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
fn entity_beam(
    frame: &PresentedFrame,
    e: &Entity,
    sprite: (SpriteHandle, usize),
    eye_angles: [f32; 3],
) -> Streak {
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
    let points = beam_points(start, end, e.scale, v, &noise, eye_angles)
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

/// How dark the notes over the view are behind their text, the room above
/// and below their text, and the most room the list of files the view could
/// not read takes
const NOTE_SHADE: u8 = 190;
const NOTE_MARGIN_Y: i8 = 2;
const MISSING_LIST_HEIGHT: f32 = 240.;

/// How far a control group's width may change before the transport is laid
/// out again
const WIDTH_SETTLED: f32 = 0.5;

/// A demo this many seconds long or shorter shows its times without hours
const HOUR: f64 = 3600.;

/// The seek bar's height, its track's thickness at rest and under the
/// pointer, and its knob's radius
const SEEK_BAR_HEIGHT: f32 = 12.;
const SEEK_TRACK: f32 = 4.;
const SEEK_TRACK_ACTIVE: f32 = 6.;
const SEEK_KNOB: f32 = 6.;

/// The seek bar's preview's width, the room around it and how far above
/// the bar it stands, in points
const PREVIEW_WIDTH: f32 = 240.;
const PREVIEW_MARGIN: f32 = 4.;
const PREVIEW_GAP: f32 = 8.;

/// Room either side of the speed between its buttons, and the volume
/// slider's width
const RATE_PADDING: f32 = 2.;
const VOLUME_SLIDER_WIDTH: f32 = 64.;

/// Seconds the left button is held on the view before it plays faster
const HOLD_TO_BOOST: f64 = 0.3;

/// The longest a free camera flies in one frame, in seconds, so a frame
/// that came late never throws it far
const LONGEST_FLIGHT: f32 = 0.1;

/// Seconds the arrow keys move the playhead, and the skip buttons or the
/// arrow keys with Shift
const SEEK_STEP: f64 = 5.;
const SKIP_STEP: f64 = 10.;

/// Seconds the controls over the whole screen view stay after the pointer
/// last moved
const CONTROLS_LINGER: f64 = 2.5;

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

/// The model of the recorder's first-person weapon, which its crosshair
/// goes with. None while dead, at intermission, or spectating other than
/// through a player's eyes
fn weapon_in_view(frame: &PresentedFrame) -> Option<&str> {
    let camera = &frame.recorder;
    let path = camera
        .viewmodel
        .as_deref()
        .filter(|p| p.ends_with(".mdl"))?;
    let observing = camera.observer_mode != 0 && camera.observer_mode != OBS_IN_EYE;
    if camera.health <= 0 || camera.intermission || observing {
        return None;
    }
    Some(path)
}

/// The first-person weapon seen through `eyes` and its pose, at the view's
/// origin and angles, the pitch turned the other way as the game gives it
/// to a viewmodel. The recorder's plays the last weapon animation from
/// when it started, and is none whenever [`weapon_in_view`] is. It takes
/// their aim without the kick the view has, a unit under the view, and
/// bobs by the `cl_bob` of `bob`, or as recorded for none
/// (`V_CalcNormalRefdef`). A followed player's is the
/// one in their hand, idle, as their animations are not in the demo. A
/// free camera has none
fn viewmodel_pose(
    frame: &PresentedFrame,
    eyes: Eyes,
    bob: Option<f32>,
) -> Option<(String, EntityPose)> {
    let (path, sequence, elapsed, (origin, angles)) = match eyes {
        Eyes::Recorder => {
            let camera = &frame.recorder;
            let path = weapon_in_view(frame)?;
            let animation = frame.viewmodel?;
            let sequence = animation.sequence.max(0) as usize;
            let elapsed = frame.at.0 - animation.started.0;
            let bob = match bob {
                Some(amount) => game_bob(amount, frame.at.0, camera.velocity),
                None => camera.bob,
            };
            (
                path,
                sequence,
                elapsed,
                bobbed(camera.origin, camera.aim, bob),
            )
        }
        Eyes::Player(id) => {
            let player = followed(frame, id.number)?;
            let path = player.weapon.as_deref().and_then(view_model_for)?;
            (path, 0, frame.client_time, player_eye(player))
        }
        Eyes::Free => return None,
    };

    let pose = EntityPose {
        sequence,
        cycle: 0.,
        elapsed,
        framerate: 1.,
        controllers: [127.; 4],
        blending: [0.; 2],
        origin,
        angles: [-angles[0], angles[1], angles[2]],
        player: false,
        gait: None,
        previous: None,
    };
    Some((path.to_owned(), pose))
}

/// The weapon in view's place for a view at `origin` aiming at `aim`, a
/// unit under the view, `bob` units of bob along the aim and tilted by it.
/// The bob's height is the view's own, already in `origin`
/// (`V_CalcNormalRefdef`)
fn bobbed(origin: [f32; 3], aim: [f32; 3], bob: f32) -> ([f32; 3], [f32; 3]) {
    let (forward, _, _) = lambda_slicer_render::camera::angle_vectors(aim);
    let mut at = [0, 1, 2].map(|k| origin[k] + bob * 0.4 * forward[k]);
    at[2] -= 1.;

    // The tilt is of the weapon's pitch, which is the aim's turned over
    let [pitch, yaw, roll] = aim;
    (at, [pitch + bob * 0.3, yaw - bob * 0.5, roll - bob])
}

/// The bob of a player moving at `velocity` at `time`, as a `cl_bob` of
/// `amount` and the game's default `cl_bobcycle` and `cl_bobup` make it. A
/// share of `amount` times the speed along the ground, rising for the first
/// half of each 0.8 s and falling for the rest (`V_CalcBob`)
fn game_bob(amount: f32, time: f64, velocity: [f32; 3]) -> f32 {
    use std::f64::consts::PI;
    let cycle = time.rem_euclid(BOB_CYCLE) / BOB_CYCLE;
    let cycle = if cycle < BOB_UP {
        PI * cycle / BOB_UP
    } else {
        PI + PI * (cycle - BOB_UP) / (1. - BOB_UP)
    };
    let [x, y, _] = velocity.map(f64::from);
    let bob = (x * x + y * y).sqrt() * f64::from(amount);
    let bob = bob * 0.3 + bob * 0.7 * cycle.sin();
    (bob as f32).clamp(LOWEST_BOB, HIGHEST_BOB)
}

/// The game's default `cl_bobcycle` and `cl_bobup`
const BOB_CYCLE: f64 = 0.8;
const BOB_UP: f64 = 0.5;

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
/// brightness at the client's time, across `fov` or the recorded field of
/// view when none
fn view(
    frame: &PresentedFrame,
    (eye, angles): ([f32; 3], [f32; 3]),
    aspect: f32,
    fov: Option<f32>,
    settings: &ViewerSettings,
) -> View {
    let (lights, light_count) = world_lights(frame, eye);
    View {
        view_projection: view_projection(frame, (eye, angles), aspect, fov),
        eye,
        angles,
        time: frame.client_time as f32,
        light_styles: light_styles(frame),
        gamma: settings.gamma(),
        fullbright: false,
        lights,
        light_count,
    }
}

/// `EF_` effects that make a dynamic light on their entity
const EF_MUZZLEFLASH: i64 = 2;
const EF_BRIGHTLIGHT: i64 = 4;
const EF_DIMLIGHT: i64 = 8;
const EF_LIGHT: i64 = 64;

/// How long a muzzle flash's light lasts, in seconds, how far it reaches
/// and how far ahead of its gun it is
const MUZZLE_LIGHT: f64 = 0.1;
const MUZZLE_RADIUS: f32 = 200.;
const MUZZLE_AHEAD: f32 = 18.;

/// Every dynamic light in `frame`, those messages made and those of the
/// entities' effects (`CL_LinkPacketEntities`). A muzzle flash the server
/// flags lights 18 units ahead of its entity and 16 up, and a bright or dim
/// light and an entity's own light shine from 16 up. The muzzle flashes a
/// weapon's animation shows make no light, as the game only draws their
/// sprite (`R_MuzzleFlash`). The game varies the radius a little at random,
/// which is left out. The recorder's own entity stands where their own
/// frames say they are
fn dynamic_lights(frame: &PresentedFrame) -> Vec<DynamicLight> {
    let mut out = frame.lights.clone();
    let ahead = |origin: [f32; 3], angles: [f32; 3]| {
        let (forward, _, _) = lambda_slicer_render::camera::angle_vectors(angles);
        [0, 1, 2].map(|k| origin[k] + forward[k] * MUZZLE_AHEAD)
    };

    for e in frame.entities.iter() {
        let origin = if e.id.number == frame.local_entity {
            frame.recorder.player_origin
        } else {
            e.origin
        };
        let [x, y, z] = origin;
        let raised = [x, y, z + 16.];
        let mut add = |origin, radius| {
            out.push(DynamicLight {
                origin,
                radius,
                color: [255; 3],
                models_only: false,
            });
        };

        // The flag stays on until the next update, which a flash outlasts
        // only for a moment
        let since = frame.client_time - f64::from(e.updated.0);
        if e.effects & EF_MUZZLEFLASH != 0 && since < MUZZLE_LIGHT {
            add(ahead(raised, [0., e.angles[1], 0.]), MUZZLE_RADIUS);
        }
        if e.effects & EF_BRIGHTLIGHT != 0 {
            add(raised, 400.);
        }
        if e.effects & EF_DIMLIGHT != 0 {
            add(raised, 200.);
        }
        if e.effects & EF_LIGHT != 0 {
            add(origin, 200.);
        }
    }
    out
}

/// The dynamic lights that light the world in `frame`, the nearest to
/// `eye` first when there are more than the renderer takes, and how many
fn world_lights(frame: &PresentedFrame, eye: [f32; 3]) -> ([WorldLight; MOST_LIGHTS], usize) {
    let mut lights: Vec<DynamicLight> = dynamic_lights(frame)
        .into_iter()
        .filter(|l| !l.models_only)
        .collect();
    let distance = |l: &DynamicLight| {
        let d = [0, 1, 2].map(|k| l.origin[k] - eye[k]);
        d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
    };
    lights.sort_by(|a, b| distance(a).total_cmp(&distance(b)));

    let mut out = [WorldLight::default(); MOST_LIGHTS];
    let count = lights.len().min(MOST_LIGHTS);
    for (slot, light) in out.iter_mut().zip(&lights) {
        *slot = WorldLight {
            origin: light.origin,
            radius: light.radius,
            color: light.color.map(f32::from),
        };
    }
    (out, count)
}

/// The light `colour` under a model at `point`, 0 to 255, with what each
/// of `lights` adds there, what is left of its radius after the way to it
/// in its colour (`R_StudioDynamicLight`)
fn lit_by(colour: [f32; 3], point: [f32; 3], lights: &[DynamicLight]) -> [f32; 3] {
    let mut out = colour;
    for light in lights {
        let d = [0, 1, 2].map(|k| point[k] - light.origin[k]);
        let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let left = light.radius - distance;
        if left > 0. {
            for (channel, c) in out.iter_mut().zip(light.color) {
                *channel += left * f32::from(c) / 255.;
            }
        }
    }
    out
}

/// The recorder's view. The recorded field of view is across the screen's
/// width, as the game's is
fn view_projection(
    frame: &PresentedFrame,
    (origin, angles): ([f32; 3], [f32; 3]),
    aspect: f32,
    fov: Option<f32>,
) -> Matrix {
    let aspect = aspect.max(0.01);
    // A field of view chosen in Settings, else the recorded one
    let fov_x = fov
        .or(frame.recorder.fov)
        .filter(|f| (1. ..179.).contains(f))
        .unwrap_or(DEFAULT_FOV);
    let fov_y = 2. * ((fov_x.to_radians() / 2.).tan() / aspect).atan();
    let camera = Camera {
        origin,
        angles,
        fov_y: fov_y.to_degrees(),
    };
    camera.view_projection(aspect, 4., 8192.)
}

/// One frame of the scene, drawn offscreen in `prepare` and shown in `paint`,
/// and the seek bar's preview drawn first when it has a new moment
struct Frame {
    pixels: [u32; 2],
    view: View,
    draw: DrawList,
    preview: Option<PreviewDrawing>,
}

/// A moment of the seek bar's preview, to draw at `pixels`, on the
/// preview's own map when `other_map`
struct PreviewDrawing {
    pixels: [u32; 2],
    view: View,
    draw: DrawList,
    other_map: bool,
}

/// The seek bar's preview as last drawn, shown in `paint`
struct PreviewPicture;

impl CallbackTrait for PreviewPicture {
    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(r) = resources.get::<Resources>()
            && let Some(group) = &r.preview_group
        {
            r.blit.paint(pass, group);
        }
    }
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

        // The preview is sent at once, before the view's frame sends its
        // corners through the same buffers
        if let Some(p) = &self.preview {
            let (resized, commands) =
                r.scene
                    .render_preview(device, queue, p.pixels, &p.view, &p.draw, p.other_map);
            queue.submit([commands]);
            if (resized || r.preview_group.is_none())
                && let Some(view) = r.scene.preview_view()
            {
                r.preview_group = Some(r.blit.bind_group(device, view));
            }
        }

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

/// HL's default HUD colour, which the full scoreboard keeps whatever the
/// HUD's colour, as the game's does
pub(crate) const HUD_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 160, 0);

/// How many of the game's text lines above the view's bottom the watched
/// player's name stands, and the band behind it
const NAME_FROM_BOTTOM: f32 = 1.5;
const NAME_BAND: egui::Color32 = egui::Color32::from_black_alpha(140);

/// How many of the game's text lines below the view's top the map change
/// stands, where AG's countdown does
const MAP_CHANGE_FROM_TOP: f32 = 7.;

/// AG's countdown over the view where its client draws it, a line of text
/// seven of the game's text lines from the top and the number three lines
/// below, sized as on a 640 by 480 screen, where a line is a 32nd of the
/// height and the HUD's digits a 20th. The text is the game's own, in
/// English as AG writes it
fn draw_countdown(
    painter: &egui::Painter,
    rect: egui::Rect,
    countdown: &Countdown,
    colour: egui::Color32,
) {
    let line = rect.height() / 32.;
    let [first, second] = countdown.names.each_ref().map(|n| text::without_colours(n));
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
        colour,
    );
    if countdown.seconds != WAITING {
        // A digit stands about seven tenths of its font's size
        let digits = rect.height() / 20. / 0.7;
        painter.text(
            egui::pos2(centre, rect.top() + 10. * line),
            egui::Align2::CENTER_TOP,
            countdown.seconds.to_string(),
            egui::FontId::proportional(digits),
            colour,
        );
    }
}

/// How wide the transport's parts were last drawn
#[derive(Clone, Copy, Default)]
struct TransportWidths {
    /// The list of players to follow, none without the game's files
    pov: f32,
    /// The playback buttons with the speed, and how far into them the play
    /// button's middle is
    buttons: f32,
    play_middle: f32,
    /// The time, and the view and sound controls
    time: f32,
    view: f32,
}

impl TransportWidths {
    /// Whether these are close enough to `other` to lay out the same
    fn settled(&self, other: &Self) -> bool {
        let pairs = [
            (self.pov, other.pov),
            (self.buttons, other.buttons),
            (self.play_middle, other.play_middle),
            (self.time, other.time),
            (self.view, other.view),
        ];
        pairs.iter().all(|(a, b)| (a - b).abs() <= WIDTH_SETTLED)
    }
}

/// Where a group `width` wide starts to stand at `wanted`, kept from the
/// row's start and from passing `end`
fn lead(wanted: f32, width: f32, end: f32) -> f32 {
    wanted.min(end - width).max(0.)
}

/// `seconds` as minutes, seconds and milliseconds, after hours when `hours`
fn clock(seconds: f64, hours: bool) -> String {
    // To the microsecond first, so 62 read as 61.9999 is still 62
    let micros = (seconds.max(0.) * 1e6).round() as u64;
    let ms = micros / 1000;
    let (s, millis) = (ms / 1000 % 60, ms % 1000);
    if hours {
        let (h, m) = (ms / 3_600_000, ms / 60_000 % 60);
        return format!("{h:02}:{m:02}:{s:02}.{millis:03}");
    }

    let m = ms / 60_000;
    format!("{m:02}:{s:02}.{millis:03}")
}

/// The player `camera` follows among `players`, the one a spectator
/// watches or the recording player while playing, by `local` when the view
/// names none. None while a spectator roams freely
fn watched<'a>(
    camera: &scene::Camera,
    local: u16,
    players: &'a [PlayerView],
) -> Option<&'a PlayerView> {
    let entity = match camera.observer_mode {
        0 => camera.view_entity.unwrap_or(local),
        OBS_ROAMING | OBS_MAP_FREE => return None,
        _ => u16::try_from(camera.observer_target).ok()?,
    };
    players.iter().find(|p| p.entity == entity)
}

/// The watched player's name low in the middle of the view, in their team's
/// colour, on a dark band so it reads over any picture. It is sized as the
/// countdown's text is, a 32nd of the view's height
fn draw_watched(painter: &egui::Painter, rect: egui::Rect, player: &PlayerView) {
    let name = text::without_colours(&player.name);
    if name.trim().is_empty() {
        return;
    }

    let line = rect.height() / 32.;
    let colour = theme::team_colour(player.model.as_deref().unwrap_or_default());
    let galley = painter.layout_no_wrap(name, egui::FontId::proportional(line), colour);
    let size = galley.size();
    let at = egui::pos2(
        rect.center().x - size.x / 2.,
        rect.bottom() - NAME_FROM_BOTTOM * line - size.y,
    );
    let band = egui::Rect::from_min_size(at, size).expand2(egui::vec2(line / 2., line / 5.));
    painter.rect_filled(band, theme::RADIUS_SMALL, NAME_BAND);
    painter.galley(at, galley, colour);
}

/// How far over a player's origin their name is, in units, just over the
/// head of a player standing
const NAME_ABOVE: f32 = 44.;

/// How many lines of names the picture's height holds, which sets their
/// size
const NAME_LINES: f32 = 44.;

/// How far from a player's origin the points looked for stand, up and down
/// to the chest and the shins and out to the shoulders, in units
const BODY_HEIGHT: f32 = 24.;
const BODY_WIDTH: f32 = 12.;

/// Points over a player standing at `origin`, at the chest, the middle and
/// the shins, each in the middle and out to either side along the
/// camera's `right`, so a player partly in view is seen
fn body_points(origin: [f32; 3], right: [f32; 3]) -> Vec<[f32; 3]> {
    let mut points = Vec::new();
    for up in [BODY_HEIGHT, 0., -BODY_HEIGHT] {
        for side in [0., BODY_WIDTH, -BODY_WIDTH] {
            let point = [0, 1, 2].map(|k| origin[k] + right[k] * side);
            points.push([point[0], point[1], point[2] + up]);
        }
    }
    points
}

/// The brush entities drawn solid, which hide what is behind them, each
/// as its model's number in the map and where it stands. One drawn
/// see-through, such as a grate or a railing in alpha mode, hides nothing
fn solid_brushes(frame: &PresentedFrame) -> Vec<(usize, [f32; 3])> {
    let mut out = Vec::new();
    for b in brushes(frame) {
        if b.model != 0 && b.mode == RenderMode::Normal {
            out.push((b.model, b.origin));
        }
    }
    out
}

/// Whether the line from `eye` to `at` reaches it, the world and each of
/// the `solid` brush entities, unturned, letting it through. A line
/// stopping 8 units short or less still reaches it, as for a glow. A free
/// camera inside a wall sees nothing past it, so that wall hides nothing
fn seen(world: &Hull, solid: &[(usize, [f32; 3])], eye: [f32; 3], at: [f32; 3]) -> bool {
    let d = [0, 1, 2].map(|k| at[k] - eye[k]);
    let distance = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let blocked = |trace: Trace| !trace.start_solid && (1. - trace.fraction) * distance > 8.;

    if blocked(world.trace(eye, at)) {
        return false;
    }
    for &(model, origin) in solid {
        let from = [0, 1, 2].map(|k| eye[k] - origin[k]);
        let to = [0, 1, 2].map(|k| at[k] - origin[k]);
        if blocked(world.trace_model(model, from, to)) {
            return false;
        }
    }
    true
}

/// Where `point` is on the picture in `rect` through `matrix`, none behind
/// the camera or off the picture
fn on_screen(matrix: &Matrix, point: [f32; 3], rect: egui::Rect) -> Option<egui::Pos2> {
    // Column-major, so each column is weighed by one of the point's parts
    let mut clip = [0.; 4];
    let parts = [point[0], point[1], point[2], 1.];
    for (column, part) in matrix.iter().zip(parts) {
        for row in 0..4 {
            clip[row] += column[row] * part;
        }
    }
    if clip[3] <= 0. {
        return None;
    }

    let (x, y) = (clip[0] / clip[3], clip[1] / clip[3]);
    let at = egui::pos2(
        rect.left() + (x + 1.) / 2. * rect.width(),
        rect.top() + (1. - y) / 2. * rect.height(),
    );
    rect.contains(at).then_some(at)
}

/// A player's name centred over `at`, in their team's colour on a dark
/// band as the watched player's name is
fn draw_name_over(painter: &egui::Painter, at: egui::Pos2, size: f32, player: &PlayerView) {
    let name = text::without_colours(&player.name);
    if name.trim().is_empty() {
        return;
    }

    let colour = theme::team_colour(player.model.as_deref().unwrap_or_default());
    let galley = painter.layout_no_wrap(name, egui::FontId::proportional(size), colour);
    let galley_size = galley.size();
    let corner = egui::pos2(at.x - galley_size.x / 2., at.y - galley_size.y);
    let band = egui::Rect::from_min_size(corner, galley_size);
    let band = band.expand2(egui::vec2(size / 3., size / 8.));
    painter.rect_filled(band, theme::RADIUS_SMALL, NAME_BAND);
    painter.galley(corner, galley, colour);
}

/// The map the server is changing to at `at`, from the intermission until
/// the next session begins. None outside an intermission and in the last
/// session
fn changing_to(sessions: &[SessionMap], at: f64) -> Option<&SessionMap> {
    let next = sessions.iter().position(|s| f64::from(s.time) > at)?;
    let current = sessions.get(next.checked_sub(1)?)?;
    let began = current.intermission?;
    (at >= f64::from(began)).then(|| &sessions[next])
}

/// How far down the picture the position starts, as a share of its
/// height, clear of the scores above and the chat below
const POSITION_FROM_TOP: f32 = 0.3;

/// The recorder's fps's size and its room from the top left corner, in
/// the game's rows, small enough to stay above the small scoreboard
const FPS_TEXT: f32 = 10.;
const FPS_MARGIN: f32 = 6.;

/// A colour the settings give, as red, green and blue
fn rgb([r, g, b]: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

/// Where the picture is seen from, `origin`, and the way it looks,
/// `angles`, in the game's units and degrees, on the left of the picture
/// in `colour`. They are what `--viewer-free-at` takes to see the same
/// view again
fn draw_position(
    painter: &egui::Painter,
    rect: egui::Rect,
    catalog: &Catalog,
    (origin, angles): ([f32; 3], [f32; 3]),
    colour: egui::Color32,
) {
    let number = |v: f32| FluentValue::from(format!("{v:.1}"));
    let position = catalog.text_with(
        "viewer-position-line",
        &[
            ("x", number(origin[0])),
            ("y", number(origin[1])),
            ("z", number(origin[2])),
        ],
    );
    let looking = catalog.text_with(
        "viewer-angles-line",
        &[("pitch", number(angles[0])), ("yaw", number(angles[1]))],
    );

    let size = rect.height() / NAME_LINES;
    let font = egui::FontId::monospace(size);
    let mut at = egui::pos2(
        rect.left() + size,
        rect.top() + rect.height() * POSITION_FROM_TOP,
    );
    for line in [position, looking] {
        // Left to right whatever the language, so the numbers keep the
        // order `--viewer-free-at` takes, and a right-to-left label still
        // reads as it should
        let line = text::visual_pieces(&line, false).concat();
        let galley = painter.layout_no_wrap(line, font.clone(), colour);
        let band = egui::Rect::from_min_size(at, galley.size());
        let band = band.expand2(egui::vec2(size / 3., size / 8.));
        painter.rect_filled(band, theme::RADIUS_SMALL, NAME_BAND);
        let height = galley.size().y;
        painter.galley(at, galley, colour);
        at.y += height + size / 3.;
    }
}

/// The recorder's frames a second in the top left corner, over the small
/// scoreboard, in `colour`
fn draw_fps(
    painter: &egui::Painter,
    rect: egui::Rect,
    catalog: &Catalog,
    fps: f32,
    colour: egui::Color32,
) {
    let scale = rect.height() / crate::kill_feed::SCREEN_HEIGHT;
    let fps = FluentValue::from(fps.round() as i64);
    let line = catalog.text_with("viewer-fps", &[("fps", fps)]);
    let font = egui::FontId::proportional(FPS_TEXT * scale);
    let galley = painter.layout_no_wrap(line, font, colour);
    let at = egui::pos2(
        rect.left() + FPS_MARGIN * scale,
        rect.top() + FPS_MARGIN * scale,
    );
    painter.galley(at, galley, colour);
}

/// Why the view is the recorder's while a player is followed, two lines
/// over where the watched player's name stands, on the same dark band, in
/// `colour`
fn draw_unseen(painter: &egui::Painter, rect: egui::Rect, line: &str, colour: egui::Color32) {
    let size = rect.height() / 32.;
    let font = egui::FontId::proportional(size);
    let galley = painter.layout_no_wrap(line.to_owned(), font, colour);
    let at = egui::pos2(
        rect.center().x - galley.size().x / 2.,
        rect.bottom() - (NAME_FROM_BOTTOM + 2.) * size - galley.size().y,
    );
    let band = egui::Rect::from_min_size(at, galley.size());
    let band = band.expand2(egui::vec2(size / 2., size / 5.));
    painter.rect_filled(band, theme::RADIUS_SMALL, NAME_BAND);
    painter.galley(at, galley, colour);
}

/// How far under the view's middle the speedometer stands, as a share of
/// the view's height, and how tall its digits and the jump speed's text
/// are, as shares too
const SPEED_BELOW: f32 = 0.16;
const SPEED_SIZE: f32 = 1. / 20.;
const JUMP_SIZE: f32 = 1. / 32.;

/// The speeds shown under the crosshair, of the recorder or another player
struct Speeds {
    /// None while not known
    velocity: Option<[f32; 3]>,
    last_jump: Option<[f32; 3]>,
    /// Worked out from how the player moved rather than recorded, shown
    /// with a ~ before
    estimated: bool,
}

/// A speed under the crosshair in the HUD's colour, and under it the speed
/// at the last jump, each as the settings ask, counted along their axes in
/// whole units a second
fn draw_speeds(
    painter: &egui::Painter,
    rect: egui::Rect,
    catalog: &Catalog,
    speeds: &Speeds,
    settings: &ViewerSettings,
) {
    let axes = settings.speed_axes;
    let colour = rgb(settings.hud_color);
    let shown = |velocity: [f32; 3]| {
        let speed = axes.speed(velocity).round();
        if speeds.estimated {
            format!("~{speed}")
        } else {
            speed.to_string()
        }
    };

    let mut at = egui::pos2(
        rect.center().x,
        rect.center().y + rect.height() * SPEED_BELOW,
    );
    if settings.speedometer
        && let Some(velocity) = speeds.velocity
    {
        let font = egui::FontId::proportional(rect.height() * SPEED_SIZE);
        let drawn = painter.text(at, egui::Align2::CENTER_TOP, shown(velocity), font, colour);
        at.y = drawn.bottom();
    }

    if settings.jump_speed
        && let Some(jump) = speeds.last_jump
    {
        let speed = FluentValue::from(shown(jump));
        let line = catalog.text_with("viewer-jump-speed", &[("speed", speed)]);
        let font = egui::FontId::proportional(rect.height() * JUMP_SIZE);
        painter.text(at, egui::Align2::CENTER_TOP, line, font, colour);
    }
}

/// Which map the server is changing to, where AG's countdown stands, on a
/// dark band as the watched player's name is, in `colour`
fn draw_map_change(painter: &egui::Painter, rect: egui::Rect, line: &str, colour: egui::Color32) {
    let size = rect.height() / 32.;
    let font = egui::FontId::proportional(size);
    let galley = painter.layout_no_wrap(line.to_owned(), font, colour);
    let at = egui::pos2(
        rect.center().x - galley.size().x / 2.,
        rect.top() + MAP_CHANGE_FROM_TOP * size,
    );
    let band = egui::Rect::from_min_size(at, galley.size());
    let band = band.expand2(egui::vec2(size / 2., size / 5.));
    painter.rect_filled(band, theme::RADIUS_SMALL, NAME_BAND);
    painter.galley(at, galley, colour);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_player_is_looked_for_over_their_whole_body() {
        // Seen along x, so the camera's right runs along -y
        let points = body_points([100., 0., 36.], [0., -1., 0.]);
        assert_eq!(points.len(), 9);
        assert!(points.contains(&[100., 0., 60.]));
        assert!(points.contains(&[100., -12., 12.]));
        assert!(points.contains(&[100., 12., 36.]));
    }

    #[test]
    fn times_show_milliseconds_and_hours_only_past_an_hour() {
        assert_eq!(clock(62., false), "01:02.000");
        assert_eq!(clock(1204.512, false), "20:04.512");
        assert_eq!(clock(f64::from(61.999_999_f32), false), "01:02.000");
        assert_eq!(clock(3723.004, true), "01:02:03.004");
        assert_eq!(clock(-1., false), "00:00.000");
    }

    #[test]
    fn an_intermission_changes_to_the_next_sessions_map() {
        let session = |time: f32, name: &str, intermission| SessionMap {
            segment: 0,
            time,
            name: name.into(),
            intermission,
        };
        let sessions = [
            session(0., "ag_hidden_lab", Some(910.)),
            session(921.6, "elixir", None),
        ];
        let changing = |at| changing_to(&sessions, at).map(|s| s.name.as_str());
        assert_eq!(changing(915.), Some("elixir"));

        // Before the intermission, and in a session with none to come
        assert_eq!(changing(0.), None);
        assert_eq!(changing(909.), None);
        assert_eq!(changing(930.), None);
    }

    #[test]
    fn the_name_shown_is_the_followed_players() {
        let player = |entity: u16, name: &str| PlayerView {
            slot: (entity - 1) as u8,
            entity,
            name: name.into(),
            model: None,
            top_color: None,
            bottom_color: None,
        };
        let players = [player(1, "ru$ty"), player(6, "naz")];
        let name = |camera: &scene::Camera| watched(camera, 6, &players).map(|p| p.name.clone());

        // Playing, the recorder's own, and watching through someone's eyes
        // or behind them, theirs
        assert_eq!(name(&scene::Camera::default()).as_deref(), Some("naz"));
        let in_eye = scene::Camera {
            observer_mode: OBS_IN_EYE,
            observer_target: 1,
            ..scene::Camera::default()
        };
        assert_eq!(name(&in_eye).as_deref(), Some("ru$ty"));
        let chasing = scene::Camera {
            observer_mode: 1,
            ..in_eye.clone()
        };
        assert_eq!(name(&chasing).as_deref(), Some("ru$ty"));

        // Roaming follows nobody
        let roaming = scene::Camera {
            observer_mode: OBS_ROAMING,
            ..in_eye
        };
        assert_eq!(name(&roaming), None);
    }

    #[test]
    fn the_game_bobs_by_a_share_of_the_speed_rising_then_falling() {
        assert_eq!(game_bob(0.01, 0.3, [0.; 3]), 0.);
        assert_eq!(game_bob(0., 0.2, [300., 0., 0.]), 0.);

        // At 300 units a second, a third of the 3 units as the cycle starts,
        // all of them a quarter in, and no more than 4 however fast
        let close = |a: f32, b: f32| (a - b).abs() < 1e-4;
        assert!(close(game_bob(0.01, 0., [300., 0., 50.]), 0.9));
        assert!(close(game_bob(0.01, 0.2, [0., 300., 0.]), 3.));
        assert!(close(game_bob(0.01, 1.0, [0., 300., 0.]), 3.));
        assert!(close(game_bob(0.005, 0.2, [0., 300., 0.]), 1.5));
        assert_eq!(game_bob(0.01, 0.2, [2000., 0., 0.]), HIGHEST_BOB);
    }

    #[test]
    fn the_weapon_sits_a_unit_under_the_view_and_bob_pushes_and_tilts_it() {
        let (at, angles) = bobbed([0., 0., 100.], [10., 90., 0.], 0.);
        assert_eq!(at, [0., 0., 99.]);
        assert_eq!(angles, [10., 90., 0.]);

        // Looking along x, 2 units of bob push it 0.8 along and tilt it
        let (at, angles) = bobbed([0.; 3], [0.; 3], 2.);
        assert!((at[0] - 0.8).abs() < 1e-5 && at[1].abs() < 1e-5);
        assert_eq!(angles, [0.6, -1., -2.]);
    }

    fn playback(playing: bool, backward: bool, stop_at: Option<f64>) -> Playback {
        Playback {
            time: 400.,
            rate: Rate::of(0.5).unwrap(),
            backward,
            playing,
            stop_at,
            loop_range: None,
        }
    }

    #[test]
    fn a_viewer_opened_again_plays_on_as_the_one_before() {
        // Paused stays paused, wherever it stood
        assert_eq!(playback(false, false, None).resume(), None);
        assert_eq!(playback(false, true, Some(410.)).resume(), None);

        assert_eq!(playback(true, false, None).resume(), Some(Command::Play));
        assert_eq!(
            playback(true, true, None).resume(),
            Some(Command::PlayBackward)
        );

        // A played range still pauses at its end
        let range = Command::PlayRange {
            from: SourceTime(400.),
            to: SourceTime(410.),
        };
        assert_eq!(playback(true, false, Some(410.)).resume(), Some(range));
    }

    #[test]
    fn colour_codes_leave_names_but_other_carets_stay() {
        assert_eq!(text::without_colours("^1red^0 ^7naz"), "red naz");
        assert_eq!(text::without_colours("a^b ^"), "a^b ^");
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
