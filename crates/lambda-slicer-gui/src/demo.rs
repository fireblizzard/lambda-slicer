//! One demo's view, its timeline with lanes per player, its clips and its
//! events
//!
//! Selecting a demo analyses it again on a thread of its own, since weapon
//! occurrences are not kept in the cache. Only the compact timeline model
//! crosses back to the window. Choosing another demo cancels the analysis

mod clips_tab;
mod compact;
mod events;
mod scores;
mod track;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Ui, Vec2};
use fluent_bundle::FluentValue;
use lambda_slicer_assets::search::SearchOptions;
use lambda_slicer_core::analysis::analyze_demo_with;
use lambda_slicer_core::analysis::match_events::MatchChange;
use lambda_slicer_core::analysis::motion::Floors;
use lambda_slicer_core::analysis::weapons::{Action, Weapon};
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::library::cache::FileRow;
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::slice::range::Snapped;
use lambda_slicer_core::state::DecodeStatus;
use lambda_slicer_core::timeline::time::DemoTime;

use crate::app::SettingsTab;
use crate::chat;
use crate::clips::{Clip, next_name, parse_time};
use crate::content::GameFolders;
use crate::export::{ClipKey, ExportItem, ExportJob, ExportState, FileNaming};
use crate::i18n::Catalog;
use crate::kill_analysis::Walls;
use crate::kill_filters::{KillFilters, Verdict};
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::timeline::{Event, EventKind, MarkKind, Model};
use crate::widgets::{self, Icon, IconButton, Variant};
use track::{Density, MarkerBin, RowMark, Side, Transform};

/// A demo time in seconds, as the viewer takes them
fn secs(t: DemoTime) -> f32 {
    t.secs_f64() as f32
}

pub fn weapon_color(w: Option<Weapon>) -> Color32 {
    match w {
        Some(Weapon::Gauss) => Color32::from_rgb(90, 170, 255),
        Some(Weapon::Crossbow) => Color32::from_rgb(120, 220, 120),
        Some(Weapon::Rpg) => Color32::from_rgb(255, 150, 60),
        Some(Weapon::Mp5) => Color32::from_rgb(230, 210, 90),
        Some(Weapon::Shotgun) => Color32::from_rgb(220, 120, 200),
        Some(Weapon::Python) => Color32::from_rgb(170, 140, 255),
        Some(Weapon::Glock) => Color32::from_rgb(180, 180, 180),
        Some(Weapon::Crowbar) => Color32::from_rgb(200, 160, 120),
        Some(Weapon::Egon) => Color32::from_rgb(120, 230, 230),
        Some(Weapon::Hivehand) => Color32::from_rgb(170, 220, 60),
        Some(_) => Color32::from_rgb(200, 200, 140),
        None => Color32::from_rgb(160, 160, 160),
    }
}

const KILL_COLOR: Color32 = Color32::from_rgb(240, 70, 70);

/// The tint of a chosen action window, on the timeline and its event rows
const WINDOW_TINT: Color32 = Color32::from_rgba_premultiplied(40, 32, 10, 40);

/// The length of a clip made with New clip, before it is edited
const NEW_CLIP_SECONDS: f32 = 10.;

/// The fewest kills of a killing spree that shows its badge, unless the
/// settings say otherwise
pub const DEFAULT_SPREE: u32 = 20;

/// How dark the controls over the whole screen view are behind them
const FULLSCREEN_SHADE: u8 = 160;

/// What a control of the demo view asked for. Controls only collect these
/// while they draw, and the view applies them once everything has drawn, so
/// a button, a timeline click and a keyboard shortcut for the same thing
/// all go through one place
#[derive(Clone, Copy, Debug, PartialEq)]
enum DemoAction {
    /// Choose an event and seek to it. From the event list the list keeps
    /// its place, otherwise it scrolls to the event
    Select {
        event: usize,
        from_list: bool,
    },
    /// Move the playhead with no event chosen, choosing the clip there
    Seek(f32),
    /// Play an event once from its preroll through its postroll
    PlayEvent(usize),
    /// Show the whole recording on the timeline
    Fit,
    /// Zoom the timeline around the playhead, or around the middle of the
    /// view when the playhead is out of it. A factor below one zooms in
    Zoom(f32),
    /// Choose a clip from the timeline and show it in the Clips tab
    ChooseClip(usize),
    /// Shade one of the most intense moments and bring it into view
    ChooseHighlight(usize),
    /// Add a clip of ten seconds to edit by hand
    NewClip,
    /// Add a clip around the selected event, or the chosen highlight
    ClipEvent,
    ClipHighlight,
    /// Show the final scores rather than those at the selected event
    ShowFinal,
}

/// What the demo view asks the app to do, as only the app can
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Close,
    /// Open the settings window on this tab
    Settings(SettingsTab),
    /// Ask where the game is installed
    Locate,
    /// Bring back the demo list, hidden or out of room
    ShowList,
    /// A problem to show until dismissed
    Notice(String),
    /// A clip finished exporting to this file
    Exported(PathBuf),
    /// Make the demo at this path, as the library shows it, a favourite or
    /// no longer one
    Favourite(String),
}

/// The tabs of the inspector's lower pane, under the scoreboard
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorTab {
    #[default]
    Events,
    Clips,
}

/// How the inspector's scoreboard and the events and clips under it share
/// its height
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneSplit {
    /// Half each
    #[default]
    Even,
    /// The scoreboard takes it all, the events and clips folded to their
    /// header
    Scores,
    /// The events and clips take it all, the scoreboard folded
    Lists,
}

impl PaneSplit {
    /// The split after a click on the scoreboard's header, or the lists'
    /// when not `scores`. From half each the pane clicked takes it all, and
    /// a click on either while one is folded goes back to half each, so one
    /// is always open
    fn clicked(self, scores: bool) -> Self {
        match (self, scores) {
            (PaneSplit::Even, true) => PaneSplit::Scores,
            (PaneSplit::Even, false) => PaneSplit::Lists,
            _ => PaneSplit::Even,
        }
    }
}

/// The gap between the inspector's two panes, a line across the panel in
/// its middle
const PANE_GAP: f32 = 2. * theme::GAP_M;

/// Which way a fold button's triangle points
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pointing {
    Down,
    Up,
    /// Along the line, right or left in a right-to-left interface
    Along,
}

/// A pane's button that folds or unfolds it, its triangle pointing along
/// while the panes share the panel, up while this one is folded and down
/// while it is `full`. Its tooltip says what a click does, the pane taking
/// the whole panel from sharing it, or both sharing it again
fn fold_button(
    ui: &mut Ui,
    catalog: &Catalog,
    open: bool,
    full: bool,
    rtl: bool,
) -> egui::Response {
    let (pointing, tip) = if !open {
        (Pointing::Up, "pane-share")
    } else if full {
        (Pointing::Down, "pane-share")
    } else {
        (Pointing::Along, "pane-fill")
    };
    fold_triangle(ui, pointing, rtl).on_hover_text(catalog.text(tip))
}

/// A button that folds or unfolds something, a triangle `pointing` one way
pub(super) fn fold_triangle(ui: &mut Ui, pointing: Pointing, rtl: bool) -> egui::Response {
    let side = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let colour = if response.hovered() {
        theme::FOREGROUND
    } else {
        theme::MUTED
    };

    let centre = rect.center();
    let size = side / 6.;
    let along = if rtl { -1. } else { 1. };
    let points = match pointing {
        Pointing::Down => vec![
            centre + Vec2::new(-size, -size / 2.),
            centre + Vec2::new(size, -size / 2.),
            centre + Vec2::new(0., size),
        ],
        Pointing::Up => vec![
            centre + Vec2::new(-size, size / 2.),
            centre + Vec2::new(size, size / 2.),
            centre + Vec2::new(0., -size),
        ],
        Pointing::Along => vec![
            centre + Vec2::new(-size / 2. * along, -size),
            centre + Vec2::new(-size / 2. * along, size),
            centre + Vec2::new(size * along, 0.),
        ],
    };
    let shape = egui::Shape::convex_polygon(points, colour, egui::Stroke::NONE);
    ui.painter().add(shape);
    response
}

/// How the demo's panes are arranged, kept for the session whichever demo
/// is open
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorkspaceUi {
    pub inspector_tab: InspectorTab,
    pub split: PaneSplit,
    /// The share of the inspector's height the divider was dragged to give
    /// the scoreboard while both panes are open, none to give it what it
    /// needs up to half
    pub scores_share: Option<f32>,
    /// The event filters are open, rather than folded to their summary
    pub event_filters_open: bool,
    /// The inspector's width beside the view as last drawn, none before it
    /// was, and its first width before any drag, as the window's width suits
    pub inspector_width: Option<f32>,
    pub inspector_first: f32,
    /// The demo list was hidden
    pub list_hidden: bool,
    /// The window has no room for the list beside the view, so it opens
    /// over the view when asked for
    pub list_squeezed: bool,
    pub drawer_open: bool,
    /// Counts the times the default layout was restored, so the inspector
    /// forgets the size it was dragged to
    pub restored: u32,
}

impl WorkspaceUi {
    /// Whether the demo list is out of view, so the header offers it
    pub fn list_out_of_view(&self) -> bool {
        self.list_hidden || (self.list_squeezed && !self.drawer_open)
    }
}

/// Why the viewer has no game files to read, for its setup notice
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Setup {
    /// A folder the settings name that no longer holds the game
    pub missing: Option<PathBuf>,
    /// Why the last choice was refused
    pub refusal: Option<String>,
}

/// What the demo view shows that the app holds
pub struct Surroundings<'a> {
    pub catalog: &'a Catalog,
    pub rtl: bool,
    /// The library's row for the demo, none for a demo opened as a file
    pub file: Option<&'a FileRow>,
    /// The window is too narrow for the inspector beside the view, so it
    /// goes under it
    pub narrow: bool,
    pub setup: &'a Setup,
    /// The demo is a favourite
    pub favourite: bool,
}

/// The inspector beside the view at first, and in a window at least
/// `WIDE_WINDOW` wide, with the narrowest and widest it can be dragged to
pub const INSPECTOR_WIDTH: f32 = 360.;
const INSPECTOR_WIDE: f32 = 400.;
const WIDE_WINDOW: f32 = 1600.;
const INSPECTOR_RANGE: std::ops::RangeInclusive<f32> = 320.0..=520.0;

/// The inspector under the view in a narrow window at first, and the least
/// it shrinks to
const INSPECTOR_HEIGHT: f32 = 220.;
const INSPECTOR_MIN_HEIGHT: f32 = 120.;

/// The least the view shrinks to, and the most of the middle the player
/// lanes take
const MIN_VIEWPORT: f32 = 120.;
const LANES_SHARE: f32 = 0.35;

/// How many icon buttons the header keeps at its trailing end
const HEADER_ACTIONS: usize = 4;

/// The setup notice's width, the circle around its warning, and the warning
/// sign itself
const SETUP_WIDTH: f32 = 380.;
const SETUP_ICON: f32 = 44.;
const SETUP_GLYPH: f32 = 20.;

/// The least height the whole setup notice fits in, below which it keeps
/// only its title and buttons
const SETUP_FULL_HEIGHT: f32 = 260.;

/// A number for each demo view opened, so state kept by egui for one demo
/// is never taken for another's
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// A clip's start or end
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Start,
    End,
}

/// A drag on the timeline, decided by what it began on and kept until it is
/// let go, whatever it passes over
#[derive(Clone, Copy, Debug, PartialEq)]
enum Drag {
    /// The ruler or the playhead, dragging the viewer's playhead
    Scrub,
    /// A clip's edge, which goes back to `original` if Escape gives it up
    ClipEdge {
        clip: usize,
        edge: Edge,
        original: DemoTime,
    },
    /// The track moved along, keeping the moment `grab` under the pointer as
    /// the press's `transform` placed it
    Pan {
        grab: f32,
        original: (f32, f32),
        transform: Transform,
    },
    /// The overview's window moved, `offset` seconds after its start under
    /// the pointer
    OverviewPan { offset: f32, original: (f32, f32) },
    /// Escape gave the drag up, and it waits to be let go
    Cancelled,
}

