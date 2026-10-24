//! The game's own content options, which add the `_addon`, `_hd` and
//! language folders to what it searches
//!
//! The game keeps "Allow custom addon content" and "Use high quality
//! models" as `addons_folder` and `hdmodels` under the user's
//! `Software\Valve\Half-Life\Settings`, in the registry on Windows and in
//! Steam's `registry.vdf` elsewhere. Steam keeps the language it runs the
//! game in with the game's manifest, `appmanifest_70.acf` in the library's
//! `steamapps` folder, and English has no folder of its own. The viewer's
//! settings can turn each on or off for the viewer alone, leaving the
//! game's as they are

use std::path::{Path, PathBuf};

use lambda_slicer_assets::search::SearchOptions;

/// Half-Life's Steam app
const MANIFEST: &str = "appmanifest_70.acf";

/// The language that has no folder of its own
const ENGLISH: &str = "english";

/// Half-Life folders in the order searched, each with the content options
/// it is searched with
pub type GameFolders = Vec<(PathBuf, SearchOptions)>;

/// How the viewer's settings change the game's content options, each none
/// to follow the game
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Overrides {
    pub hd: Option<bool>,
    pub addon: Option<bool>,
    /// The language folders of the language Steam runs the game in, which
    /// the game uses whenever it isn't English
    pub localized: Option<bool>,
}

/// The content options for the installation at `install`, the game's own
/// unless `overrides` says otherwise
fn options(install: &Path, overrides: Overrides) -> SearchOptions {
    let (hd, addon) = switches();
    let localized = overrides.localized.unwrap_or(true);
    SearchOptions {
        language: language(install).filter(|_| localized),
        hd: overrides.hd.unwrap_or(hd),
        addon: overrides.addon.unwrap_or(addon),
    }
}

/// Each of `installs` with the content options it is searched with, in the
/// order searched
pub fn searched(installs: &[PathBuf], overrides: Overrides) -> GameFolders {
    let mut out = Vec::new();
    for install in installs {
        out.push((install.clone(), options(install, overrides)));
    }
    out
}

/// The language Steam runs the game installed at `install` in, none for
/// English or when Steam's manifest isn't beside it. A Steam library holds
/// the game in `steamapps/common/Half-Life`, and the manifest in
/// `steamapps`
pub fn language(install: &Path) -> Option<String> {
    let steamapps = install.parent()?.parent()?;
    let text = std::fs::read(steamapps.join(MANIFEST)).ok()?;
    let text = String::from_utf8_lossy(&text);
    let language = value(&text, &["AppState", "UserConfig", "language"])?;
    let language = language.trim().to_ascii_lowercase();
    (!language.is_empty() && language != ENGLISH).then_some(language)
}

/// Whether the game's high quality models and custom addon content are on
#[cfg(windows)]
pub fn switches() -> (bool, bool) {
    let on = |name: &str| registry_dword(name).is_some_and(|v| v != 0);
    (on("hdmodels"), on("addons_folder"))
}

/// Whether the game's high quality models and custom addon content are on,
/// from Steam's copy of the registry
#[cfg(not(windows))]
pub fn switches() -> (bool, bool) {
    let Some(home) = std::env::var_os("HOME") else {
        return (false, false);
    };
    let Ok(text) = std::fs::read(PathBuf::from(home).join(".steam/registry.vdf")) else {
        return (false, false);
    };
    let text = String::from_utf8_lossy(&text);
    let settings = [
        "Registry",
        "HKCU",
        "Software",
        "Valve",
        "Half-Life",
        "Settings",
    ];
    let on = |name: &str| {
        let mut path = settings.to_vec();
        path.push(name);
        value(&text, &path).is_some_and(|v| v.trim() != "0" && !v.trim().is_empty())
    };
    (on("hdmodels"), on("addons_folder"))
}

