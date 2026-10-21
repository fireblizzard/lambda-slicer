//! The floor under players on the game's maps, so a kill counts as midair
//! only well above it, not in a plain jump or a drop from a spawn point
//!
//! Each game folder a demo names is searched once, and each map read once,
//! for as long as the reader is kept

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_core::analysis::motion::{Floors, MapName};
use lambda_slicer_core::library::scan::ScanFloors;

/// The game files of each game folder, none for one that cannot be read
type Libraries = Mutex<HashMap<String, Option<Arc<Library>>>>;

/// A reader of the floor under players on the maps of the game installed
/// at `install`, named by that folder so analyses made with other game
/// files are told apart
pub fn reader(install: &Path) -> ScanFloors {
    let libraries = Libraries::default();
    let root = install.to_path_buf();

    let floors: Floors = Arc::new(move |map: &MapName, origin| {
        let library = library_for(&libraries, &root, &map.game_dir)?;
        library.player_hulls(&map.path)?.height_above_floor(origin)
    });

    ScanFloors {
        files: install.display().to_string(),
        floors,
    }
}

/// The game files in `game_dir` of the game at `root`, searched for once
fn library_for(known: &Libraries, root: &Path, game_dir: &str) -> Option<Arc<Library>> {
    let mut known = known.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(library) = known.get(game_dir) {
        return library.clone();
    }

    let options = SearchOptions::default();
    let search = SearchPath::new(root, game_dir, &options, AssetLimits::default());
    let library = search
        .ok()
        .map(|search| Arc::new(Library::new(search, AssetLimits::default())));
    known.insert(game_dir.to_owned(), library.clone());
    library
}