/// Events of different sessions under one click on the timeline, offered
/// to choose from rather than one taken at random
#[derive(Clone, Debug, PartialEq)]
struct Chooser {
    at: Pos2,
    events: Vec<usize>,
    /// Opened by this frame's click, which must not close it again
    fresh: bool,
}

/// The timeline's markers for one view, kept while the view, its width and
/// the shown events stay the same
struct MarkerCache {
    view: (f32, f32),
    width: f32,
    rows_version: u64,
    bins: Vec<MarkerBin>,
}

enum Loaded {
    Model(Arc<Model>),
    Failed(String),
}

/// A demo's analysis running in the background
struct Loader {
    cancel: Arc<AtomicBool>,
    rx: Receiver<Loaded>,
    thread: Option<JoinHandle<()>>,
}

impl Loader {
    fn start(path: PathBuf, duration: f32, floors: Option<Floors>, ctx: egui::Context) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let thread_cancel = cancel.clone();
        let thread = std::thread::Builder::new()
            .name("demo analysis".into())
            .spawn(move || {
                let cancel = Some(thread_cancel);
                let report = analyze_demo_with(&path, Limits::default(), cancel, floors.as_ref());
                let loaded = match report.decode {
                    DecodeStatus::Cancelled => return,
                    DecodeStatus::Failed | DecodeStatus::Unsupported => Loaded::Failed(
                        report
                            .decode_error
                            .map(|m| m.code.to_string())
                            .unwrap_or_default(),
                    ),
                    _ => Loaded::Model(Arc::new(Model::build(&report, duration))),
                };
                let _ = tx.send(loaded);
                ctx.request_repaint();
            })
            .expect("spawn demo analysis");
        Self {
            cancel,
            rx,
            thread: Some(thread),
        }
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Which events the list, the timeline and its density show, all through
/// [`Shown::judge`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub shots: bool,
    pub actions: bool,
    pub kills: bool,
    pub switches: bool,
    pub chat: bool,
    /// Votes, timeouts, pauses and sudden death
    pub matches: bool,
    pub weapon: Option<Weapon>,
    /// Refinements of the kills, kept while kills are hidden
    pub kill_filters: KillFilters,
}

/// Only kills at first, the rest one toggle away
impl Default for Shown {
    fn default() -> Self {
        Self {
            shots: false,
            actions: false,
            kills: true,
            switches: false,
            chat: false,
            matches: false,
            weapon: None,
            kill_filters: KillFilters::default(),
        }
    }
}

impl Shown {
    fn kind(&self, k: MarkKind) -> bool {
        match k {
            MarkKind::Shot => self.shots,
            MarkKind::Action => self.actions,
            MarkKind::Kill | MarkKind::Death => self.kills,
            MarkKind::Switch => self.switches,
            MarkKind::Chat => self.chat,
            MarkKind::Match => self.matches,
        }
    }

    fn weapon(&self, w: Option<Weapon>) -> bool {
        self.weapon.is_none() || self.weapon == w
    }

    /// Whether `event` is shown, as far as can be told with what `walls`
    /// has told of kills through a wall. Only a kill can be unknown, by a
    /// refinement its data cannot settle, and the refinements leave other
    /// events to their kind and weapon
    fn judge(&self, event: &Event, walls: &Walls) -> Verdict {
        if !self.kind(mark_kind(event)) || !self.weapon(event.weapon) {
            return Verdict::NoMatch;
        }

        match &event.kind {
            EventKind::Kill { features, .. } => {
                let wall = walls.get(&features.key);
                self.kill_filters.judge(features, wall)
            }
            _ => Verdict::Match,
        }
    }
}

/// Whether the actor of `event`, or a kill's victim, has `player` in their
/// name, ignoring colour codes. `player` is in lowercase, and empty matches
/// every event
fn has_player(event: &Event, player: &str) -> bool {
    if player.is_empty() {
        return true;
    }

    let mut names = vec![event.actor.as_deref()];
    if let EventKind::Kill { victim, .. } = &event.kind {
        names.push(victim.as_deref());
    }
    for name in names.into_iter().flatten() {
        if text::player_name(name).to_lowercase().contains(player) {
            return true;
        }
    }
    false
}

/// A vote, a timeout, a pause, sudden death, a change of map or a player
/// starting or stopping spectating, in words
fn describe_match(catalog: &Catalog, change: &MatchChange) -> String {
    let named = |s: &str| FluentValue::from(text::without_colours(s));
    match change {
        MatchChange::VoteCalled { subject, .. } => {
            catalog.text_with("event-vote-called", &[("subject", named(subject))])
        }
        MatchChange::VoteCast {
            subject,
            yes,
            no,
            tally,
        } => {
            let change = vote_change(catalog, *yes, *no);
            let tally = catalog.text_with(
                "event-vote-tally",
                &[
                    ("yes", FluentValue::from(i64::from(tally.yes))),
                    ("no", FluentValue::from(i64::from(tally.no))),
                    ("undecided", FluentValue::from(i64::from(tally.undecided))),
                ],
            );
            let args = [
                ("subject", named(subject)),
                ("change", FluentValue::from(change)),
                ("tally", FluentValue::from(tally)),
            ];
            catalog.text_with("event-vote-cast", &args)
        }
        MatchChange::VoteAccepted { subject } => {
            catalog.text_with("event-vote-accepted", &[("subject", named(subject))])
        }
        MatchChange::VoteDenied { subject } => {
            catalog.text_with("event-vote-denied", &[("subject", named(subject))])
        }
        MatchChange::TimeoutCalled => catalog.text("event-timeout-called"),
        MatchChange::TimeoutUnderWay => catalog.text("event-timeout-under-way"),
        MatchChange::TimeoutOver => catalog.text("event-timeout-over"),
        MatchChange::Paused => catalog.text("event-paused"),
        MatchChange::Resumed => catalog.text("event-resumed"),
        MatchChange::SuddenDeath => catalog.text("event-sudden-death"),
        MatchChange::SuddenDeathOver => catalog.text("event-sudden-death-over"),
        MatchChange::MapChanged { map } => catalog.text_with(
            "event-map-changed",
            &[("map", FluentValue::from(map.clone()))],
        ),
        MatchChange::Spectating => catalog.text("event-spectating"),
        MatchChange::Playing => catalog.text("event-playing"),
    }
}

/// How a vote's count changed, such as two more yes and one more no
fn vote_change(catalog: &Catalog, yes: i16, no: i16) -> String {
    let counts = [
        (yes, "event-vote-more-yes", "event-vote-fewer-yes"),
        (no, "event-vote-more-no", "event-vote-fewer-no"),
    ];
    let mut changes = Vec::new();
    for (count, more, fewer) in counts {
        if count == 0 {
            continue;
        }
        let id = if count > 0 { more } else { fewer };
        let count = FluentValue::from(i64::from(count.abs()));
        changes.push(catalog.text_with(id, &[("count", count)]));
    }
    changes.join(", ")
}

/// What a line of chat says, without its sender's name and the colon
/// after it, as the event names its sender already. A tag before the name,
/// such as AG's `(T)` for team chat, stays
fn said(line: &str, sender: Option<&str>) -> String {
    let Some(sender) = sender.map(text::without_colours) else {
        return line.to_owned();
    };
    let Some(start) = line.find(&sender).filter(|_| !sender.is_empty()) else {
        return line.to_owned();
    };

    let before = &line[..start];
    let after = line[start + sender.len()..]
        .trim_start_matches(':')
        .trim_start();
    format!("{before}{after}")
}

/// How an event is marked on the timeline. A kill's death mark in its
/// victim's lane is the same event
fn mark_kind(event: &Event) -> MarkKind {
    match event.kind {
        EventKind::Action { action, .. } if action.is_shot() => MarkKind::Shot,
        EventKind::Action { .. } => MarkKind::Action,
        EventKind::Kill { .. } => MarkKind::Kill,
        EventKind::Switch => MarkKind::Switch,
        EventKind::Chat { .. } => MarkKind::Chat,
        EventKind::Match(_) => MarkKind::Match,
    }
}

/// What the compact timeline draws of the shown event at `index`. Team
/// colours are only for a session played in teams, as players' teams in
/// other sessions are not sides
fn row_mark(index: usize, event: &Event, teams: bool) -> RowMark {
    let (side, headshot) = match &event.kind {
        EventKind::Kill {
            features, headshot, ..
        } => {
            let team = features.killer_team.as_deref().filter(|_| teams);
            (track::side_of(team), *headshot == Some(true))
        }
        _ => (Side::Unknown, false),
    };

    RowMark {
        event: index,
        segment: event.segment,
        time: event.time,
        kind: mark_kind(event),
        weapon: event.weapon,
        side,
        headshot,
    }
}

pub struct DemoView {
    pub path: PathBuf,
    /// Which demo view this is, for egui state that must not carry over
    generation: u64,
    /// What the controls asked for this frame, applied once they have drawn,
    /// and what they asked of the app
    actions: Vec<DemoAction>,
    requests: Vec<Request>,
    loader: Option<Loader>,
    /// The game folder whose maps the analysis read the floor under players
    /// from, none without the game's files
    floors_from: Option<GameFolders>,
    /// The analysis again for other game files, whose events take the
    /// place of those shown once it is in
    reanalysis: Option<Loader>,
    model: Option<Arc<Model>>,
    failed: Option<String>,
    /// The stretch of the recording the timeline shows
    view: (f32, f32),
    shown: Shown,
    /// Text in the name of a player in each shown event, ignoring case
    player: String,
    /// The shown events as indices into the model's events, in time order,
    /// and what the timeline draws of each
    rows: Vec<usize>,
    marks: Vec<RowMark>,
    /// Whether each of the model's events is shown, for the player lanes
    shown_events: Vec<bool>,
    /// What the rows were worked out for, with the player's name, and the
    /// revision of the kills through a wall then
    rows_for: Option<(Shown, String, u64)>,
    /// Counts each time the rows change, for what is worked out from them
    rows_version: u64,
    /// Each line of chat by its event, its sender's place named, once the
    /// viewer reads the game's files
    chat_lines: Option<HashMap<usize, String>>,
    /// Kills a refinement could not tell about, so not shown
    unknown_kills: usize,
    /// The shown events' density for a version of the rows, none for a
    /// recording with no length
    density: Option<(u64, Option<Density>)>,
    /// The timeline's markers for the view last drawn
    markers: Option<MarkerCache>,
    selected: Option<usize>,
    /// The scoreboard follows the playhead once it has moved since the demo
    /// opened, or since the final scores were asked for, and shows the
    /// final scores until then, while the playhead stays where this says
    scores_live: bool,
    scores_still: Option<f32>,
    /// The height the scoreboard's pane needs to show it all, as last drawn
    scores_needed: Option<f32>,
    /// Events chosen together in the list for copying, with Ctrl or Shift
    /// held or by Ctrl+A. Selecting an event chooses it alone
    marked: BTreeSet<usize>,
    /// The event a Shift click chooses from
    mark_from: Option<usize>,
    scroll_table: bool,
    /// How many shown events the playhead had passed when the list last
    /// followed it, none while paused
    followed: Option<usize>,
    /// Whether the list follows the playhead, until scrolled by hand, and
    /// whether it goes back to it on the next frame even while paused
    follow_playhead: bool,
    pan_to_playhead: bool,
    /// How far the list was scrolled when last drawn, for telling a scroll
    /// by hand
    list_offset: f32,
    /// Frames left to scroll the list to the chosen window's first event.
    /// A list seen for the first time has no size yet, so it gets a few
    scroll_to_window: u8,
    /// An inspector tab a control asked for, shown on the next frame
    wanted_tab: Option<InspectorTab>,
    /// Events to choose between after a click on the timeline
    chooser: Option<Chooser>,
    /// The drag under way on the timeline
    drag: Option<Drag>,
    /// The playhead was in view last frame, so playing out of view turns a
    /// page of the timeline rather than a view the user moved away
    playhead_in_view: bool,
    /// The action window shaded on the timeline, and one to jump to once
    /// the analysis is in
    window: Option<usize>,
    /// The highlight was chosen after the selected event, so Clip it clips
    /// the highlight
    highlight_last: bool,
    pub open_window: Option<usize>,
    /// A shown event to select once the analysis is in, by its place in the
    /// list, for checking the view from the command line
    pub open_event: Option<usize>,
    /// Export every clip once the analysis is in, for checking exports
    /// from the command line
    pub export_on_load: bool,
    /// The player lanes are hidden under the timeline until asked for, as
    /// they take some reading
    pub lanes_open: bool,
    /// This demo's clips. The app saves them when `clips_changed` is set
    pub clips: Vec<Clip>,
    pub clips_changed: bool,
    /// Each clip's identity for this view, in the order of `clips`, which
    /// stays while the clip is edited and moved, and the next one to give
    clip_ids: Vec<u64>,
    next_clip_id: u64,
    chosen_clip: Option<usize>,
    /// A clip's start or end while it is being typed
    typing: Option<(usize, Edge, String)>,
    /// Seconds a new clip keeps before and after its event
    pub padding: (f32, f32),
    /// The fewest kills of a killing spree that shows its badge, the
    /// settings' each frame
    pub spree_least: u32,
    /// What an export of each requested range would cut, worked out once
    snapped: HashMap<(DemoTime, DemoTime), Result<Snapped, Message>>,
    /// Where exports go, the demo's own folder when none is set
    pub export_folder: Option<PathBuf>,
    /// How exported clips are named
    pub file_naming: FileNaming,
    export: Option<ExportJob>,
    /// How each clip's latest export went, by its identity, with the range
    /// that export was asked for
    exports: HashMap<u64, (ClipKey, ExportState)>,
    /// The demo viewer, when a game folder is known
    pub viewer: Option<crate::viewer::Viewer>,
    /// The game folder the viewer was opened with
    pub viewer_game: Option<GameFolders>,
    /// The demo's length in seconds, as the library knows it
    pub duration: f32,
    /// The clip the viewer loops
    looping: Option<usize>,
    /// The height the header and the transport with the timeline took last
    /// frame, which the inspector under the view leaves room for
    around_view: f32,
    /// Which Gauss kills went through a wall, told in the background
    walls: Walls,
}

