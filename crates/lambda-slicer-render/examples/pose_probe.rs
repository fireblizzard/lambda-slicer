//! Poses a player model and a weapon it holds, timing each step
//!
//! `pose_probe INSTALL GAME PLAYER WEAPON SEQUENCE GAIT`

use std::time::Instant;

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_render::light::ModelLight;
use lambda_slicer_render::load::{self, EntityPose, GaitPose};
use lambda_slicer_render::model::Eye;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, game, player, weapon, sequence, gait, ..] = args.as_slice() else {
        eprintln!("usage: pose_probe INSTALL GAME PLAYER WEAPON SEQUENCE GAIT");
        std::process::exit(2);
    };
    let search = SearchPath::new(
        install.as_ref(),
        game,
        &SearchOptions::default(),
        Limits::default(),
    )
    .expect("search path");
    let started = Instant::now();
    let holder = load::player(&search, player).expect("player");
    println!("player {} in {:?}", holder.path, started.elapsed());
    let held = load::model(&search, weapon);
    println!(
        "weapon {:?} in {:?}",
        held.as_ref().map(|m| &m.path),
        started.elapsed()
    );
    if let Ok(held) = &held {
        for t in &held.mesh.textures {
            let mut colours: Vec<[u8; 4]> = t
                .pixels
                .chunks(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect();
            colours.sort();
            colours.dedup();
            println!(
                "  texture {}x{}, {} colours, first {:?}",
                t.width,
                t.height,
                colours.len(),
                &colours[..colours.len().min(3)]
            );
        }
    }

    let pose = EntityPose {
        sequence: sequence.parse().unwrap(),
        cycle: 55.,
        elapsed: 0.1,
        framerate: 1.,
        controllers: [127.; 4],
        blending: [0.; 2],
        origin: [0.; 3],
        angles: [0., 135., 0.],
        player: true,
        gait: Some(GaitPose {
            sequence: gait.parse().unwrap(),
            yaw: 135.,
            torso: 127.5,
            distance: 0.,
            elapsed: 0.93,
        }),
        previous: None,
    };
    let started = Instant::now();
    let bones = holder.pose(&pose).expect("player pose");
    println!("player posed in {:?}", started.elapsed());
    let eye = Eye::new([0.; 3], [0.; 3]);
    let vertices = holder.mesh.skin(&bones, &ModelLight::UNLIT, &eye);
    println!(
        "player skinned, {} corners, in {:?}",
        vertices.len(),
        started.elapsed()
    );
    if let Ok(held) = held {
        let merged = held
            .merged_pose(&pose, &holder, &bones)
            .expect("weapon pose");
        println!("weapon posed in {:?}", started.elapsed());
        let vertices = held.mesh.skin(&merged, &ModelLight::UNLIT, &eye);
        println!(
            "weapon skinned, {} corners, in {:?}",
            vertices.len(),
            started.elapsed()
        );
    }
}
