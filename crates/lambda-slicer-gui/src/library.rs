//! The library, as a full table beside a filter pane or as a narrow list
//! beside an open demo
//!
//! Both show the same query of the cache in the same order. Rows come from
//! the cache a page at a time as they scroll into view, so a library of any
//! size costs only the visible pages. Changing a filter or the order counts
//! the matches again and drops the loaded pages. While a scan stores files
//! the list refreshes at most once a second. The full table can list the
//! demos' [matches] instead, one a row

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, CornerRadius, Popup, PopupCloseBehavior, Pos2, Rect, Response, RichText, Role, Sense,
    TextStyle, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText,
};
use egui_extras::{Column, TableBuilder};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::context::Scoreboard;
use lambda_slicer_core::analysis::parts::PartStatus;
use lambda_slicer_core::analysis::weapons::Weapon;
use lambda_slicer_core::library::cache::{
    Cache, FileQuery, FileRow, KillQuery, LibraryStats, MatchSort, Sort,
};
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::timeline::time::DemoTime;

use crate::i18n::{Catalog, DateOrder, Ymd};
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, AnalysisState, Icon, IconButton, Variant};

mod matches;

use matches::{MatchList, Tab};

/// A date bound with a checkbox to use it, its fields in the locale's order.
/// The checkbox and the fields wrap together as one group
fn date_filter(
    ui: &mut Ui,
    catalog: &Catalog,
    label: &WidgetText,
    value: &mut Option<Ymd>,
    salt: &str,
) {
    layout::unbroken(ui, salt, |ui| date_fields(ui, catalog, label, value, salt));
}

fn date_fields(
    ui: &mut Ui,
    catalog: &Catalog,
    label: &WidgetText,
    value: &mut Option<Ymd>,
    salt: &str,
) {
    let mut on = value.is_some();
    if ui.checkbox(&mut on, label.clone()).changed() {
        *value = on.then(|| {
            let today = chrono::Local::now().date_naive();
            use chrono::Datelike;
            Ymd {
                year: today.year(),
                month: today.month(),
                day: today.day(),
            }
        });
    }
    let Some(d) = value else {
        return;
    };
    ui.push_id(salt, |ui| {
        let year = |ui: &mut Ui, d: &mut Ymd| {
            ui.add(
                egui::DragValue::new(&mut d.year)
                    .range(1998..=2100)
                    .speed(0.1),
            );
        };
        let month = |ui: &mut Ui, d: &mut Ymd| {
            ui.add(egui::DragValue::new(&mut d.month).range(1..=12).speed(0.1));
        };
        let day = |ui: &mut Ui, d: &mut Ymd| {
            let last = d.month_days();
            ui.add(egui::DragValue::new(&mut d.day).range(1..=last).speed(0.1));
        };
        let sep = |ui: &mut Ui, s: char| {
            ui.label(s.to_string());
        };
        #[derive(Clone, Copy)]
        enum Part {
            Day,
            Month,
            Year,
        }
        let (parts, mark) = match catalog.locale.date_order() {
            DateOrder::DayMonthYear(s) => ([Part::Day, Part::Month, Part::Year], s),
            DateOrder::MonthDayYear(s) => ([Part::Month, Part::Day, Part::Year], s),
            DateOrder::YearMonthDay(s) => ([Part::Year, Part::Month, Part::Day], s),
        };
        // A numeric date reads left to right even in a right-to-left row,
        // whose widgets are placed from the right, so it is added backwards
        let mut parts = parts.to_vec();
        if catalog.locale.rtl() {
            parts.reverse();
        }
        for (i, part) in parts.into_iter().enumerate() {
            if i > 0 {
                sep(ui, mark);
            }
            match part {
                Part::Day => day(ui, d),
                Part::Month => month(ui, d),
                Part::Year => year(ui, d),
            }
        }
        d.day = d.day.min(d.month_days());
    });
}

/// A number typed with either a point or a comma before the decimals
pub fn parse_number(text: &str) -> Option<f64> {
    let cleaned: String = text
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    cleaned.parse().ok()
}

/// Rows loaded from the cache at a time
const PAGE: usize = 100;

/// Shortest time between refreshes while a scan stores files
const REFRESH: Duration = Duration::from_secs(1);

/// The filter pane's width at first, and the narrowest and widest it can be
/// dragged to
const FILTER_PANE: f32 = 248.;
const FILTER_PANE_RANGE: std::ops::RangeInclusive<f32> = 220.0..=340.0;

/// The height of a table row and of a narrow list's two-line row
const TABLE_ROW: f32 = 32.;
const LIST_ROW: f32 = 48.;

/// Where a narrow row's two lines are centred, from its top
const LIST_LINES: [f32; 2] = [16., 34.];

/// The intensity bar beside its number, and how strongly its fill shows
const INTENSITY_BAR: Vec2 = Vec2::new(64., 4.);
const INTENSITY_FILL: f32 = 0.8;

/// The narrow list's order menu, and the least its search field shrinks to
const SORT_MENU_WIDTH: f32 = 104.;
const SEARCH_MIN_WIDTH: f32 = 60.;

/// The room a slider leaves beside it for its value
const SLIDER_VALUE_ROOM: f32 = 56.;

/// The longest a demo can be asked to be at least, in minutes, and the
/// step that number takes as it is dragged
pub const MAX_MINUTES: f32 = 120.;
pub const MINUTES_STEP: f64 = 0.5;

/// The most kills a demo can be asked to have at least, and the step that
/// number takes as it is dragged
pub const MAX_KILLS: u32 = 1000;

/// The most players and teams the filters take, the game's most players
/// and more teams than AG has colours for
pub const MAX_PLAYERS: u32 = 32;
pub const MAX_TEAMS: u32 = 8;
pub const KILLS_STEP: f64 = 0.2;

/// Weapons that leave events, so a shot count can show their use
pub const WEAPONS: [Weapon; 12] = [
    Weapon::Gauss,
    Weapon::Crossbow,
    Weapon::Rpg,
    Weapon::Mp5,
    Weapon::Shotgun,
    Weapon::Python,
    Weapon::Glock,
    Weapon::Crowbar,
    Weapon::Egon,
    Weapon::Hivehand,
    Weapon::Tripmine,
    Weapon::Snark,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spectating {
    #[default]
    Any,
    /// Spectating for more than half the playback
    Mostly,
    /// Spectating for at most half the playback
    Playing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filters {
    pub text: String,
    pub map: String,
    pub player: String,
    pub weapon: Option<Weapon>,
    pub min_minutes: f32,
    /// Fewest kills, 0 for any
    pub min_kills: u32,
    pub spectating: Spectating,
    /// Recording date bounds, inclusive
    pub from: Option<Ymd>,
    pub to: Option<Ymd>,
    /// Only demos with a headshot kill
    pub headshot: bool,
    /// Only demos with a kill where the killer or the victim was in the air
    pub midair: bool,
    /// Lowest accuracy of the recording player in percent, 0 for any
    pub min_accuracy: f32,
    /// Fewest and most players and teams, inclusive. The whole range is
    /// any
    pub players: (u32, u32),
    pub teams: (u32, u32),
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            text: String::new(),
            map: String::new(),
            player: String::new(),
            weapon: None,
            min_minutes: 0.,
            min_kills: 0,
            spectating: Spectating::default(),
            from: None,
            to: None,
            headshot: false,
            midair: false,
            min_accuracy: 0.,
            players: (0, MAX_PLAYERS),
            teams: (0, MAX_TEAMS),
        }
    }
}

/// The point of view after its `toggle` is clicked, `Playing` or `Mostly`,
/// while it is `now`. Both on is every demo. Turning one off leaves the
/// other's demos alone, and the last one on stays on, as nothing would be
/// left
fn toggled(now: Spectating, toggle: Spectating) -> Spectating {
    let other = match toggle {
        Spectating::Playing => Spectating::Mostly,
        _ => Spectating::Playing,
    };
    match now {
        Spectating::Any => other,
        n if n == other => Spectating::Any,
        n => n,
    }
}

/// A field for a count of `value` within `within`, inclusive
fn count_field<'a>(
    value: &'a mut u32,
    within: (u32, u32),
    catalog: &'a Catalog,
) -> egui::DragValue<'a> {
    egui::DragValue::new(value)
        .range(within.0..=within.1)
        .speed(KILLS_STEP)
        .custom_formatter(|v, _| catalog.number(v, 0))
        .custom_parser(parse_number)
}

