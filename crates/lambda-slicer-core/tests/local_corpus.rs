//! Opt-in checks against the developer's local fixture manifest. Original
//! demos are never committed
//!
//! ```powershell
//! $env:LAMBDA_SLICER_FIXTURE_MANIFEST = 'C:\path\to\local-fixtures.json'
//! cargo test -p lambda-slicer-core --test local_corpus -- --ignored
//! ```

use lambda_slicer_core::fixtures::{MANIFEST_ENV, check_fixture, load_manifest};

#[test]
#[ignore = "needs local fixtures; set LAMBDA_SLICER_FIXTURE_MANIFEST"]
fn local_fixtures_match_manifest() {
    let path = std::env::var_os(MANIFEST_ENV).expect("set LAMBDA_SLICER_FIXTURE_MANIFEST");
    let manifest = load_manifest(path.as_ref()).unwrap();

    let mut failures = Vec::new();
    for f in &manifest.fixtures {
        let r = check_fixture(f);
        if !r.passed() {
            failures.push(format!(
                "{} {}: {:?}",
                r.role,
                r.path.display(),
                r.mismatches
            ));
        }
    }

    assert!(failures.is_empty(), "{failures:#?}");
}

/// Exports from every fixture, whether published or refused, leave the
/// source's bytes as they were (implementation plan §12)
#[test]
#[ignore = "needs local fixtures; set LAMBDA_SLICER_FIXTURE_MANIFEST"]
fn exports_leave_every_fixture_unchanged() {
    use lambda_slicer_core::container::Limits;
    use lambda_slicer_core::library::fingerprint::hash_file;
    use lambda_slicer_core::slice::restart::run_restart;
    use lambda_slicer_core::state::{DecodeOptions, decode_demo};
    use lambda_slicer_core::timeline::time::DemoTime;

    let path = std::env::var_os(MANIFEST_ENV).expect("set LAMBDA_SLICER_FIXTURE_MANIFEST");
    let manifest = load_manifest(path.as_ref()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let options = DecodeOptions {
        collect_events: false,
        ..DecodeOptions::default()
    };

    let mut changed = Vec::new();
    for (i, f) in manifest.fixtures.iter().enumerate() {
        let before = hash_file(&f.path).unwrap();
        let (report, _) = decode_demo(&f.path, &options, |_, _, _| {});
        let length = report.groups.end.unwrap_or(0.);

        // Ten seconds from a third and from two thirds of the way in
        let mut outcomes = Vec::new();
        for (k, at) in [length / 3., length * 2. / 3.].into_iter().enumerate() {
            let start = DemoTime::from_f32(at).unwrap();
            let end = DemoTime::from_micros(start.micros() + 10_000_000);
            let out = dir.path().join(format!("{i}_{k}.dem"));
            let r = run_restart(&f.path, &out, start, Some(end), Limits::default());
            outcomes.push(
                r.map(|_| "published".to_string())
                    .unwrap_or_else(|e| e.to_string()),
            );
        }
        println!("{}: {outcomes:?}", f.path.display());

        if hash_file(&f.path).unwrap() != before {
            changed.push(f.path.display().to_string());
        }
    }

    assert!(changed.is_empty(), "changed by an export: {changed:?}");
}
