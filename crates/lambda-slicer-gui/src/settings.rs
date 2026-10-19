//! Settings kept between runs, as JSON in the platform's configuration
//! directory. A missing or unreadable file gives the defaults, and saving
//! writes a temporary file first so a crash never leaves half a file

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Locale;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// None follows the system language
    pub locale: Option<Locale>,
    /// Parallel analyses, 1 to 4
    pub workers: u8,
    pub folders: Vec<PathBuf>,
    /// None uses the platform's cache directory
    pub cache: Option<PathBuf>,
    /// Library columns left out, gauss shots by default
    pub hidden_columns: Vec<String>,
    /// Whether a demo shows its timeline, hidden until asked for
    pub show_timeline: bool,
    /// Seconds a new clip keeps before and after the event it is made
    /// around
    pub preroll: f32,
    pub postroll: f32,
    /// Where clips are exported, beside each demo when none is set
    pub export_folder: Option<PathBuf>,
    pub viewer: ViewerSettings,
}

/// How the demo viewer shows and plays a demo
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewerSettings {
    /// The Half-Life folder, the one holding `valve`. `--game` comes first
    pub game: Option<PathBuf>,
    /// The game's `gamma`, `brightness`, `texgamma`, `lightgamma` and
    /// `gl_overbright`
    pub gamma: f32,
    pub brightness: f32,
    pub texgamma: f32,
    pub lightgamma: f32,
    pub overbright: bool,
    /// Degrees across the view's width, none for the recorded one
    pub fov: Option<f32>,
    /// Whether the weapon in view is drawn
    pub viewmodel: bool,
    /// The sound's master volume, 0 to 1
    pub volume: f32,
    /// Players shown through walls, and with `see_through_items` the other
    /// models too, projectiles and items among them
    pub see_through: bool,
    pub see_through_items: bool,
}

impl Default for ViewerSettings {
    fn default() -> Self {
        let g = lambda_slicer_render::gamma::Gamma::default();
        Self {
            game: None,
            gamma: g.gamma,
            brightness: g.brightness,
            texgamma: g.texgamma,
            lightgamma: g.lightgamma,
            overbright: g.overbright,
            fov: None,
            viewmodel: true,
            volume: 0.5,
            see_through: false,
            see_through_items: false,
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
            volume: within(self.volume, default.volume, 0., 1.),
            ..self
        }
    }
}

/// The fields of view the viewer takes, in degrees
pub const FOV: (f32, f32) = (10., 150.);

/// The longest padding a clip takes around an event, in seconds
pub const MAX_PADDING: f32 = 120.;

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: None,
            workers: 1,
            folders: Vec::new(),
            cache: None,
            hidden_columns: vec!["gauss".into()],
            show_timeline: false,
            preroll: 5.,
            postroll: 3.,
            export_folder: None,
            viewer: ViewerSettings::default(),
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
        self.viewer = self.viewer.clamped();
        self
    }

    /// The chosen language, or the system's
    pub fn locale(&self) -> Locale {
        self.locale.unwrap_or_else(Locale::detect)
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
            cache: None,
            hidden_columns: Vec::new(),
            show_timeline: true,
            preroll: 2.5,
            postroll: 0.,
            export_folder: Some(PathBuf::from("F:/clips")),
            viewer: ViewerSettings {
                game: Some(PathBuf::from("F:/Half-Life")),
                gamma: 2.,
                brightness: 2.,
                lightgamma: 2.,
                overbright: true,
                fov: Some(110.),
                viewmodel: false,
                volume: 0.25,
                ..ViewerSettings::default()
            },
        };
        s.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path), s);

        std::fs::write(
            &path,
            b"{\"workers\": 9, \"locale\": \"zh-Hans\", \"preroll\": -4, \"postroll\": 1e9}",
        )
        .unwrap();
        let loaded = Settings::load_from(&path);
        assert_eq!((loaded.workers, loaded.locale), (4, Some(Locale::ZhHans)));
        assert_eq!((loaded.preroll, loaded.postroll), (0., MAX_PADDING));

        // The viewer's settings stay in the game's ranges
        std::fs::write(
            &path,
            b"{\"viewer\": {\"gamma\": 9, \"brightness\": -1, \"fov\": 500, \"volume\": 2}}",
        )
        .unwrap();
        let viewer = Settings::load_from(&path).viewer;
        assert_eq!((viewer.gamma, viewer.brightness), (3., 0.));
        assert_eq!((viewer.fov, viewer.volume), (Some(FOV.1), 1.));
        assert!(viewer.viewmodel);

        std::fs::write(&path, b"not json").unwrap();
        assert_eq!(Settings::load_from(&path), Settings::default());
    }
}