/// The least and most of `range` to filter by, none for an end that is
/// also the end of `whole` and so filters nothing
fn bounds(range: (u32, u32), whole: (u32, u32)) -> (Option<i64>, Option<i64>) {
    let least = (range.0 > whole.0).then_some(i64::from(range.0));
    let most = (range.1 < whole.1).then_some(i64::from(range.1));
    (least, most)
}

impl Filters {
    /// How many filters are on, each date bound on its own. Text of only
    /// spaces filters nothing, so it does not count
    pub fn active(&self) -> usize {
        let mut count = 0;
        for text in [&self.text, &self.map, &self.player] {
            if !text.trim().is_empty() {
                count += 1;
            }
        }

        let others = [
            self.weapon.is_some(),
            self.min_minutes > 0.,
            self.min_kills > 0,
            self.spectating != Spectating::Any,
            self.headshot,
            self.midair,
            self.min_accuracy > 0.,
            self.players != (0, MAX_PLAYERS),
            self.teams != (0, MAX_TEAMS),
            self.from.is_some(),
            self.to.is_some(),
        ];
        for on in others {
            if on {
                count += 1;
            }
        }
        count
    }

    /// The cache query for these filters in this order, a page long
    fn query(&self, sort: Sort, descending: bool) -> FileQuery {
        let text = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_string());
        let (min_players, max_players) = bounds(self.players, (0, MAX_PLAYERS));
        let (min_teams, max_teams) = bounds(self.teams, (0, MAX_TEAMS));
        FileQuery {
            min_players,
            max_players,
            min_teams,
            max_teams,
            text: text(&self.text),
            map: text(&self.map),
            player: text(&self.player),
            weapon_used: self.weapon.map(|w| w.name().to_string()),
            min_duration: (self.min_minutes > 0.).then_some(self.min_minutes * 60.),
            min_kills: (self.min_kills > 0).then_some(i64::from(self.min_kills)),
            min_spectating: (self.spectating == Spectating::Mostly).then_some(0.5000001),
            max_spectating: (self.spectating == Spectating::Playing).then_some(0.5),
            min_accuracy: (self.min_accuracy > 0.).then_some(f64::from(self.min_accuracy) / 100.),
            from_date: self.from.map(Ymd::iso),
            to_date: self.to.map(Ymd::iso),
            kills: KillQuery {
                headshot: self.headshot,
                midair: self.midair,
                ..Default::default()
            },
            sort,
            descending,
            limit: PAGE,
            ..Default::default()
        }
    }
}

/// The menu a right click on a demo's row opens, to show the demo in its
/// folder. Returns why the folder could not be shown
fn row_menu(response: &Response, file: &FileRow, catalog: &Catalog, rtl: bool) -> Option<String> {
    let mut failed = None;
    response.context_menu(|ui| {
        let label = text::widget_text(ui.ctx(), &catalog.text("path-show"), rtl, None, Look::Plain);
        let show = widgets::Button::new(Some(Icon::Folder), label, Variant::Default, rtl);
        if ui.add(show).clicked() {
            if let Err(e) = crate::platform::show_in_folder(Path::new(&file.path)) {
                let detail = FluentValue::from(e.to_string());
                failed = Some(catalog.text_with("path-show-failed", &[("detail", detail)]));
            }
            ui.close();
        }
    });
    failed
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Col {
    Name,
    Date,
    /// When the file was last modified, newest first for new clips
    FileDate,
    Map,
    Duration,
    Score,
    Gauss,
    Kills,
    Headshots,
    Accuracy,
    Action,
    Spectating,
    /// How many players and teams the demo's matches had
    Players,
    Teams,
}

impl Col {
    const ALL: [Col; 14] = [
        Col::Name,
        Col::Date,
        Col::FileDate,
        Col::Map,
        Col::Duration,
        Col::Score,
        Col::Players,
        Col::Teams,
        Col::Gauss,
        Col::Kills,
        Col::Headshots,
        Col::Accuracy,
        Col::Action,
        Col::Spectating,
    ];

    /// The name kept in settings. The status column of earlier versions is
    /// now a symbol beside the name, so its key no longer hides anything
    fn key(self) -> &'static str {
        match self {
            Col::Name => "name",
            Col::Date => "date",
            Col::FileDate => "modified",
            Col::Map => "map",
            Col::Duration => "duration",
            Col::Score => "score",
            Col::Gauss => "gauss",
            Col::Kills => "kills",
            Col::Headshots => "headshots",
            Col::Accuracy => "accuracy",
            Col::Action => "action",
            Col::Spectating => "spectating",
            Col::Players => "players",
            Col::Teams => "teams",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Col::Name => "column-name",
            Col::Date => "column-date",
            Col::FileDate => "column-file-date",
            Col::Map => "column-map",
            Col::Duration => "column-duration",
            Col::Score => "column-score",
            Col::Gauss => "column-gauss",
            Col::Kills => "column-kills",
            Col::Headshots => "column-headshots",
            Col::Accuracy => "column-accuracy",
            Col::Action => "column-action",
            Col::Spectating => "column-spectating",
            Col::Players => "column-players",
            Col::Teams => "column-teams",
        }
    }

    /// What the column's figure means, for its heading's tooltip
    fn help(self) -> Option<&'static str> {
        match self {
            Col::Date => Some("help-date-column"),
            Col::FileDate => Some("help-file-date"),
            Col::Headshots => Some("help-headshots-column"),
            Col::Accuracy => Some("help-accuracy"),
            Col::Players => Some("help-players-column"),
            Col::Teams => Some("help-teams-column"),
            _ => None,
        }
    }

    fn sort(self) -> Sort {
        match self {
            Col::Name => Sort::Path,
            Col::Date => Sort::Date,
            Col::FileDate => Sort::Modified,
            Col::Map => Sort::Map,
            Col::Duration => Sort::Duration,
            Col::Score => Sort::Score,
            Col::Gauss => Sort::Gauss,
            Col::Kills => Sort::Kills,
            Col::Headshots => Sort::Headshots,
            Col::Accuracy => Sort::Accuracy,
            Col::Action => Sort::Action,
            Col::Spectating => Sort::Spectating,
            Col::Players => Sort::Players,
            Col::Teams => Sort::Teams,
        }
    }

    /// Every column is a size of its own the user can change, so a table
    /// wider than the window scrolls sideways rather than dropping columns
    fn column(self) -> Column {
        let (width, least) = match self {
            Col::Name => (280., 180.),
            Col::Date | Col::FileDate => (112., 80.),
            Col::Map => (128., 60.),
            Col::Duration => (80., 64.),
            Col::Score => (128., 64.),
            Col::Gauss | Col::Kills => (64., 48.),
            Col::Headshots => (56., 48.),
            Col::Accuracy => (80., 56.),
            Col::Action => (144., 96.),
            Col::Spectating => (96., 64.),
            Col::Players | Col::Teams => (64., 48.),
        };
        Column::initial(width).at_least(least).clip(true)
    }
}

/// Every column the library can be sorted by, as the key kept in settings
/// and its title's message
pub fn sort_columns() -> impl Iterator<Item = (&'static str, &'static str)> {
    Col::ALL.into_iter().map(|c| (c.key(), c.title()))
}

/// What the narrow list's header asked for
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListAction {
    /// Back to the full library table, closing the demo
    Back,
    /// Out of the way of the demo, which keeps it open
    Hide,
}

/// Names and paths sort from the start of the alphabet, figures from the
/// highest
fn descending_first(sort: Sort) -> bool {
    !matches!(sort, Sort::Path | Sort::Map)
}

/// The first two sides' frags as `170 : 155`, and how many sides there are
/// past those two
pub fn compact_score(board: &Scoreboard) -> Option<(String, usize)> {
    let first = board.sides.first()?;
    let shown = match board.sides.get(1) {
        Some(second) => format!("{} : {}", first.frags, second.frags),
        None => first.frags.to_string(),
    };
    let more = board.sides.len().saturating_sub(2);

    Some((shown, more))
}

/// A score for display, from its scoreboard, or else the stored summary,
/// which cannot say how many sides it leaves out
fn score_of(board: Option<&Scoreboard>, summary: Option<&str>) -> Option<(String, usize)> {
    if let Some(score) = board.and_then(compact_score) {
        return Some(score);
    }
    summary.map(|summary| (summary.to_string(), 0))
}

/// The name of the file at `path` without the `.dem` every demo has
fn display_name(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    match name.strip_suffix(".dem") {
        Some(stem) if !stem.is_empty() => stem.to_string(),
        _ => name,
    }
}

