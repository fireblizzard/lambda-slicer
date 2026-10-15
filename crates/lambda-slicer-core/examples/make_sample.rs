//! Development tool that builds a reproducible scan sample
//!
//! Usage is `make_sample <problems.txt> <out.txt> <count> <seed> <roots...>`
//!
//! Every path in `problems.txt` is included, and the rest are drawn from the
//! roots round-robin across map names (read from each demo header), so
//! common maps cannot dominate. Read-only

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use lambda_slicer_core::container::read_header;

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

/// splitmix64, which is small, seedable, and good enough for sampling
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn key(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn map_name(p: &Path) -> String {
    let Ok(mut f) = File::open(p).map(BufReader::new) else {
        return "<unreadable>".into();
    };
    let Ok(len) = f.seek(SeekFrom::End(0)) else {
        return "<unreadable>".into();
    };

    match read_header(&mut f, len) {
        Ok(h) => h.map_name().to_lowercase(),
        Err(_) => "<no header>".into(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [problems, out, count, seed, roots @ ..] = args.as_slice() else {
        eprintln!("usage: make_sample <problems.txt> <out.txt> <count> <seed> <roots...>");
        std::process::exit(2);
    };
    let count: usize = count.parse().expect("count");
    let mut rng = Rng(seed.parse().expect("seed"));

    let mut files = Vec::new();
    for r in roots {
        walk(Path::new(r), &mut files);
    }
    let by_key: BTreeMap<String, PathBuf> = files.iter().map(|p| (key(p), p.clone())).collect();

    let mut chosen = Vec::new();
    let mut taken = BTreeSet::new();
    for line in std::fs::read_to_string(problems).expect("problems").lines() {
        let k = key(Path::new(line.trim()));
        if let Some(p) = by_key.get(&k)
            && taken.insert(k)
        {
            chosen.push(p.clone());
        }
    }
    let problem_count = chosen.len();

    let mut maps: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for p in &files {
        if !taken.contains(&key(p)) {
            maps.entry(map_name(p)).or_default().push(p.clone());
        }
    }

    let mut groups: Vec<Vec<PathBuf>> = maps.into_values().collect();
    // Random map order, then one random file per map per round
    for i in (1..groups.len()).rev() {
        let j = rng.below(i + 1);
        groups.swap(i, j);
    }
    while chosen.len() < count && groups.iter().any(|g| !g.is_empty()) {
        for g in groups.iter_mut() {
            if chosen.len() >= count {
                break;
            }

            if g.is_empty() {
                continue;
            }

            let i = rng.below(g.len());
            chosen.push(g.swap_remove(i));
        }
    }

    let text: String = chosen
        .iter()
        .map(|p| format!("{}\n", p.display()))
        .collect();
    std::fs::write(out, text).expect("write");

    println!(
        "{} files: {problem_count} problem files, {} sampled across {} maps, from {} demos",
        chosen.len(),
        chosen.len() - problem_count,
        groups.len(),
        files.len()
    );
}