impl DemoView {
    /// Open the demo at `path`, analysing it in the background with the
    /// floor under players read from the maps of a game folder when
    /// `floors` gives the folder and its floors
    pub fn open(
        path: PathBuf,
        duration: f32,
        floors: Option<(GameFolders, Floors)>,
        ctx: &egui::Context,
    ) -> Self {
        let (floors_from, floors) = floors.unzip();
        Self {
            loader: Some(Loader::start(path.clone(), duration, floors, ctx.clone())),
            floors_from,
            reanalysis: None,
            path,
            generation: GENERATION.fetch_add(1, Ordering::Relaxed),
            actions: Vec::new(),
            requests: Vec::new(),
            model: None,
            failed: None,
            view: track::clamp_view((0., duration), (0., duration)),
            shown: Shown::default(),
            player: String::new(),
            rows: Vec::new(),
            marks: Vec::new(),
            shown_events: Vec::new(),
            rows_for: None,
            rows_version: 0,
            chat_lines: None,
            unknown_kills: 0,
            density: None,
            markers: None,
            selected: None,
            scores_live: false,
            scores_still: None,
            scores_needed: None,
            marked: BTreeSet::new(),
            mark_from: None,
            scroll_table: false,
            followed: None,
            follow_playhead: true,
            pan_to_playhead: false,
            list_offset: 0.,
            scroll_to_window: 0,
            wanted_tab: None,
            chooser: None,
            drag: None,
            playhead_in_view: true,
            window: None,
            highlight_last: false,
            open_window: None,
            open_event: None,
            lanes_open: false,
            clips: Vec::new(),
            clips_changed: false,
            clip_ids: Vec::new(),
            next_clip_id: 0,
            chosen_clip: None,
            typing: None,
            padding: (0., 0.),
            spree_least: DEFAULT_SPREE,
            snapped: HashMap::new(),
            export_folder: None,
            file_naming: FileNaming::default(),
            export: None,
            exports: HashMap::new(),
            export_on_load: false,
            viewer: None,
            viewer_game: None,
            duration,
            looping: None,
            around_view: 0.,
            walls: Walls::default(),
        }
    }

    /// The folder exports go to
    fn export_dir(&self) -> PathBuf {
        self.export_folder
            .clone()
            .or_else(|| self.path.parent().map(Path::to_path_buf))
            .unwrap_or_default()
    }

    /// Give every clip an identity. Clips the app hands over arrive without
    fn identify_clips(&mut self) {
        while self.clip_ids.len() < self.clips.len() {
            self.clip_ids.push(self.next_clip_id);
            self.next_clip_id += 1;
        }
        self.clip_ids.truncate(self.clips.len());
    }

    /// Note how clip `i`'s export stands, for its range as it is now
    fn mark_export(&mut self, i: usize, state: ExportState) {
        self.identify_clips();
        let (Some(clip), Some(&id)) = (self.clips.get(i), self.clip_ids.get(i)) else {
            return;
        };
        self.exports.insert(id, ((clip.start, clip.end), state));
    }

    /// Export these clips one after another with their ranges as they are
    /// now, unless an export is under way
    fn start_export(&mut self, ctx: &egui::Context, clips: &[usize]) {
        if self.export.is_some() || clips.is_empty() {
            return;
        }

        self.identify_clips();
        let mut items = Vec::new();
        for &i in clips {
            let (Some(clip), Some(&id)) = (self.clips.get(i), self.clip_ids.get(i)) else {
                continue;
            };
            let range = (clip.start, clip.end);
            self.exports.insert(id, (range, ExportState::Waiting));
            let file_name = self.file_naming.file_name(&self.path, clip);
            items.push(ExportItem {
                id,
                range,
                file_name,
            });
        }

        let job = ExportJob::start(self.path.clone(), self.export_dir(), items, ctx.clone());
        self.export = Some(job);
    }

    fn poll_export(&mut self) {
        let exports = &mut self.exports;
        let mut written = Vec::new();
        let update = |id: u64, state: ExportState| {
            if let ExportState::Done(done) = &state {
                written.push(done.path.clone());
            }
            if let Some(export) = exports.get_mut(&id) {
                export.1 = state;
            }
        };
        if let Some(job) = &mut self.export
            && job.poll(update)
        {
            self.export = None;
        }

        // The app adds the new files to the library
        for path in written {
            self.requests.push(Request::Exported(path));
        }
    }

    pub fn loading(&self) -> bool {
        self.loader.is_some()
    }

    /// The game folders whose maps the analysis read the floor under
    /// players from, none without the game's files
    pub fn floors_from(&self) -> Option<&[(PathBuf, SearchOptions)]> {
        self.floors_from.as_deref()
    }

    /// Analyse the demo again with `floors`, a game folder and the floor
    /// under players on its maps, or without any, so midair kills are told
    /// with them. The events shown stay, with the clips, the selection,
    /// the filters and the playhead, until the new analysis takes their
    /// place
    pub fn reanalyse(&mut self, floors: Option<(GameFolders, Floors)>, ctx: &egui::Context) {
        let (floors_from, floors) = floors.unzip();
        self.floors_from = floors_from;

        // A first analysis still running starts over instead
        let loader = Loader::start(self.path.clone(), self.duration, floors, ctx.clone());
        if self.loader.is_some() {
            self.loader = Some(loader);
        } else {
            self.reanalysis = Some(loader);
        }
    }

    /// Turn on these kill refinements, for checking them from the command
    /// line
    pub fn refine(&mut self, filters: KillFilters) {
        self.shown.kill_filters = filters;
    }

    /// Show the kinds of event named in the comma-separated `list`, from
    /// `kills`, `shots`, `switches`, `actions`, `chat` and `match`, and no
    /// others, for checking them from the command line
    pub fn show_kinds(&mut self, list: &str) {
        let names: Vec<&str> = list.split(',').map(str::trim).collect();
        let shown = &mut self.shown;
        shown.kills = names.contains(&"kills");
        shown.shots = names.contains(&"shots");
        shown.switches = names.contains(&"switches");
        shown.actions = names.contains(&"actions");
        shown.chat = names.contains(&"chat");
        shown.matches = names.contains(&"match");
    }

    /// Show only events of players with `player` in their name, for
    /// checking it from the command line
    pub fn filter_player(&mut self, player: String) {
        self.player = player;
    }

    /// The view fills the whole screen, so the app gives it the whole
    /// window and the window the whole screen
    pub fn fullscreen(&self) -> bool {
        self.viewer.as_ref().is_some_and(|v| v.fullscreen)
    }

    /// Still analysing or exporting
    pub fn busy(&self) -> bool {
        self.loading() || self.export.is_some() || self.export_on_load
    }

    fn poll(&mut self) {
        self.poll_reanalysis();
        let Some(loader) = &self.loader else {
            return;
        };
        match loader.rx.try_recv() {
            Ok(Loaded::Model(m)) => {
                let extent = (m.start, m.end);
                self.view = extent;
                if let Some(w) = self
                    .open_window
                    .and_then(|i| m.windows.get(i).map(|w| (i, w)))
                {
                    let pad = ((w.1.end - w.1.start) * 0.1).max(3.);
                    self.view = track::clamp_view((w.1.start - pad, w.1.end + pad), extent);
                    self.window = Some(w.0);
                    self.highlight_last = true;
                    self.scroll_to_window = 3;
                }

                // A demo the library gave no length, such as one opened from
                // the file explorer, takes the analysis's
                if self.duration <= 0. {
                    self.duration = m.end;
                    if let Some(viewer) = &mut self.viewer {
                        viewer.learn_length(f64::from(m.end));
                    }
                }
                self.model = Some(m);
                self.loader = None;
            }
            Ok(Loaded::Failed(code)) => {
                self.failed = Some(code);
                self.loader = None;
            }
            Err(_) => {}
        }
    }

    /// Take in the analysis made again, its events in place of those shown.
    /// One that failed leaves the events shown as they are
    fn poll_reanalysis(&mut self) {
        let Some(loader) = &self.reanalysis else {
            return;
        };
        let loaded = match loader.rx.try_recv() {
            Ok(loaded) => loaded,
            Err(_) => return,
        };
        self.reanalysis = None;

        // The same recording gives the same events in the same order, so
        // the selection still points at the same one
        if let Loaded::Model(m) = loaded {
            if self.selected.is_some_and(|i| i >= m.events.len()) {
                self.selected = None;
            }
            self.model = Some(m);
            self.rows_for = None;
            self.chat_lines = None;
        }
    }

    /// Work out the shown events again once what is shown has changed, with
    /// what the timeline draws of them and the kills that could not be told
    fn refresh_rows(&mut self) {
        let Some(m) = self.model.clone() else {
            return;
        };
        let wanted = (self.shown, self.player.clone(), self.walls.revision());
        if self.rows_for.as_ref() == Some(&wanted) {
            return;
        }

        self.rows.clear();
        self.marks.clear();
        self.shown_events = vec![false; m.events.len()];
        self.unknown_kills = 0;

        let player = self.player.trim().to_lowercase();
        for (index, event) in m.events.iter().enumerate() {
            if !has_player(event, &player) {
                continue;
            }

            match self.shown.judge(event, &self.walls) {
                Verdict::Match => {}
                Verdict::Unknown => {
                    self.unknown_kills += 1;
                    continue;
                }
                Verdict::NoMatch => continue,
            }

            self.rows.push(index);
            self.marks
                .push(row_mark(index, event, m.team_play(event.segment)));
            self.shown_events[index] = true;
        }

        self.rows_for = Some(wanted);
        self.rows_version += 1;
    }

