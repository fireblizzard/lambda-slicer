use crate::xenoxide::studio::fixture::{Bone, Controller, Fixture, Hitbox, Run, Sequence, run};
use sha2::{Digest, Sha256};

use super::*;

const AT_ORIGIN: Placement = Placement {
    origin: [0.0; 3],
    angles: [0.0; 3],
};

fn model(fixture: &Fixture) -> StudioModel {
    StudioModel::parse(&fixture.build().bytes)
        .unwrap()
        .resolve(&[])
        .unwrap()
}

/// A bone with no controllers; translations scale 1, rotations 0.01 radians per unit
fn bone(name: &'static str, parent: i32, value: [f32; 6]) -> Bone {
    Bone {
        name,
        parent,
        controllers: [-1; 6],
        value,
        scale: [1.0, 1.0, 1.0, 0.01, 0.01, 0.01],
    }
}

type BoneTracks = [Option<Vec<Run>>; 6];

/// No tracks for any of `bones` bones
fn still(bones: usize) -> Vec<BoneTracks> {
    vec![Default::default(); bones]
}

/// One track of `frames` frames holding `value`
fn constant(value: i16, frames: u8) -> Option<Vec<Run>> {
    Some(vec![run(&[value], frames)])
}

fn fixture(bones: Vec<Bone>, sequences: Vec<Sequence>) -> Fixture {
    Fixture {
        bones,
        controllers: Vec::new(),
        hitboxes: Vec::new(),
        groups: vec![("default", "")],
        sequences,
        textures: 0,
        body_parts: Vec::new(),
        texture_data: Vec::new(),
        skins: Vec::new(),
        textures_in_companion: false,
    }
}

fn inputs(sequence: usize, frame: f64) -> PoseInputs {
    PoseInputs {
        current: SequenceState {
            sequence,
            frame,
            blend: [0.0; 2],
        },
        controllers: [0.0; 4],
        mouth: 0.0,
        transition: None,
        gait: None,
    }
}

fn pose(model: &StudioModel, inputs: &PoseInputs) -> EvaluatedPose {
    evaluate_pose(model, inputs, &AT_ORIGIN).unwrap()
}

fn translation(pose: &EvaluatedPose, bone: usize) -> [f64; 3] {
    [0, 1, 2].map(|row| pose.bones()[bone][row][3])
}

/// Equal within `1e-9`, for expectations derived by hand
fn close(left: [f64; 3], right: [f64; 3]) -> bool {
    left.iter()
        .zip(right)
        .all(|(left, right)| (left - right).abs() < 1e-9)
}