/// A length in seconds as `HH:MM:SS`
fn clock(seconds: Option<f32>) -> Option<String> {
    let time = DemoTime::from_f32(seconds?)?.to_string();
    time.split('.').next().map(str::to_string)
}

/// What the lists were last counted for
#[derive(Clone, Debug, PartialEq)]
struct Applied {
    filters: Filters,
    sort: Sort,
    descending: bool,
    matches: (MatchSort, bool, Vec<PartStatus>),
}

pub struct LibraryView {
    cache_path: PathBuf,
    cache: Option<Cache>,
    pub filters: Filters,
    applied: Option<Applied>,
    sort: Sort,
    descending: bool,
    /// Whether the full table lists demos or their matches, and the
    /// matches' own order and count
    tab: Tab,
    matches: MatchList,
    /// Where to start watching the selected demo, as a match's row asked
    pub open_at: Option<f32>,
    /// The demo player is playing, so files a scan stores wait to be
    /// listed rather than hold up its frames
    pub hold: bool,
    /// Files matching the filters, and figures over the whole library
    total: u64,
    stats: LibraryStats,
    pages: HashMap<usize, Vec<FileRow>>,
    stale: bool,
    last_refresh: Instant,
    pub selected: Option<FileRow>,
    error: Option<Message>,
    /// Column keys left out, and whether that changed this frame
    pub hidden: Vec<String>,
    pub columns_changed: bool,
    /// The filter pane beside the full table is open
    pub filters_open: bool,
    /// The narrow list shows its filters in place of its rows
    pub list_filters_open: bool,
    /// Name every map a demo recorded, rather than leaving out those of a
    /// demo of several maps that were barely played
    every_map: bool,
    /// The table row the arrow keys have reached, and that it should be
    /// scrolled into view
    cursor: Option<usize>,
    scroll_to_cursor: bool,
    /// How far the table scrolled sideways can go, as last drawn, and that
    /// it should go there so a right-to-left table starts at its leading
    /// columns
    max_scroll_x: Option<f32>,
    reach_leading: bool,
    /// The direction the table was last drawn in
    last_rtl: Option<bool>,
    /// Why a demo's folder could not be shown, for the app to tell
    pub path_notice: Option<String>,
}

impl LibraryView {
    pub fn new(cache_path: PathBuf, hidden: Vec<String>) -> Self {
        Self {
            hidden,
            columns_changed: false,
            cache_path,
            cache: None,
            filters: Filters::default(),
            applied: None,
            // The newest files first, so clips just exported are on top
            sort: Sort::Modified,
            descending: true,
            tab: Tab::Demos,
            matches: MatchList::default(),
            open_at: None,
            hold: false,
            total: 0,
            stats: LibraryStats::default(),
            pages: HashMap::new(),
            stale: true,
            last_refresh: Instant::now() - REFRESH,
            selected: None,
            error: None,
            filters_open: true,
            list_filters_open: false,
            every_map: false,
            cursor: None,
            path_notice: None,
            scroll_to_cursor: false,
            max_scroll_x: None,
            reach_leading: false,
            last_rtl: None,
        }
    }

    /// Select the cached file at `path`, as if its row were clicked
    pub fn select_path(&mut self, path: &Path) {
        let wanted = path.display().to_string();
        let query = FileQuery {
            text: Some(wanted.clone()),
            limit: 50,
            ..Default::default()
        };
        if let Some(Ok(rows)) = self.cache().map(|c| c.query(&query)) {
            self.selected = rows.into_iter().find(|r| r.path == wanted);
        }
    }

    /// Sort by the column kept in settings as `key`, highest or newest
    /// first when `descending`. A key this version does not know keeps the
    /// order there was
    pub fn sort_by_key(&mut self, key: &str, descending: bool) {
        if let Some(col) = Col::ALL.into_iter().find(|c| c.key() == key) {
            self.sort = col.sort();
            self.descending = descending;
        }
    }

    /// Forget the demos under a library folder that was removed, unless one
    /// of the `kept` folders still holds them
    pub fn forget_folder(&mut self, folder: &Path, kept: &[PathBuf]) {
        self.cache();
        if let Some(cache) = &mut self.cache
            && let Err(e) = cache.forget_folder(folder, kept)
        {
            self.error = Some(e.message());
        }
        self.mark_stale();
    }

    /// Forget everything stored about every demo, for a scan to find again
    pub fn clear(&mut self) {
        self.cache();
        if let Some(cache) = &mut self.cache
            && let Err(e) = cache.clear()
        {
            self.error = Some(e.message());
        }
        self.mark_stale();
    }

    /// Stored files may have changed what the list shows
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    /// Current files in the cache, whatever the filters leave out
    pub fn library_files(&self) -> u64 {
        self.stats.files
    }

    /// Whether the files have been counted yet, which happens as the list
    /// is first drawn
    pub fn counted(&self) -> bool {
        self.applied.is_some()
    }

    fn query(&self) -> FileQuery {
        FileQuery {
            every_map: self.every_map,
            ..self.filters.query(self.sort, self.descending)
        }
    }

    /// Name every map a demo recorded, or leave out those of a demo of
    /// several maps that were barely played
    pub fn set_every_map(&mut self, every_map: bool) {
        if self.every_map != every_map {
            self.every_map = every_map;
            self.mark_stale();
        }
    }

    fn cache(&mut self) -> Option<&Cache> {
        if self.cache.is_none() && self.cache_path.exists() {
            match Cache::open(&self.cache_path) {
                Ok(c) => self.cache = Some(c),
                Err(e) => self.error = Some(e.message()),
            }
        }
        self.cache.as_ref()
    }

    /// Count again and drop loaded pages when the filters, the order or the
    /// stored files changed. Stored files refresh at most once per
    /// [`REFRESH`], and a refresh held back is woken for when it is due, as
    /// nothing else may draw a frame once a scan has finished. While the
    /// demo player plays they wait for it to stop, as a big library takes
    /// long enough to count that its frames would stall
    fn refresh(&mut self, ctx: &egui::Context) {
        let wanted = Applied {
            filters: self.filters.clone(),
            sort: self.sort,
            descending: self.descending,
            matches: self.matches.order(),
        };
        let changed = self.applied.as_ref() != Some(&wanted);
        let waited = self.last_refresh.elapsed();
        let due = self.stale && waited >= REFRESH && !self.hold;
        if self.stale && !due && !self.hold {
            ctx.request_repaint_after(REFRESH - waited);
        }
        if !changed && !due {
            return;
        }

        let query = self.query();
        let count = self.cache().map(|c| c.count(&query));
        match count {
            Some(Ok(n)) => {
                self.total = n;
                self.error = None;
            }
            Some(Err(e)) => self.error = Some(e.message()),
            None => self.total = 0,
        }

        let query = self.match_query();
        let count = self.cache().map(|c| c.count_matches(&query));
        match count {
            Some(Ok(n)) => self.matches.total = n,
            Some(Err(e)) => self.error = Some(e.message()),
            None => self.matches.total = 0,
        }

        let stats = self.cache().map(Cache::library_stats);
        match stats {
            Some(Ok(stats)) => self.stats = stats,
            Some(Err(e)) => self.error = Some(e.message()),
            None => self.stats = LibraryStats::default(),
        }

        // A row the keyboard reached means another file in another list
        if changed {
            self.cursor = None;
        }
        self.pages.clear();
        self.matches.pages.clear();
        self.applied = Some(wanted);
        self.stale = false;
        self.last_refresh = Instant::now();
    }

    fn row(&mut self, index: usize) -> Option<FileRow> {
        let page = index / PAGE;
        if !self.pages.contains_key(&page) {
            let query = FileQuery {
                offset: page * PAGE,
                ..self.query()
            };
            let rows = self.cache().map(|c| c.query(&query));
            match rows {
                Some(Ok(rows)) => {
                    self.pages.insert(page, rows);
                }
                Some(Err(e)) => self.error = Some(e.message()),
                None => return None,
            }
        }
        self.pages.get(&page)?.get(index % PAGE).cloned()
    }

    /// Sort by `sort`, turning the order round when it is already the order
    fn sort_by(&mut self, sort: Sort) {
        if sort == self.sort {
            self.descending = !self.descending;
        } else {
            self.sort = sort;
            self.descending = descending_first(sort);
        }
    }

    /// How many filters are on, with the matches' own on their `tab`
    fn active(&self, tab: Tab) -> usize {
        let statuses = tab == Tab::Matches && !self.matches.hidden.is_empty();
        self.filters.active() + usize::from(statuses)
    }

