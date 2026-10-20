//! Finding the Half-Life installation the viewer reads its files from (GUI
//! redesign plan §8.4)
//!
//! The viewer draws and plays demos itself, so `hl.exe` only says where the
//! installation is. The folder holding it and `valve` is what the viewer
//! reads. A folder chosen as a folder needs no executable, which is how
//! Linux and settings saved before the executable was kept find it

use std::path::{Path, PathBuf};

use fluent_bundle::FluentValue;

use crate::i18n::Catalog;

/// The game's executable on this platform, by name
pub const EXECUTABLE: &str = if cfg!(windows) { "hl.exe" } else { "hl_linux" };

/// Files of which every Half-Life installation's `valve` folder holds at
/// least one
const VALVE_FILES: [&str; 3] = ["liblist.gam", "pak0.pak", "halflife.wad"];

/// Whether `root` holds a Half-Life installation the viewer can read. A few
/// files in its `valve` folder tell, rather than a walk through all of it
pub fn looks_installed(root: &Path) -> bool {
    let valve = root.join("valve");
    if !valve.is_dir() {
        return false;
    }

    for name in VALVE_FILES {
        if valve.join(name).is_file() {
            return true;
        }
    }
    false
}

/// The executable a saved folder holds, for settings kept before the
/// executable was
pub fn executable_in(root: &Path) -> Option<PathBuf> {
    let executable = root.join(EXECUTABLE);
    executable.is_file().then_some(executable)
}

/// An installation a choice names
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installation {
    pub root: PathBuf,
    pub executable: Option<PathBuf>,
}

/// Why a choice is not an installation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A file that is not the game's executable, or nothing, by its name
    NotExecutable(String),
    /// A folder, or an executable's folder, that holds no installation
    NotInstalled(PathBuf),
}

impl Refusal {
    pub fn message(&self, catalog: &Catalog) -> String {
        match self {
            Refusal::NotExecutable(name) => catalog.text_with(
                "setup-not-executable",
                &[
                    ("name", FluentValue::from(name.clone())),
                    ("executable", FluentValue::from(EXECUTABLE)),
                ],
            ),
            Refusal::NotInstalled(folder) => {
                let folder = FluentValue::from(folder.display().to_string());
                catalog.text_with("setup-not-installed", &[("folder", folder)])
            }
        }
    }
}

/// The installation `chosen` names, the game's executable or the folder
/// holding `valve`. A folder that holds the executable keeps it too
pub fn accept(chosen: &Path) -> Result<Installation, Refusal> {
    if chosen.is_dir() {
        if !looks_installed(chosen) {
            return Err(Refusal::NotInstalled(chosen.to_path_buf()));
        }
        return Ok(Installation {
            root: chosen.to_path_buf(),
            executable: executable_in(chosen),
        });
    }

    // A shortcut or another program is not the executable, even when it
    // starts the game
    let name = chosen
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| chosen.display().to_string());
    if !chosen.is_file() || !name.eq_ignore_ascii_case(EXECUTABLE) {
        return Err(Refusal::NotExecutable(name));
    }

    let Some(root) = chosen.parent() else {
        return Err(Refusal::NotInstalled(chosen.to_path_buf()));
    };
    if !looks_installed(root) {
        return Err(Refusal::NotInstalled(root.to_path_buf()));
    }

    Ok(Installation {
        root: root.to_path_buf(),
        executable: Some(chosen.to_path_buf()),
    })
}

/// Ask where the installation is, starting in `start`, as the executable on
/// Windows and as the folder elsewhere. `executables` names the dialog's
/// filter. None when the dialog is cancelled
pub fn pick(start: Option<&Path>, executables: &str) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(folder) = start.filter(|f| f.is_dir()) {
        dialog = dialog.set_directory(folder);
    }

    if cfg!(windows) {
        dialog.add_filter(executables, &["exe"]).pick_file()
    } else {
        dialog.pick_folder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder with `valve` and the files asked for in it
    fn install(valve_files: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("valve")).unwrap();
        for name in valve_files {
            std::fs::write(dir.path().join("valve").join(name), b"").unwrap();
        }
        dir
    }

    #[test]
    fn the_executable_names_its_folder() {
        let dir = install(&["liblist.gam"]);
        let executable = dir.path().join(EXECUTABLE);
        std::fs::write(&executable, b"").unwrap();

        let found = accept(&executable).unwrap();
        assert_eq!(found.root, dir.path());
        assert_eq!(found.executable, Some(executable.clone()));

        // A folder holding the executable keeps it, as saved settings that
        // name only the folder do
        let found = accept(dir.path()).unwrap();
        assert_eq!(found.executable, Some(executable));
    }

    #[test]
    fn a_folder_with_the_game_files_needs_no_executable() {
        let dir = install(&["pak0.pak"]);
        let found = accept(dir.path()).unwrap();
        assert_eq!(found.root, dir.path());
        assert_eq!(found.executable, None);
    }

    #[test]
    fn other_files_and_folders_are_refused_with_the_reason() {
        let dir = install(&["halflife.wad"]);

        // Another program, and a file that is not there
        let launcher = dir.path().join("launcher.exe");
        std::fs::write(&launcher, b"").unwrap();
        let refused = accept(&launcher).unwrap_err();
        assert_eq!(refused, Refusal::NotExecutable("launcher.exe".into()));
        let missing = dir.path().join(EXECUTABLE);
        assert_eq!(
            accept(&missing).unwrap_err(),
            Refusal::NotExecutable(EXECUTABLE.into())
        );

        // The executable or a folder away from the game's files
        let empty = tempfile::tempdir().unwrap();
        let stray = empty.path().join(EXECUTABLE);
        std::fs::write(&stray, b"").unwrap();
        assert_eq!(
            accept(&stray).unwrap_err(),
            Refusal::NotInstalled(empty.path().to_path_buf())
        );
        assert_eq!(
            accept(empty.path()).unwrap_err(),
            Refusal::NotInstalled(empty.path().to_path_buf())
        );

        // A `valve` folder without any of the game's files
        let bare = install(&[]);
        assert!(!looks_installed(bare.path()));
    }

    #[test]
    fn the_executable_name_ignores_case() {
        let dir = install(&["liblist.gam"]);
        let shouting = dir.path().join(EXECUTABLE.to_uppercase());
        std::fs::write(&shouting, b"").unwrap();
        assert_eq!(accept(&shouting).unwrap().root, dir.path());
    }
}