/// SHA-256 over the bits of every value of every transform, in order: equal digests
/// mean bit-identical poses
fn digest(pose: &EvaluatedPose) -> String {
    let mut hasher = Sha256::new();
    for transform in pose.bones() {
        for row in transform {
            for value in row {
                hasher.update(value.to_bits().to_le_bytes());
            }
        }
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// A three-bone chain with only default translations: every rotation is the identity,
/// so each bone's world translation is the placement's origin plus the chain's offsets
#[test]
fn a_still_chain_adds_its_offsets_to_the_origin() {
    let model = model(&fixture(
        vec![
            bone("root", -1, [1.0, 2.0, 3.0, 0.0, 0.0, 0.0]),
            bone("upper", 0, [10.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            bone("hand", 1, [5.0, 0.5, 0.0, 0.0, 0.0, 0.0]),
        ],
        vec![Sequence::new("still", 10.0, 1, vec![still(3)])],
    ));
    let placement = Placement {
        origin: [100.0, 200.0, 300.0],
        angles: [0.0; 3],
    };
    let pose = evaluate_pose(&model, &inputs(0, 0.0), &placement).unwrap();
    assert_eq!(translation(&pose, 0), [101.0, 202.0, 303.0]);
    assert_eq!(translation(&pose, 1), [111.0, 202.0, 303.0]);
    assert_eq!(translation(&pose, 2), [116.0, 202.5, 303.0]);
    for transform in pose.bones() {
        assert_eq!(
            [transform[0][0], transform[1][1], transform[2][2]],
            [1.0; 3]
        );
    }
}

/// A bone's rotation turns its children: yaw (ZR) of a quarter turn on the upper arm
/// carries the hand's offset from X to Y; and the placement's yaw turns the whole model
#[test]
fn rotations_turn_children_and_the_placement_turns_the_model() {
    let quarter = std::f32::consts::FRAC_PI_2;
    let model = model(&fixture(
        vec![
            bone("root", -1, [0.0; 6]),
            bone("upper", 0, [10.0, 0.0, 0.0, 0.0, 0.0, quarter]),
            bone("hand", 1, [5.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        ],
        vec![Sequence::new("still", 10.0, 1, vec![still(3)])],
    ));
    // The quarter turn is stored as an `f32`, a little over a quarter
    let (sin, cos) = (quarter as f64).sin_cos();
    let pose = pose(&model, &inputs(0, 0.0));
    assert!(
        close(translation(&pose, 2), [10.0 + 5.0 * cos, 5.0 * sin, 0.0]),
        "{:?}",
        translation(&pose, 2)
    );
    // A quarter turn of yaw: model X to world Y, model Y to world -X
    let turned = evaluate_pose(
        &model,
        &inputs(0, 0.0),
        &Placement {
            origin: [0.0; 3],
            angles: [0.0, 90.0, 0.0],
        },
    )
    .unwrap();
    let (sin90, cos90) = std::f64::consts::FRAC_PI_2.sin_cos();
    let hand = [10.0 + 5.0 * cos, 5.0 * sin];
    assert!(close(
        translation(&turned, 2),
        [
            hand[0] * cos90 - hand[1] * sin90,
            hand[0] * sin90 + hand[1] * cos90,
            0.0
        ]
    ));
    // Pitch is negated for studio models: a positive pitch raises model X
    let pitched = evaluate_pose(
        &model,
        &inputs(0, 0.0),
        &Placement {
            origin: [0.0; 3],
            angles: [30.0, 0.0, 0.0],
        },
    )
    .unwrap();
    let upper = translation(&pitched, 1);
    assert!(
        close(upper, [10.0 * 0.75_f64.sqrt(), 0.0, 5.0]),
        "{upper:?}"
    );
}

/// Translations and rotations read their tracks as the renderer does: halfway through
/// frame 0, two one-frame runs of 5 and 6 translate by 5, one run of [5, 6] by 5.5;
/// a rotation interpolates towards the next frame in both cases
#[test]
fn tracks_interpolate_and_hold_as_the_renderer_reads_them() {
    let split = || Some(vec![run(&[5], 1), run(&[6], 1)]);
    let joined = || Some(vec![run(&[5, 6], 2)]);
    let mut tracks = still(3);
    tracks[0][0] = split();
    tracks[1][0] = joined();
    // Rotation about Z: 0.01 rad per unit
    tracks[2][5] = Some(vec![run(&[0], 1), run(&[100], 1)]);
    let model = model(&fixture(
        vec![
            bone("a", -1, [0.0; 6]),
            bone("b", -1, [0.0; 6]),
            bone("c", -1, [0.0; 6]),
        ],
        vec![Sequence::new("track", 10.0, 2, vec![tracks])],
    ));
    let pose = pose(&model, &inputs(0, 0.5));
    assert_eq!(translation(&pose, 0)[0], 5.0);
    assert_eq!(translation(&pose, 1)[0], 5.5);
    // Halfway from 0 to 100 units of 0.01 radians (an `f32`) about Z
    let transform = pose.bones()[2];
    let (sin, cos) = (50.0 * 0.01_f32 as f64).sin_cos();
    assert!((transform[0][0] - cos).abs() < 1e-12);
    assert!((transform[1][0] - sin).abs() < 1e-12);
}

/// The frame's limits: the exact last frame reads that frame; past it the renderer
/// restarts at 0; just below 0 the fraction is negative; below -0.01 it is held there.
/// A single-frame sequence reads its only frame.
#[test]
fn frames_at_and_past_their_limits() {
    let mut tracks = still(1);
    tracks[0][0] = Some(vec![run(&[0, 10, 20, 30], 4)]);
    let model = model(&fixture(
        vec![bone("a", -1, [0.0; 6])],
        vec![
            Sequence::new("four", 10.0, 4, vec![tracks]),
            Sequence::new(
                "one",
                10.0,
                1,
                vec![{
                    let mut tracks = still(1);
                    tracks[0][0] = constant(7, 1);
                    tracks
                }],
            ),
        ],
    ));
    let x = |sequence, frame| translation(&pose(&model, &inputs(sequence, frame)), 0)[0];
    assert_eq!(x(0, 3.0), 30.0);
    assert_eq!(x(0, 2.5), 25.0);
    assert_eq!(x(0, 3.5), 0.0);
    assert!((x(0, -0.005) - -0.05).abs() < 1e-12);
    assert!((x(0, -0.5) - -0.1).abs() < 1e-12);
    assert_eq!(x(1, 0.0), 7.0);
    assert_eq!(x(1, 0.75), 7.0);
}

#[test]
fn frames_are_estimated_without_clocks() {
    let sequence = |frames, looping| {
        let fixture = fixture(
            vec![bone("a", -1, [0.0; 6])],
            vec![Sequence {
                flags: if looping { 1 } else { 0 },
                ..Sequence::new("s", 10.0, frames, vec![still(1)])
            }],
        );
        model(&fixture).sequences()[0].clone()
    };
    let looping = sequence(5, true);
    assert_eq!(estimate_frame(&looping, 0.0, 6.0), 2.0);
    assert_eq!(estimate_frame(&looping, 0.0, -1.0), 3.0);
    assert_eq!(estimate_frame(&looping, 128.0, 0.0), 2.0);
    assert_eq!(estimate_frame(&looping, 0.0, 4.0), 0.0);
    let once = sequence(5, false);
    assert_eq!(estimate_frame(&once, 0.0, 10.0), 5.0 - 1.001);
    assert_eq!(estimate_frame(&once, 0.0, -3.0), 0.0);
    assert_eq!(estimate_frame(&once, 64.0, 0.5), 1.5);
    let single = sequence(1, false);
    assert_eq!(estimate_frame(&single, 200.0, 0.3), 0.0);
    let single_looping = sequence(1, true);
    assert_eq!(estimate_frame(&single_looping, 200.0, 0.3), 0.3);
}

/// A player's pitch becomes the first blend control as `StudioPlayerBlend` makes it:
/// three times the pitch, truncated, mapped into the blend range and truncated again;
/// beyond the range the control saturates and the rest of the pitch is left for the
/// model; a range under 0.1 (a compiler error) gives 127
#[test]
fn a_players_pitch_becomes_its_blend() {
    let sequence = |start: f32, end: f32| {
        let fixture = fixture(
            vec![bone("a", -1, [0.0; 6])],
            vec![Sequence {
                blend_start: [start, 0.0],
                blend_end: [end, 0.0],
                ..Sequence::new("aim", 10.0, 1, vec![still(1), still(1)])
            }],
        );
        model(&fixture).sequences()[0].clone()
    };
    let aim = sequence(-90.0, 90.0);
    assert_eq!(player_blend(&aim, 10.0), (170.0, 0.0));
    assert_eq!(player_blend(&aim, 10.1), (170.0, 0.0));
    assert_eq!(player_blend(&aim, 11.0), (174.0, 0.0));
    // The first truncation matters: 30.9 maps from 30 (not 171), and 90.6 truncates to
    // 90, inside the range, so no pitch is left over
    assert_eq!(player_blend(&aim, 10.3), (170.0, 0.0));
    assert_eq!(player_blend(&aim, 30.2), (255.0, 0.0));
    assert_eq!(player_blend(&aim, 40.0), (255.0, 10.0));
    assert_eq!(player_blend(&aim, -40.0), (0.0, -10.0));
    assert_eq!(player_blend(&aim, 30.0), (255.0, 0.0));
    let flat = sequence(0.0, 0.0);
    assert_eq!(player_blend(&flat, 0.0), (127.0, 0.0));
}

/// Controllers map as `StudioCalcBoneAdj` does: an ordinary controller clamps its
/// fraction of 255 into its range; a wrapping one maps linearly, 360 degrees to 256;
/// the mouth saturates at 64; rotations are in degrees and become radians
#[test]
fn controllers_map_clamp_and_wrap() {
    let driven = |controller: usize| {
        let mut controllers = [-1; 6];
        controllers[0] = controller as i32;
        Bone {
            controllers,
            ..bone("driven", -1, [0.0; 6])
        }
    };
    let mut yawed = bone("yawed", -1, [0.0; 6]);
    yawed.controllers[5] = 3;
    let mut fixture = fixture(
        vec![driven(0), driven(1), driven(2), yawed],
        vec![Sequence::new("still", 10.0, 1, vec![still(4)])],
    );
    fixture.controllers = vec![
        Controller {
            bone: 0,
            kind: STUDIO_X,
            start: 0.0,
            end: 10.0,
            rest: 0,
            index: 0,
        },
        Controller {
            bone: 1,
            kind: STUDIO_X | 0x8000,
            start: 2.0,
            end: 0.0,
            rest: 0,
            index: 1,
        },
        Controller {
            bone: 2,
            kind: STUDIO_X,
            start: 0.0,
            end: 8.0,
            rest: 0,
            index: 4,
        },
        Controller {
            bone: 3,
            kind: STUDIO_ZR,
            start: -90.0,
            end: 90.0,
            rest: 0,
            index: 2,
        },
    ];
    let model = model(&fixture);
    let at = |controllers: [f64; 4], mouth: f64| {
        pose(
            &model,
            &PoseInputs {
                controllers,
                mouth,
                ..inputs(0, 0.0)
            },
        )
    };
    let x = |pose: &EvaluatedPose, bone| translation(pose, bone)[0];
    assert_eq!(x(&at([0.0, 0.0, 0.0, 0.0], 0.0), 0), 0.0);
    assert_eq!(x(&at([255.0, 0.0, 0.0, 0.0], 0.0), 0), 10.0);
    assert_eq!(x(&at([127.5, 0.0, 0.0, 0.0], 0.0), 0), 5.0);
    assert_eq!(x(&at([400.0, 0.0, 0.0, 0.0], 0.0), 0), 10.0);
    assert_eq!(x(&at([-5.0, 0.0, 0.0, 0.0], 0.0), 0), 0.0);
    // Wrapping: 64 is a quarter turn, plus the start; any value maps linearly
    assert_eq!(x(&at([0.0, 64.0, 0.0, 0.0], 0.0), 1), 92.0);
    assert_eq!(x(&at([0.0, -0.5, 0.0, 0.0], 0.0), 1), 2.0 - 0.703125);
    // The mouth: half open at 32, fully at 64 and beyond
    assert_eq!(x(&at([0.0; 4], 32.0), 2), 4.0);
    assert_eq!(x(&at([0.0; 4], 100.0), 2), 8.0);
    // Yaw from -90 to 90 degrees: 255 is a quarter turn
    let yaw = at([0.0, 0.0, 255.0, 0.0], 0.0).bones()[3];
    assert!((yaw[1][0] - 1.0).abs() < 1e-12 && yaw[0][0].abs() < 1e-12);

    // Interpolation between updates: the short way round for a wrapping controller
    assert_eq!(interpolate_controller(250, 5, 0.5, true), -0.5);
    assert_eq!(interpolate_controller(250, 5, 0.5, false), 127.5);
    assert_eq!(interpolate_controller(10, 20, 0.25, true), 12.5);
    assert_eq!(
        interpolate_controller(5, 250, 0.0, true),
        5.0 + 128.0 - 128.0 - 0.0
    );
}

/// Blends at and beyond their boundaries: two blends slerp by the first control, four
/// as the renderer pairs them
#[test]
fn blends_meet_their_boundaries() {
    let blend = |value: i16| {
        let mut tracks = still(1);
        tracks[0][0] = constant(value, 1);
        tracks
    };
    let model = model(&fixture(
        vec![bone("a", -1, [0.0; 6])],
        vec![
            Sequence::new("two", 10.0, 1, vec![blend(0), blend(100)]),
            Sequence::new(
                "four",
                10.0,
                1,
                vec![blend(0), blend(100), blend(200), blend(300)],
            ),
        ],
    ));
    let x = |sequence, blend: [f64; 2]| {
        let mut inputs = inputs(sequence, 0.0);
        inputs.current.blend = blend;
        translation(&pose(&model, &inputs), 0)[0]
    };
    assert_eq!(x(0, [0.0, 0.0]), 0.0);
    assert_eq!(x(0, [255.0, 0.0]), 100.0);
    assert_eq!(x(0, [127.5, 0.0]), 50.0);
    assert_eq!(x(0, [400.0, 0.0]), 100.0);
    assert_eq!(x(0, [-1.0, 0.0]), 0.0);
    assert_eq!(x(1, [0.0, 0.0]), 0.0);
    assert_eq!(x(1, [255.0, 0.0]), 100.0);
    assert_eq!(x(1, [0.0, 255.0]), 200.0);
    assert_eq!(x(1, [255.0, 255.0]), 300.0);
    assert_eq!(x(1, [127.5, 127.5]), 150.0);
}

/// A transition weighs the previous sequence's pose: 0 is the current pose, 1 the
/// previous, clamped beyond
#[test]
fn transitions_weigh_the_previous_pose() {
    let holding = |value: i16| {
        let mut tracks = still(1);
        tracks[0][0] = constant(value, 1);
        tracks
    };
    let model = model(&fixture(
        vec![bone("a", -1, [0.0; 6])],
        vec![
            Sequence::new("now", 10.0, 1, vec![holding(0)]),
            Sequence::new("before", 10.0, 1, vec![holding(100)]),
        ],
    ));
    let x = |weight| {
        let inputs = PoseInputs {
            transition: Some(Transition {
                from: SequenceState {
                    sequence: 1,
                    frame: 0.0,
                    blend: [0.0; 2],
                },
                weight,
            }),
            ..inputs(0, 0.0)
        };
        translation(&pose(&model, &inputs), 0)[0]
    };
    assert_eq!(x(0.0), 0.0);
    assert_eq!(x(1.0), 100.0);
    assert_eq!(x(0.25), 25.0);
    assert_eq!(x(2.0), 100.0);
    assert_eq!(x(-1.0), 0.0);
}

/// The gait replaces the bones from the root to the pelvis and from each child of the
/// pelvis that is not the spine chain: here the root, the pelvis and both legs, while
/// the spine keeps the current sequence
#[test]
fn the_gait_replaces_the_legs() {
    let names = [
        ("Bip01", -1),
        ("Bip01 Pelvis", 0),
        ("Bip01 Spine", 1),
        ("Bip01 Spine1", 2),
        ("Bip01 L Leg", 1),
        ("Bip01 R Leg", 1),
    ];
    let holding = |value: i16| {
        let mut tracks = still(names.len());
        for bone in &mut tracks {
            bone[0] = constant(value, 1);
        }
        tracks
    };
    let model = model(&fixture(
        names
            .iter()
            .map(|(name, parent)| bone(name, *parent, [0.0; 6]))
            .collect(),
        vec![
            Sequence::new("upper", 10.0, 1, vec![holding(1)]),
            Sequence::new("run", 10.0, 1, vec![holding(50)]),
        ],
    ));
    let pose = pose(
        &model,
        &PoseInputs {
            gait: Some(GaitState {
                sequence: 1,
                frame: 0.0,
            }),
            ..inputs(0, 0.0)
        },
    );
    let local: Vec<f64> = names
        .iter()
        .enumerate()
        .map(|(index, (_, parent))| {
            let own = translation(&pose, index)[0];
            if *parent < 0 {
                own
            } else {
                own - translation(&pose, *parent as usize)[0]
            }
        })
        .collect();
    assert_eq!(local, [50.0, 50.0, 1.0, 1.0, 50.0, 50.0]);
}

/// Motion flags zero the motion bone's translation on their axes; the linear motion
/// flags change nothing
#[test]
fn motion_flags_zero_the_motion_bones_axes() {
    let moving = || {
        let mut tracks = still(2);
        for channel in 0..3 {
            tracks[1][channel] = constant(7, 1);
        }
        tracks
    };
    let model = model(&fixture(
        vec![bone("a", -1, [0.0; 6]), bone("b", -1, [0.0; 6])],
        vec![
            Sequence {
                motion_type: STUDIO_X | STUDIO_Z,
                motion_bone: 1,
                ..Sequence::new("xz", 10.0, 1, vec![moving()])
            },
            Sequence {
                motion_type: 0x40 | 0x80 | 0x100,
                motion_bone: 1,
                ..Sequence::new("linear", 10.0, 1, vec![moving()])
            },
        ],
    ));
    assert_eq!(
        translation(&pose(&model, &inputs(0, 0.0)), 1),
        [0.0, 7.0, 0.0]
    );
    assert_eq!(
        translation(&pose(&model, &inputs(1, 0.0)), 1),
        [7.0, 7.0, 7.0]
    );
}

/// A hitbox model: one bone with a box 2 by 4 by 6 in its space (group 4), turned a
/// quarter of yaw by the placement, so the box spans 4 in world X and 2 in world Y
fn boxed() -> (StudioModel, EvaluatedPose) {
    let mut fixture = fixture(
        vec![bone("a", -1, [0.0; 6])],
        vec![Sequence::new("still", 10.0, 1, vec![still(1)])],
    );
    fixture.hitboxes = vec![Hitbox {
        bone: 0,
        group: 4,
        min: [-1.0, -2.0, -3.0],
        max: [1.0, 2.0, 3.0],
    }];
    let model = model(&fixture);
    let pose = evaluate_pose(
        &model,
        &inputs(0, 0.0),
        &Placement {
            origin: [0.0; 3],
            angles: [0.0, 90.0, 0.0],
        },
    )
    .unwrap();
    (model, pose)
}

#[test]
fn a_rotated_hitbox_is_met_on_its_turned_face() {
    let (model, pose) = boxed();
    let hit = trace_hitboxes(&model, &pose, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0])
        .unwrap()
        .unwrap();
    assert_eq!((hit.hitbox, hit.bone, hit.group), (0, 0, 4));
    assert!((hit.fraction - 0.4).abs() < 1e-12, "{}", hit.fraction);
    assert!(close(hit.point, [-2.0, 0.0, 0.0]));
    // The box's +Y face, turned to face world -X
    assert!(close(hit.normal, [-1.0, 0.0, 0.0]), "{:?}", hit.normal);
    assert!(!hit.starts_inside);
    // Along world Y the narrow side: 1 from the centre
    let hit = trace_hitboxes(&model, &pose, [0.0, 10.0, 0.0], [0.0, -10.0, 0.0])
        .unwrap()
        .unwrap();
    assert!((hit.fraction - 0.45).abs() < 1e-12);
    assert!(close(hit.normal, [0.0, 1.0, 0.0]));
    // The other way along world Y: the box's minimum face, along its positive axis
    let hit = trace_hitboxes(&model, &pose, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0])
        .unwrap()
        .unwrap();
    assert!((hit.fraction - 0.45).abs() < 1e-12);
    assert!(close(hit.normal, [0.0, -1.0, 0.0]), "{:?}", hit.normal);
    // Above the box, short of it, and along a still axis outside its slab: missed
    assert_eq!(
        trace_hitboxes(&model, &pose, [-10.0, 0.0, 4.0], [10.0, 0.0, 4.0]).unwrap(),
        None
    );
    assert_eq!(
        trace_hitboxes(&model, &pose, [-10.0, 0.0, 0.0], [-3.0, 0.0, 0.0]).unwrap(),
        None
    );
    assert_eq!(
        trace_hitboxes(&model, &pose, [0.0, 5.0, -10.0], [0.0, 5.0, 10.0]).unwrap(),
        None
    );
    // Starting inside
    let hit = trace_hitboxes(&model, &pose, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0])
        .unwrap()
        .unwrap();
    assert_eq!(
        (hit.fraction, hit.normal, hit.starts_inside),
        (0.0, [0.0; 3], true)
    );
}

/// The nearest box wins whatever its index; among boxes met at the same fraction, the
/// lowest index
#[test]
fn the_nearest_hitbox_wins_and_ties_go_to_the_lowest_index() {
    let mut fixture = fixture(
        vec![bone("a", -1, [0.0; 6])],
        vec![Sequence::new("still", 10.0, 1, vec![still(1)])],
    );
    let cube = |group, centre: f32| Hitbox {
        bone: 0,
        group,
        min: [centre - 1.0, -1.0, -1.0],
        max: [centre + 1.0, 1.0, 1.0],
    };
    fixture.hitboxes = vec![cube(1, 5.0), cube(2, 0.0), cube(3, 0.0), cube(4, 5.0)];
    let model = model(&fixture);
    let pose = pose(&model, &inputs(0, 0.0));
    let hit = |from: [f64; 3], to: [f64; 3]| {
        trace_hitboxes(&model, &pose, from, to)
            .unwrap()
            .map(|hit| (hit.hitbox, hit.group))
    };
    assert_eq!(hit([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]), Some((1, 2)));
    assert_eq!(hit([10.0, 0.0, 0.0], [-10.0, 0.0, 0.0]), Some((0, 1)));
}

/// A world point from a bone's local point, by its stored transform
fn to_world(pose: &EvaluatedPose, bone: usize, local: [f64; 3]) -> [f64; 3] {
    let transform = pose.bones()[bone];
    [0, 1, 2].map(|row| {
        transform[row][0] * local[0]
            + transform[row][1] * local[1]
            + transform[row][2] * local[2]
            + transform[row][3]
    })
}

/// The review's case: a bone turning from about pi to pi + 0.002 radians, halfway
/// through a frame. The slerp takes its linear branch and, as the renderer's does, does
/// not normalize, so the bone's transform is slightly scaled; a child bone inherits it.
/// Each has a unit box.
fn slightly_scaled() -> (StudioModel, EvaluatedPose) {
    let mut turning = bone(
        "turning",
        -1,
        [0.0, 0.0, 0.0, 0.0, 0.0, std::f32::consts::PI],
    );
    turning.scale[5] = 0.001;
    let mut tracks = still(2);
    tracks[0][5] = Some(vec![run(&[0], 1), run(&[2], 1)]);
    let mut fixture = fixture(
        vec![turning, bone("child", 0, [2.0, 0.0, 0.0, 0.0, 0.0, 0.0])],
        vec![Sequence::new("turn", 10.0, 2, vec![tracks])],
    );
    let unit = |bone| Hitbox {
        bone,
        group: bone + 1,
        min: [-1.0; 3],
        max: [1.0; 3],
    };
    fixture.hitboxes = vec![unit(0), unit(1)];
    let model = model(&fixture);
    let pose = pose(&model, &inputs(0, 0.5));
    (model, pose)
}

/// Hit queries invert the stored transform, not its transpose: a point just outside
/// the box is outside, a crossing meets the box at its true fraction, and the normal is
/// the face's, carried by the inverse transpose and normalized, on the bone and on its
/// child
#[test]
fn hits_invert_the_transforms_the_evaluator_produces() {
    let (model, pose) = slightly_scaled();
    // Not orthonormal: the rotation is scaled
    let transform = pose.bones()[0];
    let column = [transform[0][1], transform[1][1], transform[2][1]];
    let length = (column[0] * column[0] + column[1] * column[1] + column[2] * column[2]).sqrt();
    assert!((1.0 - length).abs() > 1e-8, "{length}");

    for bone in [0, 1] {
        // From just outside the box, moving away: missed (the transpose put it inside)
        let outside = to_world(&pose, bone, [0.0, 1.0000001, 0.0]);
        let further = to_world(&pose, bone, [0.0, 3.0, 0.0]);
        let missed = trace_hitboxes(&model, &pose, outside, further).unwrap();
        assert!(
            missed.is_none_or(|hit| hit.hitbox != bone),
            "bone {bone}: {missed:?}"
        );
        // From 3 to -5 along the bone's Y: the box's +Y face at a quarter of the way
        let start = to_world(&pose, bone, [0.0, 3.0, 0.0]);
        let end = to_world(&pose, bone, [0.0, -5.0, 0.0]);
        let hit = trace_hitboxes(&model, &pose, start, end).unwrap().unwrap();
        assert_eq!(hit.hitbox, bone);
        assert!(
            (hit.fraction - 0.25).abs() < 1e-12,
            "bone {bone}: {}",
            hit.fraction
        );
        // A uniformly scaled rotation's face normal is its normalized column
        let transform = pose.bones()[bone];
        let column = [transform[0][1], transform[1][1], transform[2][1]];
        let length = (column[0] * column[0] + column[1] * column[1] + column[2] * column[2]).sqrt();
        let expected = column.map(|component| component / length);
        assert!(close(hit.normal, expected), "bone {bone}: {:?}", hit.normal);
        let unit = hit
            .normal
            .iter()
            .map(|component| component * component)
            .sum::<f64>();
        assert!((unit - 1.0).abs() < 1e-15);
    }
}

/// The face normal is carried by the inverse transpose: on a sheared transform the
/// box's +Y face stays horizontal, though the bone's Y column leans
#[test]
fn normals_follow_the_inverse_transpose() {
    let (model, pose) = boxed();
    let sheared = EvaluatedPose {
        bones: vec![[
            [1.0, 1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ]],
        ..pose
    };
    // The box spans 2 either side in Y: its +Y face is the line y = 2, x from 1 to 3
    let hit = trace_hitboxes(&model, &sheared, [2.0, 10.0, 0.0], [2.0, -10.0, 0.0])
        .unwrap()
        .unwrap();
    assert!(close(hit.normal, [0.0, 1.0, 0.0]), "{:?}", hit.normal);
    // At x = 0 the segment enters the -X face, slanted by the shear to x = y - 1
    let hit = trace_hitboxes(&model, &sheared, [0.0, 10.0, 0.0], [0.0, -10.0, 0.0])
        .unwrap()
        .unwrap();
    let diagonal = 0.5_f64.sqrt();
    assert!(
        close(hit.normal, [-diagonal, diagonal, 0.0]),
        "{:?}",
        hit.normal
    );
}

/// Finite inputs whose arithmetic overflows make no pose and no hit: the review's
/// wrapping controller at the largest finite value; a large but finite translation
/// controller added to a placement's origin at the largest value (an `f32` offset alone
/// rounds away there, so the origin by itself is a valid pose); and a segment spanning
/// the whole range
#[test]
fn overflowing_arithmetic_is_refused() {
    let mut wrapped = bone("wrapped", -1, [0.0; 6]);
    wrapped.controllers[5] = 0;
    wrapped.controllers[0] = 1;
    let mut fixture = fixture(
        vec![wrapped, bone("child", 0, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0])],
        vec![Sequence::new("still", 10.0, 1, vec![still(2)])],
    );
    fixture.controllers = vec![
        Controller {
            bone: 0,
            kind: STUDIO_ZR | 0x8000,
            start: 0.0,
            end: 0.0,
            rest: 0,
            index: 0,
        },
        Controller {
            bone: 0,
            kind: STUDIO_X | 0x8000,
            start: 0.0,
            end: 0.0,
            rest: 0,
            index: 1,
        },
    ];
    fixture.hitboxes = vec![Hitbox {
        bone: 0,
        group: 1,
        min: [-1.0; 3],
        max: [1.0; 3],
    }];
    let model = model(&fixture);
    assert_eq!(
        evaluate_pose(
            &model,
            &PoseInputs {
                controllers: [f64::MAX, 0.0, 0.0, 0.0],
                ..inputs(0, 0.0)
            },
            &AT_ORIGIN
        ),
        Err(PoseError::Overflow {
            stage: "controllers"
        })
    );
    let far = Placement {
        origin: [f64::MAX, 0.0, 0.0],
        angles: [0.0; 3],
    };
    assert!(evaluate_pose(&model, &inputs(0, 0.0), &far).is_ok());
    assert_eq!(
        evaluate_pose(
            &model,
            &PoseInputs {
                controllers: [0.0, 1e308, 0.0, 0.0],
                ..inputs(0, 0.0)
            },
            &far
        ),
        Err(PoseError::Overflow { stage: "pose" })
    );
    let pose = pose(&model, &inputs(0, 0.0));
    assert_eq!(
        trace_hitboxes(&model, &pose, [-f64::MAX, 0.0, 0.0], [f64::MAX, 0.0, 0.0]),
        Err(PoseError::Overflow { stage: "segment" })
    );
    // The segment's movement fits, but carried into a bone placed far the other way
    // its coordinates do not
    let opposite = evaluate_pose(
        &model,
        &inputs(0, 0.0),
        &Placement {
            origin: [-1e308, 0.0, 0.0],
            angles: [0.0; 3],
        },
    )
    .unwrap();
    assert_eq!(
        trace_hitboxes(
            &model,
            &opposite,
            [f64::MAX, 0.0, 0.0],
            [f64::MAX, 1.0, 0.0]
        ),
        Err(PoseError::Overflow { stage: "segment" })
    );
    // A bone scaled by two halves the segment in its space, where it fits and misses the
    // box; the world movement still overflows, so no miss is reported
    let doubled = EvaluatedPose {
        bones: vec![
            [
                [2.0, 0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0, 0.0],
                [0.0, 0.0, 2.0, 0.0],
            ],
            pose.bones()[1],
        ],
        ..pose.clone()
    };
    assert_eq!(
        trace_hitboxes(
            &model,
            &doubled,
            [-f64::MAX, 10.0, 0.0],
            [f64::MAX, 10.0, 0.0]
        ),
        Err(PoseError::Overflow { stage: "segment" })
    );
}

/// A transform whose determinant is zero, subnormal or overflowing, or whose inverse
/// overflows, cannot be inverted, and a hit query on its bone says so
#[test]
fn an_unusable_transform_is_refused() {
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];
    assert!(inverse(&identity).is_some());
    let scaled = |scale: f64| {
        let mut transform = identity;
        for (axis, row) in transform.iter_mut().enumerate() {
            row[axis] = scale;
        }
        transform
    };
    assert!(inverse(&scaled(0.0)).is_none());
    assert!(inverse(&scaled(1e-110)).is_none());
    assert!(inverse(&scaled(1e110)).is_none());
    let mut flat = identity;
    flat[2] = [1.0, 0.0, 0.0, 0.0];
    assert!(inverse(&flat).is_none());
    // A normal determinant whose inverse still overflows
    let mut thin = scaled(4.0);
    thin[2][2] = 2e-309;
    assert!(inverse(&thin).is_none());

    let (model, pose) = boxed();
    let singular = EvaluatedPose {
        bones: vec![scaled(0.0)],
        ..pose
    };
    assert_eq!(
        trace_hitboxes(&model, &singular, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
        Err(PoseError::UnusableTransform { bone: 0 })
    );
}

#[test]
fn a_pose_is_bound_to_its_model_and_evaluator() {
    let (model, pose) = boxed();
    assert_eq!(pose.model(), model.identity().combined);
    assert_eq!(pose.evaluator_revision(), POSE_EVALUATOR_REVISION);
    assert_eq!(pose.inputs(), &inputs(0, 0.0));
    assert_eq!(pose.placement().angles, [0.0, 90.0, 0.0]);
    // Another model's pose is refused
    let other = self::model(&fixture(
        vec![bone("b", -1, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0])],
        vec![Sequence::new("still", 10.0, 1, vec![still(1)])],
    ));
    assert_eq!(
        trace_hitboxes(&other, &pose, [0.0; 3], [1.0; 3]),
        Err(PoseError::ModelMismatch)
    );
}

#[test]
fn invalid_inputs_are_refused() {
    let model = model(&fixture(
        vec![bone("a", -1, [0.0; 6])],
        vec![Sequence::new("still", 10.0, 1, vec![still(1)])],
    ));
    let evaluate = |inputs: PoseInputs| evaluate_pose(&model, &inputs, &AT_ORIGIN).err();
    let state = |sequence| SequenceState {
        sequence,
        frame: 0.0,
        blend: [0.0; 2],
    };
    assert_eq!(
        evaluate(inputs(1, 0.0)),
        Some(PoseError::SequenceOutOfRange { sequence: 1 })
    );
    assert_eq!(
        evaluate(PoseInputs {
            transition: Some(Transition {
                from: state(3),
                weight: 0.5
            }),
            ..inputs(0, 0.0)
        }),
        Some(PoseError::SequenceOutOfRange { sequence: 3 })
    );
    assert_eq!(
        evaluate(PoseInputs {
            gait: Some(GaitState {
                sequence: 2,
                frame: 0.0
            }),
            ..inputs(0, 0.0)
        }),
        Some(PoseError::SequenceOutOfRange { sequence: 2 })
    );
    assert_eq!(
        evaluate(inputs(0, f64::NAN)),
        Some(PoseError::InvalidInput { input: "current" })
    );
    assert_eq!(
        evaluate(PoseInputs {
            mouth: -1.0,
            ..inputs(0, 0.0)
        }),
        Some(PoseError::InvalidInput { input: "mouth" })
    );
    assert_eq!(
        evaluate(PoseInputs {
            controllers: [0.0, f64::INFINITY, 0.0, 0.0],
            ..inputs(0, 0.0)
        }),
        Some(PoseError::InvalidInput {
            input: "controllers"
        })
    );
    assert_eq!(
        evaluate_pose(
            &model,
            &inputs(0, 0.0),
            &Placement {
                origin: [f64::NAN, 0.0, 0.0],
                angles: [0.0; 3]
            }
        )
        .err(),
        Some(PoseError::InvalidInput { input: "placement" })
    );
    let pose = pose(&model, &inputs(0, 0.0));
    assert_eq!(
        trace_hitboxes(&model, &pose, [f64::NAN, 0.0, 0.0], [1.0; 3]),
        Err(PoseError::InvalidInput { input: "segment" })
    );
}

/// The slerp's cases: its ends, the nearer of q and -q, nearly equal rotations
/// (linear), and opposite ones
#[test]
fn the_slerp_takes_its_documented_branches() {
    let p = angle_quaternion([0.0, 0.0, 0.0]);
    let q = angle_quaternion([0.0, 0.0, 1.0]);
    assert_eq!(quaternion_slerp(p, q, 0.0), p);
    let end = quaternion_slerp(p, q, 1.0);
    assert!(end.iter().zip(q).all(|(a, b)| (a - b).abs() < 1e-15));
    let negated = q.map(|component| -component);
    assert_eq!(
        quaternion_slerp(p, q, 0.3),
        quaternion_slerp(p, negated, 0.3)
    );
    // Nearly equal: the linear branch
    let near = angle_quaternion([0.0, 0.0, 1e-7]);
    let mid = quaternion_slerp(p, near, 0.5);
    let linear = [0, 1, 2, 3].map(|i| 0.5 * p[i] + 0.5 * near[i]);
    assert_eq!(mid, linear);
    // Opposite: p and -p are the same rotation, chosen as the nearer, so the result
    // is p; and p with its perpendicular forms its own branch
    assert_eq!(quaternion_slerp(p, p.map(|c| -c), 0.5), p);
}

/// The combined case, frozen: controllers (one wrapping), two blends, a transition
/// and a gait on a skeleton with rotations, placed off the origin and turned. Equal
/// digests are bit-identical transforms; the targets that ran it are recorded in
/// `STUDIO_POSE.md`.
#[test]
fn the_combined_case_matches_its_golden_transforms() {
    let names = [
        ("Bip01", -1),
        ("Bip01 Pelvis", 0),
        ("Bip01 Spine", 1),
        ("Bip01 Head", 2),
        ("Bip01 L Leg", 1),
    ];
    let animated = |offset: i16| {
        let mut tracks = still(names.len());
        for (index, bone) in tracks.iter_mut().enumerate() {
            let base = offset + index as i16 * 7;
            bone[0] = Some(vec![run(&[base, base + 3], 2), run(&[base + 5], 2)]);
            bone[4] = Some(vec![run(&[base * 2, base * 2 + 40, base * 2 - 25, 0], 4)]);
            bone[5] = Some(vec![run(&[base], 1), run(&[-base], 3)]);
        }
        tracks
    };
    let mut bones: Vec<Bone> = names
        .iter()
        .map(|(name, parent)| bone(name, *parent, [3.0, 1.0, 2.0, 0.1, 0.2, 0.3]))
        .collect();
    bones[2].controllers[5] = 0;
    bones[3].controllers[3] = 1;
    let mut fixture = fixture(
        bones,
        vec![
            Sequence::new("aim", 15.0, 4, vec![animated(10), animated(-30)]),
            Sequence::new("idle", 12.0, 4, vec![animated(4)]),
            Sequence::new("run", 30.0, 4, vec![animated(60)]),
        ],
    );
    fixture.controllers = vec![
        Controller {
            bone: 2,
            kind: STUDIO_ZR,
            start: -30.0,
            end: 30.0,
            rest: 0,
            index: 0,
        },
        Controller {
            bone: 3,
            kind: STUDIO_XR | 0x8000,
            start: 0.0,
            end: 0.0,
            rest: 0,
            index: 1,
        },
    ];
    let model = model(&fixture);
    let inputs = PoseInputs {
        current: SequenceState {
            sequence: 0,
            frame: 1.25,
            blend: [80.0, 0.0],
        },
        controllers: [200.0, interpolate_controller(250, 10, 0.4, true), 0.0, 0.0],
        mouth: 0.0,
        transition: Some(Transition {
            from: SequenceState {
                sequence: 1,
                frame: 2.75,
                blend: [0.0; 2],
            },
            weight: 0.35,
        }),
        gait: Some(GaitState {
            sequence: 2,
            frame: 0.6,
        }),
    };
    let placement = Placement {
        origin: [-120.5, 64.25, 36.0],
        angles: [12.0, 135.0, -4.0],
    };
    let pose = evaluate_pose(&model, &inputs, &placement).unwrap();
    assert_eq!(digest(&pose), COMBINED_CASE_SHA256);
    let hitless = trace_hitboxes(&model, &pose, [0.0; 3], [1.0; 3]).unwrap();
    assert_eq!(hitless, None);
}

const COMBINED_CASE_SHA256: &str =
    "d0dfabab0a484ec72d16a1f662d0294643b90b5de1a0ea12f0ae789c63f53143";

/// The stock player model from a Half-Life installation
/// (`XENOXIDE_GAME_DIRECTORY`, the directory holding `valve`)
fn stock_player() -> StudioModel {
    let directory = std::env::var_os("XENOXIDE_GAME_DIRECTORY")
        .expect("XENOXIDE_GAME_DIRECTORY must point to a Half-Life installation");
    let bytes = std::fs::read(std::path::Path::new(&directory).join("valve/models/player.mdl"))
        .expect("player.mdl");
    StudioModel::parse(&bytes).unwrap().resolve(&[]).unwrap()
}

fn sequence_index(model: &StudioModel, label: &str) -> usize {
    model
        .sequences()
        .iter()
        .position(|sequence| sequence.label == label)
        .unwrap()
}

fn bone_index(model: &StudioModel, name: &str) -> usize {
    model
        .bones()
        .iter()
        .position(|bone| bone.name == name)
        .unwrap()
}

/// The world centre of the first hitbox on `bone`, and the bone's model forward axis
fn box_centre(model: &StudioModel, pose: &EvaluatedPose, bone: usize) -> [f64; 3] {
    let hitbox = model
        .hitboxes()
        .iter()
        .find(|hitbox| hitbox.bone == bone)
        .unwrap();
    let centre = [0, 1, 2].map(|axis| (hitbox.min[axis] as f64 + hitbox.max[axis] as f64) / 2.0);
    let transform = pose.bones()[bone];
    [0, 1, 2].map(|row| {
        transform[row][0] * centre[0]
            + transform[row][1] * centre[1]
            + transform[row][2] * centre[2]
            + transform[row][3]
    })
}

/// The stock player model's pose, frozen: aiming the MP5 with the blend from a view
/// pitch of 10 degrees, the torso controllers centred (as the renderer's gait sets
/// them for no yaw offset), part way out of the python aim sequence, running, placed
/// off the origin and turned 45 degrees. A ray down onto the head meets the head's box
/// (hit group 1); one from behind into the chest meets the upper spine's (group 2).
/// The rays use no platform trigonometry: 45 degrees is the exact square root of 1/2.
#[test]
#[ignore = "requires XENOXIDE_GAME_DIRECTORY"]
fn the_stock_player_pose_matches_its_golden_transforms() {
    let model = stock_player();
    let aim = sequence_index(&model, "ref_aim_mp5");
    let (blend, pitch) = player_blend(&model.sequences()[aim], 10.0);
    assert_eq!((blend, pitch), (212.0, 0.0));
    let inputs = PoseInputs {
        current: SequenceState {
            sequence: aim,
            frame: estimate_frame(&model.sequences()[aim], 0.0, 0.4),
            blend: [blend, 0.0],
        },
        controllers: [127.0; 4],
        mouth: 0.0,
        transition: Some(Transition {
            from: SequenceState {
                sequence: sequence_index(&model, "ref_aim_python"),
                frame: 3.0,
                blend: [blend, 0.0],
            },
            weight: 0.4,
        }),
        gait: Some(GaitState {
            sequence: sequence_index(&model, "run"),
            frame: 5.5,
        }),
    };
    let placement = Placement {
        origin: [256.0, -128.0, 36.0],
        angles: [pitch, 45.0, 0.0],
    };
    let pose = evaluate_pose(&model, &inputs, &placement).unwrap();
    assert_eq!(digest(&pose), STOCK_PLAYER_SHA256);

    let head = box_centre(&model, &pose, bone_index(&model, "Bip01 Head"));
    let above = trace_hitboxes(
        &model,
        &pose,
        [head[0], head[1], head[2] + 200.0],
        [head[0], head[1], head[2] - 200.0],
    )
    .unwrap()
    .unwrap();
    assert_eq!((above.hitbox, above.group), (11, 1));
    assert_eq!(above.fraction, STOCK_HEAD_FRACTION);

    let chest = box_centre(&model, &pose, bone_index(&model, "Bip01 Spine2"));
    let diagonal = 0.5_f64.sqrt();
    let behind = [
        chest[0] - 200.0 * diagonal,
        chest[1] - 200.0 * diagonal,
        chest[2],
    ];
    let front = [
        chest[0] + 200.0 * diagonal,
        chest[1] + 200.0 * diagonal,
        chest[2],
    ];
    let from_behind = trace_hitboxes(&model, &pose, behind, front)
        .unwrap()
        .unwrap();
    assert_eq!((from_behind.hitbox, from_behind.group), (9, 2));
    assert_eq!(from_behind.fraction, STOCK_CHEST_FRACTION);
}

const STOCK_PLAYER_SHA256: &str =
    "87cfe51aa2e5f3e67d4430b78b18fee427f74246e574b23447ef3187616e613a";
const STOCK_HEAD_FRACTION: f64 = 0.4892395494417107;
const STOCK_CHEST_FRACTION: f64 = 0.4791508449222621;
