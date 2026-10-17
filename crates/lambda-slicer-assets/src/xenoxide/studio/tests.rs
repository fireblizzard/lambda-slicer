use std::path::{Path, PathBuf};

use super::*;

use super::fixture;
use super::fixture::{Fixture, run};

fn parse(bytes: &[u8]) -> Result<StudioModel, StudioError> {
    StudioModel::parse(bytes)?.resolve(&[])
}

/// The standard fixture's bytes, frozen: a change to the builder changes this digest
const STANDARD_FIXTURE_SHA256: &str =
    "2287e5b3ec919efa05a195ccb60d621642e927e316ef8d6d3c5e2a747b09a1f9";

#[test]
fn the_standard_fixture_reads_completely() {
    let built = Fixture::standard().build();
    assert_eq!(
        StudioDigest::of(&built.bytes).to_string(),
        STANDARD_FIXTURE_SHA256
    );
    let unresolved = StudioModel::parse(&built.bytes).unwrap();
    assert!(unresolved.dependencies().is_empty());
    let model = unresolved.resolve(&[]).unwrap();
    assert_eq!(model.name(), b"fixture.mdl");
    assert_eq!(model.eye_position(), [0.0, 0.0, 28.0]);
    assert_eq!(model.hull(), ([-16.0, -16.0, -36.0], [16.0, 16.0, 36.0]));
    assert!(model.texture_companion());

    let bones = model.bones();
    assert_eq!(bones.len(), 3);
    assert_eq!(bones[2].name, "Bip01 Head");
    assert_eq!(
        bones.iter().map(|bone| bone.parent).collect::<Vec<_>>(),
        [None, Some(0), Some(1)]
    );
    assert_eq!(
        bones[1].controllers,
        [None, None, None, None, Some(0), None]
    );
    assert_eq!(bones[0].value, [0.0, 0.0, 10.0, 0.0, 0.0, 0.0]);
    assert_eq!(bones[0].scale, [0.5, 0.5, 0.5, 0.001, 0.001, 0.001]);

    assert_eq!(
        model.bone_controllers(),
        [StudioBoneController {
            bone: 1,
            axis: 0x10,
            wraps: true,
            start: -90.0,
            end: 90.0,
            rest: 0,
            index: 0,
        }]
    );
    let hitboxes = model.hitboxes();
    assert_eq!(
        hitboxes
            .iter()
            .map(|hitbox| (hitbox.bone, hitbox.group))
            .collect::<Vec<_>>(),
        [(2, 1), (1, 2)]
    );
    assert_eq!(hitboxes[0].min, [-4.0, -4.0, -2.0]);
    assert_eq!(hitboxes[0].max, [4.0, 4.0, 6.0]);

    let idle = &model.sequences()[0];
    assert_eq!(idle.label, "idle");
    assert_eq!((idle.fps, idle.frames, idle.group), (15.0, 4, 0));
    assert_eq!(
        idle.events,
        [StudioEvent {
            frame: 1,
            event: 5001,
            kind: 0,
            options: "30".into(),
        }]
    );
    // Runs [10, 20] over 3 frames and [30] over 1: the last value repeats in a run
    let values = |track: &Option<StudioTrack>| track.as_ref().map(|track| track.values().to_vec());
    assert_eq!(idle.animation.len(), 1);
    assert_eq!(values(&idle.animation[0][0][0]), Some(vec![10, 20, 20, 30]));
    assert_eq!(values(&idle.animation[0][1][4]), Some(vec![5, 6, 7, 8]));
    assert_eq!(idle.animation[0][0][1], None);
    // The same values as one run or as runs ending at their last explicit value: a
    // translation holds at such a frame, a rotation interpolates on
    let bone = &idle.animation[0][2];
    let (split, joined, rotation) = (
        bone[0].as_ref().unwrap(),
        bone[1].as_ref().unwrap(),
        bone[5].as_ref().unwrap(),
    );
    assert_eq!(split.values(), joined.values());
    assert_eq!(
        (0..4)
            .map(|frame| split.translation(frame))
            .collect::<Vec<_>>(),
        [(5, 5), (6, 6), (7, 8), (8, 8)]
    );
    assert_eq!(
        (0..4)
            .map(|frame| joined.translation(frame))
            .collect::<Vec<_>>(),
        [(5, 6), (6, 7), (7, 8), (8, 8)]
    );
    assert_eq!(
        (0..4)
            .map(|frame| rotation.rotation(frame))
            .collect::<Vec<_>>(),
        [(5, 6), (6, 7), (7, 8), (8, 8)]
    );

    let aim = &model.sequences()[1];
    assert_eq!(aim.animation.len(), 2);
    assert_eq!(values(&aim.animation[0][0][3]), Some(vec![-100, -200]));
    assert_eq!(values(&aim.animation[1][0][3]), Some(vec![100, 200]));

    let identity = model.identity();
    assert_eq!(identity.main, StudioDigest::of(&built.bytes));
    assert!(identity.sequence_groups.is_empty());
    assert_eq!(
        identity.combined,
        StudioModelIdentity::new(identity.main, Vec::new()).combined
    );
    assert_ne!(identity.combined, identity.main);
}

