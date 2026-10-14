//! Lambda Slicer diagnostic CLI.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use lambda_slicer_core::container::Limits;
use lambda_slicer_core::fixtures::{MANIFEST_ENV, check_fixture, load_manifest};
use lambda_slicer_core::library::fingerprint::full_fingerprint;
use lambda_slicer_core::slice::controls::{Control, run_control};
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_core::validate::structural::{InspectReport, Severity, inspect};

#[derive(Parser)]
#[command(
    name = "lambda-slicer",
    version,
    about = "GoldSrc demo inspection and slicing"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read the header, directory, and every frame record of a demo.
    Inspect {
        path: PathBuf,
        /// Write the JSON report to a file, or `-` for stdout.
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Structural validation; exits non-zero when errors are found.
    Validate {
        path: PathBuf,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Write an S0 control export for in-game validation. Never overwrites.
    Control {
        kind: ControlKind,
        source: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Exclusive end for `truncate-end`: seconds, M:SS, or HH:MM:SS.fff.
        #[arg(long)]
        end: Option<DemoTime>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Size, modification time, and BLAKE3 hashes of files.
    Hash { paths: Vec<PathBuf> },
    /// Check the local fixture manifest (default: $LAMBDA_SLICER_FIXTURE_MANIFEST).
    Fixtures {
        #[arg(long)]
        manifest: Option<PathBuf>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ControlKind {
    /// S0a: exact copy.
    ExactCopy,
    /// S0b: container rebuilt from decoded frames.
    Roundtrip,
    /// S0c: keep [0, end) and regenerate the terminal record and directory.
    TruncateEnd,
}

fn write_json<T: Serialize>(target: &Path, value: &T) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    if target == Path::new("-") {
        let mut out = std::io::stdout().lock();
        writeln!(out, "{text}")?;
    } else {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .with_context(|| {
                format!(
                    "creating {} (existing files are never overwritten)",
                    target.display()
                )
            })?;
        writeln!(f, "{text}")?;
    }
    Ok(())
}

fn fmt_time(t: Option<f32>) -> String {
    t.and_then(DemoTime::from_f32)
        .map_or_else(|| "-".into(), |t| t.to_string())
}

fn print_inspect(r: &InspectReport) {
    println!("{}", r.path);
    println!("  status {:?}, {} bytes", r.status, r.size);
    if let Some(h) = &r.header {
        println!(
            "  protocol {}/{}  game {:?}  map {:?}  checksum {:#010x}",
            h.demo_protocol, h.network_protocol, h.game_dir, h.map_name, h.map_checksum
        );
    }
    for e in &r.entries {
        println!(
            "  entry {} {:?} type {}: offset {} length {}  directory frames {} time {}",
            e.index, e.description, e.entry_type, e.offset, e.length, e.frame_count, e.track_time
        );
        println!(
            "    {} records ({} network, {} event, {} sound), time {} .. {}, frame numbers {:?}..{:?}, {} terminal",
            e.records,
            e.network_records,
            e.counts.event,
            e.counts.sound,
            fmt_time(e.first_time),
            fmt_time(e.last_time),
            e.min_frame_number.unwrap_or_default(),
            e.max_frame_number.unwrap_or_default(),
            e.terminal_records,
        );
    }
    for f in &r.findings {
        let entry = f.entry.map_or(String::new(), |e| format!(" [entry {e}]"));
        println!("  {:?} {}{}: {}", f.severity, f.code, entry, f.message);
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let limits = Limits::default();
    match cli.command {
        Command::Inspect { path, json } => return inspect_cmd(&path, json.as_deref(), false),
        Command::Validate { path, json } => return inspect_cmd(&path, json.as_deref(), true),
        Command::Control {
            kind,
            source,
            output,
            end,
            json,
        } => {
            let control = match (kind, end) {
                (ControlKind::ExactCopy, None) => Control::ExactCopy,
                (ControlKind::Roundtrip, None) => Control::Roundtrip,
                (ControlKind::TruncateEnd, Some(end)) => Control::TruncateEnd(end),
                (ControlKind::TruncateEnd, None) => bail!("truncate-end needs --end"),
                (_, Some(_)) => bail!("--end applies only to truncate-end"),
            };
            let report = run_control(&source, &output, control, limits)?;
            match &json {
                Some(target) => write_json(target, &report)?,
                None => {
                    println!("{} -> {}", report.control, report.published.path.display());
                    if let Some(h) = &report.published.source.full_hash {
                        println!("  source blake3 {h}");
                    }
                    println!("  output blake3 {}", report.published.output_hash);
                    if let Some(same) = report.byte_identical {
                        println!("  byte-identical to source: {same}");
                    }
                    if let Some(c) = &report.cut {
                        println!(
                            "  requested [0, {}), effective [0, {}); {} records, {} network frames kept",
                            c.requested_end,
                            c.effective_end,
                            c.retained_records,
                            c.retained_network_records
                        );
                    }
                    let name = report
                        .published
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    println!(
                        "  in-game: copy into the mod directory, then run: playdemo {}",
                        name.trim_end_matches(".dem")
                    );
                }
            }
        }
        Command::Hash { paths } => {
            for p in paths {
                let fp = full_fingerprint(&p).with_context(|| p.display().to_string())?;
                println!(
                    "{}  {} bytes  blake3 {}",
                    p.display(),
                    fp.size,
                    fp.full_hash.unwrap_or_default()
                );
            }
        }
        Command::Fixtures { manifest, json } => {
            let path = match manifest.or_else(|| std::env::var_os(MANIFEST_ENV).map(PathBuf::from))
            {
                Some(p) => p,
                None => bail!("pass --manifest or set {MANIFEST_ENV}"),
            };
            let m = load_manifest(&path).map_err(anyhow::Error::msg)?;
            let results: Vec<_> = m.fixtures.iter().map(check_fixture).collect();
            let all_passed = results.iter().all(|r| r.passed());
            match &json {
                Some(target) => write_json(target, &results)?,
                None => {
                    for r in &results {
                        let state = if r.passed() { "ok  " } else { "FAIL" };
                        println!("{state} {} {}", r.role, r.path.display());
                        if let Some(h) = &r.blake3 {
                            println!("     blake3 {h}");
                        }
                        for mm in &r.mismatches {
                            println!("     {mm}");
                        }
                        if let Some(rep) = &r.inspect {
                            for f in rep
                                .findings
                                .iter()
                                .filter(|f| f.severity >= Severity::Warning)
                            {
                                println!("     {:?} {}: {}", f.severity, f.code, f.message);
                            }
                        }
                    }
                }
            }
            if !all_passed {
                return Ok(ExitCode::FAILURE);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `inspect` fails only when the header is unreadable; `validate` fails on
/// any structural error.
fn inspect_cmd(path: &Path, json: Option<&Path>, strict: bool) -> Result<ExitCode> {
    let report = inspect(path, Limits::default());
    match json {
        Some(target) => write_json(target, &report)?,
        None => print_inspect(&report),
    }
    let failed = if strict {
        report.has_errors()
    } else {
        report.header.is_none()
    };
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
