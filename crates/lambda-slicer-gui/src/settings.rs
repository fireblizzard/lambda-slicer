//! Settings kept between runs, as JSON in the platform's configuration
//! directory. A missing or unreadable file gives the defaults, and saving
//! writes a temporary file first so a crash never leaves half a file

use std::path::PathBuf;

use lambda_slicer_playback::decals::MAX_DECALS;
use serde::{Deserialize, Serialize};

use crate::i18n::Locale;
use crate::library::{MAX_KILLS, MAX_MINUTES};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// None follows the system language. A language this version does not
    /// know follows the system's too, rather than losing every other setting
    #[serde(deserialize_with = "known_locale")]
    pub locale: Option<Locale>,
    /// Parallel analyses, 1 to 4
    pub workers: u8,
    pub folders: Vec<PathBuf>,
    /// The demos marked as favourites, by their path as the library shows
    /// it
    pub favourites: Vec<String>,
    /// The demos hidden from the library's list, by their path as the
    /// library shows it
    pub hidden_demos: Vec<String>,
    /// None uses the platform's cache directory
    pub cache: Option<PathBuf>,
    /// Library columns left out, gauss shots by default
    pub hidden_columns: Vec<String>,
    /// The library's tab when the app starts, by its key
    pub start_tab: String,
    /// The library's order when the app starts, by a column's key, newest
    /// or highest first when `sort_descending`
    pub sort_column: String,
    pub sort_descending: bool,
    /// The least length in minutes and the fewest kills the library's
    /// filters ask of a demo when the app starts, 0 for any
    pub start_min_minutes: f32,
    pub start_min_kills: u32,
    /// The fewest kills of a killing spree whose last kill shows a badge
    /// in the events
    pub spree_badge: u32,
    /// Whether the library leaves out the maps of a demo of several maps
    /// that nobody was killed on and that were recorded for less than five
    /// minutes, rather than naming every map
    pub hide_unplayed_maps: bool,
    /// Whether a demo opens with its player lanes under the timeline. Files
    /// from before the timeline was always shown name it `show_timeline`
    #[serde(alias = "show_timeline")]
    pub player_lanes: bool,
    /// Seconds a new clip keeps before and after the event it is made
    /// around
    pub preroll: f32,
    pub postroll: f32,
    /// Where clips are exported, beside each demo when none is set
    pub export_folder: Option<PathBuf>,
    /// Exported clips are named after the clip rather than its times, with
    /// the demo's name first when `demo_name_first` is also set
    pub clip_name_files: bool,
    pub demo_name_first: bool,
    pub viewer: ViewerSettings,
    /// A layout this version cannot read opens the first way, keeping every
    /// other setting
    #[serde(deserialize_with = "readable_layout")]
    pub layout: LayoutSettings,
}

/// How the window was laid out, kept so the app opens the way it was left
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutSettings {
    /// The filter pane beside the library's table, and the filters over the
    /// narrow list beside an open demo
    pub filters_open: bool,
    pub list_filters_open: bool,
    /// The demo list was hidden beside an open demo
    pub list_hidden: bool,
    /// The width the inspector was dragged to, none for its first width
    pub inspector_width: Option<f32>,
    /// The share of the inspector's height the divider under the scoreboard
    /// was dragged to give it, none to give it what it needs
    pub scores_share: Option<f32>,
    /// Where the settings window was left, none to open it in the middle
    pub settings_at: Option<[f32; 2]>,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            filters_open: true,
            list_filters_open: false,
            list_hidden: false,
            inspector_width: None,
            scores_share: None,
            settings_at: None,
        }
    }
}

impl LayoutSettings {
    /// Sizes and places that are numbers, others forgotten
    fn clamped(self) -> Self {
        let width = self.inspector_width.filter(|w| w.is_finite() && *w > 0.);
        let share = self.scores_share.filter(|s| (0. ..=1.).contains(s));
        let settings_at = self
            .settings_at
            .filter(|at| at.iter().all(|c| c.is_finite()));
        Self {
            inspector_width: width,
            scores_share: share,
            settings_at,
            ..self
        }
    }
}

/// The saved layout, or the first one when it cannot be read
fn readable_layout<'de, D>(deserializer: D) -> Result<LayoutSettings, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}

/// A saved language this version knows, or none to follow the system's.
/// Anything else, even a value that is not text, follows the system's
fn known_locale<'de, D>(deserializer: D) -> Result<Option<Locale>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let locale = value.as_str().and_then(Locale::from_tag);
    Ok(locale)
}