/// Runs as the 16-bit slots the renderer indexes: each run's header, then its values
fn slots(runs: &[(u8, u8, Vec<i16>)]) -> Vec<i16> {
    let mut slots = Vec::new();
    for (valid, total, values) in runs {
        slots.push(i16::from_le_bytes([*valid, *total]));
        slots.extend(values);
    }
    slots
}

/// The run holding frame `k`: its first slot and `k` within it, as the renderer walks
fn walk(slots: &[i16], mut k: usize) -> (usize, usize, usize, usize) {
    let header = |at: usize| {
        let [valid, total] = slots[at].to_le_bytes();
        (valid as usize, total as usize)
    };
    let mut at = 0;
    while header(at).1 <= k {
        k -= header(at).1;
        at += header(at).0 + 1;
    }
    let (valid, total) = header(at);
    (at, k, valid, total)
}

/// `StudioCalcBonePosition`, transcribed from the pinned renderer: the raw translation
/// at frame `frame` plus `s`, with unit scale and zero default
fn renderer_translation(runs: &[(u8, u8, Vec<i16>)], frame: usize, s: f64) -> f64 {
    let slots = slots(runs);
    let (at, k, valid, total) = walk(&slots, frame);
    let value = |slot: usize| slots[at + slot] as f64;
    if valid > k {
        if valid > k + 1 {
            value(k + 1) * (1.0 - s) + s * value(k + 2)
        } else {
            value(k + 1)
        }
    } else if total <= k + 1 {
        value(valid) * (1.0 - s) + s * value(valid + 2)
    } else {
        value(valid)
    }
}

/// `StudioCalcBoneQuaterion`, transcribed from the pinned renderer: the two raw angles
/// it builds the quaternions it interpolates from
fn renderer_rotation(runs: &[(u8, u8, Vec<i16>)], frame: usize) -> (i16, i16) {
    let slots = slots(runs);
    let (at, k, valid, total) = walk(&slots, frame);
    let value = |slot: usize| slots[at + slot];
    if valid > k {
        let first = value(k + 1);
        let second = if valid > k + 1 {
            value(k + 2)
        } else if total > k + 1 {
            first
        } else {
            value(valid + 2)
        };
        (first, second)
    } else {
        let first = value(valid);
        let second = if total > k + 1 {
            first
        } else {
            value(valid + 2)
        };
        (first, second)
    }
}

fn decoded(runs: &[(u8, u8, Vec<i16>)]) -> StudioTrack {
    let frames: usize = runs.iter().map(|(_, total, _)| *total as usize).sum();
    let mut bytes = Vec::new();
    for (valid, total, values) in runs {
        bytes.push(*valid);
        bytes.push(*total);
        values
            .iter()
            .for_each(|value| bytes.extend_from_slice(&value.to_le_bytes()));
    }
    decode_track(&bytes, 0, frames).unwrap()
}

/// The decoded track reads as the renderer reads the runs, translations and rotations
/// separately, at whole and fractional frames up to the last (the renderer's frames
/// never pass it). Layouts include runs ending exactly at their last explicit value,
/// where the two readings differ.
#[test]
fn decoded_tracks_read_as_the_renderer_reads_runs() {
    let layouts: Vec<Vec<(u8, u8, Vec<i16>)>> = vec![
        vec![(4, 4, vec![1, 2, 3, 4])],
        vec![(1, 5, vec![7])],
        vec![
            (2, 3, vec![10, 20]),
            (1, 1, vec![30]),
            (3, 6, vec![1, 2, 3]),
        ],
        vec![(1, 1, vec![5]), (1, 1, vec![6]), (2, 2, vec![7, 8])],
        vec![(3, 3, vec![-1, -2, -3]), (2, 5, vec![9, 8])],
        vec![(2, 2, vec![4, 9]), (1, 3, vec![-6]), (1, 1, vec![2])],
    ];
    for runs in layouts {
        let track = decoded(&runs);
        for frame in 0..track.frames() - 1 {
            let (from, towards) = track.translation(frame);
            for s in [0.0, 0.25, 0.5, 0.75] {
                let ours = from as f64 * (1.0 - s) + s * towards as f64;
                assert_eq!(
                    ours,
                    renderer_translation(&runs, frame, s),
                    "{runs:?} translation at {frame} + {s}"
                );
            }
            assert_eq!(
                track.rotation(frame),
                renderer_rotation(&runs, frame),
                "{runs:?} rotation at {frame}"
            );
        }
    }
}