    /// The recording's extent, the analysis's once it is in and the
    /// library's length until then
    fn extent(&self) -> (f32, f32) {
        match &self.model {
            Some(m) => (m.start, m.end),
            None => (0., self.duration),
        }
    }

    /// Whether the viewer can go to an event. It seeks by time alone, so an
    /// event of a later session whose time an earlier session also has is
    /// out of its reach
    fn reachable(&self, event: usize) -> bool {
        let Some(m) = &self.model else {
            return false;
        };
        let Some(e) = m.events.get(event) else {
            return false;
        };
        let Some(time) = DemoTime::from_f32(e.time) else {
            return false;
        };

        // Past the last group there is no other session to mistake it for
        match m.groups.segment_at(time) {
            Some(segment) => segment == e.segment,
            None => true,
        }
    }

    /// Choose an event and bring it into view, moving the viewer's playhead
    /// to it when the viewer can go there
    fn select(&mut self, event: usize, from_list: bool) {
        let Some(time) = self
            .model
            .as_ref()
            .and_then(|m| m.events.get(event))
            .map(|e| e.time)
        else {
            return;
        };

        self.selected = Some(event);
        self.marked = BTreeSet::from([event]);
        self.mark_from = Some(event);
        self.highlight_last = false;
        self.scroll_table = !from_list;

        let players = self.event_players(event);
        if self.reachable(event)
            && let Some(viewer) = &mut self.viewer
        {
            viewer.seek(time);
            viewer.follow_first_seen(players);
        }

        let (start, end) = self.view;
        if time < start || time > end {
            self.view = track::centre_on(self.view, self.extent(), time);
        }
    }

    /// The players whose view an event is best seen from, by slot, a
    /// kill's killer and then its victim. Other events have none, and are
    /// seen from the recorder's view
    fn event_players(&self, event: usize) -> [Option<u8>; 2] {
        let kind = self
            .model
            .as_ref()
            .and_then(|m| m.events.get(event))
            .map(|e| &e.kind);
        match kind {
            Some(EventKind::Kill {
                killer,
                victim_player,
                ..
            }) => [killer.map(|k| k.0), victim_player.map(|v| v.0)],
            _ => [None, None],
        }
    }

    /// The range clipping the selected event would take, or with several
    /// events chosen, the one from the first of them to the last, whatever
    /// lies between, padded the same. A start there must fall in the first
    /// event's session, which the last must share. Times restart with some
    /// sessions, and an export starts at the first group with the time
    fn proposal(&self) -> Option<(f32, f32)> {
        let m = self.model.as_ref()?;
        let (first, last) = match self.chosen_span() {
            Some(span) => span,
            None => (self.selected?, self.selected?),
        };
        let (first, last) = (m.events.get(first)?, m.events.get(last)?);
        if first.segment != last.segment {
            return None;
        }

        let range = self.clamped(first.time - self.padding.0, last.time + self.padding.1);
        self.starts_in(range.0, first.segment).then_some(range)
    }

    /// The first and last of several chosen events that show, none with
    /// fewer than two chosen. Events are kept in time order, so their
    /// places say which comes first
    fn chosen_span(&self) -> Option<(usize, usize)> {
        if self.marked.len() < 2 {
            return None;
        }

        let shown = |i: &usize| self.shown_events.get(*i).copied().unwrap_or(false);
        let first = *self.marked.iter().find(|i| shown(i))?;
        let last = *self.marked.iter().rev().find(|i| shown(i))?;
        Some((first, last))
    }

    /// What Clip it clips, the highlight when it was chosen after the event
    /// and otherwise the event, none when that cannot start where it should
    pub(super) fn clip_it_range(&self) -> Option<(f32, f32)> {
        if self.highlight_last && self.window.is_some() {
            return self.highlight_range();
        }
        self.proposal()
    }

    /// Whether Clip it clips the chosen highlight rather than an event
    pub(super) fn clips_highlight(&self) -> bool {
        self.highlight_last && self.window.is_some()
    }

    /// The range clipping the chosen highlight would take, padded as an
    /// event's is, when it starts in the highlight's own session
    fn highlight_range(&self) -> Option<(f32, f32)> {
        let m = self.model.as_ref()?;
        let w = m.windows.get(self.window?)?;
        let range = self.clamped(w.start - self.padding.0, w.end + self.padding.1);
        self.starts_in(range.0, w.segment).then_some(range)
    }

    /// Whether a clip starting at `time` starts in `segment`
    fn starts_in(&self, time: f32, segment: u32) -> bool {
        let Some(m) = &self.model else {
            return false;
        };
        DemoTime::from_f32(time).is_some_and(|t| m.groups.segment_at(t) == Some(segment))
    }

    /// A range kept within the recording
    fn clamped(&self, from: f32, to: f32) -> (f32, f32) {
        let last = self
            .model
            .as_ref()
            .map_or(f32::INFINITY, |m| m.groups.end.unwrap_or(m.end));
        let from = from.clamp(0., last);
        (from, to.clamp(from, last))
    }

    /// Add a clip, named after the next free number, and choose it
    fn add_clip(&mut self, catalog: &Catalog, (from, to): (f32, f32)) {
        let (Some(start), Some(end)) = (DemoTime::from_f32(from), DemoTime::from_f32(to)) else {
            return;
        };
        // A name is kept and typed in, so it holds none of the marks that
        // keep a message's number apart from its text
        let name = next_name(&self.clips, |n| {
            let name = catalog.text_with("clip-default-name", &[("number", n.into())]);
            name.replace(['\u{2068}', '\u{2069}'], "")
        });
        self.identify_clips();
        self.clips.push(Clip { name, start, end });
        self.identify_clips();
        self.chosen_clip = Some(self.clips.len() - 1);
        self.clips_changed = true;
    }

    /// What an export of a clip would cut, or why it would refuse
    fn snap(&mut self, clip: &Clip) -> Result<Snapped, Message> {
        let Some(m) = &self.model else {
            return Err(Message::new("export-empty-range", String::new()));
        };
        self.snapped
            .entry((clip.start, clip.end))
            .or_insert_with(|| {
                m.groups
                    .snap(clip.start, Some(clip.end))
                    .map_err(|e| e.message())
            })
            .clone()
    }

    /// Bring a clip into view with a little room on both sides
    fn show_clip(&mut self, i: usize) {
        let Some(c) = self.clips.get(i) else {
            return;
        };
        let (s, e) = (c.start.secs_f64() as f32, c.end.secs_f64() as f32);
        let pad = ((e - s) * 0.1).max(2.);
        self.view = track::clamp_view((s - pad, e + pad), self.extent());
        self.chosen_clip = Some(i);
    }

    /// What an event was, in words
    pub fn describe(catalog: &Catalog, kind: &EventKind) -> String {
        match kind {
            EventKind::Action { action, mode } => {
                let a = catalog.text(match action {
                    Action::Swing => "action-swing",
                    Action::Fire => "action-fire",
                    Action::Launch => "action-launch",
                    Action::ChargeActivity => "action-charge-activity",
                    Action::ChargeStop => "action-charge-stop",
                    Action::BeamStart => "action-beam-start",
                    Action::BeamSustain => "action-beam-sustain",
                    Action::BeamStop => "action-beam-stop",
                    Action::PlacementAttempt => "action-placement-attempt",
                    Action::ReleaseAttempt => "action-release-attempt",
                });
                match mode {
                    Some(m) => {
                        let mode = catalog.text_or(&format!("mode-{m}"), m);
                        format!("{a} · {mode}")
                    }
                    None => a,
                }
            }
            EventKind::Kill {
                victim, headshot, ..
            } => {
                let killed = catalog.text_with(
                    "event-kill",
                    &[(
                        "victim",
                        FluentValue::from(
                            victim
                                .as_deref()
                                .map_or_else(|| "?".into(), text::player_name),
                        ),
                    )],
                );
                if *headshot == Some(true) {
                    format!("{killed} · {}", catalog.text("event-headshot"))
                } else {
                    killed
                }
            }
            EventKind::Switch => catalog.text("event-switch"),
            EventKind::Chat { text, .. } => text::without_colours(text),
            EventKind::Match(change) => describe_match(catalog, change),
        }
    }

    /// Name the places the analysis's lines of chat stand for, once the
    /// viewer reads the game's files
    fn name_chat_places(&mut self) {
        if self.chat_lines.is_some() {
            return;
        }
        let (Some(model), Some(viewer)) = (&self.model, &mut self.viewer) else {
            return;
        };
        if !viewer.reads_game_files() {
            return;
        }

        let mut lines = HashMap::new();
        for (index, event) in model.events.iter().enumerate() {
            let EventKind::Chat { text, place } = &event.kind else {
                continue;
            };
            let map = model.maps.iter().find(|m| m.segment == event.segment);
            let name = match (map, place) {
                (Some(map), Some(at)) => viewer.place_name(&map.name, *at),
                _ => None,
            };
            lines.insert(index, chat::with_place(text, &name.unwrap_or_default()));
        }
        self.chat_lines = Some(lines);
    }

    /// What the event at `index` was, in words, a line of chat with its
    /// sender's place named once the game's files are read
    pub(super) fn what(&self, catalog: &Catalog, index: usize) -> String {
        let Some(event) = self.model.as_ref().and_then(|m| m.events.get(index)) else {
            return String::new();
        };
        if !matches!(event.kind, EventKind::Chat { .. }) {
            return Self::describe(catalog, &event.kind);
        }

        let line = match self.chat_lines.as_ref().and_then(|l| l.get(&index)) {
            Some(line) => text::without_colours(line),
            None => Self::describe(catalog, &event.kind),
        };
        said(&line, event.actor.as_deref())
    }

    /// Take in the analysis, export results and playback state, and the
    /// keys that drive playback when `keys` says they are the demo's. Called
    /// once a frame before anything of the demo draws, whichever of its
    /// parts are shown
    pub fn begin_frame(&mut self, ctx: &egui::Context, keys: bool) {
        self.poll();
        self.poll_export();

        self.identify_clips();
        if self.export_on_load && self.model.is_some() {
            self.export_on_load = false;
            let mut eligible = Vec::new();
            for (i, clip) in self.clips.clone().iter().enumerate() {
                if self.snap(clip).is_ok() {
                    eligible.push(i);
                }
            }
            self.start_export(ctx, &eligible);
        }

        // Which Gauss kills went through a wall, once the analysis is in,
        // with the game's files the viewer reads
        if let Some(model) = self.model.clone() {
            let install = self.viewer_game.clone();
            let viewer = self.viewer.as_ref();
            let hulls = || viewer.and_then(|v| v.map_hulls());
            self.walls
                .update(&self.path, &model, install.as_deref(), hulls, ctx);
        }
        self.walls.poll();

        // The viewer names the map an intermission changes to from the
        // sessions the analysis found
        if let (Some(model), Some(viewer)) = (&self.model, &mut self.viewer)
            && viewer.sessions != model.maps
        {
            viewer.sessions = model.maps.clone();
        }
        if let (Some(model), Some(viewer)) = (&self.model, &mut self.viewer)
            && viewer.standings != model.standings
        {
            viewer.standings = model.standings.clone();
        }
        self.name_chat_places();

        // The event the command line asked for, once there are rows
        if self.model.is_some()
            && let Some(row) = self.open_event.take()
        {
            self.refresh_rows();
            if let Some(&event) = self.rows.get(row) {
                self.actions.push(DemoAction::Select {
                    event,
                    from_list: false,
                });
            }
        }

        if keys {
            if let Some(viewer) = &mut self.viewer {
                viewer.shortcuts(ctx);
            }
            self.shortcuts(ctx);
        }
        if let Some(viewer) = &mut self.viewer {
            viewer.begin_frame(ctx);
        }
    }

