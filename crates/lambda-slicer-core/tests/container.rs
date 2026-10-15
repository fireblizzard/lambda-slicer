mod common;

use std::io::Cursor;
use std::path::{Path, PathBuf};

use lambda_slicer_core::container::{
    ContainerError, DemoReader, FrameBody, FrameCause, Limits, index_entry,
};
use lambda_slicer_core::slice::ExportError;
use lambda_slicer_core::slice::controls::{Control, run_control};
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_core::validate::structural::{InspectStatus, inspect};
use proptest::prelude::*;

fn reader(bytes: Vec<u8>) -> Result<DemoReader<Cursor<Vec<u8>>>, ContainerError> {
    DemoReader::new(Cursor::new(bytes), Limits::default())
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, bytes).unwrap();
    p
}

fn frame_error(bytes: Vec<u8>, entry: u32) -> FrameCause {
    let mut r = reader(bytes).unwrap();
    let idx = index_entry(&mut r, entry, false).unwrap();
    match idx.error {
        Some(ContainerError::Frame { cause, .. }) => cause,
        other => panic!("expected frame error, got {other:?}"),
    }
}

/// Byte offset of the playback entry, read from the directory
fn playback_offset(bytes: &[u8]) -> usize {
    let r = reader(bytes.to_vec()).unwrap();
    r.directory[1].offset as usize
}

#[test]
fn synthetic_demo_indexes_completely() {
    let bytes = demo(4);
    let mut r = reader(bytes).unwrap();

    assert_eq!(r.header.map_name(), "crossfire");
    assert_eq!(r.header.game_dir(), "ag");
    assert_eq!(r.directory.len(), 2);

    let idx = index_entry(&mut r, 1, true).unwrap();
    assert!(idx.is_complete(), "{:?}", idx.error);
    assert_eq!(idx.counts.network, 4);
    assert_eq!(idx.counts.event, 4);
    assert_eq!(idx.counts.demo_start, 1);
    assert_eq!(idx.terminal_records, 1);
    assert_eq!(idx.spans.len(), 1 + 4 * 7 + 1);
}

#[test]
fn event_args_accessors_read_documented_offsets() {
    let mut r = reader(demo(1)).unwrap();

    let ev = r
        .frames(1)
        .unwrap()
        .filter_map(Result::ok)
        .find_map(|lf| match lf.frame.body {
            FrameBody::Event(e) => Some(e),
            _ => None,
        })
        .unwrap();

    assert_eq!(ev.index, 9);
    assert_eq!(ev.bparam2(), 0x0303_0303);
}

#[test]
fn inspect_reports_complete_with_no_findings() {
    let dir = tempfile::tempdir().unwrap();
    let p = write(dir.path(), "a.dem", &demo(3));

    let r = inspect(&p, Limits::default());

    assert_eq!(r.status, InspectStatus::Complete);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn repeated_terminal_records_are_accepted_and_preserved() {
    // Mirrors a real recording whose LOADING entry holds many NextSection
    // records after the first
    let mut bytes = demo(2);
    let r = reader(bytes.clone()).unwrap();
    let (off, len) = (
        r.directory[0].offset as usize,
        r.directory[0].length as usize,
    );

    let mut extra = Vec::new();
    for i in 0..3u8 {
        extra.push(5);
        extra.extend_from_slice(&(1.0f32 + f32::from(i)).to_bits().to_le_bytes());
        extra.extend_from_slice(&(200 + i32::from(i)).to_le_bytes());
    }

    let insert_at = off + len;
    bytes.splice(insert_at..insert_at, extra.iter().copied());

    // Fix up directory, because entry 0 grows and later regions shift
    let shift = extra.len() as i32;
    let dir_off = i32::from_le_bytes(bytes[540..544].try_into().unwrap()) + shift;
    bytes[540..544].copy_from_slice(&dir_off.to_le_bytes());

    let d = dir_off as usize + 4;
    let e0_len = d + 88;
    let e1_off = d + 92 + 84;
    let l = i32::from_le_bytes(bytes[e0_len..e0_len + 4].try_into().unwrap()) + shift;
    bytes[e0_len..e0_len + 4].copy_from_slice(&l.to_le_bytes());
    let o = i32::from_le_bytes(bytes[e1_off..e1_off + 4].try_into().unwrap()) + shift;
    bytes[e1_off..e1_off + 4].copy_from_slice(&o.to_le_bytes());

    let mut r = reader(bytes.clone()).unwrap();
    let idx = index_entry(&mut r, 0, false).unwrap();
    assert!(idx.is_complete());
    assert_eq!(idx.terminal_records, 4);

    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &bytes);
    let rep = run_control(
        &src,
        &dir.path().join("rt.dem"),
        Control::Roundtrip,
        Limits::default(),
    )
    .unwrap();

    assert_eq!(rep.byte_identical, Some(true));
}