/// The review's reproduction: two one-frame runs of 5 and 6 against one two-frame run
/// of [5, 6]. Halfway through frame 0 the translation is 5 for the first and 5.5 for
/// the second, as in the renderer; the rotation interpolates towards 6 in both.
#[test]
fn a_translation_holds_at_the_end_of_a_run_and_a_rotation_does_not() {
    let separate = vec![(1, 1, vec![5]), (1, 1, vec![6])];
    let joined = vec![(2, 2, vec![5, 6])];
    let midway = |track: &StudioTrack| {
        let (from, towards) = track.translation(0);
        (from as f64 + towards as f64) / 2.0
    };
    assert_eq!(decoded(&separate).values(), decoded(&joined).values());
    assert_eq!(midway(&decoded(&separate)), 5.0);
    assert_eq!(renderer_translation(&separate, 0, 0.5), 5.0);
    assert_eq!(midway(&decoded(&joined)), 5.5);
    assert_eq!(renderer_translation(&joined, 0, 0.5), 5.5);
    assert_eq!(decoded(&separate).rotation(0), (5, 6));
    assert_eq!(decoded(&joined).rotation(0), (5, 6));
    assert_eq!(renderer_rotation(&separate, 0), (5, 6));
    // At the last frame there is nothing to interpolate towards
    assert_eq!(decoded(&joined).translation(1), (6, 6));
    assert_eq!(decoded(&joined).rotation(1), (6, 6));
}

/// A sequence group holds the animation of the sequences assigned to it: the model
/// names the file, it is required, and nothing else is accepted in its place
#[test]
fn external_sequence_groups_are_required_dependencies() {
    let fixture = Fixture::with_external_group();
    let main = fixture.build();
    let group = fixture.build_group(1);
    let unresolved = || StudioModel::parse(&main.bytes).unwrap();
    assert_eq!(
        unresolved().dependencies(),
        [SequenceGroupDependency {
            index: 1,
            name: "models\\fixture01.mdl".into(),
        }]
    );
    assert_eq!(
        unresolved().resolve(&[]),
        Err(StudioError::MissingDependency {
            group: 1,
            name: "models\\fixture01.mdl".into(),
        })
    );
    assert_eq!(
        unresolved().resolve(&[(1, &group.bytes), (2, &group.bytes)]),
        Err(StudioError::UnexpectedDependency { group: 2 })
    );
    assert_eq!(
        unresolved().resolve(&[(1, &group.bytes), (1, &group.bytes)]),
        Err(StudioError::DuplicateDependency { group: 1 })
    );
    // The main model is no sequence group
    assert_eq!(
        unresolved().resolve(&[(1, &main.bytes)]),
        Err(StudioError::InvalidMagic {
            what: "sequence group file"
        })
    );
    let mut wrong_version = group.bytes.clone();
    wrong_version[4..8].copy_from_slice(&9_i32.to_le_bytes());
    assert_eq!(
        unresolved().resolve(&[(1, &wrong_version)]),
        Err(StudioError::UnsupportedVersion {
            what: "sequence group file",
            version: 9,
        })
    );
    let short = &group.bytes[..group.bytes.len() - 1];
    assert!(matches!(
        unresolved().resolve(&[(1, short)]),
        Err(StudioError::LengthMismatch { .. })
    ));

    let model = unresolved().resolve(&[(1, &group.bytes)]).unwrap();
    let walk = &model.sequences()[2];
    assert_eq!(walk.group, 1);
    assert_eq!(
        walk.animation[0][0][1].as_ref().map(StudioTrack::values),
        Some(&[1, 2, 3][..])
    );
    let identity = model.identity();
    assert_eq!(identity.main, StudioDigest::of(&main.bytes));
    assert_eq!(
        identity.sequence_groups,
        [(
            1,
            "models\\fixture01.mdl".to_string(),
            StudioDigest::of(&group.bytes)
        )]
    );
    // Other animation in the group file is another identity, the main file unchanged
    let mut changed = Fixture::with_external_group();
    changed.sequences[2].tracks[0][0][1] = Some(vec![run(&[1, 2, 4], 3)]);
    let other_group = changed.build_group(1);
    let other = StudioModel::parse(&changed.build().bytes)
        .unwrap()
        .resolve(&[(1, &other_group.bytes)])
        .unwrap();
    assert_eq!(other.identity().main, identity.main);
    assert_ne!(other.identity().combined, identity.combined);

    // A declared group no sequence uses is no dependency
    let mut unused = Fixture::standard();
    unused.groups.push(("spare", "models\\fixture02.mdl"));
    let unresolved = StudioModel::parse(&unused.build().bytes).unwrap();
    assert!(unresolved.dependencies().is_empty());
}