    /// Ctrl+Left and Ctrl+Right go to the shown event before or after, and
    /// I and O set the chosen clip's start and end to the current moment.
    /// While text is typed every key is the text's
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if text::typing(ctx) {
            return;
        }

        let command = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, key));
        let back = command(egui::Key::ArrowLeft);
        let forward = command(egui::Key::ArrowRight);
        if (back || forward)
            && let Some(model) = self.model.clone()
        {
            let (previous, next) = self.neighbours(&model);
            let row = if back { previous } else { next };
            if let Some(&event) = row.and_then(|r| self.rows.get(r)) {
                self.actions.push(DemoAction::Select {
                    event,
                    from_list: false,
                });
            }
        }

        // Only the clip chosen, never a new one
        let plain = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
        let edge = if plain(egui::Key::I) {
            Some(Edge::Start)
        } else if plain(egui::Key::O) {
            Some(Edge::End)
        } else {
            None
        };
        if let (Some(edge), Some(i)) = (edge, self.chosen_clip) {
            self.edge_to_playhead(i, edge);
        }
    }

    /// Apply what the controls asked for, then bring the loop and the sound
    /// in line with the edits. Called once a frame after everything has
    /// drawn. Returns what was asked of the app
    pub fn finish_frame(&mut self, ctx: &egui::Context, catalog: &Catalog) -> Vec<Request> {
        let actions = std::mem::take(&mut self.actions);
        for action in &actions {
            self.apply(*action, catalog);
        }

        // What an action changed shows on the next frame, asked for here as
        // nothing else may come to ask for one
        if !actions.is_empty() {
            ctx.request_repaint();
        }

        // A looped clip's edits move its loop too
        if let (Some(c), Some(viewer)) = (
            self.looping.and_then(|i| self.clips.get(i)),
            &mut self.viewer,
        ) {
            viewer.set_loop(Some((secs(c.start), secs(c.end))));
        }

        if let Some(viewer) = &mut self.viewer {
            viewer.finish_frame(ctx);
        }

        std::mem::take(&mut self.requests)
    }

    fn apply(&mut self, action: DemoAction, catalog: &Catalog) {
        match action {
            DemoAction::Select { event, from_list } => {
                self.select(event, from_list);

                // An event chosen elsewhere shows in its list
                if !from_list {
                    self.wanted_tab = Some(InspectorTab::Events);
                }
            }
            DemoAction::Seek(at) => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.seek(at);
                }

                let t = DemoTime::from_f32(at);
                if let Some(i) = self
                    .clips
                    .iter()
                    .rposition(|c| t.is_some_and(|t| t >= c.start && t < c.end))
                {
                    self.chosen_clip = Some(i);
                }
            }
            DemoAction::PlayEvent(event) => {
                // Played from the wrong session's moment is not this event
                if !self.reachable(event) {
                    return;
                }

                let Some(e) = self.model.as_ref().and_then(|m| m.events.get(event)) else {
                    return;
                };
                let range = (e.time - self.padding.0, e.time + self.padding.1);

                let players = self.event_players(event);
                if let Some(viewer) = &mut self.viewer {
                    // Played once, through any clip's loop
                    self.looping = None;
                    viewer.set_loop(None);
                    viewer.play_range(range.0, range.1);
                    viewer.follow_while_played(players, range);
                }
            }
            DemoAction::Fit => {
                self.view = self.extent();
                self.window = None;
            }
            DemoAction::Zoom(factor) => {
                let (start, end) = self.view;
                let playhead = self.viewer.as_ref().map(|v| v.time());
                let anchor = match playhead {
                    Some(t) if t >= start && t <= end => t,
                    _ => (start + end) / 2.,
                };
                self.view = track::zoom(self.view, self.extent(), anchor, factor);
            }
            DemoAction::ChooseClip(i) => {
                if i < self.clips.len() {
                    self.chosen_clip = Some(i);
                    self.wanted_tab = Some(InspectorTab::Clips);
                }
            }
            DemoAction::ChooseHighlight(i) => {
                let Some(w) = self.model.as_ref().and_then(|m| m.windows.get(i)) else {
                    return;
                };
                let (start, end, segment) = (w.start, w.end, w.segment);

                // The minute with a little room on both sides
                let pad = ((end - start) * 0.1).max(3.);
                self.view = track::clamp_view((start - pad, end + pad), self.extent());
                self.window = Some(i);
                self.highlight_last = true;
                self.scroll_to_window = 3;

                // To its start, unless an earlier session has the same time
                if self.starts_in(start, segment)
                    && let Some(viewer) = &mut self.viewer
                {
                    viewer.seek(start);
                }
            }
            DemoAction::NewClip => {
                // Any range, for a demo with no events or moments to start
                // from, ten seconds from the playhead, or from where the
                // timeline shows without a viewer. A range the recording's
                // end leaves empty says why it cannot be exported
                let from = match &self.viewer {
                    Some(viewer) => viewer.time(),
                    None => self.view.0,
                };
                let range = self.clamped(from, from + NEW_CLIP_SECONDS);
                self.add_clip(catalog, range);
                self.wanted_tab = Some(InspectorTab::Clips);
            }
            DemoAction::ClipEvent => {
                if let Some(range) = self.clip_it_range() {
                    self.add_clip(catalog, range);
                    self.wanted_tab = Some(InspectorTab::Clips);
                }
            }
            DemoAction::ClipHighlight => {
                if let Some(range) = self.highlight_range() {
                    self.add_clip(catalog, range);
                    self.wanted_tab = Some(InspectorTab::Clips);
                }
            }
            DemoAction::ShowFinal => {
                // Until the playhead moves again, or without a viewer until
                // another event is chosen
                self.scores_live = false;
                self.scores_still = None;
                if self.viewer.is_none() {
                    self.selected = None;
                }
            }
        }
    }

    /// Keep a clip time still being typed when it reads as a time, as
    /// leaving its field would, for when the field goes out of view with
    /// its tab or its demo. Text that is not a time stays as typed, outlined
    /// when the field shows again, and the clip keeps its time
    pub fn commit_typing(&mut self) {
        let Some((i, edge, text)) = &self.typing else {
            return;
        };
        let (Some(clip), Some(t)) = (self.clips.get_mut(*i), parse_time(text)) else {
            return;
        };

        let edge = *edge;
        self.typing = None;

        let current = match edge {
            Edge::Start => &mut clip.start,
            Edge::End => &mut clip.end,
        };
        if *current != t {
            *current = t;
            self.clips_changed = true;
        }
    }

    /// The whole demo, its inspector beside the view or under it in a
    /// narrow window, and the view between its header and its transport,
    /// or the view alone while it fills the screen. What the controls ask for is applied by [`Self::finish_frame`]
    pub fn ui(&mut self, ui: &mut Ui, s: &Surroundings, workspace: &mut WorkspaceUi) {
        let (catalog, rtl) = (s.catalog, s.rtl);
        self.refresh_rows();

        if let Some(viewer) = &mut self.viewer
            && viewer.fullscreen
        {
            fullscreen_ui(viewer, ui, catalog, rtl, &mut self.requests);
            return;
        }

        // The inspector beside the view, or under it in a narrow window
        let frame = egui::Frame::new()
            .fill(theme::BACKGROUND)
            .inner_margin(theme::GAP_S as i8);
        if s.narrow {
            let room = ui.available_height() - MIN_VIEWPORT - self.around_view;
            let most = room.max(INSPECTOR_MIN_HEIGHT);
            egui::Panel::bottom(format!("inspector below {}", workspace.restored))
                .frame(frame)
                .resizable(true)
                .default_size(INSPECTOR_HEIGHT)
                .size_range(INSPECTOR_MIN_HEIGHT..=most)
                .show(ui, |ui| self.inspector(ui, catalog, rtl, workspace));
        } else {
            let width = if ui.available_width() >= WIDE_WINDOW {
                INSPECTOR_WIDE
            } else {
                INSPECTOR_WIDTH
            };
            let id = format!("inspector {}", workspace.restored);
            let shown = layout::trailing_panel(&id, rtl)
                .frame(frame)
                .resizable(true)
                .default_size(workspace.inspector_width.unwrap_or(width))
                .size_range(INSPECTOR_RANGE)
                .show(ui, |ui| self.inspector(ui, catalog, rtl, workspace));

            // As wide as it was dragged, for the app to know how much room
            // the view has left
            workspace.inspector_width = Some(shown.response.rect.width());
            workspace.inspector_first = width;
        }

        self.middle(ui, s, workspace);
    }

    /// The header over the view, and the transport and player lanes under
    /// it, kept room for before the view takes what is left
    fn middle(&mut self, ui: &mut Ui, s: &Surroundings, workspace: &WorkspaceUi) {
        let (catalog, rtl) = (s.catalog, s.rtl);
        let most_lanes = ui.available_height() * LANES_SHARE;
        let margin = egui::Margin::symmetric(theme::GAP_S as i8, 0);

        let header = egui::Frame::new().fill(theme::SURFACE).inner_margin(margin);
        let header = egui::Panel::top("demo header")
            .exact_size(theme::TOOLBAR_HEIGHT)
            .frame(header)
            .show(ui, |ui| self.header(ui, s, workspace));

        let transport = egui::Frame::new().inner_margin(margin);
        let transport = egui::Panel::bottom("transport")
            .frame(transport)
            .show(ui, |ui| {
                // The seek bar spans the picture above it, past the margin
                if let Some(viewer) = &mut self.viewer {
                    viewer.seek_bar_ui(ui, catalog, f32::from(margin.left));
                }
                if let Some(viewer) = &mut self.viewer
                    && viewer.transport_ui(ui, catalog, rtl)
                {
                    self.requests.push(Request::Settings(SettingsTab::Viewer));
                }

                self.timeline_ui(ui, catalog, rtl, most_lanes);
            });
        self.around_view = header.response.rect.height() + transport.response.rect.height();

        // The view takes what is left, the setup notice in its place while
        // the game's files are not found
        let height = ui.available_height().max(MIN_VIEWPORT);
        match &mut self.viewer {
            Some(viewer) if viewer.has_assets() => viewer.viewport_ui(ui, catalog, rtl, height),
            _ => self.setup_notice(ui, s, height),
        }
    }

    /// The demo's name and facts, its actions at the trailing end, and the
    /// way back to the demo list leading while the list is out of view
    fn header(&mut self, ui: &mut Ui, s: &Surroundings, workspace: &WorkspaceUi) {
        let (catalog, rtl) = (s.catalog, s.rtl);
        let name = demo_name(&self.path);
        let facts = self.facts(s);

        layout::row(ui, rtl, |ui| {
            if workspace.list_out_of_view() {
                let label = catalog.text("list-show");
                let show = IconButton::new(Icon::PanelOpen, &label).mirrored(rtl);
                if ui.add(show).clicked() {
                    self.requests.push(Request::ShowList);
                }
            }

            // The name and its facts take what the actions leave, cut short
            // rather than pushing them away
            let action = theme::CONTROL_HEIGHT + ui.spacing().item_spacing.x;
            let room = (ui.available_width() - action * HEADER_ACTIONS as f32).max(0.);
            layout::cell(ui, rtl, room, |ui| {
                let path = self.path.display().to_string();
                let title =
                    text::widget_text(ui.ctx(), &name, text::starts_rtl(&name), None, Look::Strong);
                ui.add(egui::Label::new(title).truncate())
                    .on_hover_text(path);

                let look = Look::Color(theme::MUTED);
                let facts = text::styled_text(ui.ctx(), &facts, rtl, look, &theme::secondary());
                ui.add(egui::Label::new(facts).truncate());
            });

            layout::trailing(ui, rtl, |ui| self.header_actions(ui, s));
        });
    }

    /// Close, show in folder, copy the path, the details and the favourite's
    /// star, placed from the trailing end
    fn header_actions(&mut self, ui: &mut Ui, s: &Surroundings) {
        let catalog = s.catalog;

        let close = catalog.text("demo-close");
        if ui.add(IconButton::new(Icon::Close, &close)).clicked() {
            self.requests.push(Request::Close);
        }

        let show = catalog.text("path-show");
        if ui.add(IconButton::new(Icon::Folder, &show)).clicked()
            && let Err(e) = crate::platform::show_in_folder(&self.path)
        {
            let detail = FluentValue::from(e.to_string());
            let notice = catalog.text_with("path-show-failed", &[("detail", detail)]);
            self.requests.push(Request::Notice(notice));
        }

        // The whole path, however the header cuts it short
        let copy = catalog.text("path-copy");
        if ui.add(IconButton::new(Icon::Copy, &copy)).clicked() {
            ui.ctx().copy_text(self.path.display().to_string());
        }

        let details = catalog.text("demo-details");
        let response = ui.add(IconButton::new(Icon::Info, &details));
        egui::Popup::menu(&response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| self.details(ui, s));

        // Filled in the favourite's yellow while the demo is one
        let id = if s.favourite {
            "menu-favourite-remove"
        } else {
            "menu-favourite-add"
        };
        let response = ui.add(IconButton::new(Icon::Star, &catalog.text(id)));
        if s.favourite {
            let radius = theme::ICON_SIZE * 0.4;
            widgets::paint_star(ui.painter(), response.rect.center(), radius, widgets::STAR);
        }
        if response.clicked() {
            let path = match s.file {
                Some(file) => file.path.clone(),
                None => self.path.display().to_string(),
            };
            self.requests.push(Request::Favourite(path));
        }
    }

    /// The map, date, length and point of view, as far as they are known
    fn facts(&self, s: &Surroundings) -> String {
        let catalog = s.catalog;
        let mut facts = Vec::new();
        if let Some(map) = s.file.and_then(|f| f.map.clone()) {
            facts.push(map);
        }
        if let Some(day) = s
            .file
            .and_then(|f| f.recorded_at.as_deref())
            .and_then(crate::i18n::Ymd::parse)
        {
            facts.push(catalog.date(day));
        }

        // The analysis's extent once it is in, the library's until then
        let length = match &self.model {
            Some(model) => Some(model.end),
            None => s.file.and_then(|f| f.duration),
        };
        if let Some(length) = length.and_then(DemoTime::from_f32) {
            let length = length.to_string();
            facts.push(length.split('.').next().unwrap_or("").to_string());
        }

        if let Some(kind) = s.file.and_then(|f| f.recording_kind.as_deref()) {
            facts.push(catalog.text(&format!("kind-{}", kind.replace('_', "-"))));
        }

        facts.join(" · ")
    }

    /// Everything the library knows of the demo, for the details popup
    fn details(&self, ui: &mut Ui, s: &Surroundings) {
        let (catalog, rtl) = (s.catalog, s.rtl);
        ui.set_max_width(text::HELP_WIDTH);
        ui.label(egui::RichText::new(self.path.display().to_string()).monospace());

        let Some(f) = s.file else {
            return;
        };

        let mut facts = Vec::new();
        if let Some(origin) = &f.clip_of {
            facts.push(crate::library::clip_of(origin, catalog));
        }
        if let Some(r) = &f.recorder {
            let name = FluentValue::from(text::player_name(r));
            facts.push(catalog.text_with("details-recorder", &[("name", name)]));
        }
        if let Some(k) = &f.recording_kind {
            facts.push(catalog.text(&format!("kind-{}", k.replace('_', "-"))));
        }
        if let Some(n) = f.shots {
            facts.push(catalog.text_with("details-shots", &[("count", FluentValue::from(n))]));
        }
        if let Some(n) = f.kills {
            facts.push(catalog.text_with("details-kills", &[("count", FluentValue::from(n))]));
        }
        if let Some(score) = f.best_window {
            let score = FluentValue::from(score);
            facts.push(catalog.text_with("details-action", &[("score", score)]));
        }

        for fact in facts {
            text::label(ui, &fact, rtl);
        }
        if let Some(e) = &f.error {
            text::label(ui, &catalog.stored(e), rtl);
        }
    }

    /// In the view's place while the game's files are not found, what the
    /// viewer needs them for and the way to give them, and that the rest
    /// works without them
    fn setup_notice(&mut self, ui: &mut Ui, s: &Surroundings, height: f32) {
        let (catalog, rtl) = (s.catalog, s.rtl);
        let size = Vec2::new(ui.available_width(), height);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().rect_filled(rect, 0., theme::VIEWPORT);

        // Without a GPU there is no viewer, and nothing the game's files
        // would change
        if self.viewer.is_none() {
            return;
        }

        // Centred by the height it took last frame, as that is only known
        // once it is laid out
        let height_id = ui.scope_id().with(("setup notice", self.generation));
        let last_height: f32 = ui.ctx().data(|d| d.get_temp(height_id)).unwrap_or(0.);
        let top = rect.min.y + ((rect.height() - last_height) / 2.).max(theme::GAP_M);
        let width = SETUP_WIDTH.min(rect.width() - theme::GAP_M * 2.);
        let area = Rect::from_min_max(
            Pos2::new(rect.center().x - width / 2., top),
            Pos2::new(rect.center().x + width / 2., rect.max.y),
        );
        let layout = egui::Layout::top_down(egui::Align::Center);
        let mut notice = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(layout));
        notice.set_clip_rect(rect);

        // A short view keeps only the title and the way to set up
        let compact = rect.height() < SETUP_FULL_HEIGHT;

        let mut locate = false;
        let mut settings = false;

        if !compact {
            warning_mark(&mut notice);
        }

        let title = catalog.text("setup-title");
        let title = text::styled_text(
            notice.ctx(),
            &title,
            rtl,
            Look::Strong,
            &egui::TextStyle::Heading,
        );
        notice.label(title);

        if !compact {
            let muted = Look::Color(theme::MUTED);
            let body = text::widget_text(
                notice.ctx(),
                &catalog.text("setup-body"),
                rtl,
                Some(SETUP_WIDTH),
                muted,
            );
            notice.add(egui::Label::new(body).wrap());
        }

        // A folder the settings name that no longer holds the game
        if let Some(folder) = &s.setup.missing {
            let folder = FluentValue::from(folder.display().to_string());
            let line = catalog.text_with("setup-relocate", &[("folder", folder)]);
            let line = text::widget_text(
                notice.ctx(),
                &line,
                rtl,
                Some(SETUP_WIDTH),
                Look::Color(theme::WARNING),
            );
            notice.add(egui::Label::new(line).wrap());
        }

        let id = if cfg!(windows) {
            "setup-locate-exe"
        } else {
            "setup-locate-folder"
        };
        let buttons =
            egui::Layout::left_to_right(egui::Align::Center).with_main_align(egui::Align::Center);
        let row = Vec2::new(notice.available_width(), theme::CONTROL_HEIGHT);
        notice.allocate_ui_with_layout(row, buttons, |ui| {
            let label = text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain);
            let primary = widgets::Button::new(Some(Icon::Folder), label, Variant::Primary, rtl);
            locate = ui.add(primary).clicked();
            let label = text::widget_text(
                ui.ctx(),
                &catalog.text("setup-settings"),
                rtl,
                None,
                Look::Plain,
            );
            let ghost = widgets::Button::new(None, label, Variant::Ghost, rtl);
            settings = ui.add(ghost).clicked();
        });

        // Why the last choice was refused, beside the way to choose again
        if let Some(refusal) = &s.setup.refusal {
            let line = text::widget_text(
                notice.ctx(),
                refusal,
                rtl,
                Some(SETUP_WIDTH),
                Look::Color(theme::ERROR),
            );
            notice.add(egui::Label::new(line).wrap());
        }

        if !compact {
            let still = catalog.text("setup-still-works");
            let still = text::widget_text(
                notice.ctx(),
                &still,
                rtl,
                Some(SETUP_WIDTH),
                Look::Color(theme::FAINT),
            );
            notice.add(egui::Label::new(still).wrap());
        }

        let height = notice.min_rect().height();
        if height != last_height {
            ui.ctx().data_mut(|d| d.insert_temp(height_id, height));
            ui.ctx().request_repaint();
        }

        if locate {
            self.requests.push(Request::Locate);
        }
        if settings {
            self.requests.push(Request::Settings(SettingsTab::General));
        }
    }

    /// The scoreboard over the events and clips, sharing the panel's height
    /// half each or one taking it all while the other is folded to its
    /// header
    fn inspector(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        workspace: &mut WorkspaceUi,
    ) {
        // A list asked for unfolds
        if let Some(tab) = self.wanted_tab.take() {
            self.choose_tab(workspace, tab);
            if workspace.split == PaneSplit::Scores {
                workspace.split = PaneSplit::Even;
            }
        }

        // A folded pane keeps the room for its header, to be unfolded by it.
        // Sharing the panel, the scoreboard has the share it was dragged to,
        // or what it needs up to half
        let whole = ui.available_rect_before_wrap();
        let header = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
        let usable = (whole.height() - PANE_GAP).max(2. * header);
        let shared = match (workspace.scores_share, self.scores_needed) {
            (Some(share), _) => usable * share,
            (None, Some(needed)) => needed.min(usable / 2.),
            (None, None) => usable / 2.,
        };
        let upper = match workspace.split {
            PaneSplit::Even => shared.clamp(header, usable - header),
            PaneSplit::Scores => usable - header,
            PaneSplit::Lists => header,
        };
        let upper_rect = Rect::from_min_size(whole.min, Vec2::new(whole.width(), upper));
        let mut lower_rect = whole;
        lower_rect.min.y = upper_rect.max.y + PANE_GAP;

        // The line between the panes drags to share the panel another way,
        // sharing it again when one was folded, and a double click goes back
        // to what the scoreboard needs
        let gap = Rect::from_x_y_ranges(whole.x_range(), upper_rect.max.y..=lower_rect.min.y);
        let divider = ui
            .interact(
                gap,
                ui.scope_id().with("pane divider"),
                Sense::click_and_drag(),
            )
            .on_hover_cursor(egui::CursorIcon::ResizeVertical);
        if divider.dragged()
            && let Some(pointer) = divider.interact_pointer_pos()
        {
            let height = (pointer.y - whole.min.y - PANE_GAP / 2.).clamp(header, usable - header);
            workspace.scores_share = Some(height / usable);
            workspace.split = PaneSplit::Even;
        }
        if divider.double_clicked() {
            workspace.scores_share = None;
        }

        // The line runs from edge to edge of the panel, and each pane's
        // header is on a band of its own, so the lower one's header never
        // reads as part of the scores above it
        let divide = upper_rect.max.y + PANE_GAP / 2.;
        let across = whole.x_range().expand(theme::GAP_S);
        let colour = if divider.hovered() || divider.dragged() {
            theme::MUTED
        } else {
            theme::HOVER
        };
        ui.painter()
            .hline(across, divide, egui::Stroke::new(1., colour));
        let band_height = ui.spacing().interact_size.y;
        for top in [upper_rect.min, lower_rect.min] {
            let band = Rect::from_min_size(top, Vec2::new(whole.width(), band_height));
            ui.painter()
                .rect_filled(band, theme::RADIUS_CONTROL, theme::SURFACE);
        }

        let mut scores = ui.new_child(egui::UiBuilder::new().max_rect(upper_rect));
        scores.set_clip_rect(upper_rect.intersect(ui.clip_rect()));
        self.scores_pane(&mut scores, catalog, rtl, workspace);

        let mut lists = ui.new_child(egui::UiBuilder::new().max_rect(lower_rect));
        lists.set_clip_rect(lower_rect.intersect(ui.clip_rect()));
        self.lists_pane(&mut lists, catalog, rtl, workspace);

        // The panes fill the whole panel, so it keeps its height whatever
        // they hold
        ui.allocate_rect(whole, Sense::hover());
    }

    /// The scoreboard's header, which folds or unfolds it, and the scores
    /// under it while open
    fn scores_pane(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        workspace: &mut WorkspaceUi,
    ) {
        let open = workspace.split != PaneSplit::Lists;
        let full = workspace.split == PaneSplit::Scores;
        layout::row(ui, rtl, |ui| {
            let mut clicked = fold_button(ui, catalog, open, full, rtl).clicked();
            let title = catalog.text("tab-scoreboard");
            let title = text::widget_text(ui.ctx(), &title, rtl, None, Look::Plain);
            clicked |= ui.add(egui::Button::selectable(false, title)).clicked();
            if clicked {
                workspace.split = workspace.split.clicked(true);
            }
        });
        if !open {
            return;
        }

        // All the scores would fill this much of the panel, scrolled or not
        ui.separator();
        let hidden = self.scores_tab(ui, catalog, rtl);
        self.scores_needed = Some(ui.min_rect().height() + hidden);
    }

    /// The events and clips' tabs with their counts, after the button that
    /// folds or unfolds them, and the chosen one under them while open. A
    /// tab chosen while folded unfolds them
    fn lists_pane(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        workspace: &mut WorkspaceUi,
    ) {
        let open = workspace.split != PaneSplit::Scores;
        let full = workspace.split == PaneSplit::Lists;
        let events = self.model.as_ref().map(|_| self.rows.len());
        let tabs = [
            (InspectorTab::Events, "tab-events", events),
            (InspectorTab::Clips, "tab-clips", Some(self.clips.len())),
        ];
        layout::row(ui, rtl, |ui| {
            if fold_button(ui, catalog, open, full, rtl).clicked() {
                workspace.split = workspace.split.clicked(false);
            }
            for (tab, id, count) in tabs {
                let chosen = workspace.inspector_tab == tab;
                let title = text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain);
                if ui.add(egui::Button::selectable(chosen, title)).clicked() {
                    self.choose_tab(workspace, tab);
                    if !open {
                        workspace.split = PaneSplit::Even;
                    }
                }
                if let Some(count) = count {
                    let count = egui::RichText::new(count.to_string())
                        .monospace()
                        .color(theme::FAINT);
                    ui.label(count);
                }
            }
        });
        if !open {
            return;
        }

        ui.separator();
        if !self.analysis_ready(ui, catalog, rtl) {
            return;
        }
        match workspace.inspector_tab {
            InspectorTab::Events => {
                let filters_open = &mut workspace.event_filters_open;
                self.events_tab(ui, catalog, rtl, filters_open);
            }
            InspectorTab::Clips => self.clips_tab(ui, catalog, rtl),
        }
    }

    /// Show another of the inspector's tabs. A clip time still being typed
    /// is kept first, as its field goes out of view before it could lose
    /// the focus and keep it itself, and an export would read the old time
    fn choose_tab(&mut self, workspace: &mut WorkspaceUi, tab: InspectorTab) {
        if workspace.inspector_tab == InspectorTab::Clips && tab != InspectorTab::Clips {
            self.commit_typing();
        }
        workspace.inspector_tab = tab;
    }

    /// Whether the analysis is in, and while it is not, that it is under
    /// way or why it failed
    fn analysis_ready(&self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> bool {
        if self.loader.is_some() {
            let name = FluentValue::from(demo_name(&self.path));
            let line = catalog.text_with("demo-loading", &[("name", name)]);
            layout::row(ui, rtl, |ui| {
                ui.spinner();
                ui.label(text::widget_text(ui.ctx(), &line, rtl, None, Look::Plain));
            });
            return false;
        }

        if let Some(code) = &self.failed {
            text::label(
                ui,
                &catalog.text_or(code, &catalog.text("status-failed")),
                rtl,
            );
            return false;
        }

        self.model.is_some()
    }
}

