//! Optional local fixture manifest. Original demos stay outside Git; the
//! manifest records their paths, hashes, and verified expectations.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::container::Limits;
use crate::library::fingerprint::hash_file;
use crate::state::{DecodeOptions, DecodeStatus, decode_demo};
use crate::validate::crosscheck::crosscheck_demo;
use crate::validate::structural::{InspectReport, InspectStatus, inspect};

pub const MANIFEST_ENV: &str = "LAMBDA_SLICER_FIXTURE_MANIFEST";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub fixtures: Vec<Fixture>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fixture {
    pub role: String,
    pub path: PathBuf,
    pub bytes: Option<u64>,
    /// Full-file BLAKE3, lowercase hex.
    pub blake3: Option<String>,
    #[serde(default)]
    pub expect: Expectations,
    #[serde(default)]
    pub notes: String,
}

/// Container-level expectations. Every field is optional; only present
/// fields are checked.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Expectations {
    pub demo_protocol: Option<i32>,
    pub network_protocol: Option<i32>,
    pub game_dir: Option<String>,
    pub map_name: Option<String>,
    pub entries: Option<Vec<EntryExpectation>>,
    /// Network decode must complete (`true`) or stop (`false`).
    pub decode_complete: Option<bool>,
    /// Exact event counts by resource path, all sources combined.
    pub events: Option<BTreeMap<String, u64>>,
    /// Every identity cross-check passes and every server-copy sample
    /// agrees exactly (see `validate::crosscheck`).
    pub crosscheck_exact: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntryExpectation {
    pub entry_type: Option<i32>,
    pub description: Option<String>,
    pub offset: Option<i32>,
    pub length: Option<i32>,
    pub frame_count: Option<i32>,
    pub network_records: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FixtureResult {
    pub role: String,
    pub path: PathBuf,
    pub blake3: Option<String>,
    pub mismatches: Vec<String>,
    pub inspect: Option<InspectReport>,
}

impl FixtureResult {
    pub fn passed(&self) -> bool {
        self.mismatches.is_empty()
    }
}

pub fn load_manifest(path: &Path) -> Result<Manifest, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let m: Manifest =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if m.version != 1 {
        return Err(format!("unsupported manifest version {}", m.version));
    }
    Ok(m)
}

fn check<T: PartialEq + std::fmt::Debug>(
    out: &mut Vec<String>,
    what: &str,
    expected: &Option<T>,
    actual: T,
) {
    if let Some(e) = expected
        && *e != actual
    {
        out.push(format!("{what}: expected {e:?}, found {actual:?}"));
    }
}

/// Hash and inspect one fixture, comparing against its expectations. Reads
/// the file only.
pub fn check_fixture(f: &Fixture) -> FixtureResult {
    let mut m = Vec::new();
    let hash = match hash_file(&f.path) {
        Ok(h) => Some(h),
        Err(e) => {
            m.push(format!("cannot read: {e}"));
            None
        }
    };
    if let (Some(expected), Some(actual)) = (&f.blake3, &hash)
        && !expected.eq_ignore_ascii_case(actual)
    {
        m.push(format!("blake3: expected {expected}, found {actual}"));
    }
    let report = hash.is_some().then(|| inspect(&f.path, Limits::default()));
    if let Some(r) = &report {
        check(&mut m, "bytes", &f.bytes, r.size);
        if r.status != InspectStatus::Complete || r.has_errors() {
            m.push(format!("inspect status {:?} with errors", r.status));
        }
        if let Some(h) = &r.header {
            let e = &f.expect;
            check(&mut m, "demo_protocol", &e.demo_protocol, h.demo_protocol);
            check(
                &mut m,
                "network_protocol",
                &e.network_protocol,
                h.network_protocol,
            );
            check(&mut m, "game_dir", &e.game_dir, h.game_dir.clone());
            check(&mut m, "map_name", &e.map_name, h.map_name.clone());
        }
        if let Some(entries) = &f.expect.entries {
            if entries.len() != r.entries.len() {
                m.push(format!(
                    "entries: expected {}, found {}",
                    entries.len(),
                    r.entries.len()
                ));
            }
            for (i, (e, a)) in entries.iter().zip(&r.entries).enumerate() {
                let p = |s: &str| format!("entry {i} {s}");
                check(&mut m, &p("type"), &e.entry_type, a.entry_type);
                check(
                    &mut m,
                    &p("description"),
                    &e.description,
                    a.description.clone(),
                );
                check(&mut m, &p("offset"), &e.offset, a.offset);
                check(&mut m, &p("length"), &e.length, a.length);
                check(&mut m, &p("frame_count"), &e.frame_count, a.frame_count);
                check(
                    &mut m,
                    &p("network_records"),
                    &e.network_records,
                    a.network_records,
                );
            }
        }
    }
    if report.is_some() && (f.expect.decode_complete.is_some() || f.expect.events.is_some()) {
        let options = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };
        let (d, _) = decode_demo(&f.path, &options, |_, _, _| {});
        let complete = d.status == DecodeStatus::Complete;
        if f.expect
            .decode_complete
            .is_some_and(|want| want != complete)
        {
            m.push(format!("decode status {:?}: {:?}", d.status, d.error));
        }
        if let Some(want) = &f.expect.events {
            for (name, n) in want {
                let got = d.stats.events_by_resource.get(name).copied().unwrap_or(0);
                if got != *n {
                    m.push(format!("events {name}: expected {n}, found {got}"));
                }
            }
        }
    }
    if report.is_some() && f.expect.crosscheck_exact.is_some() {
        let r = crosscheck_demo(&f.path, Limits::default());
        let (ok, n) = r.server_copy_agreement();
        let exact = r.decode_error.is_none() && r.identity_failures().is_empty() && ok == n;
        if f.expect.crosscheck_exact != Some(exact) {
            m.push(format!(
                "crosscheck: identity failures {:?}, server copy {ok}/{n}",
                r.identity_failures()
                    .iter()
                    .map(|c| c.name)
                    .collect::<Vec<_>>()
            ));
        }
    }
    FixtureResult {
        role: f.role.clone(),
        path: f.path.clone(),
        blake3: hash,
        mismatches: m,
        inspect: report,
    }
}