/// A file cut anywhere, with its declared length made to match, is refused: every
/// record it refers to is read and checked against the file
#[test]
fn a_truncated_file_is_refused_wherever_it_is_cut() {
    let built = Fixture::standard().build();
    for length in 0..built.bytes.len() {
        let bytes = if length < 76 {
            built.bytes[..length].to_vec()
        } else {
            built.truncated(length)
        };
        assert!(parse(&bytes).is_err(), "cut at {length}");
    }
    let group_fixture = Fixture::with_external_group();
    let main = group_fixture.build();
    let group = group_fixture.build_group(1);
    for length in 76..group.bytes.len() {
        let bytes = group.truncated(length);
        let result = StudioModel::parse(&main.bytes)
            .unwrap()
            .resolve(&[(1, &bytes)]);
        assert!(result.is_err(), "group cut at {length}");
    }
}

/// No corruption of any single byte panics; each either reads or is refused
#[test]
fn corrupting_any_byte_never_panics() {
    let fixture = Fixture::with_external_group();
    let main = fixture.build();
    let group = fixture.build_group(1);
    for at in 0..main.bytes.len() {
        for value in [0x00, 0x7f, 0x80, 0xff] {
            let mut bytes = main.bytes.clone();
            bytes[at] = value;
            if let Ok(unresolved) = StudioModel::parse(&bytes) {
                let _ = unresolved.resolve(&[(1, &group.bytes)]);
            }
        }
    }
    for at in 0..group.bytes.len() {
        for value in [0x00, 0x7f, 0x80, 0xff] {
            let mut bytes = group.bytes.clone();
            bytes[at] = value;
            let _ = StudioModel::parse(&main.bytes)
                .unwrap()
                .resolve(&[(1, &bytes)]);
        }
    }
}

fn refused(mutate: impl FnOnce(&mut fixture::Built)) -> StudioError {
    let mut built = Fixture::standard().build();
    mutate(&mut built);
    parse(&built.bytes).unwrap_err()
}

#[test]
fn malformed_headers_are_refused() {
    let mut wrong_magic = Fixture::standard().build().bytes;
    wrong_magic[0..4].copy_from_slice(b"IDSQ");
    assert_eq!(
        parse(&wrong_magic),
        Err(StudioError::InvalidMagic { what: "header" })
    );
    assert_eq!(
        refused(|built| built.put_i32(4, 6)),
        StudioError::UnsupportedVersion {
            what: "header",
            version: 6
        }
    );
    assert!(matches!(
        refused(|built| built.put_i32(72, 100)),
        StudioError::LengthMismatch { declared: 100, .. }
    ));
    assert_eq!(
        parse(&[0; 100]),
        Err(StudioError::Truncated { what: "header" })
    );
    assert_eq!(
        parse(&vec![0; MAX_STUDIO_MODEL_BYTES + 1]),
        Err(StudioError::TooLarge {
            bytes: MAX_STUDIO_MODEL_BYTES + 1
        })
    );
    assert_eq!(
        refused(|built| built.put_f32(80, f32::NAN)),
        StudioError::NonFinite {
            what: "header",
            index: 0
        }
    );
    // The header's name is informational: any bytes, terminated or not
    let mut built = Fixture::standard().build();
    built.bytes[8..72].fill(0xd0);
    assert_eq!(parse(&built.bytes).unwrap().name(), [0xd0; 64]);
}