/// A demo's file name without the `.dem` every demo has
/// The view over the whole of `ui`, with the seek bar and the transport
/// over its bottom while the pointer is about, as video players have
fn fullscreen_ui(
    viewer: &mut crate::viewer::Viewer,
    ui: &mut Ui,
    catalog: &Catalog,
    rtl: bool,
    requests: &mut Vec<Request>,
) {
    let ctx = ui.ctx().clone();
    let whole = ui.max_rect();
    viewer.viewport_ui(ui, catalog, rtl, whole.height());

    // Where the controls were last drawn, which the pointer keeps them over
    let id = egui::Id::unique("fullscreen controls");
    let last = ctx
        .memory(|m| m.area_rect(id))
        .unwrap_or(egui::Rect::NOTHING);
    if !viewer.fullscreen_controls_shown(&ctx, last) {
        return;
    }

    let margin = egui::Margin::symmetric(theme::GAP_S as i8, theme::GAP_S as i8);
    let frame = egui::Frame::new()
        .fill(egui::Color32::from_black_alpha(FULLSCREEN_SHADE))
        .inner_margin(margin);
    egui::Area::new(id)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::ZERO)
        .show(&ctx, |ui| {
            frame.show(ui, |ui| {
                ui.set_width(whole.width() - margin.sum().x);
                viewer.seek_bar_ui(ui, catalog, f32::from(margin.left));
                if viewer.transport_ui(ui, catalog, rtl) {
                    requests.push(Request::Settings(SettingsTab::Viewer));
                }
            });
        });
}