    /// How many rows `tab` lists
    fn shown_rows(&self, tab: Tab) -> u64 {
        match tab {
            Tab::Demos => self.total,
            Tab::Matches => self.matches.total,
        }
    }

    /// How many demos or matches `tab` lists, and of how many when the
    /// filters leave some out
    fn count_line(&self, tab: Tab, catalog: &Catalog) -> String {
        let (shown, total, ids) = match tab {
            Tab::Demos => (
                self.total,
                self.stats.files,
                ("library-demos", "library-demos-of"),
            ),
            Tab::Matches => (
                self.matches.total,
                self.stats.matches,
                ("library-matches", "library-matches-of"),
            ),
        };
        let (shown, total) = (shown as i64, total as i64);
        if self.active(tab) == 0 || shown == total {
            return catalog.text_with(ids.0, &[("count", shown.into())]);
        }

        catalog.text_with(ids.1, &[("shown", shown.into()), ("total", total.into())])
    }

    /// Why nothing is listed, or the cache's error
    fn empty_ui(&self, ui: &mut Ui, tab: Tab, catalog: &Catalog, rtl: bool) {
        layout::column(ui, rtl, |ui| {
            if let Some(e) = &self.error {
                text::label(ui, &catalog.message(e), rtl);
            }

            if self.shown_rows(tab) == 0 {
                let filtered = self.active(tab) > 0;
                let id = match (tab, filtered) {
                    (Tab::Demos, false) => "library-empty",
                    (Tab::Demos, true) => "library-no-matches",
                    (Tab::Matches, _) if self.stats.files == 0 => "library-empty",
                    (Tab::Matches, false) => "library-no-games",
                    (Tab::Matches, true) => "library-no-games-filtered",
                };
                text::label(ui, &catalog.text(id), rtl);
            }
        });
    }

    /// The filters as groups one under another, for the filter pane and the
    /// narrow list alike
    fn filter_form(&mut self, ui: &mut Ui, tab: Tab, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);

        // Find, with the reset of every filter beside its title
        let active = self.active(tab);
        layout::row(ui, rtl, |ui| {
            widgets::section_title(ui, &catalog.text("filters-find"), rtl);
            if active > 0 {
                layout::trailing(ui, rtl, |ui| {
                    let reset = widgets::Button::new(None, t("filter-clear"), Variant::Ghost, rtl);
                    if ui.add(reset).clicked() {
                        self.filters = Filters::default();
                        self.matches.hidden.clear();
                    }
                });
            }
        });

        let fields = [
            (&mut self.filters.text, "filter-search"),
            (&mut self.filters.player, "filter-player"),
            (&mut self.filters.map, "filter-map"),
        ];
        for (value, hint) in fields {
            let field = egui::TextEdit::singleline(value)
                .hint_text(t(hint))
                .desired_width(f32::INFINITY);
            ui.add(field);
        }

