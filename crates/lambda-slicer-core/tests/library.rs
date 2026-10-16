//! Library scan and cache behaviour on generated files

use std::path::{Path, PathBuf};

use lambda_slicer_core::container::Limits;
use lambda_slicer_core::library::cache::{Cache, Entry, FileQuery};
use lambda_slicer_core::library::scan::{Outcome, ScanControl, ScanEvent, ScanOptions, scan};

fn options(roots: Vec<PathBuf>, cache: &Path) -> ScanOptions {
    ScanOptions {
        roots,
        cache: cache.to_path_buf(),
        workers: 2,
        force: false,
        full_hash: true,
        limits: Limits::default(),
    }
}

/// Every file's outcome, by file name
fn outcomes(o: &ScanOptions) -> Vec<(String, Outcome)> {
    let control = ScanControl::new();
    let mut out = Vec::new();
    scan(o, &control, |e| {
        if let ScanEvent::File { path, outcome } = e {
            out.push((
                path.file_name().unwrap().to_string_lossy().into_owned(),
                outcome.clone(),
            ));
        }
    })
    .unwrap();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn failed() -> Outcome {
    Outcome::Analyzed {
        status: "failed".into(),
    }
}

#[test]
fn a_bad_file_gets_its_own_result_and_a_rescan_hits_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let demos = dir.path().join("demos");
    std::fs::create_dir(&demos).unwrap();
    std::fs::write(demos.join("a.dem"), b"not a demo").unwrap();
    std::fs::write(demos.join("b.dem"), b"also not a demo").unwrap();
    let cache = dir.path().join("cache.sqlite");
    let o = options(vec![demos.clone()], &cache);

    assert_eq!(
        outcomes(&o),
        [("a.dem".into(), failed()), ("b.dem".into(), failed())]
    );
    assert_eq!(
        outcomes(&o),
        [
            ("a.dem".into(), Outcome::CacheHit),
            ("b.dem".into(), Outcome::CacheHit)
        ]
    );

    // A changed file is analysed again, the other stays cached
    std::fs::write(demos.join("b.dem"), b"changed and longer").unwrap();
    assert_eq!(
        outcomes(&o),
        [
            ("a.dem".into(), Outcome::CacheHit),
            ("b.dem".into(), failed())
        ]
    );

    let mut forced = o.clone();
    forced.force = true;
    assert_eq!(
        outcomes(&forced),
        [("a.dem".into(), failed()), ("b.dem".into(), failed())]
    );

    let c = Cache::open(&cache).unwrap();
    let rows = c
        .query(&FileQuery {
            limit: 10,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.status == "failed"));
    assert_eq!(c.stats().unwrap().stale, 0);
}

#[test]
fn hard_links_are_one_file() {
    let dir = tempfile::tempdir().unwrap();
    let demos = dir.path().join("demos");
    std::fs::create_dir(&demos).unwrap();
    std::fs::write(demos.join("a.dem"), b"x").unwrap();
    std::fs::hard_link(demos.join("a.dem"), demos.join("b.dem")).unwrap();
    let mut o = options(vec![demos.clone()], &dir.path().join("cache.sqlite"));
    o.workers = 1;

    let got = outcomes(&o);
    assert_eq!(got[0], ("a.dem".into(), failed()));
    assert!(matches!(got[1].1, Outcome::Duplicate { .. }), "{got:?}");

    // Still one file when the first path is a cache hit
    let got = outcomes(&o);
    assert_eq!(got[0], ("a.dem".into(), Outcome::CacheHit));
    assert!(matches!(got[1].1, Outcome::Duplicate { .. }), "{got:?}");
}

#[test]
fn a_cancelled_scan_stores_nothing_and_keeps_earlier_results() {
    let dir = tempfile::tempdir().unwrap();
    let demos = dir.path().join("demos");
    std::fs::create_dir(&demos).unwrap();
    std::fs::write(demos.join("a.dem"), b"x").unwrap();
    let cache = dir.path().join("cache.sqlite");
    let o = options(vec![demos.clone()], &cache);
    outcomes(&o);

    let control = ScanControl::new();
    control.cancel();
    let mut forced = o.clone();
    forced.force = true;
    let progress = scan(&forced, &control, |_| {}).unwrap();
    assert!(progress.cancelled);
    assert_eq!(progress.analyzed, 0);
    assert_eq!(Cache::open(&cache).unwrap().stats().unwrap().files, 1);
}

