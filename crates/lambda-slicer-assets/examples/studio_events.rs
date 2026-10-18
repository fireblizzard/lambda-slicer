//! Lists the client events in the sequences of every studio model in the
//! given folders, the events the game's client acts on itself while it
//! draws a model, such as muzzle flashes and sounds
//!
//! `studio_events FOLDER...`
//!
//! Events from 5000 on are the client's. Each line gives the model, the
//! sequence, the frame, the event and its options

use std::collections::BTreeMap;
use std::path::Path;

use lambda_slicer_assets::xenoxide::studio::StudioModel;

fn main() {
    let mut kinds: BTreeMap<i32, usize> = BTreeMap::new();
    let (mut read, mut failed) = (0, 0);
    for folder in std::env::args().skip(1) {
        let Ok(entries) = std::fs::read_dir(Path::new(&folder)) else {
            eprintln!("{folder}: not a folder");
            continue;
        };
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            if !bytes.starts_with(b"IDST") {
                continue;
            }
            let Ok(unresolved) = StudioModel::parse(&bytes) else {
                continue;
            };
            // Sequence group files sit beside the model
            let groups: Vec<(usize, Vec<u8>)> = unresolved
                .dependencies()
                .iter()
                .filter_map(|d| {
                    let file = Path::new(&d.name).file_name()?;
                    Some((d.index, std::fs::read(path.with_file_name(file)).ok()?))
                })
                .collect();
            let files: Vec<(usize, &[u8])> =
                groups.iter().map(|(i, b)| (*i, b.as_slice())).collect();
            let model = match unresolved.resolve(&files) {
                Ok(model) => model,
                Err(e) => {
                    failed += 1;
                    eprintln!("{}: {e:?}", path.display());
                    continue;
                }
            };
            read += 1;
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            for sequence in model.sequences() {
                for event in sequence.events.iter().filter(|e| e.event >= 5000) {
                    *kinds.entry(event.event).or_insert(0) += 1;
                    println!(
                        "{name} {} frame {} event {} {:?}",
                        sequence.label, event.frame, event.event, event.options
                    );
                }
            }
        }
    }
    println!("{read} models read, {failed} failed");
    for (event, count) in kinds {
        println!("  event {event}: {count}");
    }
}