#[test]
fn roundtrip_is_byte_identical() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(5));
    let out = dir.path().join("out.dem");

    let rep = run_control(&src, &out, Control::Roundtrip, Limits::default()).unwrap();

    assert_eq!(rep.byte_identical, Some(true));
    assert_eq!(std::fs::read(&out).unwrap(), std::fs::read(&src).unwrap());
}

#[test]
fn exact_copy_matches_source() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(2));
    let out = dir.path().join("copy.dem");

    run_control(&src, &out, Control::ExactCopy, Limits::default()).unwrap();

    assert_eq!(std::fs::read(&out).unwrap(), std::fs::read(&src).unwrap());
}

#[test]
fn truncate_end_snaps_to_next_group_and_rewrites_directory() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(10));
    let out = dir.path().join("cut.dem");

    // Groups sit at 0.0, 0.5, and so on, so [0, 1.2) keeps groups 0..=2 and
    // the terminal takes group 3's time and frame number
    let end: DemoTime = "1.2".parse().unwrap();
    let rep = run_control(&src, &out, Control::TruncateEnd(end), Limits::default()).unwrap();
    let cut = rep.cut.unwrap();

    assert_eq!(cut.retained_network_records, 3);
    assert_eq!(cut.effective_end_us, 1_500_000);
    assert_eq!(cut.terminal_frame_number, 3);

    let r = inspect(&out, Limits::default());
    assert_eq!(r.status, InspectStatus::Complete);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    let e = &r.entries[1];
    assert_eq!(e.frame_count, 3);
    assert_eq!(e.track_time, 1.5);
    assert_eq!(e.network_records, 3);

    // Header (except the directory offset) and LOADING entry are unchanged
    let src_bytes = std::fs::read(&src).unwrap();
    let out_bytes = std::fs::read(&out).unwrap();
    let pb = playback_offset(&src_bytes);

    assert!(src_bytes[..540] == out_bytes[..540]);
    assert!(src_bytes[544..pb] == out_bytes[544..pb]);
}

#[test]
fn truncate_end_on_exact_group_time_excludes_that_group() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(10));
    let out = dir.path().join("cut.dem");

    let rep = run_control(
        &src,
        &out,
        Control::TruncateEnd("1.5".parse().unwrap()),
        Limits::default(),
    )
    .unwrap();

    assert_eq!(rep.cut.unwrap().retained_network_records, 3);
}

#[test]
fn truncate_errors_for_empty_and_out_of_range() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(4));

    let empty = run_control(
        &src,
        &dir.path().join("e.dem"),
        Control::TruncateEnd(DemoTime::ZERO),
        Limits::default(),
    );
    assert!(matches!(empty, Err(ExportError::EmptyRange)), "{empty:?}");

    let beyond = run_control(
        &src,
        &dir.path().join("b.dem"),
        Control::TruncateEnd("10".parse().unwrap()),
        Limits::default(),
    );
    assert!(
        matches!(beyond, Err(ExportError::RangeBeyondEnd { .. })),
        "{beyond:?}"
    );

    // Failed exports leave nothing behind but the source
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("src.dem")]);
}

#[test]
fn export_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(2));
    let existing = write(dir.path(), "existing.dem", b"keep me");

    let r = run_control(&src, &existing, Control::Roundtrip, Limits::default());
    assert!(matches!(r, Err(ExportError::DestinationExists(_))), "{r:?}");
    assert_eq!(std::fs::read(&existing).unwrap(), b"keep me");

    let r = run_control(&src, &src, Control::ExactCopy, Limits::default());
    assert!(matches!(r, Err(ExportError::SameAsSource(_))), "{r:?}");

    let r = run_control(
        &src,
        &dir.path().join("missing/out.dem"),
        Control::ExactCopy,
        Limits::default(),
    );
    assert!(matches!(r, Err(ExportError::MissingParent(_))), "{r:?}");
}

#[cfg(windows)]
#[test]
fn export_rejects_hardlink_alias_of_source() {
    let dir = tempfile::tempdir().unwrap();
    let src = write(dir.path(), "src.dem", &demo(2));
    let link = dir.path().join("alias.dem");
    std::fs::hard_link(&src, &link).unwrap();

    let r = run_control(&src, &link, Control::Roundtrip, Limits::default());
    assert!(matches!(r, Err(ExportError::SameAsSource(_))), "{r:?}");
}

// Malformed input

#[test]
fn rejects_bad_magic_and_short_files() {
    let mut b = demo(1);
    b[0] = b'X';
    assert!(matches!(reader(b), Err(ContainerError::BadMagic)));

    assert!(matches!(
        reader(vec![0; 100]),
        Err(ContainerError::TooShort { len: 100 })
    ));
}