fn demo_name(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    match name.strip_suffix(".dem") {
        Some(stem) if !stem.is_empty() => stem.to_string(),
        _ => name,
    }
}

/// A warning sign in a soft circle, heading the setup notice
fn warning_mark(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(SETUP_ICON), Sense::hover());
    let painter = ui.painter();
    painter.circle_filled(
        rect.center(),
        SETUP_ICON / 2.,
        theme::WARNING.gamma_multiply(0.15),
    );

    let glyph = Rect::from_center_size(rect.center(), Vec2::splat(SETUP_GLYPH));
    widgets::paint_icon(painter, glyph, Icon::Warning, theme::WARNING);
}

#[cfg(test)]
mod tests {
    use lambda_slicer_core::analysis::context::KillCause;
    use lambda_slicer_core::analysis::match_events::Tally;
    use lambda_slicer_core::analysis::motion::MotionEvidence;
    use lambda_slicer_core::slice::range::{GroupIndex, GroupStart};
    use lambda_slicer_playback::kill_features::{OtherRoute, WallDetails, WallGauss, WallRoute};

    use super::*;
    use crate::kill_filters::MidairMode;
    use crate::timeline::{KillFeatures, KillKey, KillMotion, SampledAt, Window};

    /// A demo view of a file that is not there, with one clip from 1 to 2
    /// seconds whose start is being typed as `typed`
    fn view_typing(typed: &str) -> DemoView {
        let ctx = egui::Context::default();
        let mut view = DemoView::open(PathBuf::from("not-there.dem"), 10., None, &ctx);
        view.clips.push(Clip {
            name: "Clip 1".into(),
            start: DemoTime::from_micros(1_000_000),
            end: DemoTime::from_micros(2_000_000),
        });
        view.typing = Some((0, Edge::Start, typed.into()));
        view
    }

    #[test]
    fn a_click_on_either_pane_from_half_each_gives_it_all_and_any_click_after_halves_again() {
        let even = PaneSplit::Even;
        assert_eq!(even.clicked(true), PaneSplit::Scores);
        assert_eq!(even.clicked(false), PaneSplit::Lists);

        // The open pane clicked or the folded one, both go back to half each
        for folded in [PaneSplit::Scores, PaneSplit::Lists] {
            assert_eq!(folded.clicked(true), PaneSplit::Even);
            assert_eq!(folded.clicked(false), PaneSplit::Even);
        }
    }

    #[test]
    fn leaving_the_clips_tab_keeps_a_time_still_being_typed() {
        let mut view = view_typing("00:00:01.500");
        let mut workspace = WorkspaceUi {
            inspector_tab: InspectorTab::Clips,
            ..Default::default()
        };

        // The field goes out of view before it could lose the focus, and an
        // export afterwards reads the clip, so the clip has the typed time
        view.choose_tab(&mut workspace, InspectorTab::Events);
        assert_eq!(workspace.inspector_tab, InspectorTab::Events);
        assert_eq!(view.clips[0].start, DemoTime::from_micros(1_500_000));
        assert!(view.clips_changed);
        assert_eq!(view.typing, None);
    }

    #[test]
    fn text_that_is_not_a_time_stays_typed_and_the_clip_keeps_its_time() {
        let mut view = view_typing("1:3x");
        let mut workspace = WorkspaceUi {
            inspector_tab: InspectorTab::Clips,
            ..Default::default()
        };

        view.choose_tab(&mut workspace, InspectorTab::Events);
        assert_eq!(view.clips[0].start, DemoTime::from_micros(1_000_000));
        assert!(!view.clips_changed);
        assert_eq!(view.typing, Some((0, Edge::Start, "1:3x".to_string())));
    }

    #[test]
    fn other_tabs_leave_a_time_being_typed_alone() {
        let mut view = view_typing("00:00:01.500");
        let mut workspace = WorkspaceUi::default();

        // Only the clips tab holds the fields, so nothing is typed elsewhere
        // to keep, and what is typed waits for its field
        view.choose_tab(&mut workspace, InspectorTab::Clips);
        assert_eq!(view.clips[0].start, DemoTime::from_micros(1_000_000));
        assert!(view.typing.is_some());
    }

    /// A shot at `time` in the first session
    fn shot(time: f32) -> Event {
        Event {
            segment: 0,
            time,
            lane: 0,
            actor: None,
            weapon: Some(Weapon::Gauss),
            kind: EventKind::Action {
                action: Action::Fire,
                mode: None,
            },
        }
    }

    /// A kill at `time` in `segment` whose killer's airborne state is
    /// `airborne`, the victim known to be on the ground. Its place among the
    /// kills is its whole seconds
    fn kill(segment: u32, time: f32, airborne: Option<bool>) -> Event {
        let motion = |airborne| KillMotion {
            airborne,
            horizontal_speed: None,
            evidence: MotionEvidence::Trajectory,
            sampled_at: SampledAt::Kill,
        };
        let features = KillFeatures {
            key: KillKey {
                segment,
                ordinal: time as u32,
            },
            killer_motion: Some(motion(airborne)),
            victim_motion: Some(motion(Some(false))),
            shot: None,
            victim: None,
            killer_team: None,
            victim_team: None,
            gauss: true,
            gauss_kill: None,
        };

        Event {
            segment,
            time,
            lane: 0,
            actor: None,
            weapon: Some(Weapon::Gauss),
            kind: EventKind::Kill {
                victim: None,
                weapon_name: "gauss".into(),
                cause: KillCause::PlayerWeapon,
                killer: None,
                victim_player: None,
                headshot: None,
                features: Box::new(features),
            },
        }
    }

    /// A demo view whose analysis is in with `events` and `groups`
    fn view_of(events: Vec<Event>, groups: GroupIndex) -> DemoView {
        let ctx = egui::Context::default();
        let mut view = DemoView::open(PathBuf::from("not-there.dem"), 100., None, &ctx);
        view.loader = None;
        view.model = Some(Arc::new(Model {
            events,
            end: 100.,
            groups,
            ..Default::default()
        }));
        view
    }

    #[test]
    fn refinements_hide_only_kills_and_every_part_reads_the_same_rows() {
        let events = vec![
            shot(1.),
            kill(0, 2., Some(true)),
            kill(0, 3., None),
            kill(0, 4., Some(false)),
        ];
        let mut view = view_of(events, GroupIndex::default());
        view.shown.shots = true;
        view.shown.kill_filters.midair = MidairMode::Either;
        view.refresh_rows();

        // The shot stays, the kill known in the air shows, and the kill
        // whose state is unknown is counted rather than taken as grounded
        assert_eq!(view.rows, [0, 1]);
        assert_eq!(view.unknown_kills, 1);
        assert_eq!(view.shown_events, [true, true, false, false]);

        // The timeline's marks, which its density counts too, are the rows
        let marked: Vec<usize> = view.marks.iter().map(|m| m.event).collect();
        assert_eq!(marked, view.rows);

        // Hiding the kills keeps their refinements for when they come back
        view.shown.kills = false;
        view.refresh_rows();
        assert_eq!(view.rows, [0]);
        assert_eq!(view.shown.kill_filters.midair, MidairMode::Either);
    }