        let weapon = self
            .filters
            .weapon
            .map_or_else(|| t("filter-weapon-any"), |w| WidgetText::from(w.name()));
        egui::ComboBox::from_id_salt("weapon")
            .selected_text(weapon)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.filters.weapon, None, t("filter-weapon-any"));
                for w in WEAPONS {
                    ui.selectable_value(&mut self.filters.weapon, Some(w), w.name());
                }
            });
        ui.add_space(theme::GAP_M);

        // Point of view, by the share of the recording spent spectating
        layout::row(ui, rtl, |ui| {
            widgets::section_title(ui, &catalog.text("filters-pov"), rtl);
            widgets::help(ui, &catalog.text("help-pov"), rtl);
        });

        // Two toggles, both on for every demo
        let spectating = self.filters.spectating;
        let toggles = [
            (Spectating::Playing, "pov-playing"),
            (Spectating::Mostly, "pov-spectating"),
        ];
        layout::row(ui, rtl, |ui| {
            let spacing = ui.spacing().item_spacing.x;
            let width = (ui.available_width() - spacing) / 2.;
            for (toggle, id) in toggles {
                let on = spectating == Spectating::Any || spectating == toggle;
                let button = egui::Button::selectable(on, t(id)).min_size(Vec2::new(width, 0.));
                if ui.add(button).clicked() {
                    self.filters.spectating = toggled(spectating, toggle);
                }
            }
        });
        ui.add_space(theme::GAP_M);

        // How much of each match was recorded, for the matches alone
        if tab == Tab::Matches {
            layout::row(ui, rtl, |ui| {
                widgets::section_title(ui, &catalog.text("filter-status"), rtl);
                widgets::help(ui, &catalog.text("help-status-column"), rtl);
            });
            self.status_filter(ui, catalog, rtl);
            ui.add_space(theme::GAP_M);
        }

        // Quality
        widgets::section_title(ui, &catalog.text("filters-quality"), rtl);

        let minutes = &mut self.filters.min_minutes;
        layout::row(ui, rtl, |ui| {
            ui.label(t("filter-min-minutes"));
            layout::trailing(ui, rtl, |ui| {
                ui.add(
                    egui::DragValue::new(minutes)
                        .range(0.0..=MAX_MINUTES)
                        .speed(MINUTES_STEP)
                        .custom_formatter(|v, _| catalog.number(v, 1))
                        .custom_parser(parse_number),
                );
            });
        });

        let kills = &mut self.filters.min_kills;
        layout::row(ui, rtl, |ui| {
            ui.label(t("filter-min-kills"));
            layout::trailing(ui, rtl, |ui| {
                ui.add(
                    egui::DragValue::new(kills)
                        .range(0..=MAX_KILLS)
                        .speed(KILLS_STEP)
                        .custom_formatter(|v, _| catalog.number(v, 0))
                        .custom_parser(parse_number),
                );
            });
        });

        // Players and teams from one count to another
        let ranges = [
            ("filter-players", &mut self.filters.players, MAX_PLAYERS),
            ("filter-teams", &mut self.filters.teams, MAX_TEAMS),
        ];
        for (id, range, most) in ranges {
            layout::row(ui, rtl, |ui| {
                ui.label(t(id));
                // Placed from the trailing side, so the upper end goes first
                layout::trailing(ui, rtl, |ui| {
                    let (from, to) = *range;
                    ui.add(count_field(&mut range.1, (from, most), catalog));
                    ui.label("–");
                    ui.add(count_field(&mut range.0, (0, to), catalog));
                });
            });
        }

        let accuracy_help = catalog.text("help-accuracy-filter");
        let min_accuracy = &mut self.filters.min_accuracy;
        layout::row(ui, rtl, |ui| {
            ui.label(t("filter-min-accuracy"));
            widgets::help(ui, &accuracy_help, rtl);
        });
        ui.spacing_mut().slider_width = ui.available_width() - SLIDER_VALUE_ROOM;
        ui.add(
            egui::Slider::new(min_accuracy, 0.0..=100.0)
                .step_by(1.)
                .custom_formatter(|v, _| catalog.number(v, 0))
                .custom_parser(parse_number),
        );

        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut self.filters.headshot, t("filter-headshots"));
            widgets::help(ui, &catalog.text("help-headshot-filter"), rtl);
        });
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut self.filters.midair, t("filter-midair"));
            widgets::help(ui, &catalog.text("help-midair-filter"), rtl);
        });
        ui.add_space(theme::GAP_M);

        // Recorded, both bounds inclusive
        widgets::section_title(ui, &catalog.text("filters-recorded"), rtl);

        layout::wrapping_row(ui, rtl, |ui| {
            date_filter(
                ui,
                catalog,
                &t("filter-from"),
                &mut self.filters.from,
                "from",
            );
        });
        layout::wrapping_row(ui, rtl, |ui| {
            date_filter(ui, catalog, &t("filter-to"), &mut self.filters.to, "to");
        });
    }

    /// The full library, its filter pane beside the table while open
    pub fn table_ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        self.refresh(ui.ctx());

        if self.filters_open {
            let frame = egui::Frame::new()
                .fill(theme::PANE)
                .inner_margin(theme::GAP_M as i8);
            layout::leading_panel("filters", rtl)
                .resizable(true)
                .default_size(FILTER_PANE)
                .size_range(FILTER_PANE_RANGE)
                .frame(frame)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("filter pane")
                        .show(ui, |ui| self.filter_form(ui, self.tab, catalog, rtl));
                });
        }

        self.table_header(ui, catalog, rtl);
        ui.separator();

        if self.error.is_some() || self.shown_rows(self.tab) == 0 {
            self.empty_ui(ui, self.tab, catalog, rtl);
            return;
        }
        match self.tab {
            Tab::Demos => self.table(ui, catalog, rtl),
            Tab::Matches => self.match_table(ui, catalog, rtl),
        }
    }

    /// The demos and matches tabs, filters with how many are on, the count
    /// of rows, and the demos' columns
    fn table_header(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let active = self.active(self.tab);
        let count = self.count_line(self.tab, catalog);

        layout::row(ui, rtl, |ui| {
            let tabs = [(Tab::Demos, "tab-demos"), (Tab::Matches, "tab-matches")];
            for (tab, id) in tabs {
                let chosen = self.tab == tab;
                let title = text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
                if ui.add(egui::Button::selectable(chosen, title)).clicked() && !chosen {
                    self.tab = tab;
                    self.cursor = None;
                }
            }
            ui.separator();

            let variant = if self.filters_open {
                Variant::Default
            } else {
                Variant::Ghost
            };
            let label =
                text::widget_text(&ctx, &catalog.text("filters-title"), rtl, None, Look::Plain);
            let filters = widgets::Button::new(Some(Icon::Filter), label, variant, rtl);
            if ui.add(filters).clicked() {
                self.filters_open = !self.filters_open;
            }
            if active > 0 {
                widgets::badge(ui, active);
            }

            ui.label(text::widget_text(&ctx, &count, rtl, None, Look::Weak));
            if self.tab == Tab::Matches {
                return;
            }

            layout::trailing(ui, rtl, |ui| {
                let label = text::widget_text(
                    &ctx,
                    &catalog.text("columns-button"),
                    rtl,
                    None,
                    Look::Plain,
                );
                let columns = widgets::Button::new(Some(Icon::Columns), label, Variant::Ghost, rtl);
                let response = ui.add(columns);
                Popup::menu(&response)
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| self.column_choices(ui, catalog, rtl));
            });
        });
    }

    /// A checkbox for every column but the demo's name, which always shows
    fn column_choices(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        for col in Col::ALL {
            if col == Col::Name {
                continue;
            }

            let mut shown = !self.hidden.iter().any(|h| h == col.key());
            let label =
                text::widget_text(ui.ctx(), &catalog.text(col.title()), rtl, None, Look::Plain);
            if ui.checkbox(&mut shown, label).changed() {
                self.hidden.retain(|h| h != col.key());
                if !shown {
                    self.hidden.push(col.key().to_string());
                }
                self.columns_changed = true;
            }
        }
    }

    /// The arrow keys move through the table's `rows` and Enter opens the
    /// one reached, unless a text field or another control has the
    /// keyboard. Returns the row to open
    fn keyboard(&mut self, ctx: &egui::Context, rows: u64) -> Option<usize> {
        if ctx.text_edit_focused() || ctx.memory(|m| m.focused().is_some()) {
            return None;
        }

        let rows = rows as usize;
        if rows == 0 {
            return None;
        }

        let pressed = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
        let last = rows - 1;
        let moved = if pressed(egui::Key::ArrowDown) {
            Some(self.cursor.map_or(0, |c| (c + 1).min(last)))
        } else if pressed(egui::Key::ArrowUp) {
            Some(self.cursor.map_or(0, |c| c.saturating_sub(1)))
        } else {
            None
        };
        if let Some(row) = moved {
            self.cursor = Some(row);
            self.scroll_to_cursor = true;
        }

        if pressed(egui::Key::Enter) {
            return self.cursor.filter(|&c| c < rows);
        }
        None
    }

    /// The columns shown, in the interface's reading order
    fn shown_columns(&self, rtl: bool) -> Vec<Col> {
        // The name always shows, whatever an older setting hid
        let mut cols = Vec::new();
        for col in Col::ALL {
            let hidden = self.hidden.iter().any(|h| h == col.key());
            if col == Col::Name || !hidden {
                cols.push(col);
            }
        }

        // Columns run right to left in a right-to-left interface
        if rtl {
            cols.reverse();
        }
        cols
    }

    fn table(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let open = self.keyboard(&ctx, self.total);
        let cols = self.shown_columns(rtl);
        let mut clicked = None;
        let mut sort_by = None;

        // A right-to-left table starts scrolled to its leading columns, on the
        // right, once its width is known
        if self.last_rtl != Some(rtl) {
            self.last_rtl = Some(rtl);
            self.reach_leading = rtl;
        }
        let mut scroll = egui::ScrollArea::horizontal().id_salt("library columns");
        if self.reach_leading
            && let Some(max) = self.max_scroll_x
        {
            scroll = scroll.horizontal_scroll_offset(max);
            self.reach_leading = false;
        }

        // Cells are centred in their rows and start on the leading side
        let cell_layout = if rtl {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };

        // The heading and the rows scroll sideways together when the columns
        // are wider than the window
        let output = scroll.show(ui, |ui| {
            let mut table = TableBuilder::new(ui)
                .id_salt("library")
                .cell_layout(cell_layout)
                .resizable(true)
                .sense(Sense::click())
                .max_scroll_height(f32::INFINITY)
                .min_scrolled_height(0.)
                .auto_shrink(false);
            for c in &cols {
                table = table.column(c.column());
            }
            if std::mem::take(&mut self.scroll_to_cursor)
                && let Some(cursor) = self.cursor
            {
                table = table.scroll_to_row(cursor, None);
            }

            table
                .header(TABLE_ROW, |mut header| {
                    for c in &cols {
                        header.col(|ui| {
                            if self.heading(ui, *c, catalog, rtl) {
                                sort_by = Some(c.sort());
                            }
                        });
                    }
                })
                .body(|body| {
                    body.rows(TABLE_ROW, self.total as usize, |mut row| {
                        let index = row.index();
                        let Some(file) = self.row(index) else {
                            return;
                        };
                        let selected = self.selected.as_ref().is_some_and(|s| s.id == file.id);
                        row.set_selected(selected);
                        row.set_hovered(self.cursor == Some(index));

                        for c in &cols {
                            row.col(|ui| self.cell(ui, *c, &file, selected, catalog, rtl));
                        }
                        let response = row.response();
                        if response.clicked() {
                            clicked = Some(index);
                        }
                        if let Some(failed) = row_menu(&response, &file, catalog, rtl) {
                            self.path_notice = Some(failed);
                        }
                    });
                });
        });
        let overflow = output.content_size.x - output.inner_rect.width();
        self.max_scroll_x = Some(overflow.max(0.));

        if let Some(sort) = sort_by {
            self.sort_by(sort);
        }

        let chosen = clicked.or(open);
        if let Some(index) = chosen {
            self.cursor = Some(index);
            self.selected = self.row(index);
        }
    }

    /// A column's heading, which sorts by it when clicked and offers the
    /// columns when right-clicked. Returns true when clicked
    fn heading(&mut self, ui: &mut Ui, col: Col, catalog: &Catalog, rtl: bool) -> bool {
        let sorted = (col.sort() == self.sort).then_some(self.descending);
        let help = col.help().map(|key| catalog.text(key));
        let title = catalog.text(col.title());
        let response = sort_heading(ui, &title, help.as_deref(), sorted, rtl);
        response.context_menu(|ui| self.column_choices(ui, catalog, rtl));
        response.clicked()
    }

    /// One cell of a row. A figure that is not known shows as a dash, and a
    /// known zero as 0
    fn cell(
        &self,
        ui: &mut Ui,
        col: Col,
        file: &FileRow,
        selected: bool,
        catalog: &Catalog,
        rtl: bool,
    ) {
        let percent = |fraction: f64| {
            catalog.text_with(
                "library-percent",
                &[("percent", FluentValue::from((fraction * 100.).round()))],
            )
        };
        let whole = |n: i64| catalog.number(n as f64, 0);

        // A weapon count is known once a file was analysed, and none means
        // the weapon was never used
        let analysed = file.shots.is_some();

        match col {
            Col::Name => self.name_cell(ui, file, selected, catalog, rtl),
            Col::Date => {
                let (day, source) = (file.recorded_at.as_deref(), file.date_source.as_deref());
                day_cell(ui, day, source, catalog, rtl);
            }
            Col::FileDate => file_date_cell(ui, file, catalog, rtl),
            Col::Map => {
                let map = file.map.as_deref().unwrap_or("");
                text_cell(ui, map, text::starts_rtl(map), Look::Plain);
            }
            Col::Score => {
                let (board, summary) = (file.scoreboard.as_ref(), file.score.as_deref());
                score_cell(ui, board, summary, catalog, rtl);
            }
            Col::Action => self.intensity_cell(ui, file.best_window, catalog, rtl),
            figure => {
                let value = match figure {
                    Col::Duration => clock(file.duration),
                    Col::Gauss => file.gauss.or(analysed.then_some(0)).map(whole),
                    Col::Kills => file.kills.map(whole),
                    Col::Headshots => file.headshots.map(whole),
                    Col::Accuracy => file.accuracy.map(percent),
                    Col::Players => file.players.map(whole),
                    Col::Teams => file.teams.map(whole),
                    _ => file.spectator_fraction.map(percent),
                };
                number_cell(ui, value, catalog, rtl);
            }
        }
    }

    /// The status symbol and the file's name, the whole path on hover. A
    /// chosen row has an accent edge on its leading side
    fn name_cell(&self, ui: &mut Ui, file: &FileRow, selected: bool, catalog: &Catalog, rtl: bool) {
        if selected {
            let cell = ui.max_rect();
            let edge_x = if rtl {
                cell.max.x - theme::SELECTED_EDGE
            } else {
                cell.min.x
            };
            let size = Vec2::new(theme::SELECTED_EDGE, cell.height());
            let edge = Rect::from_min_size(Pos2::new(edge_x, cell.min.y), size);
            ui.painter().rect_filled(edge, 0., theme::ACCENT);
        }

        let state = AnalysisState::of(&file.status, file.shots);
        let mut status = catalog.text(state.message_id());
        if let Some(error) = &file.error {
            status = format!("{status}\n{}", catalog.stored(error));
        }
        let name = display_name(&file.path);

        layout::row(ui, rtl, |ui| {
            widgets::status_symbol(ui, state, &status);
            if let Some(origin) = &file.clip_of {
                widgets::tag(ui, &catalog.text("badge-clip"))
                    .on_hover_text(clip_of(origin, catalog));
            }
            text_cell(ui, &name, text::starts_rtl(&name), Look::Plain).on_hover_text(&file.path);
        });
    }

    /// The best action window's score with a bar scaled to the highest in
    /// the library, so filtering never changes what a bar's length means
    fn intensity_cell(&self, ui: &mut Ui, score: Option<f64>, catalog: &Catalog, rtl: bool) {
        let Some(score) = score else {
            number_cell(ui, None, catalog, rtl);
            return;
        };

        let fill = match self.stats.max_action {
            Some(max) if score > 0. => (score / max).clamp(0., 1.) as f32,
            _ => 0.,
        };
        let max = self.stats.max_action.unwrap_or(0.);
        let tip = catalog.text_with(
            "intensity-help",
            &[
                ("score", FluentValue::from(catalog.number(score, 2))),
                ("max", FluentValue::from(catalog.number(max, 2))),
            ],
        );

        layout::row(ui, rtl, |ui| {
            let (rect, bar) = ui.allocate_exact_size(INTENSITY_BAR, Sense::hover());
            // Ends rounded fully, half the bar's thickness
            let radius = CornerRadius::same((INTENSITY_BAR.y / 2.) as u8);
            ui.painter().rect_filled(rect, radius, theme::RAISED);

            // The bar fills from the interface's leading side
            let length = rect.width() * fill;
            let filled = if rtl {
                Rect::from_min_max(Pos2::new(rect.max.x - length, rect.min.y), rect.max)
            } else {
                Rect::from_min_size(rect.min, Vec2::new(length, rect.height()))
            };
            ui.painter()
                .rect_filled(filled, radius, theme::ACCENT.gamma_multiply(INTENSITY_FILL));

            let ctx = ui.ctx().clone();
            let help = text::help(&ctx, &tip, rtl);
            bar.on_hover_text(help.clone());
            number_cell(ui, Some(catalog.number(score, 1)), catalog, rtl).on_hover_text(help);
        });
    }

    /// The narrow list beside an open demo, two lines a demo or a match as
    /// the table showed, or its filters in place of the rows. Returns what
    /// its header asked for
    pub fn list_ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> Option<ListAction> {
        self.refresh(ui.ctx());

        let action = self.list_header(ui, catalog, rtl);
        ui.separator();

        if self.list_filters_open {
            egui::ScrollArea::vertical()
                .id_salt("list filters")
                .show(ui, |ui| self.filter_form(ui, self.tab, catalog, rtl));
            return action;
        }

        self.list_search(ui, catalog, rtl);
        ui.separator();

        if self.error.is_some() || self.shown_rows(self.tab) == 0 {
            self.empty_ui(ui, self.tab, catalog, rtl);
            return action;
        }
        if self.tab == Tab::Matches {
            self.match_list(ui, catalog, rtl);
            return action;
        }

        let mut clicked = None;

        // Rows touch, so the rows the scroll area works out are the rows it
        // draws, at the same places
        ui.spacing_mut().item_spacing.y = 0.;
        egui::ScrollArea::vertical()
            .id_salt("demo list")
            .auto_shrink(false)
            .show_rows(ui, LIST_ROW, self.total as usize, |ui, rows| {
                for index in rows {
                    let Some(file) = self.row(index) else {
                        continue;
                    };
                    let response = self.list_row(ui, &file, catalog, rtl);
                    if let Some(failed) = row_menu(&response, &file, catalog, rtl) {
                        self.path_notice = Some(failed);
                    }
                    if response.clicked() {
                        clicked = Some(file);
                    }
                }
            });

        if let Some(file) = clicked {
            self.selected = Some(file);
        }
        action
    }

    /// Back to the table, the count, the filters with how many are on, and
    /// hiding the list. Returns what was asked for
    fn list_header(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> Option<ListAction> {
        let ctx = ui.ctx().clone();
        let active = self.active(self.tab);
        let count = self.count_line(self.tab, catalog);

        layout::row(ui, rtl, |ui| {
            let mut action = None;
            let back_label = catalog.text("list-back");
            if ui.add(IconButton::new(Icon::Table, &back_label)).clicked() {
                action = Some(ListAction::Back);
            }
            ui.label(text::widget_text(&ctx, &count, rtl, None, Look::Weak));

            layout::trailing(ui, rtl, |ui| {
                let hide_label = catalog.text("list-hide");
                let hide = IconButton::new(Icon::PanelClose, &hide_label).mirrored(rtl);
                if ui.add(hide).clicked() {
                    action = Some(ListAction::Hide);
                }

                let label = catalog.text("filters-title");
                let filters =
                    IconButton::new(Icon::Filter, &label).selected(self.list_filters_open);
                if ui.add(filters).clicked() {
                    self.list_filters_open = !self.list_filters_open;
                }
                if active > 0 && !self.list_filters_open {
                    widgets::badge(ui, active);
                }
            });
            action
        })
    }

    /// The path search and the order, the same as the table's
    fn list_search(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);

        layout::row(ui, rtl, |ui| {
            let spacing = ui.spacing().item_spacing.x;
            let search_width =
                ui.available_width() - SORT_MENU_WIDTH - theme::CONTROL_HEIGHT - spacing * 2.;
            let search = egui::TextEdit::singleline(&mut self.filters.text)
                .hint_text(t("filter-search"))
                .desired_width(search_width.max(SEARCH_MIN_WIDTH));
            ui.add(search);

            if self.tab == Tab::Matches {
                self.match_sort_menu(ui, catalog, rtl);
                return;
            }

            let mut sort = self.sort;
            let current = Col::ALL
                .into_iter()
                .find(|c| c.sort() == sort)
                .map_or_else(String::new, |c| catalog.text(c.title()));
            egui::ComboBox::from_id_salt("list sort")
                .width(SORT_MENU_WIDTH)
                .selected_text(text::widget_text(&ctx, &current, rtl, None, Look::Plain))
                .show_ui(ui, |ui| {
                    for col in Col::ALL {
                        ui.selectable_value(&mut sort, col.sort(), t(col.title()));
                    }
                })
                .response
                .on_hover_text(catalog.text("sort-by"));
            if sort != self.sort {
                self.sort = sort;
                self.descending = descending_first(sort);
            }

            direction_button(ui, &mut self.descending, catalog);
        });
    }

    /// One demo in two lines, its status and name, then its map, length and
    /// score. The date, recorder, intensity, accuracy and path show on hover
    fn list_row(&self, ui: &mut Ui, file: &FileRow, catalog: &Catalog, rtl: bool) -> Response {
        let size = Vec2::new(ui.available_width(), LIST_ROW);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let selected = self.selected.as_ref().is_some_and(|s| s.id == file.id);
        let name = display_name(&file.path);
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, &name));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        // A chosen row in the soft accent with an accent edge on its leading
        // side, a hovered one raised
        let painter = ui.painter();
        if selected {
            painter.rect_filled(rect, 0., theme::ACCENT_SOFT);
            let edge_x = if rtl {
                rect.max.x - theme::SELECTED_EDGE
            } else {
                rect.min.x
            };
            let size = Vec2::new(theme::SELECTED_EDGE, LIST_ROW);
            let edge = Rect::from_min_size(Pos2::new(edge_x, rect.min.y), size);
            painter.rect_filled(edge, 0., theme::ACCENT);
        } else if response.hovered() {
            painter.rect_filled(rect, 0., theme::RAISED);
        }

        // Places measured from the leading side, the right one in a
        // right-to-left interface
        let inner = rect.shrink2(Vec2::new(theme::GAP_M, 0.));
        let lead = |offset: f32, width: f32| {
            if rtl {
                inner.max.x - offset - width
            } else {
                inner.min.x + offset
            }
        };
        let first_line = rect.min.y + LIST_LINES[0];
        let second_line = rect.min.y + LIST_LINES[1];

        // The first line
        let state = AnalysisState::of(&file.status, file.shots);
        let symbol_x = lead(0., theme::ICON_SIZE) + theme::ICON_SIZE / 2.;
        widgets::paint_status(painter, Pos2::new(symbol_x, first_line), state);

        // A clip's tag after the status, before the name it shortens
        let mut name_offset = theme::ICON_SIZE + theme::GAP_S;
        if file.clip_of.is_some() {
            let galley = widgets::tag_galley(ui, &catalog.text("badge-clip"));
            let size = widgets::tag_size(&galley);
            let tag_at = Pos2::new(lead(name_offset, size.x), first_line - size.y / 2.);
            widgets::paint_tag(ui.painter(), Rect::from_min_size(tag_at, size), galley);
            name_offset += size.x + theme::GAP_S;
        }

        let name_room = inner.width() - name_offset;
        let look = if selected {
            Look::Color(theme::ACCENT)
        } else {
            Look::Plain
        };
        let name_text = text::widget_text(ui.ctx(), &name, text::starts_rtl(&name), None, look);
        let name_galley =
            name_text.into_galley(ui, Some(TextWrapMode::Truncate), name_room, TextStyle::Body);
        let name_x = lead(name_offset, name_galley.size().x);
        let name_at = Pos2::new(name_x, first_line - name_galley.size().y / 2.);
        ui.painter().galley(name_at, name_galley, theme::FOREGROUND);

        // The second line, the score on the trailing side
        let score = score_of(file.scoreboard.as_ref(), file.score.as_deref()).map(
            |(shown, more)| match more {
                0 => shown,
                more => format!("{shown} +{more}"),
            },
        );
        let score_galley = score.map(|score| {
            let text = RichText::new(score).monospace().color(theme::MUTED);
            WidgetText::from(text).into_galley(
                ui,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                TextStyle::Monospace,
            )
        });
        let score_width = score_galley.as_ref().map_or(0., |g| g.size().x);
        if let Some(galley) = score_galley {
            let x = if rtl {
                inner.min.x
            } else {
                inner.max.x - galley.size().x
            };
            let at = Pos2::new(x, second_line - galley.size().y / 2.);
            ui.painter().galley(at, galley, theme::MUTED);
        }

        let mut facts = Vec::new();
        if let Some(map) = &file.map {
            facts.push(map.clone());
        }
        if let Some(length) = clock(file.duration) {
            facts.push(length);
        }
        let facts = facts.join(" · ");

        let facts_room =
            inner.width() - theme::ICON_SIZE - theme::GAP_S - score_width - theme::GAP_S;
        let facts_text = text::styled_text(
            ui.ctx(),
            &facts,
            rtl,
            Look::Color(theme::MUTED),
            &theme::secondary(),
        );
        let facts_galley = facts_text.into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            facts_room.max(0.),
            theme::secondary(),
        );
        let facts_x = lead(theme::ICON_SIZE + theme::GAP_S, facts_galley.size().x);
        let facts_at = Pos2::new(facts_x, second_line - facts_galley.size().y / 2.);
        ui.painter().galley(facts_at, facts_galley, theme::MUTED);

        response.on_hover_ui(|ui| self.row_details(ui, file, catalog, rtl))
    }

    /// What a narrow row leaves out, for its tooltip
    fn row_details(&self, ui: &mut Ui, file: &FileRow, catalog: &Catalog, rtl: bool) {
        let mut lines = Vec::new();
        if let Some(origin) = &file.clip_of {
            lines.push(clip_of(origin, catalog));
        }
        if let Some(day) = file.recorded_at.as_deref().and_then(Ymd::parse) {
            lines.push(catalog.date(day));
        }
        if let Some(recorder) = &file.recorder {
            let name = FluentValue::from(text::player_name(recorder));
            lines.push(catalog.text_with("details-recorder", &[("name", name)]));
        }
        if let Some(score) = file.best_window {
            let score = FluentValue::from(score);
            lines.push(catalog.text_with("details-action", &[("score", score)]));
        }
        if let Some(accuracy) = file.accuracy {
            let title = catalog.text("column-accuracy");
            let percent = FluentValue::from((accuracy * 100.).round());
            let value = catalog.text_with("library-percent", &[("percent", percent)]);
            lines.push(format!("{title} {value}"));
        }

        for line in lines {
            text::label(ui, &line, rtl);
        }
        ui.label(RichText::new(&file.path).monospace());
    }
}

