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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: None,
            workers: 1,
            folders: Vec::new(),
            cache: None,
            hidden_columns: vec!["gauss".into()],
        }
    }
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
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&temp, path)
    }

    fn clamped(mut self) -> Self {
        self.workers = self.workers.clamp(1, 4);
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
        };
        s.save_to(&path).unwrap();
        assert_eq!(Settings::load_from(&path), s);

        std::fs::write(&path, b"{\"workers\": 9, \"locale\": \"zh-Hans\"}").unwrap();
        let loaded = Settings::load_from(&path);
        assert_eq!((loaded.workers, loaded.locale), (4, Some(Locale::ZhHans)));

        std::fs::write(&path, b"not json").unwrap();
        assert_eq!(Settings::load_from(&path), Settings::default());
    }
}