#[test]
fn out_of_range_tables_are_refused() {
    let len = Fixture::standard().build().bytes.len() as i32;
    assert_eq!(
        refused(|built| built.put_i32(144, len)),
        StudioError::OutOfBounds {
            what: "bone",
            index: 0
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(144, -4)),
        StudioError::OutOfBounds {
            what: "bone",
            index: 0
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(144, i32::MAX)),
        StudioError::OutOfBounds {
            what: "bone",
            index: 0
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(140, -1)),
        StudioError::InvalidCount {
            what: "bone",
            count: -1
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(140, 129)),
        StudioError::CountOverLimit {
            what: "bone",
            count: 129,
            limit: MAX_STUDIO_BONES
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(148, 9)),
        StudioError::CountOverLimit {
            what: "bone controller",
            count: 9,
            limit: MAX_STUDIO_BONE_CONTROLLERS
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(156, 40)),
        StudioError::OutOfBounds {
            what: "hitbox",
            index: 0
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(164, MAX_STUDIO_SEQUENCES as i32 + 1)),
        StudioError::CountOverLimit {
            what: "sequence",
            count: MAX_STUDIO_SEQUENCES + 1,
            limit: MAX_STUDIO_SEQUENCES
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(172, 17)),
        StudioError::CountOverLimit {
            what: "sequence group",
            count: 17,
            limit: MAX_STUDIO_SEQUENCE_GROUPS
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(172, 0)),
        StudioError::InvalidSequenceGroup { index: 0 }
    );
    assert_eq!(
        refused(|built| {
            let at = built.sequence(0) + 48;
            built.put_i32(at, MAX_STUDIO_EVENTS as i32 + 1);
        }),
        StudioError::CountOverLimit {
            what: "event",
            count: MAX_STUDIO_EVENTS + 1,
            limit: MAX_STUDIO_EVENTS
        }
    );
    assert_eq!(
        refused(|built| {
            let at = built.sequence(0) + 124;
            built.put_i32(at, len - 4);
        }),
        StudioError::InvalidSequence {
            index: 0,
            reason: "animation offsets lie outside the file"
        }
    );
}

#[test]
fn invalid_bone_references_are_refused() {
    // A bone that is its own parent, one whose parent comes later (a cycle through
    // bone 2, whose ancestors include bone 0), and parents outside the skeleton
    assert_eq!(
        refused(|built| built.put_i32(built.bone(1) + 32, 1)),
        StudioError::InvalidBoneParent { bone: 1, parent: 1 }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.bone(0) + 32, 2)),
        StudioError::InvalidBoneParent { bone: 0, parent: 2 }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.bone(2) + 32, 7)),
        StudioError::InvalidBoneParent { bone: 2, parent: 7 }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.bone(2) + 32, -2)),
        StudioError::InvalidBoneParent {
            bone: 2,
            parent: -2
        }
    );
    // Bone controller references
    assert_eq!(
        refused(|built| built.put_i32(built.bone(0) + 40 + 4, 1)),
        StudioError::InvalidBoneController {
            bone: 0,
            channel: 1,
            controller: 1
        }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.bone(2) + 40, -3)),
        StudioError::InvalidBoneController {
            bone: 2,
            channel: 0,
            controller: -3
        }
    );
    // Controllers, hitboxes and their bones
    assert_eq!(
        refused(|built| built.put_i32(built.controller(0), 3)),
        StudioError::InvalidController { index: 0 }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.controller(0) + 4, 0x40)),
        StudioError::InvalidController { index: 0 }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.controller(0) + 20, 5)),
        StudioError::InvalidController { index: 0 }
    );
    assert_eq!(
        refused(|built| built.put_i32(built.hitbox(1), 3)),
        StudioError::InvalidHitbox { index: 1 }
    );
    assert_eq!(
        refused(|built| built.put_f32(built.hitbox(0) + 8, 5.0)),
        StudioError::InvalidHitbox { index: 0 }
    );
    // A controller of -1 means bone 0
    let mut built = Fixture::standard().build();
    let at = built.controller(0);
    built.put_i32(at, -1);
    assert_eq!(parse(&built.bytes).unwrap().bone_controllers()[0].bone, 0);
}

#[test]
fn nonfinite_values_are_refused() {
    let cases: [(fn(&fixture::Built) -> usize, &str, usize); 7] = [
        (|built| built.bone(1) + 64, "bone", 1),
        (|built| built.bone(2) + 88 + 12, "bone", 2),
        (|built| built.controller(0) + 12, "bone controller", 0),
        (|built| built.hitbox(1) + 20, "hitbox", 1),
        (|built| built.sequence(1) + 32, "sequence", 1),
        (|built| built.sequence(0) + 80, "sequence", 0),
        (|built| built.sequence(0) + 136, "sequence", 0),
    ];
    for (field, what, index) in cases {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                refused(|built| built.put_f32(field(built), value)),
                StudioError::NonFinite { what, index },
                "{what} {index} {value}"
            );
        }
    }
}

#[test]
fn invalid_names_are_refused() {
    assert_eq!(
        refused(|built| {
            let at = built.bone(1);
            built.bytes[at..at + 32].fill(b'A');
        }),
        StudioError::InvalidName {
            what: "bone",
            index: 1
        }
    );
    assert_eq!(
        refused(|built| {
            let at = built.bone(0);
            built.bytes[at + 1] = 0xd0;
        }),
        StudioError::InvalidName {
            what: "bone",
            index: 0
        }
    );
    assert_eq!(
        refused(|built| {
            let at = built.group(0) + 32;
            built.bytes[at] = 0x07;
        }),
        StudioError::InvalidName {
            what: "sequence group",
            index: 0
        }
    );
    assert_eq!(
        refused(|built| {
            let at = built.sequence(1);
            built.bytes[at..at + 32].fill(b'x');
        }),
        StudioError::InvalidName {
            what: "sequence",
            index: 1
        }
    );
}