/// What a clip was exported from, by the name its mark gives
pub fn clip_of(origin: &str, catalog: &Catalog) -> String {
    let name = FluentValue::from(origin.to_owned());
    catalog.text_with("clip-of", &[("name", name)])
}

/// The button turning the order round, highest first when `descending`
fn direction_button(ui: &mut Ui, descending: &mut bool, catalog: &Catalog) {
    let (icon, id) = if *descending {
        (Icon::SortDown, "sort-descending")
    } else {
        (Icon::SortUp, "sort-ascending")
    };
    let label = catalog.text(id);
    if ui.add(IconButton::new(icon, &label)).clicked() {
        *descending = !*descending;
    }
}

/// A column's heading `title`, with `help` on hover, and an arrow when the
/// table is sorted by it, down when `sorted` says it is descending
fn sort_heading(
    ui: &mut Ui,
    title: &str,
    help: Option<&str>,
    sorted: Option<bool>,
    rtl: bool,
) -> Response {
    let ctx = ui.ctx().clone();
    let color = if sorted.is_some() {
        theme::FOREGROUND
    } else {
        theme::MUTED
    };

    layout::row(ui, rtl, |ui| {
        let title = text::styled_text(&ctx, title, rtl, Look::Color(color), &theme::secondary());
        let response = ui.add(
            egui::Label::new(title)
                .selectable(false)
                .sense(Sense::click()),
        );

        if let Some(descending) = sorted {
            let arrow = if descending {
                Icon::SortDown
            } else {
                Icon::SortUp
            };
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.), Sense::hover());
            widgets::paint_icon(ui.painter(), rect, arrow, theme::ACCENT);
        }

        match help {
            Some(help) => response.on_hover_text(text::help(&ctx, help, rtl)),
            None => response,
        }
    })
}

