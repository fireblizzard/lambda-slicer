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
