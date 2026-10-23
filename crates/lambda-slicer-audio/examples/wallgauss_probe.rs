//! Says for each Gauss kill of real demos whether it went through a wall,
//! as the viewer's enrichment reconstructs it
//!
//! `wallgauss_probe INSTALL DEMO...`
//!
//! Each kill is listed with its session, playback time, killer, victim, the
//! shots of the killer's taken as its candidates and the verdict, so the
//! kills can be watched in the viewer and the verdicts checked by eye

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_audio::library::Library;
use lambda_slicer_core::analysis::analyze_demo_with;
use lambda_slicer_core::analysis::context::Kill;
use lambda_slicer_core::container::{Limits, read_header};
use lambda_slicer_playback::kill_features::{self, GaussKill};

fn probe(install: &Path, demo: &Path) {
    let game = std::fs::File::open(demo).ok().and_then(|mut file| {
        let len = file.metadata().ok()?.len();
        Some(read_header(&mut file, len).ok()?.game_dir())
    });
    let Some(game) = game else {
        println!("{}: no header", demo.display());
        return;
    };
    let search = match SearchPath::new(
        install,
        &game,
        &SearchOptions::default(),
        AssetLimits::default(),
    ) {
        Ok(search) => search,
        Err(e) => {
            println!("{}: {e}", demo.display());
            return;
        }
    };
    let library = Arc::new(Library::new(search, AssetLimits::default()));

    let report = analyze_demo_with(demo, Limits::default(), None, None);
    let mut gauss: Vec<&Kill> = Vec::new();
    let mut kills: Vec<GaussKill> = Vec::new();
    for kill in &report.kills {
        let found = GaussKill::of(kill, &report.occurrences, report.max_clients);
        if let Some(found) = found {
            gauss.push(kill);
            kills.push(found);
        }
    }

    println!("{}: {} Gauss kills", demo.display(), kills.len());
    let hulls = library.game_files().hulls;
    let cancel = AtomicBool::new(false);
    let result = kill_features::classify(demo, hulls, &kills, &cancel, |index, verdict, _| {
        let kill = gauss[index];
        let name = |p: &Option<lambda_slicer_core::state::PlayerIdentity>| {
            p.as_ref().map_or("?".to_string(), |p| p.name.clone())
        };
        println!(
            "  {} {:9.3} {} -> {} shots {} {:?}",
            kill.segment,
            kill.time,
            name(&kill.killer),
            name(&kill.victim),
            kills[index].shots.len(),
            verdict
        );
    });
    if let Err(e) = result {
        println!("  stopped: {}", e.code);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((install, demos)) = args.split_first() else {
        eprintln!("wallgauss_probe INSTALL DEMO...");
        std::process::exit(2);
    };

    for demo in demos {
        probe(Path::new(install), Path::new(demo));
    }
}