    #[test]
    fn a_votes_change_reads_as_more_or_fewer_of_each() {
        let catalog = Catalog::new(crate::i18n::Locale::En);

        // Fluent marks where each value it places starts and ends
        let words = |s: String| s.replace(['\u{2068}', '\u{2069}'], "");
        assert_eq!(
            words(vote_change(&catalog, 2, 1)),
            "2 more yes, one more no"
        );
        assert_eq!(words(vote_change(&catalog, 0, -1)), "one less no");

        let cast = MatchChange::VoteCast {
            subject: "agstart".into(),
            yes: 1,
            no: 0,
            tally: Tally {
                yes: 2,
                no: 0,
                undecided: 6,
            },
        };
        assert_eq!(
            words(describe_match(&catalog, &cast)),
            "Vote on agstart: one more yes (2 yes, 0 no, 6 undecided)"
        );
    }

    #[test]
    fn a_line_of_chat_says_what_follows_its_senders_name() {
        let team = "(T) -<[ATK]>-naz: at Bridge";
        assert_eq!(said(team, Some("-<[ATK]>-naz")), "(T) at Bridge");

        // Lines come without colour codes, and names lose theirs to match
        assert_eq!(said("red: hi", Some("^1red^0")), "hi");

        // The server's own lines, and a name the line does not have
        assert_eq!(said("- p has left the game", None), "- p has left the game");
        assert_eq!(said("hello", Some("naz")), "hello");
    }

    #[test]
    fn the_player_filter_matches_the_killer_or_the_victim() {
        let named = |actor: &str, victim: Option<&str>| {
            let mut event = kill(0, 1., Some(false));
            event.actor = Some(actor.into());
            if let EventKind::Kill { victim: v, .. } = &mut event.kind {
                *v = victim.map(String::from);
            }
            event
        };
        let events = vec![
            named("^1Naz", Some("bob")),
            named("alice", Some("^4nAz^0!")),
            named("alice", None),
        ];
        let mut view = view_of(events, GroupIndex::default());

        // Colour codes and case make no difference, nor do spaces around
        view.player = " NAZ ".into();
        view.refresh_rows();
        assert_eq!(view.rows, [0, 1]);

        view.player = "alice".into();
        view.refresh_rows();
        assert_eq!(view.rows, [1, 2]);

        // An empty field shows every event
        view.player.clear();
        view.refresh_rows();
        assert_eq!(view.rows, [0, 1, 2]);
    }

    #[test]
    fn clip_it_clips_whichever_was_chosen_last() {
        let group = |time: f32| GroupStart {
            time,
            segment: 0,
            demo_start: false,
        };
        let groups = GroupIndex {
            groups: vec![group(0.), group(50.)],
            ..Default::default()
        };
        let mut view = view_of(vec![kill(0, 30., None)], groups);
        Arc::get_mut(view.model.as_mut().unwrap())
            .unwrap()
            .windows
            .push(Window {
                segment: 0,
                start: 40.,
                end: 70.,
                score: 1.,
                sides: Vec::new(),
            });
        view.padding = (4., 1.);
        let catalog = Catalog::new(crate::i18n::Locale::En);

        // The event, then the highlight chosen after it, then the event again
        view.select(0, true);
        assert_eq!(view.clip_it_range(), Some((26., 31.)));
        view.apply(DemoAction::ChooseHighlight(0), &catalog);
        assert_eq!(view.clip_it_range(), Some((36., 71.)));
        view.select(0, true);
        assert_eq!(view.clip_it_range(), Some((26., 31.)));
    }

    #[test]
    fn clip_it_spans_from_the_first_chosen_event_to_the_last() {
        let group = |time: f32| GroupStart {
            time,
            segment: 0,
            demo_start: false,
        };
        let groups = GroupIndex {
            groups: vec![group(0.), group(50.)],
            ..Default::default()
        };
        let kills = vec![
            kill(0, 10., None),
            kill(0, 20., None),
            kill(0, 30., None),
            kill(0, 40., None),
        ];
        let mut view = view_of(kills, groups);
        view.refresh_rows();
        view.padding = (4., 1.);

        // The kill at 30 s between the two chosen is in the clip anyway
        view.select(0, true);
        view.marked = BTreeSet::from([1, 3]);
        assert_eq!(view.clip_it_range(), Some((16., 41.)));

        // Selecting one event chooses it alone again
        view.select(2, true);
        assert_eq!(view.clip_it_range(), Some((26., 31.)));
    }

    #[test]
    fn an_event_whose_time_an_earlier_session_has_is_out_of_reach() {
        let group = |time: f32, segment: u32| GroupStart {
            time,
            segment,
            demo_start: false,
        };
        let groups = GroupIndex {
            groups: vec![group(0., 0), group(10., 0), group(0., 1), group(10., 1)],
            ..Default::default()
        };
        let mut view = view_of(vec![kill(0, 5., None), kill(1, 5., None)], groups);

        // The viewer seeks by time alone, which finds the first session
        assert!(view.reachable(0));
        assert!(!view.reachable(1));

        // It can still be selected and read, without being played
        view.select(1, true);
        assert_eq!(view.selected, Some(1));
    }

    #[test]
    fn only_kills_told_through_a_wall_show_and_the_rest_wait() {
        let events = vec![kill(0, 1., None), kill(0, 2., None), kill(0, 3., None)];
        let mut view = view_of(events, GroupIndex::default());
        view.shown.kill_filters.wallgauss = true;
        view.refresh_rows();

        // Nothing told yet, so every Gauss kill waits as unknown
        assert!(view.rows.is_empty());
        assert_eq!(view.unknown_kills, 3);

        let key = |event: usize| match &view.model.as_ref().unwrap().events[event].kind {
            EventKind::Kill { features, .. } => features.key,
            _ => unreachable!(),
        };
        let (first, second) = (key(0), key(1));
        let yes = WallGauss::Yes(WallDetails {
            route: WallRoute::Beyond,
            thickness: 12.,
            shot_time: 1.,
        });
        view.walls.insert(first, yes);
        view.walls.insert(second, WallGauss::No(OtherRoute::Direct));

        // A verdict arriving works the rows out again
        view.refresh_rows();
        assert_eq!(view.rows, [0]);
        assert_eq!(view.unknown_kills, 1);
    }

    /// A demo view with clips from 1 to 2, 3 to 4 and 5 to 6 seconds
    fn view_with_clips() -> DemoView {
        let ctx = egui::Context::default();
        let mut view = DemoView::open(PathBuf::from("not-there.dem"), 10., None, &ctx);
        for k in 0..3 {
            let start = DemoTime::from_micros((1 + 2 * k) * 1_000_000);
            let end = DemoTime::from_micros((2 + 2 * k) * 1_000_000);
            let name = format!("Clip {k}");
            view.clips.push(Clip { name, start, end });
        }
        view.identify_clips();
        view
    }

    #[test]
    fn an_export_result_stays_with_its_clip_and_its_range() {
        let mut view = view_with_clips();
        view.mark_export(2, ExportState::Cancelled);
        view.looping = Some(2);

        // Removing an earlier clip moves the result and the loop with their
        // clip, never onto another
        view.remove_clip(0);
        let id = view.clip_ids[1];
        let (range, state) = view.exports[&id].clone();
        assert_eq!(state, ExportState::Cancelled);
        assert_eq!(view.looping, Some(1));

        // Editing the clip keeps the range the export was of
        view.clips[1].end = DemoTime::from_micros(9_000_000);
        assert_eq!(
            range,
            (
                DemoTime::from_micros(5_000_000),
                DemoTime::from_micros(6_000_000)
            )
        );

        // Removing the clip takes its result and its loop with it
        view.remove_clip(1);
        assert!(!view.exports.contains_key(&id));
        assert_eq!(view.looping, None);
    }

    #[test]
    fn exporting_every_clip_says_which_were_left_out() {
        let ctx = egui::Context::default();
        let mut view = view_with_clips();

        // Without the analysis no clip can be cut, so each is skipped with
        // why, and nothing starts
        view.export_clips(&ctx, clips_tab::ExportRequest::All);
        assert!(view.export.is_none());
        for id in &view.clip_ids {
            assert!(matches!(view.exports[id].1, ExportState::Skipped(_)));
        }
    }

    #[test]
    fn a_new_clip_without_a_viewer_starts_where_the_timeline_shows() {
        let catalog = Catalog::new(crate::i18n::Locale::En);
        let ctx = egui::Context::default();
        let mut view = DemoView::open(PathBuf::from("not-there.dem"), 60., None, &ctx);
        view.view = (20., 40.);

        view.apply(DemoAction::NewClip, &catalog);
        let clip = &view.clips[0];
        assert_eq!(clip.start, DemoTime::from_micros(20_000_000));
        assert_eq!(clip.end, DemoTime::from_micros(30_000_000));
        assert_eq!(view.wanted_tab, Some(InspectorTab::Clips));
        assert_eq!(view.chosen_clip, Some(0));

        // Its name is plain text, without the marks around a message's number
        assert_eq!(clip.name, "Clip 1");
    }

    /// The actions `view` asks for after `key` is pressed with Ctrl
    fn press_with_ctrl(view: &mut DemoView, key: egui::Key) -> Vec<DemoAction> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }],
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| view.shortcuts(ui.ctx()));

        // Nothing is drawn, so the fonts egui made go unused
        output.textures_delta.clear();
        std::mem::take(&mut view.actions)
    }

    #[test]
    fn ctrl_and_the_arrows_step_through_the_shown_events() {
        let events = vec![kill(0, 1., Some(true)), shot(2.), kill(0, 3., Some(true))];
        let mut view = view_of(events, GroupIndex::default());
        view.refresh_rows();
        view.selected = Some(0);

        // The shot is hidden, so the next shown event is the second kill
        let next = press_with_ctrl(&mut view, egui::Key::ArrowRight);
        assert_eq!(
            next,
            [DemoAction::Select {
                event: 2,
                from_list: false
            }]
        );

        // Nothing comes before the first
        assert!(press_with_ctrl(&mut view, egui::Key::ArrowLeft).is_empty());
    }

    #[test]
    fn a_demo_opened_without_a_length_takes_the_analysiss() {
        let ctx = egui::Context::default();
        let mut view = DemoView::open(PathBuf::from("not-there.dem"), 0., None, &ctx);
        let (tx, rx) = mpsc::channel();
        view.loader = Some(Loader {
            cancel: Arc::new(AtomicBool::new(false)),
            rx,
            thread: None,
        });
        let model = Model {
            end: 1442.,
            ..Default::default()
        };
        tx.send(Loaded::Model(Arc::new(model))).unwrap();
        view.poll();
        assert_eq!(view.duration, 1442.);
    }

    #[test]
    fn an_analysis_made_again_replaces_the_events_and_keeps_the_selection() {
        let mut view = view_of(vec![kill(0, 5., Some(true))], GroupIndex::default());
        view.selected = Some(0);
        view.refresh_rows();
        assert!(view.rows_for.is_some());

        // The game's files told the killer was on the ground after all
        let (tx, rx) = mpsc::channel();
        view.reanalysis = Some(Loader {
            cancel: Arc::new(AtomicBool::new(false)),
            rx,
            thread: None,
        });
        let again = Model {
            events: vec![kill(0, 5., Some(false))],
            end: 100.,
            ..Default::default()
        };
        tx.send(Loaded::Model(Arc::new(again))).unwrap();
        view.poll();

        assert!(view.reanalysis.is_none());
        assert_eq!(view.selected, Some(0));
        assert_eq!(view.rows_for, None);
        let model = view.model.as_ref().unwrap();
        assert_eq!(model.events[0], kill(0, 5., Some(false)));
    }
}
