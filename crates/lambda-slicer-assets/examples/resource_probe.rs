//! Development survey of how a game installation resolves demos' resources
//!
//! Usage is `resource_probe <install> <game> <demos...>`. Decodes each demo,
//! takes every session's resource list, looks each resource up and prints
//! how many came from each folder and every one that is missing

use std::collections::BTreeMap;
use std::path::PathBuf;

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::resource::{Listed, Outcome, resolve_all};
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_core::protocol::messages::{Message, consistency_positions};
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

fn main() {
    let mut args = std::env::args().skip(1);
    let install = PathBuf::from(args.next().expect("install folder"));
    let game = args.next().expect("game folder");
    let search = SearchPath::new(
        &install,
        &game,
        &SearchOptions::default(),
        Limits::default(),
    )
    .unwrap();
    println!(
        "search: {}",
        search
            .sources()
            .iter()
            .map(|s| s.label.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for demo in args {
        let options = DecodeOptions {
            collect_events: false,
            ..DecodeOptions::default()
        };
        let mut lists = Vec::new();
        let _ = decode_demo(&PathBuf::from(&demo), &options, |_, d, m| {
            if let Message::ResourceList {
                resources,
                consistency,
            } = &m.message
            {
                let checked = consistency_positions(consistency.as_deref().unwrap_or_default());
                lists.push((d.segment, resources.clone(), checked));
            }
        });
        println!("{demo}");
        for (segment, list, checked) in lists {
            let listed = list.iter().enumerate().map(|(i, r)| Listed {
                kind: r.kind,
                name: r.name.as_slice(),
                checked: checked.contains(&i),
            });
            let resolved = resolve_all(&search, listed, Limits::default());
            let mut by_source: BTreeMap<String, usize> = BTreeMap::new();
            let mut missing = Vec::new();
            for r in &resolved {
                let key = match &r.outcome {
                    Outcome::Found(f) => f.source.clone(),
                    Outcome::InDecals(f) => format!("{} (decals.wad)", f.source),
                    Outcome::MapModel => "map brush model".to_string(),
                    Outcome::Sentence => "sentence".to_string(),
                    Outcome::Check => "file check".to_string(),
                    Outcome::Missing(e) => {
                        missing.push(format!("{:?} {}: {e}", r.kind, r.name));
                        "missing".to_string()
                    }
                };
                *by_source.entry(key).or_default() += 1;
            }
            println!(
                "  session {segment}: {} resources, {by_source:?}",
                resolved.len()
            );
            for m in missing {
                println!("    {m}");
            }
        }
    }
}
