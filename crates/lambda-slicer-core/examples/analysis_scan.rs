//! Development scan of the weapon analysis over a list of demos
//!
//! Usage is `analysis_scan --list FILE`. Prints one tab-separated line per
//! demo, then totals. Read-only

use std::collections::BTreeMap;

use lambda_slicer_core::analysis::analyze_demo;
use lambda_slicer_core::analysis::weapons::{Action, Actor, Evidence};
use lambda_slicer_core::container::Limits;

/// Local and network records of one shot arrive this close together
const PAIR_WINDOW: f32 = 0.1;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [flag, list] = args.as_slice() else {
        eprintln!("usage: analysis_scan --list FILE");
        std::process::exit(2);
    };
    assert_eq!(flag, "--list");

    let mut actions: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut unknown_actors = 0;
    let mut recorder_local = 0;
    let mut recorder_network = 0;
    let mut paired = 0;

    println!(
        "path\tkind\tdecode\tshots\tunknown actors\trecorder local\trecorder network\tpaired\tkills\tspectating\tweapons"
    );
    for path in std::fs::read_to_string(list)
        .unwrap()
        .lines()
        .map(str::trim)
    {
        if path.is_empty() {
            continue;
        }

        let r = analyze_demo(path.as_ref(), Limits::default());
        let recorder = r
            .recorder
            .as_ref()
            .map(|p| (p.slot, p.incarnation, p.segment));
        let by_recorder = |a: &Actor| match a {
            Actor::Player { identity, .. } => {
                Some((identity.slot, identity.incarnation, identity.segment)) == recorder
            }
            _ => false,
        };

        let shots: Vec<_> = r.occurrences.iter().filter(|o| o.shot).collect();
        let unknown = r
            .occurrences
            .iter()
            .filter(|o| !matches!(o.actor, Actor::Player { .. }))
            .count();
        let local: Vec<_> = shots
            .iter()
            .filter(|o| by_recorder(&o.actor) && o.evidence == Evidence::LocalEvent)
            .collect();
        let network: Vec<_> = shots
            .iter()
            .filter(|o| by_recorder(&o.actor) && o.evidence != Evidence::LocalEvent)
            .collect();
        let pairs = network
            .iter()
            .filter(|n| {
                local
                    .iter()
                    .any(|l| l.weapon == n.weapon && (l.time - n.time).abs() <= PAIR_WINDOW)
            })
            .count();

        for o in &r.occurrences {
            let key = format!("{:?} {:?} {}", o.weapon, o.action, o.mode.unwrap_or(""));
            let e = actions.entry(key).or_default();
            e.0 += 1;
            if matches!(o.action, Action::Fire | Action::Launch)
                && !matches!(o.actor, Actor::Player { .. })
            {
                e.1 += 1;
            }
        }
        unknown_actors += unknown;
        recorder_local += local.len();
        recorder_network += network.len();
        paired += pairs;

        let weapons: Vec<_> = r
            .weapons
            .iter()
            .filter(|w| w.shots > 0)
            .map(|w| format!("{}:{}", w.weapon.name(), w.shots))
            .collect();
        println!(
            "{path}\t{:?}\t{:?}\t{}\t{unknown}\t{}\t{}\t{pairs}\t{}\t{:.2}\t{}",
            r.coverage.kind,
            r.decode,
            shots.len(),
            local.len(),
            network.len(),
            r.kills.len(),
            r.coverage.spectator_fraction,
            weapons.join(",")
        );
    }

    println!(
        "TOTAL unknown actors {unknown_actors}, recorder shots local {recorder_local} network {recorder_network}, paired {paired}"
    );
    for (k, (n, unattributed)) in &actions {
        println!("ACTION {k:<36} {n:>9} {unattributed:>7}");
    }
}
