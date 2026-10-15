//! `--json -` keeps stdout a single JSON document

use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lambda-slicer"))
        .args(args)
        .output()
        .unwrap()
}

fn assert_json_stdout(out: &Output) {
    let text = String::from_utf8(out.stdout.clone()).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&text);
    assert!(parsed.is_ok(), "stdout is not one JSON document:\n{text}");
}

#[test]
fn decode_listings_are_refused_with_json_on_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let demo = dir.path().join("missing.dem");
    let demo = demo.to_str().unwrap();

    for extra in [&["--events"][..], &["--trace", "0", "10"][..]] {
        let mut args = vec!["decode", demo, "--json", "-"];
        args.extend_from_slice(extra);
        let out = run(&args);

        assert!(!out.status.success());
        assert!(out.stdout.is_empty(), "{extra:?} wrote to stdout");
        let err = String::from_utf8(out.stderr).unwrap();
        assert!(err.contains("--json FILE"), "{err}");
    }
}

#[test]
fn decode_listings_are_allowed_with_a_json_file() {
    let dir = tempfile::tempdir().unwrap();
    let demo = dir.path().join("missing.dem");
    let report = dir.path().join("report.json");

    let out = run(&[
        "decode",
        demo.to_str().unwrap(),
        "--events",
        "--json",
        report.to_str().unwrap(),
    ]);

    assert!(Path::new(&report).exists());
    let text = std::fs::read_to_string(&report).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&text).is_ok());
    assert!(out.stderr.is_empty());
}

#[test]
fn json_on_stdout_is_a_single_document() {
    let dir = tempfile::tempdir().unwrap();
    let demo = dir.path().join("missing.dem");
    let demo = demo.to_str().unwrap();

    for command in ["inspect", "validate", "decode", "crosscheck"] {
        let out = run(&[command, demo, "--json", "-"]);
        assert_json_stdout(&out);
    }
}