#[test]
fn invalid_sequences_are_refused() {
    let invalid = |index, reason| StudioError::InvalidSequence { index, reason };
    assert_eq!(
        refused(|built| built.put_i32(built.sequence(0) + 56, 0)),
        invalid(0, "frame count")
    );
    assert_eq!(
        refused(|built| built.put_i32(built.sequence(0) + 56, MAX_STUDIO_FRAMES as i32 + 1)),
        invalid(0, "frame count")
    );
    assert_eq!(
        refused(|built| built.put_f32(built.sequence(1) + 32, -1.0)),
        invalid(1, "negative frame rate")
    );
    assert_eq!(
        refused(|built| built.put_i32(built.sequence(1) + 156, 1)),
        invalid(1, "sequence group")
    );
    assert_eq!(
        refused(|built| built.put_i32(built.sequence(0) + 172, 2)),
        invalid(0, "next sequence")
    );
    for blends in [0, 3, 5, -1] {
        assert_eq!(
            refused(|built| built.put_i32(built.sequence(1) + 120, blends)),
            StudioError::UnsupportedBlends {
                sequence: 1,
                blends
            }
        );
    }
}

#[test]
fn malformed_animation_runs_are_refused() {
    let malformed = |reason| StudioError::MalformedAnimation {
        sequence: 0,
        blend: 0,
        bone: 0,
        channel: 0,
        reason,
    };
    let track = |built: &fixture::Built| built.track_starts[0][0][0][0].unwrap();
    assert_eq!(
        refused(|built| {
            let at = track(built);
            built.bytes[at + 1] = 0;
        }),
        malformed("run covers no frames")
    );
    assert_eq!(
        refused(|built| {
            let at = track(built);
            built.bytes[at] = 0;
        }),
        malformed("run has no values")
    );
    assert_eq!(
        refused(|built| {
            let at = track(built);
            built.bytes[at] = 4;
        }),
        malformed("run has more values than frames")
    );
    // A track starting past the end of the file
    assert_eq!(
        refused(|built| {
            let block = built.animation[0].unwrap();
            built.put_u16(block, u16::MAX);
        }),
        malformed("run lies outside the file")
    );
    // The last track's runs cut by the end of the file
    let built = Fixture::standard().build();
    let bytes = built.truncated(built.bytes.len() - 2);
    assert_eq!(
        parse(&bytes),
        Err(StudioError::MalformedAnimation {
            sequence: 1,
            blend: 1,
            bone: 0,
            channel: 3,
            reason: "run lies outside the file"
        })
    );
}

/// The motion bone is checked where the renderer reads it: whenever the motion type
/// has X, Y, Z, LX, LY or LZ. Otherwise it is kept as stored, leftovers included.
#[test]
fn an_active_motion_bone_must_exist() {
    let with_motion = |kind: i32, bone: i32| {
        let mut built = Fixture::standard().build();
        let at = built.sequence(0);
        built.put_i32(at + 72, kind);
        built.put_i32(at + 76, bone);
        parse(&built.bytes)
    };
    // Inactive: no flag, or only rotation flags, with a stock leftover value
    for kind in [0, 0x08, 0x20] {
        let model = with_motion(kind, 1128127587).unwrap();
        let idle = &model.sequences()[0];
        assert_eq!(idle.motion_bone, 1128127587);
        assert_eq!(idle.active_motion_bone(), None);
    }
    // Active, on an existing bone
    for kind in [0x01, 0x02, 0x04, 0x40, 0x80, 0x100, 0x07] {
        let model = with_motion(kind, 2).unwrap();
        assert_eq!(
            model.sequences()[0].active_motion_bone(),
            Some(2),
            "{kind:#x}"
        );
    }
    // Active, on no bone: the review's three-bone case with STUDIO_X and bone 3, and
    // the linear-motion flags the renderer reads the bone for too
    for (kind, bone) in [(0x01, 3), (0x02, -1), (0x04, 1128127587), (0x100, 3)] {
        assert_eq!(
            with_motion(kind, bone),
            Err(StudioError::InvalidSequence {
                index: 0,
                reason: "motion bone"
            }),
            "{kind:#x} {bone}"
        );
    }
}

