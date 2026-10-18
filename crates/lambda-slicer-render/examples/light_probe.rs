//! Samples a map's light under points, timing the traces
//!
//! `light_probe INSTALL GAME MAP X Y Z`

use std::time::Instant;

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_render::load;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, game, map, x, y, z, ..] = args.as_slice() else {
        eprintln!("usage: light_probe INSTALL GAME MAP X Y Z");
        std::process::exit(2);
    };
    let point = [x, y, z].map(|v| v.parse::<f32>().unwrap());
    let search = SearchPath::new(
        install.as_ref(),
        game,
        &SearchOptions::default(),
        Limits::default(),
    )
    .expect("search path");
    let (mesh, _) = load::map(&search, map, Limits::default()).expect("map");
    let styles = [1.; 64];

    let started = Instant::now();
    println!(
        "at {point:?}: {:?} in {:?}",
        mesh.light.at(point, &styles),
        started.elapsed()
    );

    // A grid around the point, to find a trace that takes long
    let started = Instant::now();
    let mut lit = 0;
    for i in -20..=20 {
        for j in -20..=20 {
            let p = [
                point[0] + i as f32 * 50.,
                point[1] + j as f32 * 50.,
                point[2],
            ];
            let one = Instant::now();
            if mesh.light.at(p, &styles).is_some() {
                lit += 1;
            }
            if one.elapsed().as_millis() > 50 {
                println!("slow at {p:?}: {:?}", one.elapsed());
            }
        }
    }
    println!("{lit} of 1681 lit in {:?}", started.elapsed());
}