/// A number the game keeps under the user's Half-Life settings
#[cfg(windows)]
fn registry_dword(name: &str) -> Option<u32> {
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};

    let wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain([0]).collect() };
    let key = wide(r"Software\Valve\Half-Life\Settings");
    let value = wide(name);
    let mut data: u32 = 0;
    let mut size = size_of::<u32>() as u32;

    // SAFETY The key and value names end in nought, and the data is a u32
    // of the size given, which the call only fills with a DWORD
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut data).cast(),
            &mut size,
        )
    };
    (status == ERROR_SUCCESS).then_some(data)
}

/// A part of a Valve key-values text, a quoted or bare word or a brace
#[derive(Debug, PartialEq)]
enum Token {
    Word(String),
    Open,
    Close,
}

/// The words and braces of a key-values `text`, its comments left out
fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => out.push(Token::Open),
            '}' => out.push(Token::Close),
            '"' => {
                let mut word = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => {
                            if let Some(escaped) = chars.next() {
                                word.push(escaped);
                            }
                        }
                        c => word.push(c),
                    }
                }
                out.push(Token::Word(word));
            }
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            c if c.is_whitespace() => {}
            c => {
                let mut word = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '{' || next == '}' || next == '"' {
                        break;
                    }
                    word.push(next);
                    chars.next();
                }
                out.push(Token::Word(word));
            }
        }
    }
    out
}

/// The value at `path` in a key-values `text`, each key matched ignoring
/// case, as Steam's files are
fn value(text: &str, path: &[&str]) -> Option<String> {
    let tokens = tokens(text);
    let mut depth = 0;
    // How many of the path's keys the blocks open so far have matched
    let mut matched = 0;
    let mut k = 0;
    while k < tokens.len() {
        match &tokens[k] {
            Token::Open => depth += 1,
            Token::Close => {
                if depth == matched && matched > 0 {
                    // Leaving a matched block, the path cannot be in it
                    return None;
                }
                depth -= 1;
            }
            Token::Word(key) => {
                let wanted = depth == matched && key.eq_ignore_ascii_case(path[matched]);
                match tokens.get(k + 1) {
                    Some(Token::Word(found)) => {
                        if wanted && matched + 1 == path.len() {
                            return Some(found.clone());
                        }
                        k += 1;
                    }
                    Some(Token::Open) if wanted && matched + 1 < path.len() => matched += 1,
                    _ => {}
                }
            }
        }
        k += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST_TEXT: &str = r#"
"AppState"
{
	"appid"		"70"
	"name"		"Half-Life"
	"InstalledDepots"
	{
		"1"		{ "language"	"french" }
	}
	"UserConfig"
	{
		"language"		"spanish"
		"BetaKey"		"steam_legacy"
	}
}
"#;

    #[test]
    fn a_value_is_found_by_its_path_and_not_in_another_block() {
        let found = value(MANIFEST_TEXT, &["AppState", "UserConfig", "language"]);
        assert_eq!(found.as_deref(), Some("spanish"));
        assert_eq!(
            value(MANIFEST_TEXT, &["appstate", "userconfig", "betakey"]).as_deref(),
            Some("steam_legacy")
        );
        assert_eq!(value(MANIFEST_TEXT, &["AppState", "language"]), None);
        assert_eq!(value(MANIFEST_TEXT, &["AppState", "UserConfig", "x"]), None);
    }

    #[test]
    fn the_language_comes_from_the_manifest_of_the_library_holding_the_game() {
        let dir = tempfile::tempdir().unwrap();
        let steamapps = dir.path().join("steamapps");
        let install = steamapps.join("common").join("Half-Life");
        std::fs::create_dir_all(&install).unwrap();
        assert_eq!(language(&install), None);

        std::fs::write(steamapps.join(MANIFEST), MANIFEST_TEXT).unwrap();
        assert_eq!(language(&install).as_deref(), Some("spanish"));

        // English has no folder of its own
        let english = MANIFEST_TEXT.replace("spanish", "English");
        std::fs::write(steamapps.join(MANIFEST), english).unwrap();
        assert_eq!(language(&install), None);
    }
}