/// Text that is cut short with an ellipsis rather than wrapped
fn text_cell(ui: &mut Ui, text: &str, rtl: bool, look: Look) -> Response {
    let text = text::widget_text(ui.ctx(), text, rtl, None, look);
    ui.add(egui::Label::new(text).selectable(false).truncate())
}

/// The recording date `recorded_at` in the locale's form, with the time
/// and where the date came from, `source`, on hover
fn day_cell(
    ui: &mut Ui,
    recorded_at: Option<&str>,
    source: Option<&str>,
    catalog: &Catalog,
    rtl: bool,
) {
    let Some(day) = recorded_at.and_then(Ymd::parse) else {
        number_cell(ui, None, catalog, rtl);
        return;
    };

    let source = match source {
        Some("file_name") => catalog.text("date-from-name"),
        _ => catalog.text("date-from-modified"),
    };
    let time = recorded_at.and_then(|t| t.get(11..)).unwrap_or("");
    let tip = format!("{} {time}\n{source}", catalog.date(day));

    let ctx = ui.ctx().clone();
    let tip = text::widget_text(&ctx, &tip, rtl, None, Look::Plain);
    number_cell(ui, Some(catalog.date(day)), catalog, rtl).on_hover_text(tip);
}

/// The day the file was last modified, with the time on hover
fn file_date_cell(ui: &mut Ui, file: &FileRow, catalog: &Catalog, rtl: bool) {
    let modified = file.modified_at.as_deref();
    let Some(day) = modified.and_then(Ymd::parse) else {
        number_cell(ui, None, catalog, rtl);
        return;
    };

    let time = modified.and_then(|t| t.get(11..)).unwrap_or("");
    let tip = format!("{} {time}", catalog.date(day));
    let tip = text::widget_text(ui.ctx(), &tip, rtl, None, Look::Plain);
    number_cell(ui, Some(catalog.date(day)), catalog, rtl).on_hover_text(tip);
}

