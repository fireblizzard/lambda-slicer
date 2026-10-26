//! Development survey of a demo's sessions and where a restart can start
//!
//! Usage is `session_probe <demo> [START-END...]`. Prints the first session,
//! the group from which each later session can be restarted, and the groups
//! where sessions change, then snaps each range as an export would

use std::path::PathBuf;

use lambda_slicer_core::container::Limits;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};
use lambda_slicer_core::timeline::time::DemoTime;

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).unwrap());
    let options = DecodeOptions {
        limits: Limits::default(),
        collect_events: false,
        cancel: None,
    };
    let (r, _) = decode_demo(&path, &options, |_, _, _| {});
    let g = &r.groups;
    println!(
        "loading {:?} ready {:?} changes {:?}",
        g.loading_segment, g.ready, g.session_changes
    );
    for pair in std::env::args().skip(2) {
        let (a, b) = pair.split_once('-').unwrap();
        let (a, b): (DemoTime, DemoTime) = (a.parse().unwrap(), b.parse().unwrap());
        match g.snap(a, Some(b), None) {
            Ok(s) => println!(
                "{a}..{b}: {}..{} segment {:?}",
                s.start,
                s.end,
                g.segment_at(a)
            ),
            Err(e) => println!("{a}..{b}: {e}"),
        }
    }
}