/// Which way a speed is counted, across the ground, up or down, or both
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeedAxes {
    #[default]
    Horizontal,
    Vertical,
    Both,
}

impl SpeedAxes {
    pub const ALL: [SpeedAxes; 3] = [SpeedAxes::Horizontal, SpeedAxes::Vertical, SpeedAxes::Both];

    /// How fast `velocity` goes counted this way, in units a second
    pub fn speed(self, [x, y, z]: [f32; 3]) -> f32 {
        match self {
            SpeedAxes::Horizontal => (x * x + y * y).sqrt(),
            SpeedAxes::Vertical => z.abs(),
            SpeedAxes::Both => (x * x + y * y + z * z).sqrt(),
        }
    }

    /// The name the settings give it
    pub fn text_id(self) -> &'static str {
        match self {
            SpeedAxes::Horizontal => "settings-speed-horizontal",
            SpeedAxes::Vertical => "settings-speed-vertical",
            SpeedAxes::Both => "settings-speed-both",
        }
    }
}

/// How the demo viewer shows and plays a demo
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewerSettings {
    /// The Half-Life folder, the one holding `valve`. `--game` comes first
    pub game: Option<PathBuf>,
    /// Other Half-Life folders searched in turn for what the first lacks,
    /// such as maps or a model, the first first
    pub more_games: Vec<PathBuf>,
    /// The game's HD models, custom addon content and localized content,
    /// each turned on or off for the viewer alone, none to follow the game
    pub hd_models: Option<bool>,
    pub addon_content: Option<bool>,
    pub localized_content: Option<bool>,
    /// The game's `gamma`, `brightness`, `texgamma`, `lightgamma` and
    /// `gl_overbright`
    pub gamma: f32,
    pub brightness: f32,
    pub texgamma: f32,
    pub lightgamma: f32,
    pub overbright: bool,
    /// Degrees across the view's width, none for the recorded one
    pub fov: Option<f32>,
    /// Degrees across the view's width while zoomed in, as `default_fov`
    /// would set them in the game
    pub zoom_fov: f32,
    /// Whether the weapon in view is drawn, its crosshair and the chat
    pub viewmodel: bool,
    pub crosshair: bool,
    pub chat: bool,
    /// Whether the recorder's health, armour, ammo and pickups show, and
    /// their size against the game's
    pub vitals: bool,
    pub vitals_size: f32,
    /// Whether they show with the free camera too, for watching from a
    /// place of one's own while still seeing the recorder's state
    pub free_vitals: bool,
    /// Whether the small scoreboard shows at the top left, and its most
    /// rows
    pub scores: bool,
    pub score_rows: usize,
    /// Whether the server's settings show at the top right when it sends
    /// them
    pub server_settings: bool,
    /// Whether the text the server places on the HUD shows, such as a bhop
    /// or KZ server's timer
    pub hud_messages: bool,
    /// The sound's master volume, 0 to 1
    pub volume: f32,
    /// Players shown through walls, and with `see_through_items` the other
    /// models too, projectiles and items among them
    pub see_through: bool,
    pub see_through_items: bool,
    /// Each player's name over them, hidden by walls unless players are
    /// seen through them
    pub player_names: bool,
    /// Where the picture is seen from and the way it looks, over it
    pub show_position: bool,
    /// The recorder's speed under the crosshair, and their speed as they
    /// last jumped under it, each counted along `speed_axes`
    pub speedometer: bool,
    pub jump_speed: bool,
    pub speed_axes: SpeedAxes,
    /// Whether choosing an event leaves the free camera for the view it is
    /// best seen from, rather than keeping the camera where it is
    pub events_leave_free: bool,
    /// The most decals left in play that are shown, the newest. The map's
    /// own are always shown
    pub decals: usize,
}

impl Default for ViewerSettings {
    fn default() -> Self {
        let g = lambda_slicer_render::gamma::Gamma::default();
        Self {
            game: None,
            more_games: Vec::new(),
            hd_models: None,
            addon_content: None,
            localized_content: None,
            gamma: g.gamma,
            brightness: g.brightness,
            texgamma: g.texgamma,
            lightgamma: g.lightgamma,
            overbright: g.overbright,
            fov: None,
            zoom_fov: 30.,
            viewmodel: true,
            crosshair: true,
            chat: true,
            vitals: true,
            vitals_size: 0.6,
            free_vitals: false,
            scores: true,
            score_rows: 4,
            server_settings: true,
            hud_messages: true,
            volume: 0.1,
            see_through: false,
            see_through_items: false,
            player_names: true,
            show_position: false,
            speedometer: false,
            jump_speed: false,
            speed_axes: SpeedAxes::Horizontal,
            events_leave_free: false,
            decals: 100,
        }
    }
}

