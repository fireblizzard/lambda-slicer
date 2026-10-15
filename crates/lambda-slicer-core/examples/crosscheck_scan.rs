//! Development probe: run decode cross-checks over every `.dem` under the
//! given roots (read-only) and aggregate the results.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lambda_slicer_core::container::Limits;
use lambda_slicer_core::validate::crosscheck::crosscheck_demo;

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        let p = e.path();
        if ft.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("dem")) {
            out.push(p);
        }
    }
}

fn main() {
    let mut files = Vec::new();
    for root in std::env::args().skip(1) {
        walk(Path::new(&root), &mut files);
    }
    let start = std::time::Instant::now();
    let mut totals = BTreeMap::<String, (u64, u64)>::new();
    let (mut errors, mut identity_fail, mut skipped_spec, mut skipped_first) =
        (0u64, Vec::new(), 0u64, 0u64);
    let mut imperfect: Vec<(f64, bool, u64, String)> = Vec::new();
    let (mut sc_ok, mut sc_n, mut perfect_files, mut files_with_copy) = (0u64, 0u64, 0u64, 0u64);
    for p in &files {
        let r = crosscheck_demo(p, Limits::default());
        if r.decode_error.is_some() {
            errors += 1;
            continue;
        }
        skipped_spec += r.skipped_spectator_frames;
        skipped_first += r.skipped_before_first_packet;
        for c in &r.exact {
            let t = totals
                .entry(format!("{:?} {}", c.kind, c.name))
                .or_default();
            t.0 += c.match_either;
            t.1 += c.compared;
        }
        for c in &r.distance {
            let t = totals
                .entry(format!("{:?} {} (exact)", c.kind, c.name))
                .or_default();
            t.0 += c.counts[0];
            t.1 += c.compared;
        }
        if !r.identity_failures().is_empty() && identity_fail.len() < 20 {
            identity_fail.push(format!(
                "{}: {:?}",
                p.display(),
                r.identity_failures()
                    .iter()
                    .map(|c| c.name)
                    .collect::<Vec<_>>()
            ));
        }
        let (ok, n) = r.server_copy_agreement();
        sc_ok += ok;
        sc_n += n;
        if n > 0 {
            files_with_copy += 1;
            if ok == n {
                perfect_files += 1;
            } else {
                imperfect.push((
                    ok as f64 / n as f64,
                    r.inferred_delta_bases,
                    n - ok,
                    p.display().to_string(),
                ));
            }
        }
    }
    println!(
        "{} files, {:.1}s, {errors} with decode errors",
        files.len(),
        start.elapsed().as_secs_f64()
    );
    println!(
        "skipped spectator/intermission frames {skipped_spec}, records before first playback packet {skipped_first}"
    );
    for (k, (ok, n)) in &totals {
        println!(
            "{ok:>12}/{n:<12} {:>9.5}%  {k}",
            if *n > 0 {
                100.0 * *ok as f64 / *n as f64
            } else {
                100.0
            }
        );
    }
    println!(
        "identity failures in {} files (first 20):",
        identity_fail.len()
    );
    for f in &identity_fail {
        println!("  {f}");
    }
    println!(
        "server-copy agreement {sc_ok}/{sc_n}; {perfect_files}/{files_with_copy} files perfect"
    );
    imperfect.sort_by(|a, b| a.0.total_cmp(&b.0));
    let inferred = imperfect.iter().filter(|x| x.1).count();
    println!(
        "{} imperfect files ({inferred} with inferred delta bases); worst 30:",
        imperfect.len()
    );
    for (rate, inf, miss, p) in imperfect.iter().take(30) {
        println!(
            "  {:>9.5}% misses {miss:>6} inferred {inf}  {p}",
            rate * 100.0
        );
    }
}