#[test]
fn unreadable_roots_are_reported_and_the_scan_goes_on() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.dem"), b"x").unwrap();
    let o = options(
        vec![dir.path().join("missing"), dir.path().to_path_buf()],
        &dir.path().join("cache.sqlite"),
    );

    let control = ScanControl::new();
    let mut errors = 0;
    let progress = scan(&o, &control, |e| {
        if let ScanEvent::DiscoveryError { .. } = e {
            errors += 1;
        }
    })
    .unwrap();
    assert_eq!(errors, 1);
    assert_eq!(progress.completed, 1);
}

#[test]
fn cache_entries_round_trip() {
    let mut c = Cache::in_memory().unwrap();
    let path = Path::new("x/y.dem");
    let entry = Entry {
        path,
        size: 5,
        mtime_ns: Some(7),
        file_identity: Some("id".into()),
        quick_hash: None,
        full_hash: None,
        header: None,
        report: None,
        error: Some(lambda_slicer_core::message::Message::new(
            "test-broken",
            "broken",
        )),
    };
    c.store(&entry).unwrap();
    c.store(&entry).unwrap();

    let got = c.lookup(path).unwrap().unwrap();
    assert_eq!(
        (got.size, got.mtime_ns, got.status.as_str()),
        (5, Some(7), "failed")
    );
    assert_eq!(c.stats().unwrap().files, 1);
}

fn header(map: &str, duration: f32) -> lambda_slicer_core::library::cache::HeaderInfo {
    lambda_slicer_core::library::cache::HeaderInfo {
        map: map.into(),
        game_dir: "ag".into(),
        demo_protocol: 5,
        network_protocol: 48,
        duration,
    }
}

#[test]
fn queries_fold_case_page_count_and_sort() {
    use lambda_slicer_core::library::cache::{Sort, fold};

    let mut c = Cache::in_memory().unwrap();
    for (name, map, duration) in [
        ("b/Iğdır_final.dem", "crossfire", 300.),
        ("a/stalk.dem", "stalkyard", 60.),
        ("c/ISTANBUL.dem", "İzmir", 120.),
    ] {
        c.store(&Entry {
            path: Path::new(name),
            size: 1,
            mtime_ns: None,
            file_identity: None,
            quick_hash: None,
            full_hash: None,
            header: Some(header(map, duration)),
            report: None,
            error: None,
        })
        .unwrap();
    }

    assert_eq!(fold("İSTANBUL Iğdır"), "istanbul iğdir");
    let q = |f: FileQuery| -> Vec<String> {
        c.query(&FileQuery { limit: 10, ..f })
            .unwrap()
            .into_iter()
            .map(|r| r.path)
            .collect()
    };
    let all = FileQuery::default();
    assert_eq!(c.count(&all).unwrap(), 3);
    assert_eq!(
        q(all.clone()),
        ["a/stalk.dem", "b/Iğdır_final.dem", "c/ISTANBUL.dem"]
    );

    // Turkish i forms match each other, other letters ignore case
    let text = |t: &str| FileQuery {
        text: Some(t.into()),
        ..Default::default()
    };
    assert_eq!(q(text("istanbul")), ["c/ISTANBUL.dem"]);
    assert_eq!(q(text("IĞDIR")), ["b/Iğdır_final.dem"]);
    let map = FileQuery {
        map: Some("izmir".into()),
        ..Default::default()
    };
    assert_eq!(q(map), ["c/ISTANBUL.dem"]);

    let longest = FileQuery {
        sort: Sort::Duration,
        descending: true,
        ..Default::default()
    };
    assert_eq!(q(longest.clone())[0], "b/Iğdır_final.dem");
    let page = c
        .query(&FileQuery {
            offset: 1,
            limit: 1,
            ..longest
        })
        .unwrap();
    assert_eq!(page[0].path, "c/ISTANBUL.dem");

    let long = FileQuery {
        min_duration: Some(100.),
        ..Default::default()
    };
    assert_eq!(c.count(&long).unwrap(), 2);
}

#[test]
fn a_new_cache_opened_at_once_is_built_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    std::thread::scope(|s| {
        let handles: Vec<_> = (0..8)
            .map(|_| s.spawn(|| Cache::open(&path).map(|_| ())))
            .collect();
        for h in handles {
            h.join().unwrap().unwrap();
        }
    });
    assert_eq!(Cache::open(&path).unwrap().stats().unwrap().files, 0);
}