impl ViewerSettings {
    pub fn gamma(&self) -> lambda_slicer_render::gamma::Gamma {
        lambda_slicer_render::gamma::Gamma {
            gamma: self.gamma,
            brightness: self.brightness,
            texgamma: self.texgamma,
            lightgamma: self.lightgamma,
            overbright: self.overbright,
        }
    }

    /// Within the game's ranges, a number that is not one taking its
    /// default
    fn clamped(self) -> Self {
        let default = Self::default();
        let within =
            |v: f32, d: f32, lo: f32, hi: f32| if v.is_finite() { v.clamp(lo, hi) } else { d };
        Self {
            gamma: within(self.gamma, default.gamma, 1.8, 3.),
            brightness: within(self.brightness, default.brightness, 0., 3.),
            texgamma: within(self.texgamma, default.texgamma, 1.8, 3.),
            lightgamma: within(self.lightgamma, default.lightgamma, 1.8, 3.),
            fov: self
                .fov
                .filter(|f| f.is_finite())
                .map(|f| f.clamp(FOV.0, FOV.1)),
            zoom_fov: within(self.zoom_fov, default.zoom_fov, FOV.0, FOV.1),
            volume: within(self.volume, default.volume, 0., 1.),
            vitals_size: within(
                self.vitals_size,
                default.vitals_size,
                VITALS_SIZE.0,
                VITALS_SIZE.1,
            ),
            decals: self.decals.min(MAX_DECALS),
            score_rows: self.score_rows.min(MAX_SCORE_ROWS),
            ..self
        }
    }
}

/// The fields of view the viewer takes, in degrees
pub const FOV: (f32, f32) = (10., 150.);

/// The most rows the small scoreboard takes, the game's most players
pub const MAX_SCORE_ROWS: usize = 32;

/// The sizes health, armour and ammo take against the game's
pub const VITALS_SIZE: (f32, f32) = (0.3, 1.5);

/// The longest padding a clip takes around an event, in seconds
pub const MAX_PADDING: f32 = 120.;

/// The fewest and most kills a killing spree's badge can be asked to show
/// from
pub const SPREE_LEAST: u32 = 2;
pub const SPREE_MOST: u32 = 999;

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: None,
            workers: 1,
            folders: Vec::new(),
            favourites: Vec::new(),
            hidden_demos: Vec::new(),
            cache: None,
            hidden_columns: vec!["gauss".into()],
            start_tab: "demos".into(),
            sort_column: "modified".into(),
            sort_descending: true,
            start_min_minutes: 0.,
            start_min_kills: 0,
            spree_badge: crate::demo::DEFAULT_SPREE,
            hide_unplayed_maps: true,
            player_lanes: false,
            preroll: 4.,
            postroll: 1.,
            export_folder: None,
            clip_name_files: true,
            demo_name_first: true,
            viewer: ViewerSettings::default(),
            layout: LayoutSettings::default(),
        }
    }
}

/// Write `bytes` to a temporary file beside `path`, then put it in place,
/// so a crash never leaves half a file
pub fn write_json(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path)
}

/// `settings.json` in the platform's configuration directory
pub fn path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }?;

    Some(base.join("lambda-slicer").join("settings.json"))
}