/// Decoded animation counts against one budget across the model and its groups
#[test]
fn decoded_animation_is_budgeted() {
    let fixture = Fixture::with_external_group();
    let main = fixture.build();
    let group = fixture.build_group(1);
    // idle: 5 tracks of 4 frames; aim: 2 of 2; walk: 1 of 3; 3 bytes per frame
    let needed = (5 * 4 + 2 * 2 + 3) * 3;
    let resolve = |budget| {
        parse_main(&main.bytes, budget).and_then(|model| model.resolve(&[(1, &group.bytes)]))
    };
    assert!(resolve(needed).is_ok());
    assert_eq!(
        resolve(needed - 1),
        Err(StudioError::AnimationBudgetExceeded {
            limit: MAX_STUDIO_ANIMATION_BYTES
        })
    );
    assert_eq!(
        parse_main(&main.bytes, (5 * 4 + 2 * 2) * 3 - 1).unwrap_err(),
        StudioError::AnimationBudgetExceeded {
            limit: MAX_STUDIO_ANIMATION_BYTES
        }
    );
}

/// The game directory of a local Half-Life installation, for the checks against stock
/// files (`XENOXIDE_GAME_DIRECTORY`, the directory holding `valve`)
fn game_directory() -> PathBuf {
    PathBuf::from(
        std::env::var_os("XENOXIDE_GAME_DIRECTORY")
            .expect("XENOXIDE_GAME_DIRECTORY must point to a Half-Life installation"),
    )
}

/// Reads a model below `mod_directory` and the sequence groups it needs, which it names
/// relative to the same directory with Windows separators (`models\scientist01.mdl`).
/// A group file that cannot be read is left out, so resolving reports it missing.
fn load(mod_directory: &Path, path: &str) -> Result<StudioModel, StudioError> {
    let bytes = std::fs::read(mod_directory.join(path)).expect("model file");
    let unresolved = StudioModel::parse(&bytes)?;
    let files: Vec<(usize, Vec<u8>)> = unresolved
        .dependencies()
        .iter()
        .filter_map(|dependency| {
            let relative = dependency.name.replace('\\', "/");
            let bytes = std::fs::read(mod_directory.join(relative)).ok()?;
            Some((dependency.index, bytes))
        })
        .collect();
    let borrowed: Vec<(usize, &[u8])> = files
        .iter()
        .map(|(index, bytes)| (*index, bytes.as_slice()))
        .collect();
    unresolved.resolve(&borrowed)
}

