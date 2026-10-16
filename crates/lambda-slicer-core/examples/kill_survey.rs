//! Development survey of `DeathMsg` weapon strings across demos
//!
//! Usage is `kill_survey --list FILE`. Counts each string by whether the
//! killer slot holds a player, the world (0), or the victim itself

use std::collections::BTreeMap;

use lambda_slicer_core::analysis::analyze_demo;
use lambda_slicer_core::container::Limits;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [flag, list] = args.as_slice() else {
        eprintln!("usage: kill_survey --list FILE");
        std::process::exit(2);
    };
    assert_eq!(flag, "--list");

    // (string, killer kind) -> (kills, files)
    let mut counts: BTreeMap<(String, &str), (u64, u64)> = BTreeMap::new();
    for path in std::fs::read_to_string(list)
        .unwrap()
        .lines()
        .map(str::trim)
    {
        if path.is_empty() {
            continue;
        }

        let r = analyze_demo(path.as_ref(), Limits::default());
        let mut seen = std::collections::BTreeSet::new();
        for k in &r.kills {
            let killer = if k.killer_entity == 0 {
                "world slot"
            } else if k.killer_entity == k.victim_entity {
                "self"
            } else {
                "player"
            };
            let key = (k.weapon_name.clone(), killer);
            let e = counts.entry(key.clone()).or_default();
            e.0 += 1;
            if seen.insert(key) {
                e.1 += 1;
            }
        }
    }

    for ((name, killer), (n, files)) in &counts {
        println!("{name:24} {killer:11} {n:8} {files:6}");
    }
}