/// A figure in monospace, or a faint dash saying it is not known
fn number_cell(ui: &mut Ui, value: Option<String>, catalog: &Catalog, rtl: bool) -> Response {
    match value {
        Some(value) => {
            let text = RichText::new(value).monospace();
            ui.add(egui::Label::new(text).selectable(false).truncate())
        }
        None => {
            let dash = RichText::new("—").color(theme::FAINT);
            let unknown = text::widget_text(
                ui.ctx(),
                &catalog.text("value-unknown"),
                rtl,
                None,
                Look::Plain,
            );
            ui.add(egui::Label::new(dash).selectable(false))
                .on_hover_text(unknown)
        }
    }
}

/// The compact score of `board`, or else of its stored `summary`, the
/// sides past the first two counted after it, and every side with its
/// frags on hover
fn score_cell(
    ui: &mut Ui,
    board: Option<&Scoreboard>,
    summary: Option<&str>,
    catalog: &Catalog,
    rtl: bool,
) {
    let Some((shown, more)) = score_of(board, summary) else {
        number_cell(ui, None, catalog, rtl);
        return;
    };

    let mut sides = Vec::new();
    if let Some(board) = board {
        for side in &board.sides {
            sides.push(format!("{}  {}", text::player_name(&side.name), side.frags));
        }
    }

    layout::row(ui, rtl, |ui| {
        let score = number_cell(ui, Some(shown), catalog, rtl);
        if !sides.is_empty() {
            let ctx = ui.ctx().clone();
            let list = text::widget_text(&ctx, &sides.join("\n"), rtl, None, Look::Plain);
            score.on_hover_text(list);
        }

        if more > 0 {
            let count = FluentValue::from(more as i64);
            let tip = catalog.text_with("score-more-sides", &[("count", count)]);
            let text = RichText::new(format!("+{more}"))
                .monospace()
                .color(theme::FAINT);
            ui.add(egui::Label::new(text).selectable(false))
                .on_hover_text(tip);
        }
    });
}

#[cfg(test)]
mod tests {
    use lambda_slicer_core::analysis::context::ScoreSide;

    use super::*;

    /// A library row for the file at `path`, with nothing analysed
    fn file_row(path: &str) -> FileRow {
        FileRow {
            id: 0,
            path: path.into(),
            status: "complete".into(),
            error: None,
            map: None,
            duration: None,
            recording_kind: None,
            recorder: None,
            recorded_at: None,
            date_source: None,
            modified_at: None,
            clip_of: None,
            score: None,
            scoreboard: None,
            spectator_fraction: None,
            shots: None,
            gauss: None,
            kills: None,
            headshots: None,
            accuracy: None,
            players: None,
            teams: None,
            best_window: None,
            matching_kills: Vec::new(),
            native: Vec::new(),
        }
    }

    #[test]
    fn the_starting_order_is_set_by_its_key() {
        let mut view = LibraryView::new(PathBuf::from("not-there.sqlite"), Vec::new());
        assert_eq!((view.sort, view.descending), (Sort::Modified, true));

        view.sort_by_key("kills", false);
        assert_eq!((view.sort, view.descending), (Sort::Kills, false));

        // A column this version does not know changes nothing
        view.sort_by_key("bananas", true);
        assert_eq!((view.sort, view.descending), (Sort::Kills, false));
    }

    #[test]
    fn a_right_click_on_a_demo_offers_its_folder() {
        let ctx = egui::Context::default();
        theme::install(&ctx);
        let catalog = Catalog::new(crate::i18n::Locale::En);
        let view = LibraryView::new(PathBuf::from("not-there.sqlite"), Vec::new());
        let file = file_row("F:/demos/run_92.dem");

        // The row drawn once with `events` for input, and its middle
        let frame = |events: Vec<egui::Event>| {
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            let mut middle = egui::Pos2::ZERO;
            let mut output = ctx.run_ui(input, |ui| {
                let response = view.list_row(ui, &file, &catalog, false);
                middle = response.rect.center();
                row_menu(&response, &file, &catalog, false);
            });
            output.textures_delta.clear();
            middle
        };
        let at = frame(Vec::new());
        let button = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frame(vec![egui::Event::PointerMoved(at)]);
        frame(vec![button(true)]);
        frame(vec![button(false)]);
        frame(Vec::new());
        assert!(egui::Popup::is_any_open(&ctx));
    }

    #[test]
    fn numbers_parse_with_either_decimal_mark() {
        assert_eq!(parse_number("2,5"), Some(2.5));
        assert_eq!(parse_number(" 2.5 "), Some(2.5));
        assert_eq!(parse_number("1\u{a0}000,5"), Some(1000.5));
        assert_eq!(parse_number("x"), None);
    }

    fn board(frags: &[i32]) -> Scoreboard {
        let mut sides = Vec::new();
        for (i, frags) in frags.iter().enumerate() {
            sides.push(ScoreSide {
                name: format!("side {i}"),
                frags: *frags,
                deaths: 0,
            });
        }
        Scoreboard { teams: true, sides }
    }

    #[test]
    fn scores_show_two_sides_and_count_the_rest() {
        assert_eq!(compact_score(&board(&[])), None);
        assert_eq!(compact_score(&board(&[7])), Some(("7".into(), 0)));
        assert_eq!(
            compact_score(&board(&[170, 155])),
            Some(("170 : 155".into(), 0))
        );
        assert_eq!(
            compact_score(&board(&[50, 44, 38, 20])),
            Some(("50 : 44".into(), 2))
        );
        assert_eq!(compact_score(&board(&[3, -1])), Some(("3 : -1".into(), 0)));
    }

    #[test]
    fn point_of_view_toggles_keep_one_on_at_least() {
        use Spectating::{Any, Mostly, Playing};
        assert_eq!(toggled(Any, Playing), Mostly);
        assert_eq!(toggled(Any, Mostly), Playing);
        assert_eq!(toggled(Mostly, Playing), Any);
        assert_eq!(toggled(Playing, Mostly), Any);
        assert_eq!(toggled(Playing, Playing), Playing);
        assert_eq!(toggled(Mostly, Mostly), Mostly);
    }

    #[test]
    fn active_filters_count_what_filters_and_bind_the_same_query() {
        let mut filters = Filters::default();
        assert_eq!(filters.active(), 0);

        // Spaces alone filter nothing, in the count and in the query
        filters.text = "   ".into();
        assert_eq!(filters.active(), 0);
        assert_eq!(filters.query(Sort::Action, true).text, None);

        let day = Ymd {
            year: 2021,
            month: 12,
            day: 31,
        };
        filters.map = " crossfire ".into();
        filters.from = Some(day);
        filters.to = Some(day);
        filters.spectating = Spectating::Mostly;
        assert_eq!(filters.active(), 4);

        let query = filters.query(Sort::Score, false);
        assert_eq!(query.map.as_deref(), Some("crossfire"));
        assert_eq!(query.from_date.as_deref(), Some("2021-12-31"));
        assert_eq!(query.to_date.as_deref(), Some("2021-12-31"));
        // More than half the playback spectating, and at most half playing
        assert!(query.min_spectating.is_some_and(|s| s > 0.5));
        assert_eq!(query.max_spectating, None);
        assert_eq!((query.sort, query.descending), (Sort::Score, false));

        filters.spectating = Spectating::Playing;
        let query = filters.query(Sort::Score, false);
        assert_eq!(
            (query.min_spectating, query.max_spectating),
            (None, Some(0.5))
        );
    }

    #[test]
    fn names_lose_only_their_dem_ending() {
        assert_eq!(display_name("F:/demos/run_92.dem"), "run_92");
        assert_eq!(display_name("F:/demos/.dem"), ".dem");
        assert_eq!(display_name("F:/demos/notes.txt"), "notes.txt");
    }
}
