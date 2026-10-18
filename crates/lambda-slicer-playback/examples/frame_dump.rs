//! Prints what playback presents at a moment, nearest entities first
//!
//! `frame_dump DEMO SECONDS`

use std::path::Path;

use lambda_slicer_core::container::Limits;
use lambda_slicer_playback::controller::{CameraMode, Stamp};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::SourceTime;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [demo, seconds, ..] = args.as_slice() else {
        eprintln!("usage: frame_dump DEMO SECONDS");
        std::process::exit(2);
    };
    let at = SourceTime(seconds.parse().expect("seconds"));

    let mut p = Playback::open(Path::new(demo), Limits::default()).expect("demo");
    p.advance_to(at).expect("playback");
    let f = p.present(Stamp::default(), at, CameraMode::Recorder, false);
    println!(
        "{:?} at {:.3}, server time {:.3}, map {:?}, local entity {}",
        f.position, f.at.0, f.server_time.0, f.map, f.local_entity
    );
    println!("recorder {:?}", f.recorder);
    for p in &f.players {
        println!("player {:?}", p);
    }

    let eye = f.recorder.origin;
    let distance = |o: [f32; 3]| {
        ((o[0] - eye[0]).powi(2) + (o[1] - eye[1]).powi(2) + (o[2] - eye[2]).powi(2)).sqrt()
    };
    let mut entities: Vec<_> = f.entities.iter().collect();
    entities.sort_by(|a, b| distance(a.origin).total_cmp(&distance(b.origin)));
    for e in entities {
        println!(
            "{:5.0} away  #{} life {} {:?} seq {} frame {:.0} rate {:.2} animtime {:.3} effects {} iuser {} {} origin {:?} angles {:?}",
            distance(e.origin),
            e.id.number,
            e.id.life,
            e.model.as_deref().unwrap_or("?"),
            e.sequence,
            e.frame,
            e.framerate,
            e.animtime,
            e.effects,
            e.iuser1,
            e.iuser2,
            e.origin,
            e.angles,
        );
    }
}