/// Every stock main model in `valve/models` parses with its sequence groups, except
/// two refused for cause: `doctor.mdl` is version 9, and `strooper.mdl` names a
/// sequence group, `models\strooper01.mdl`, that the installation does not contain
#[test]
#[ignore = "requires XENOXIDE_GAME_DIRECTORY"]
fn every_stock_model_parses() {
    let valve = game_directory().join("valve");
    let mut entries: Vec<_> = std::fs::read_dir(valve.join("models"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.to_ascii_lowercase().ends_with(".mdl"))
        .collect();
    entries.sort();
    let mut failures = Vec::new();
    let mut parsed = 0;
    for name in &entries {
        let bytes = std::fs::read(valve.join("models").join(name)).unwrap();
        // Texture companions and sequence groups are not main models
        if bytes.get(0..4) != Some(b"IDST") {
            continue;
        }
        let header_bones = i32_at(&bytes, 140);
        let header_sequences = i32_at(&bytes, 164);
        if header_bones == 0 && header_sequences == 0 {
            continue;
        }
        match (name.as_str(), load(&valve, &format!("models/{name}"))) {
            (_, Ok(_)) => parsed += 1,
            (
                "doctor.mdl",
                Err(StudioError::UnsupportedVersion {
                    what: "header",
                    version: 9,
                }),
            ) => {}
            ("strooper.mdl", Err(StudioError::MissingDependency { group: 1, name }))
                if name == "models\\strooper01.mdl" => {}
            (_, Err(error)) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
    assert_eq!(parsed, 238);
}

/// The stock player model, used by multiplayer servers for player hitboxes, read in
/// full; its digest, as installed in both `valve` and `ag`, recorded
#[test]
#[ignore = "requires XENOXIDE_GAME_DIRECTORY"]
fn the_stock_player_model_reads() {
    let game = game_directory();
    for mod_directory in ["valve", "ag"] {
        let bytes = std::fs::read(game.join(mod_directory).join("models/player.mdl")).unwrap();
        assert_eq!(
            StudioDigest::of(&bytes).to_string(),
            "fb59b69ff909b5e1e8d22616d4ffdaa24deb40f698fc8711cfc63487a35582cd",
            "{mod_directory}"
        );
    }
    let model = load(&game.join("valve"), "models/player.mdl").unwrap();
    assert_eq!(model.bones().len(), 50);
    assert!(model.texture_companion());
    assert_eq!(model.sequence_groups().len(), 1);
    assert_eq!(
        model.identity().combined.to_string(),
        "678db174e6f7479d7c968688b73878a40ebaadf511d1cc841ced5fd82e8f26b5"
    );
    // Torso twist on four spine bones (controllers 0-3) and the mouth (4)
    let controllers: Vec<(u8, &str, i32)> = model
        .bone_controllers()
        .iter()
        .map(|controller| {
            let bone = model.bones()[controller.bone].name.as_str();
            (controller.index, bone, controller.axis)
        })
        .collect();
    assert_eq!(
        controllers,
        [
            (4, "Bone02", 0x20),
            (0, "Bip01 Spine", 0x08),
            (1, "Bip01 Spine1", 0x08),
            (2, "Bip01 Spine2", 0x08),
            (3, "Bip01 Spine3", 0x08),
        ]
    );
    // Twenty boxes in the standard hit groups: 1 head, 2 chest, 3 stomach, 4 and 5 the
    // arms, 6 and 7 the legs
    let hitboxes: Vec<(&str, i32)> = model
        .hitboxes()
        .iter()
        .map(|hitbox| (model.bones()[hitbox.bone].name.as_str(), hitbox.group))
        .collect();
    assert_eq!(
        hitboxes,
        [
            ("Bip01 Pelvis", 3),
            ("Bip01 L Leg", 6),
            ("Bip01 L Leg1", 6),
            ("Bip01 L Foot", 6),
            ("Bip01 R Leg", 7),
            ("Bip01 R Leg1", 7),
            ("Bip01 R Foot", 7),
            ("Bip01 Spine1", 3),
            ("Bip01 Spine2", 2),
            ("Bip01 Spine3", 2),
            ("Bip01 Neck", 2),
            ("Bip01 Head", 1),
            ("Bip01 L Arm", 4),
            ("Bip01 L Arm1", 4),
            ("Bip01 L Arm2", 4),
            ("Bip01 L Hand", 4),
            ("Bip01 R Arm", 5),
            ("Bip01 R Arm1", 5),
            ("Bip01 R Arm2", 5),
            ("Bip01 R Hand", 5),
        ]
    );
    // 77 sequences: the 52 weapon aim and shoot sequences have two blends, by pitch
    let sequences = model.sequences();
    assert_eq!(sequences.len(), 77);
    let blended = sequences
        .iter()
        .filter(|sequence| sequence.animation.len() == 2)
        .count();
    assert_eq!(blended, 52);
    let mut expected: std::collections::BTreeSet<String> = ["aim_1", "aim_2", "shoot_1", "shoot_2"]
        .map(String::from)
        .into();
    for weapon in [
        "crowbar",
        "trip",
        "onehanded",
        "python",
        "shotgun",
        "gauss",
        "mp5",
        "rpg",
        "egon",
        "squeak",
        "hive",
        "bow",
    ] {
        for stance in ["ref", "crouch"] {
            for action in ["aim", "shoot"] {
                expected.insert(format!("{stance}_{action}_{weapon}"));
            }
        }
    }
    let two_blend: std::collections::BTreeSet<String> = sequences
        .iter()
        .filter(|sequence| sequence.animation.len() == 2)
        .map(|sequence| sequence.label.clone())
        .collect();
    assert_eq!(two_blend, expected);
    let look_idle = &sequences[0];
    assert_eq!(
        (look_idle.label.as_str(), look_idle.frames, look_idle.fps),
        ("look_idle", 31, 14.0)
    );
}

/// A stock model with external sequence groups: all seven are dependencies, each in
/// the identity by name and digest
#[test]
#[ignore = "requires XENOXIDE_GAME_DIRECTORY"]
fn a_stock_model_with_sequence_groups_reads() {
    let valve = game_directory().join("valve");
    let model = load(&valve, "models/scientist.mdl").unwrap();
    let identity = model.identity();
    assert_eq!(
        identity.main.to_string(),
        "5995e3caea74e4f15ca8e382a90199795b30ea5d216d6bc4738c1f2ad23311c5"
    );
    assert_eq!(
        identity
            .sequence_groups
            .iter()
            .map(|(index, name, _)| (*index, name.as_str()))
            .collect::<Vec<_>>(),
        (1..=7)
            .map(|index| (index, format!("models\\scientist0{index}.mdl")))
            .collect::<Vec<_>>()
            .iter()
            .map(|(index, name)| (*index, name.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        identity.combined.to_string(),
        "8d84a6637fe0f4eb0b1515b507247092310d88883c04aa6069f7f679ad323e6f"
    );
}