#[test]
fn zero_directory_offset_is_reported_as_incomplete() {
    let mut b = demo(1);
    b[540..544].copy_from_slice(&0i32.to_le_bytes());

    assert!(matches!(reader(b), Err(ContainerError::NoDirectory)));
}

#[test]
fn rejects_invalid_directory_values() {
    let base = demo(1);
    let dir_off = i32::from_le_bytes(base[540..544].try_into().unwrap()) as usize;

    let mut b = base.clone();
    b[540..544].copy_from_slice(&(-5i32).to_le_bytes());
    assert!(matches!(reader(b), Err(ContainerError::Range { .. })));

    let mut b = base.clone();
    b[540..544].copy_from_slice(&i32::MAX.to_le_bytes());
    assert!(matches!(reader(b), Err(ContainerError::Range { .. })));

    let mut b = base.clone();
    b[dir_off..dir_off + 4].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(matches!(reader(b), Err(ContainerError::Range { .. })));

    let mut b = base.clone();
    b[dir_off..dir_off + 4].copy_from_slice(&1000i32.to_le_bytes());
    assert!(matches!(reader(b), Err(ContainerError::Range { .. })));

    // Entry length running past the end of the file
    let mut b = base.clone();
    let len_field = dir_off + 4 + 92 + 88;
    b[len_field..len_field + 4].copy_from_slice(&i32::MAX.to_le_bytes());
    let mut r = reader(b).unwrap();
    assert!(matches!(r.frames(1), Err(ContainerError::Range { .. })));
}

#[test]
fn unknown_tag_stops_with_location() {
    let mut b = demo(2);
    let pb = playback_offset(&b);
    b[pb] = 42;

    let mut r = reader(b).unwrap();
    let idx = index_entry(&mut r, 1, false).unwrap();
    match idx.error {
        Some(ContainerError::Frame {
            entry: 1,
            ordinal: 0,
            offset,
            cause: FrameCause::UnknownTag(42),
        }) => {
            assert_eq!(offset as usize, pb)
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn non_finite_time_is_rejected() {
    let mut b = demo(1);
    let pb = playback_offset(&b);
    b[pb + 1..pb + 5].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());

    assert!(matches!(frame_error(b, 1), FrameCause::NonFiniteTime(_)));
}

/// Offset of the first network message length field in the playback entry
fn first_network_length_field(b: &[u8]) -> usize {
    let mut r = reader(b.to_vec()).unwrap();
    let lf = r
        .frames(1)
        .unwrap()
        .map(Result::unwrap)
        .find(|lf| matches!(lf.frame.body, FrameBody::Network(_)))
        .unwrap();

    lf.offset as usize + 9 + 436 + 28
}

#[test]
fn negative_and_oversized_lengths_are_rejected() {
    let base = demo(2);
    let at = first_network_length_field(&base);

    let mut b = base.clone();
    b[at..at + 4].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(matches!(
        frame_error(b, 1),
        FrameCause::NegativeLength { .. }
    ));

    let mut b = base.clone();
    b[at..at + 4].copy_from_slice(&(64i32 << 20).to_le_bytes());
    assert!(matches!(frame_error(b, 1), FrameCause::OverLimit { .. }));

    let mut b = base.clone();
    b[at..at + 4].copy_from_slice(&(1i32 << 20).to_le_bytes());
    assert!(matches!(frame_error(b, 1), FrameCause::Truncated { .. }));
}

#[test]
fn missing_terminal_is_an_error() {
    let mut b = demo(1);

    // Shorten the playback entry by the 9-byte terminal record
    let dir_off = i32::from_le_bytes(b[540..544].try_into().unwrap()) as usize;
    let len_field = dir_off + 4 + 92 + 88;
    let len = i32::from_le_bytes(b[len_field..len_field + 4].try_into().unwrap()) - 9;
    b[len_field..len_field + 4].copy_from_slice(&len.to_le_bytes());

    assert_eq!(frame_error(b, 1), FrameCause::MissingTerminal);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Arbitrary corruption of a valid file never panics and never reads
    /// outside the entry
    #[test]
    fn mutated_files_never_panic(muts in prop::collection::vec((0usize..4096, any::<u8>()), 1..16)) {
        let mut b = demo(3);
        for (at, v) in muts {
            let at = at % b.len();
            b[at] = v;
        }

        if let Ok(mut r) = reader(b) {
            for e in 0..r.directory.len() as u32 {
                let _ = index_entry(&mut r, e, true);
            }
        }
    }

    #[test]
    fn random_bytes_never_panic(tail in prop::collection::vec(any::<u8>(), 0..2048)) {
        let mut b = demo(0)[..544].to_vec();
        b.extend(tail);
        let off = 544i32;
        b[540..544].copy_from_slice(&off.to_le_bytes());

        if let Ok(mut r) = reader(b) {
            for e in 0..r.directory.len() as u32 {
                let _ = index_entry(&mut r, e, false);
            }
        }
    }
}

use common::demo;