impl Settings {
    pub fn load_from(path: &std::path::Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .map(Settings::clamped)
            .unwrap_or_default()
    }

    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        write_json(path, &serde_json::to_vec_pretty(self)?)
    }

    fn clamped(mut self) -> Self {
        self.workers = self.workers.clamp(1, 4);
        let padding = |p: f32| {
            if p.is_finite() {
                p.clamp(0., MAX_PADDING)
            } else {
                0.
            }
        };
        self.preroll = padding(self.preroll);
        self.postroll = padding(self.postroll);
        self.start_min_minutes = if self.start_min_minutes.is_finite() {
            self.start_min_minutes.clamp(0., MAX_MINUTES)
        } else {
            0.
        };
        self.start_min_kills = self.start_min_kills.min(MAX_KILLS);
        self.spree_badge = self.spree_badge.clamp(SPREE_LEAST, SPREE_MOST);
        self.viewer = self.viewer.clamped();
        self.layout = self.layout.clamped();
        self
    }

    /// The chosen language, or the system's
    pub fn locale(&self) -> Locale {
        self.locale.unwrap_or_else(Locale::detect)
    }

    /// Every setting back to its default, but for where things are, the
    /// game, the library's folders and where the library is kept, and where
    /// the settings window sits
    pub fn restored(&self) -> Self {
        let mut restored = Self {
            folders: self.folders.clone(),
            favourites: self.favourites.clone(),
            hidden_demos: self.hidden_demos.clone(),
            cache: self.cache.clone(),
            ..Self::default()
        };
        restored.viewer.game = self.viewer.game.clone();
        restored.viewer.more_games = self.viewer.more_games.clone();
        restored.layout.settings_at = self.layout.settings_at;
        restored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_bad_files_give_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("settings.json");
        assert_eq!(Settings::load_from(&path), Settings::default());

        let s = Settings {
            locale: Some(Locale::Uk),
            workers: 3,
            folders: vec![PathBuf::from("F:/demos")],
            favourites: vec!["F:/demos/run_92.dem".into()],
            hidden_demos: vec!["F:/demos/run_93.dem".into()],
            cache: None,
            hidden_columns: Vec::new(),
            start_tab: "matches".into(),
            sort_column: "kills".into(),
            sort_descending: false,
            start_min_minutes: 5.,
            start_min_kills: 25,
            spree_badge: 15,
            hide_unplayed_maps: false,
            player_lanes: true,
            preroll: 2.5,
            postroll: 0.,
            export_folder: Some(PathBuf::from("F:/clips")),
            clip_name_files: false,
            demo_name_first: false,
            viewer: ViewerSettings {
                game: Some(PathBuf::from("F:/Half-Life")),
                gamma: 2.,
                brightness: 2.,
                lightgamma: 2.,
                overbright: true,
                fov: Some(110.),
                zoom_fov: 45.,
                viewmodel: false,
                volume: 0.25,
                ..ViewerSettings::default()
            },
            layout: LayoutSettings {
                filters_open: false,
                list_hidden: true,
                inspector_width: Some(400.),
                settings_at: Some([120., 80.]),
                ..LayoutSettings::default()
            },
        };
        s.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path), s);

        std::fs::write(
            &path,
            b"{\"workers\": 9, \"locale\": \"zh-Hans\", \"preroll\": -4, \"postroll\": 1e9, \"start_min_minutes\": 1e9, \"start_min_kills\": 99999}",
        )
        .unwrap();
        let loaded = Settings::load_from(&path);
        assert_eq!((loaded.workers, loaded.locale), (4, Some(Locale::ZhHans)));
        assert_eq!((loaded.preroll, loaded.postroll), (0., MAX_PADDING));
        assert_eq!(loaded.start_min_minutes, MAX_MINUTES);
        assert_eq!(loaded.start_min_kills, MAX_KILLS);

        // The viewer's settings stay in the game's ranges
        std::fs::write(
            &path,
            b"{\"viewer\": {\"gamma\": 9, \"brightness\": -1, \"fov\": 500, \"zoom_fov\": 1, \"volume\": 2}}",
        )
        .unwrap();
        let viewer = Settings::load_from(&path).viewer;
        assert_eq!((viewer.gamma, viewer.brightness), (3., 0.));
        assert_eq!((viewer.fov, viewer.volume), (Some(FOV.1), 1.));
        assert_eq!(viewer.zoom_fov, FOV.0);
        assert!(viewer.viewmodel);

        // No more decals than playback keeps
        std::fs::write(&path, b"{\"viewer\": {\"decals\": 99999}}").unwrap();
        assert_eq!(Settings::load_from(&path).viewer.decals, MAX_DECALS);

        std::fs::write(&path, b"not json").unwrap();
        assert_eq!(Settings::load_from(&path), Settings::default());
    }

    #[test]
    fn restoring_keeps_where_things_are() {
        let mut s = Settings {
            locale: Some(Locale::Fr),
            workers: 3,
            folders: vec![PathBuf::from("F:/demos")],
            cache: Some(PathBuf::from("F:/cache.sqlite")),
            player_lanes: true,
            ..Settings::default()
        };
        s.favourites = vec!["F:/demos/run_92.dem".into()];
        s.hidden_demos = vec!["F:/demos/run_93.dem".into()];
        s.viewer.game = Some(PathBuf::from("F:/Half-Life"));
        s.viewer.more_games = vec![PathBuf::from("G:/Half-Life")];
        s.viewer.decals = 5;
        s.layout.inspector_width = Some(400.);
        s.layout.settings_at = Some([10., 20.]);

        let restored = s.restored();
        assert_eq!(restored.folders, s.folders);
        assert_eq!(restored.cache, s.cache);
        assert_eq!(restored.viewer.game, s.viewer.game);
        assert_eq!(restored.viewer.more_games, s.viewer.more_games);
        assert_eq!(restored.favourites, s.favourites);
        assert_eq!(restored.hidden_demos, s.hidden_demos);
        assert_eq!(restored.layout.settings_at, s.layout.settings_at);

        // Everything else is the default
        let mut expected = restored.clone();
        expected.folders.clear();
        expected.cache = None;
        expected.viewer.game = None;
        expected.viewer.more_games.clear();
        expected.favourites.clear();
        expected.hidden_demos.clear();
        expected.layout.settings_at = None;
        assert_eq!(expected, Settings::default());
    }

    #[test]
    fn settings_that_kept_the_game_program_still_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let saved = br#"{"locale": "en", "folders": ["F:/demos"], "preroll": 2.0,
            "viewer": {"game": "F:/Half-Life", "executable": "F:/Half-Life/hl.exe",
            "volume": 0.5}}"#;
        std::fs::write(&path, saved).unwrap();

        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.viewer.game, Some(PathBuf::from("F:/Half-Life")));
        assert_eq!((loaded.viewer.volume, loaded.preroll), (0.5, 2.));
        assert_eq!(loaded.folders, [PathBuf::from("F:/demos")]);
    }

    #[test]
    fn speeds_count_across_the_ground_up_and_down_or_both() {
        let velocity = [300., 400., -1200.];
        assert_eq!(SpeedAxes::Horizontal.speed(velocity), 500.);
        assert_eq!(SpeedAxes::Vertical.speed(velocity), 1200.);
        assert_eq!(SpeedAxes::Both.speed(velocity), 1300.);
    }

    #[test]
    fn an_old_timeline_choice_opens_the_player_lanes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        // Shown before means the lanes now, and hidden only leaves them shut,
        // as the timeline itself is always there
        std::fs::write(&path, br#"{"show_timeline": true}"#).unwrap();
        assert!(Settings::load_from(&path).player_lanes);
        std::fs::write(&path, br#"{"show_timeline": false}"#).unwrap();
        assert!(!Settings::load_from(&path).player_lanes);

        // Saved again, it goes under its new name and reads back the same
        let lanes = Settings {
            player_lanes: true,
            ..Settings::default()
        };
        lanes.save_to(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("player_lanes") && !text.contains("show_timeline"));
        assert!(Settings::load_from(&path).player_lanes);
    }

    #[test]
    fn clips_take_their_names_for_file_names_until_asked_not_to() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        // A file from before the choice was there
        std::fs::write(&path, br#"{"workers": 2}"#).unwrap();
        let loaded = Settings::load_from(&path);
        assert!(loaded.clip_name_files);
        assert!(loaded.demo_name_first);

        // A choice made stays
        std::fs::write(&path, br#"{"clip_name_files": false}"#).unwrap();
        assert!(!Settings::load_from(&path).clip_name_files);
    }

    #[test]
    fn the_library_starts_with_the_newest_files_until_asked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        // A file from before the choice was there
        std::fs::write(&path, br#"{"workers": 2}"#).unwrap();
        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.sort_column, "modified");
        assert!(loaded.sort_descending);
    }

    #[test]
    fn values_this_version_cannot_read_lose_nothing_else() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let saved = br#"{"locale": "xx-Unknown", "folders": ["F:/demos"],
            "viewer": {"game": "F:/Half-Life"},
            "layout": {"inspector_width": "wide", "settings_at": [1, 3]}}"#;
        std::fs::write(&path, saved).unwrap();

        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.locale, None);
        assert_eq!(loaded.folders, [PathBuf::from("F:/demos")]);
        assert_eq!(loaded.viewer.game, Some(PathBuf::from("F:/Half-Life")));
        assert_eq!(loaded.layout, LayoutSettings::default());

        // A language saved as something other than text follows the system's
        std::fs::write(&path, br#"{"locale": 3, "workers": 2}"#).unwrap();
        let loaded = Settings::load_from(&path);
        assert_eq!((loaded.locale, loaded.workers), (None, 2));
    }
}
