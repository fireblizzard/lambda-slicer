//! Lambda Slicer diagnostic CLI

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

use lambda_slicer_core::analysis::context::{Kill, KillCause};
use lambda_slicer_core::analysis::motion::{MotionEvidence, MotionState};
use lambda_slicer_core::analysis::weapons::{Actor, IntervalEnd};
use lambda_slicer_core::analysis::{AnalysisReport, analyze_demo};
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::fixtures::{MANIFEST_ENV, check_fixture, load_manifest};
use lambda_slicer_core::library::cache::{
    Cache, FileQuery, FileRow, KillQuery, WindowRow, default_path as library_cache_path,
};
use lambda_slicer_core::library::fingerprint::full_fingerprint;
use lambda_slicer_core::library::scan::{
    Outcome as ScanOutcome, ScanControl, ScanEvent, ScanOptions, ScanProgress, scan,
};
use lambda_slicer_core::protocol::messages::{Message, svc};
use lambda_slicer_core::slice::controls::{Control, run_control};
use lambda_slicer_core::slice::restart::run_restart;
use lambda_slicer_core::state::{
    ActorResolution, DecodeOptions, DecodeStatus, DecodedEvent, Decoder, decode_demo,
};
use lambda_slicer_core::timeline::time::DemoTime;
use lambda_slicer_core::validate::crosscheck::{CrossCheckReport, Outcome, crosscheck_demo};
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
    /// Read the header, directory, and every frame record of a demo
    Inspect {
        path: PathBuf,
        /// Write the JSON report to a file, or `-` for stdout
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Structural validation, exits non-zero when errors are found
    Validate {
        path: PathBuf,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Write an S0 control export for in-game validation. Never overwrites
    Control {
        kind: ControlKind,
        source: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Exclusive end for `truncate-end`, given as seconds, M:SS, or HH:MM:SS.fff
        #[arg(long)]
        end: Option<DemoTime>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Experimental S1 restart, writing `[start, end)` as a demo that plays
    /// on its own. Never overwrites
    Slice {
        source: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Inclusive start, given as seconds, M:SS, or HH:MM:SS.fff
        #[arg(long)]
        start: DemoTime,
        /// Exclusive end, defaulting to the end of the recording
        #[arg(long)]
        end: Option<DemoTime>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Decode network messages and reconstruct entity/client state
    Decode {
        path: PathBuf,
        /// Write the JSON report to a file, or `-` for stdout
        #[arg(long)]
        json: Option<PathBuf>,
        /// List every decoded event (network and standalone demo events). Not
        /// allowed with `--json -`
        #[arg(long)]
        events: bool,
        /// Print each message whose frame time is in [FROM, TO). Not allowed
        /// with `--json -`
        #[arg(long, num_args = 2, value_names = ["FROM", "TO"])]
        trace: Option<Vec<DemoTime>>,
    },
    /// Weapon actions, charge intervals and actors from the recorded events
    Analyze {
        path: PathBuf,
        /// Write the JSON report to a file, or `-` for stdout
        #[arg(long)]
        json: Option<PathBuf>,
        /// List every weapon action with its actor
        #[arg(long)]
        occurrences: bool,
        /// List kills with the killer's and victim's movement, filtered by
        /// the options below
        #[arg(long)]
        kills: bool,
        /// Only kills with this weapon, as the death feed names it (`bolt`,
        /// `grenade`) or as a weapon (`crossbow`, `mp5`)
        #[arg(long, requires = "kills")]
        weapon: Option<String>,
        /// Only kills whose victim was airborne
        #[arg(long, requires = "kills")]
        victim_airborne: bool,
        /// Only kills whose killer was airborne at the shot
        #[arg(long, requires = "kills")]
        killer_airborne: bool,
        /// Only kills where the killer or the victim moved at least this fast,
        /// in units per second
        #[arg(long, requires = "kills")]
        min_speed: Option<f32>,
    },
    /// Cross-check decoded state against client-recorded values. Exit codes are
    /// 0 verified, 1 failed, 2 inconclusive
    Crosscheck {
        path: PathBuf,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Size, modification time, and BLAKE3 hashes of files
    Hash { paths: Vec<PathBuf> },
    /// Check the local fixture manifest (defaults to $LAMBDA_SLICER_FIXTURE_MANIFEST)
    Fixtures {
        #[arg(long)]
        manifest: Option<PathBuf>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Find demos under folders and analyse those missing from the library
    /// cache. Ctrl-C stops and keeps finished results
    Scan {
        #[arg(required = true)]
        roots: Vec<PathBuf>,
        /// Cache database, by default in the platform's cache directory
        #[arg(long)]
        cache: Option<PathBuf>,
        /// Parallel analyses, 1 to 4
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=4))]
        workers: u8,
        /// Analyse files even when their cache entry is current
        #[arg(long)]
        force: bool,
        /// Skip the full-content hash of analysed files
        #[arg(long)]
        no_full_hash: bool,
        /// Print every file's outcome, not only failures
        #[arg(long)]
        verbose: bool,
        /// Write the final counts as JSON to a file, or `-` for stdout
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Search the library cache. With no conditions, lists files
    Query {
        #[arg(long)]
        cache: Option<PathBuf>,
        /// Map name containing this text
        #[arg(long)]
        map: Option<String>,
        /// A player name containing this text
        #[arg(long)]
        player: Option<String>,
        /// Spectating at least this fraction of playback, 0 to 1
        #[arg(long)]
        min_spectating: Option<f64>,
        /// Spectating at most this fraction of playback, 0 to 1
        #[arg(long)]
        max_spectating: Option<f64>,
        /// A weapon with at least one shot, by profile name such as `gauss`
        #[arg(long)]
        weapon_used: Option<String>,
        /// Has a kill with this weapon (death notice string or profile name)
        #[arg(long)]
        kill_weapon: Option<String>,
        /// Has a kill whose victim was airborne
        #[arg(long)]
        victim_airborne: bool,
        /// Has a kill whose killer was airborne at the shot
        #[arg(long)]
        killer_airborne: bool,
        /// Has a kill where the killer or the victim moved at least this
        /// fast horizontally, in units per second
        #[arg(long)]
        min_speed: Option<f32>,
        /// Best action window scoring at least this
        #[arg(long)]
        min_action: Option<f64>,
        /// List the best action windows across the library instead of files
        #[arg(long, conflicts_with_all = ["map", "player", "min_spectating", "max_spectating",
            "weapon_used", "kill_weapon", "victim_airborne", "killer_airborne", "min_speed",
            "min_action"])]
        windows: bool,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        json: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ControlKind {
    /// S0a, exact copy
    ExactCopy,
    /// S0b, container rebuilt from decoded frames
    Roundtrip,
    /// S0c, keep [0, end) and regenerate the terminal record and directory
    TruncateEnd,
}

/// Whether a `--json` target names stdout
fn is_stdout(target: &Path) -> bool {
    target == Path::new("-")
}

fn write_json<T: Serialize>(target: &Path, value: &T) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    if is_stdout(target) {
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
        Command::Slice {
            source,
            output,
            start,
            end,
            json,
        } => {
            let r = run_restart(&source, &output, start, end, limits)?;
            match &json {
                Some(target) => write_json(target, &r)?,
                None => {
                    println!("slice -> {}", r.published.path.display());
                    if let Some(h) = &r.published.source.full_hash {
                        println!("  source blake3 {h}");
                    }
                    println!("  output blake3 {}", r.published.output_hash);
                    println!(
                        "  requested [{}, {}), effective [{}, {}); {} records, {} network frames kept",
                        r.requested_start,
                        r.requested_end.as_deref().unwrap_or("end"),
                        r.effective_start,
                        r.effective_end,
                        r.kept_records,
                        r.kept_network_records
                    );
                    println!(
                        "  bootstrap: {} full updates ({} bytes) for sequences {:?}, start sequence {}",
                        r.bootstrap_sequences.len(),
                        r.bootstrap_bytes,
                        r.bootstrap_sequences,
                        r.start_sequence
                    );
                    println!(
                        "  persistent state: {} messages ({} bytes) replayed, {} expired HUD notices left out",
                        r.persistent_messages, r.persistent_bytes, r.expired_notices
                    );
                    for c in &r.carried {
                        println!("  carried over the start: {c}");
                    }
                    println!(
                        "  time windows: {} inexact, {} outside the window",
                        r.inexact_time_windows, r.clamped_time_windows
                    );
                    for a in &r.approximations {
                        println!("  approximation: {a}");
                    }
                    let v = &r.validation;
                    println!(
                        "  validation: {} kept snapshots compared, {} differences explained by approximations, {} unexplained",
                        v.compared_snapshots, v.approximated_differences, v.other_differences
                    );

                    let name = r
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
        Command::Decode {
            path,
            json,
            events,
            trace,
        } => return decode_cmd(&path, json.as_deref(), events, trace.as_deref()),
        Command::Analyze {
            path,
            json,
            occurrences,
            kills,
            weapon,
            victim_airborne,
            killer_airborne,
            min_speed,
        } => {
            if json.as_deref().is_some_and(is_stdout) && (occurrences || kills) {
                bail!(
                    "--occurrences and --kills print text to stdout, so use them with --json FILE, not --json -"
                );
            }

            let r = analyze_demo(&path, limits);
            match &json {
                Some(target) => write_json(target, &r)?,
                None => print_analysis(&r),
            }
            if occurrences {
                print_occurrences(&r);
            }
            if kills {
                let filter = KillFilter {
                    weapon,
                    victim_airborne,
                    killer_airborne,
                    min_speed,
                };
                print_kills(&r, &filter);
            }

            return Ok(if r.decode == DecodeStatus::Complete {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            });
        }
        Command::Crosscheck { path, json } => {
            let r = crosscheck_demo(&path, Limits::default());
            match &json {
                Some(target) => write_json(target, &r)?,
                None => print_crosscheck(&r),
            }

            return Ok(match r.outcome {
                Outcome::Verified => ExitCode::SUCCESS,
                Outcome::Failed => ExitCode::from(1),
                Outcome::Inconclusive => ExitCode::from(2),
            });
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
        Command::Scan {
            roots,
            cache,
            workers,
            force,
            no_full_hash,
            verbose,
            json,
        } => {
            let options = ScanOptions {
                roots,
                cache: cache_path(cache)?,
                workers: usize::from(workers),
                force,
                full_hash: !no_full_hash,
                limits: Limits::default(),
            };
            return scan_cmd(&options, verbose, json.as_deref());
        }
        Command::Query {
            cache,
            map,
            player,
            min_spectating,
            max_spectating,
            weapon_used,
            kill_weapon,
            victim_airborne,
            killer_airborne,
            min_speed,
            min_action,
            windows,
            limit,
            json,
        } => {
            let path = cache_path(cache)?;
            if !path.exists() {
                bail!("no library cache at {}, run `scan` first", path.display());
            }
            let cache = Cache::open(&path)?;
            if windows {
                let rows = cache.top_windows(limit)?;
                match &json {
                    Some(target) => write_json(target, &rows)?,
                    None => print_windows(&rows),
                }
                return Ok(ExitCode::SUCCESS);
            }

            let query = FileQuery {
                map,
                player,
                min_spectating,
                max_spectating,
                weapon_used,
                kills: KillQuery {
                    weapon: kill_weapon,
                    victim_airborne,
                    killer_airborne,
                    min_speed,
                },
                min_action,
                limit,
            };
            let rows = cache.query(&query)?;
            match &json {
                Some(target) => write_json(target, &rows)?,
                None => {
                    print_files(&rows);
                    let stats = cache.stats()?;
                    println!(
                        "{} shown of {} cached files ({} stale)",
                        rows.len(),
                        stats.files,
                        stats.stale
                    );
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

/// `inspect` fails only when the header is unreadable, while `validate` fails
/// on any structural error
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

/// Printable, escaped, truncated text from recorded bytes. Recorded commands
/// are displayed, never executed
fn text(bytes: &[u8]) -> String {
    let s: String = String::from_utf8_lossy(bytes).escape_debug().collect();
    if s.chars().count() > 80 {
        format!("{}...", s.chars().take(80).collect::<String>())
    } else {
        s
    }
}

fn describe(d: &Decoder, m: &Message) -> String {
    match m {
        Message::Time(bits) => format!("{}", f32::from_bits(*bits)),
        Message::Print(t) | Message::StuffText(t) | Message::CenterPrint(t) => text(t),
        Message::PacketEntities(p) => format!("{} entities", p.declared_count),
        Message::DeltaPacketEntities(p) => format!(
            "{} entities, {} updates, from {}",
            p.declared_count,
            p.updates.len(),
            p.delta_from.unwrap_or(0)
        ),
        Message::ClientData(c) => format!(
            "from {:?}, {} weapon updates",
            c.delta_from,
            c.weapons.len()
        ),
        Message::Event(e) => {
            let names: Vec<String> = e
                .iter()
                .map(|e| {
                    d.event_name(e.index)
                        .map_or_else(|| format!("#{}", e.index), text)
                })
                .collect();
            names.join(", ")
        }
        Message::EventReliable(e) => d
            .event_name(e.index)
            .map_or_else(|| format!("#{}", e.index), text),
        Message::Sound(s) => format!(
            "sound {} on entity {} channel {}",
            s.sound_index, s.entity, s.channel
        ),
        Message::User { id, body } => format!(
            "{} ({} bytes)",
            d.user_message(*id)
                .map_or_else(|| format!("#{id}"), |u| u.name_lossy()),
            body.len()
        ),
        Message::UpdateUserInfo {
            slot,
            user_id,
            info,
            ..
        } => format!("slot {slot} user {user_id} {}", text(info)),
        Message::TempEntity { kind, .. } => format!("type {kind}"),
        Message::DeltaDescription(s) => {
            format!("{} ({} fields)", s.name_lossy(), s.fields.len())
        }
        Message::SetView(e) => format!("entity {e}"),
        Message::ServerInfo(i) => format!(
            "max clients {}, player {}, map {}",
            i.max_clients,
            i.player_num,
            text(&i.map_name)
        ),
        _ => String::new(),
    }
}

/// Actor of an event, named as the player was called when the event
/// happened, not at the end of the recording
fn actor_text(d: &Decoder, e: &DecodedEvent) -> String {
    let named = |entity: i64| match &e.player {
        Some(p) => format!(
            "entity {entity} ({}, slot {} #{})",
            p.name, p.slot, p.incarnation
        ),
        None if entity >= 1 && entity <= i64::from(d.max_clients) => {
            format!("unknown player (entity {entity})")
        }
        None => format!("entity {entity}"),
    };

    match &e.actor {
        ActorResolution::PacketEntity { entity, .. } => named((*entity).into()),
        ActorResolution::Explicit { entity } => named((*entity).into()),
        ActorResolution::OutOfRange {
            packet_index,
            entities,
        } => format!("unresolved (packet index {packet_index} of {entities})"),
        ActorResolution::NoPacketContext { packet_index } => {
            format!("unresolved (packet index {packet_index}, no entity update)")
        }
        ActorResolution::Unknown => "unknown actor".into(),
    }
}

fn decode_cmd(
    path: &Path,
    json: Option<&Path>,
    events: bool,
    trace: Option<&[DemoTime]>,
) -> Result<ExitCode> {
    // The event and trace listings are text on stdout, so they would corrupt
    // a JSON document written there
    if json.is_some_and(is_stdout) && (events || trace.is_some()) {
        bail!(
            "--events and --trace print text to stdout, so use them with --json FILE, not --json -"
        );
    }

    let window = trace.map(|t| (t[0], t[1]));
    let options = DecodeOptions {
        collect_events: events,
        ..Default::default()
    };
    let (report, decoder) = decode_demo(path, &options, |loc, d, m| {
        let Some((from, to)) = window else { return };

        let Some(t) = DemoTime::from_f32(loc.time()) else {
            return;
        };
        if loc.entry == 0 || t < from || t >= to {
            return;
        }

        println!(
            "{t} frame {:>7} {:<24} {}",
            loc.frame_number,
            svc::name(m.opcode),
            describe(d, &m.message)
        );
    });

    if let Some(target) = json {
        write_json(target, &report)?;
    } else {
        println!("{}", report.path);
        println!("  decode {:?}", report.status);
        if let Some(e) = &report.error {
            println!("  error: {e}");
        }

        let s = &report.stats;
        if report.inferred_delta_bases {
            println!(
                "  {} snapshots rest on inferred delta bases, first at {:?} (state not proven)",
                s.inferred_snapshots,
                s.first_inferred.map(|l| l.time())
            );
            println!(
                "  bases: {} strict, {} by inferred offset, {} by count search, {} empty; {} offset changes",
                s.bases_strict,
                s.bases_offset,
                s.bases_count_search,
                s.bases_empty,
                s.sequence_offset_changes
            );
        }
        println!(
            "  {} network frames, {} messages, {} snapshots ({} full, {} delta entity packets), max {} entities",
            s.network_frames,
            s.messages,
            s.snapshots,
            s.full_packets,
            s.delta_packets,
            s.max_entities
        );
        println!(
            "  max clients {}, recorder slot {}, hltv {}",
            report.max_clients, report.player_num, report.hltv
        );
        for p in &report.players {
            println!("  player slot {} user {} {:?}", p.slot, p.user_id, p.name);
        }

        const SHOWN: usize = 30;
        for c in report.name_history.iter().take(SHOWN) {
            let t =
                DemoTime::from_f32(c.location.time()).map_or_else(|| "?".into(), |t| t.to_string());
            println!(
                "  {t} {:?} slot {} #{} user {} {:?}",
                c.kind, c.player.slot, c.player.incarnation, c.player.user_id, c.player.name
            );
        }
        if report.name_history.len() > SHOWN {
            println!(
                "  ... {} more name changes (see --json)",
                report.name_history.len() - SHOWN
            );
        }

        let mut ev: Vec<_> = s.events_by_resource.iter().collect();
        ev.sort_by(|a, b| b.1.cmp(a.1));
        for (name, n) in ev {
            println!("  {n:>8} {name}");
        }

        if s.unresolved_event_actors > 0 {
            println!(
                "  {} events without a resolved actor",
                s.unresolved_event_actors
            );
        }
    }

    if events {
        let event_t = decoder.schema("event_t");
        for e in &decoder.events {
            let t =
                DemoTime::from_f32(e.location.time()).map_or_else(|| "?".into(), |t| t.to_string());
            let name = e
                .resource
                .as_deref()
                .map_or_else(|| format!("#{}", e.index), text);
            let args = match (&e.args, &event_t, &e.demo) {
                (Some(a), Some(s), _) => ["bparam1", "bparam2", "iparam1", "fparam1"]
                    .iter()
                    .filter_map(|f| a.value(s, f).map(|v| format!("{f}={v:?}")))
                    .collect::<Vec<_>>()
                    .join(" "),
                (_, _, Some(r)) => format!(
                    "bparam1={} bparam2={} iparam1={} fparam1={}",
                    r.bparam1(),
                    r.bparam2(),
                    r.iparam1(),
                    r.fparam1()
                ),
                _ => String::new(),
            };

            println!(
                "{t} {:<12} {:<22} {:<32} {args}",
                format!("{:?}", e.source),
                name,
                actor_text(&decoder, e)
            );
        }
    }

    Ok(if report.status == DecodeStatus::Complete {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn actor_name(a: &Actor) -> String {
    match a {
        Actor::Player { identity, .. } => format!("{} (slot {})", identity.name, identity.slot),
        Actor::UnknownPlayer { entity } => format!("Unknown player (entity {entity})"),
        Actor::Entity { entity } => format!("entity {entity}"),
        Actor::Unknown => "Unknown actor".into(),
    }
}

fn print_analysis(r: &AnalysisReport) {
    println!("{}", r.path);
    println!("  decode {:?}, profile {}", r.decode, r.profile);
    if let Some(e) = &r.decode_error {
        println!("  decode stopped: {e}");
    }
    if let Some(p) = &r.recorder {
        println!("  recorder: {} (slot {})", p.name, p.slot);
    }
    println!("  {} recorded events", r.events);

    println!(
        "  {:?}: {} local and {} network events, {} kills, spectating {:.0}% of {:.0} s",
        r.coverage.kind,
        r.coverage.local_events,
        r.coverage.network_events,
        r.coverage.kills,
        r.coverage.spectator_fraction * 100.,
        r.coverage.playback_seconds
    );
    for n in &r.coverage.notes {
        println!("    {n}");
    }

    println!("  weapon        shots  records  intervals  kills");
    for w in &r.weapons {
        println!(
            "  {:<12} {:>6} {:>8} {:>10} {:>6}",
            format!("{:?}", w.weapon),
            w.shots,
            w.records,
            w.intervals,
            w.kills
        );
    }
    let mut causes: Vec<(KillCause, Vec<&str>, usize)> = Vec::new();
    for k in r
        .kills
        .iter()
        .filter(|k| k.cause != KillCause::PlayerWeapon)
    {
        match causes.iter_mut().find(|(c, _, _)| *c == k.cause) {
            Some((_, names, n)) => {
                *n += 1;
                if !names.contains(&k.weapon_name.as_str()) {
                    names.push(&k.weapon_name);
                }
            }
            None => causes.push((k.cause, vec![&k.weapon_name], 1)),
        }
    }
    for (cause, names, n) in &causes {
        println!("  kills, {cause:?}: {n} ({})", names.join(", "));
    }
    println!(
        "  {} weapon switches by the recorder",
        r.weapon_switches.len()
    );
    if !r.action_windows.is_empty() {
        println!("  action windows (score, frags per side, weapons)");
    }
    for w in &r.action_windows {
        let sides: Vec<String> = w.sides.iter().map(|(n, f)| format!("{n} {f}")).collect();
        println!(
            "    {} to {}  {:.2}  {}  [{}]",
            clock(w.start),
            clock(w.end),
            w.score,
            sides.join(", "),
            w.weapons.join(", ")
        );
    }

    println!("  actions (count, with a known player)");
    for a in &r.actions {
        let action = match a.mode {
            Some(m) => format!("{:?} {:?} ({m})", a.weapon, a.action),
            None => format!("{:?} {:?}", a.weapon, a.action),
        };
        println!("    {action:<36} {:>7} {:>7}", a.count, a.attributed);
    }

    let unknown_start = r.intervals.iter().filter(|i| !i.start_known).count();
    let unknown_end = r
        .intervals
        .iter()
        .filter(|i| i.end_reason == IntervalEnd::Unknown)
        .count();
    println!(
        "  {} activity intervals, {unknown_start} with an unseen start, {unknown_end} with an unseen end",
        r.intervals.len()
    );
    for (name, n) in &r.unclassified {
        println!("  not in the profile: {name} ({n} records)");
    }
}

fn clock(t: f32) -> String {
    DemoTime::from_f32(t).map_or_else(|| t.to_string(), |t| t.to_string())
}

struct KillFilter {
    weapon: Option<String>,
    victim_airborne: bool,
    killer_airborne: bool,
    min_speed: Option<f32>,
}

impl KillFilter {
    fn keeps(&self, k: &Kill) -> bool {
        let Some(c) = &k.context else {
            return false;
        };
        if let Some(w) = &self.weapon {
            let named = k.weapon_name.eq_ignore_ascii_case(w);
            let mapped = k.weapon.is_some_and(|x| x.name().eq_ignore_ascii_case(w));
            let shot = c
                .shot
                .as_ref()
                .is_some_and(|s| s.weapon.name().eq_ignore_ascii_case(w));
            if !(named || mapped || shot) {
                return false;
            }
        }
        if self.victim_airborne && c.victim.airborne != Some(true) {
            return false;
        }
        if self.killer_airborne && c.killer.airborne != Some(true) {
            return false;
        }
        if let Some(min) = self.min_speed {
            let fast = |m: &MotionState| m.horizontal_speed.is_some_and(|s| s >= min);
            if !(fast(&c.killer) || fast(&c.victim)) {
                return false;
            }
        }

        true
    }
}

fn motion_text(m: &MotionState) -> String {
    let state = match (m.evidence, m.airborne) {
        (MotionEvidence::NotVisible, _) => return "not visible".into(),
        (_, Some(true)) => "air",
        (_, Some(false)) => "ground",
        (_, None) => "unclear",
    };
    match m.horizontal_speed {
        Some(s) => format!("{state} {s:.0}u/s"),
        None => state.into(),
    }
}

fn print_kills(r: &AnalysisReport, filter: &KillFilter) {
    let name = |p: &Option<lambda_slicer_core::state::PlayerIdentity>, entity: u8| {
        p.as_ref()
            .map_or_else(|| format!("entity {entity}"), |p| p.name.clone())
    };
    let mut shown = 0;
    for k in r.kills.iter().filter(|k| filter.keeps(k)) {
        let Some(c) = &k.context else { continue };
        let flight = c
            .shot
            .as_ref()
            .map(|s| format!(" after {:.2}s", s.flight))
            .unwrap_or_default();
        let distance = c
            .distance
            .map(|d| format!(" at {d:.0}u"))
            .unwrap_or_default();
        println!(
            "{} {:<20} killed {:<20} with {:<12} killer {:<18} victim {:<18}{distance}{flight}",
            clock(k.time),
            name(&k.killer, k.killer_entity),
            name(&k.victim, k.victim_entity),
            k.weapon_name,
            motion_text(&c.killer),
            motion_text(&c.victim)
        );
        shown += 1;
    }
    println!("{shown} of {} kills", r.kills.len());
}

fn print_occurrences(r: &AnalysisReport) {
    for o in &r.occurrences {
        let t = DemoTime::from_f32(o.time).map_or_else(|| o.time.to_string(), |t| t.to_string());
        let mode = o.mode.map(|m| format!(" ({m})")).unwrap_or_default();
        println!(
            "{t} {:<9} {:<16} {:<38} {:?}, {:?}",
            format!("{:?}", o.weapon),
            format!("{:?}{mode}", o.action),
            actor_name(&o.actor),
            o.evidence,
            o.attribution
        );
    }
}

fn print_crosscheck(r: &CrossCheckReport) {
    println!("{}", r.path);
    if let Some(e) = &r.unsupported {
        println!("  not decoded: {e}");
    }
    if let Some(e) = &r.decode_error {
        println!("  decode stopped: {e}");
    }
    if r.inferred_delta_bases {
        println!("  delta bases inferred: state around those packets is not proven");
    }

    println!(
        "  skipped: {} spectator/intermission frames, {} records before the first playback packet, {} level-change frames, {} camera-view origins",
        r.skipped_spectator_frames,
        r.skipped_before_first_packet,
        r.skipped_transition_frames,
        r.skipped_camera_view_records
    );

    for c in &r.exact {
        println!(
            "  {:<12} {:>9}/{:<9} (before {}, after {})  {}",
            format!("{:?}", c.kind),
            c.match_either,
            c.compared,
            c.match_before,
            c.match_after,
            c.name
        );
        for e in &c.examples {
            println!(
                "      t {} frame {}: recorded {} decoded before {} after {}",
                e.time, e.frame_number, e.recorded, e.before, e.after
            );
        }
    }

    for c in &r.distance {
        println!(
            "  {:<12} {:>9}/{:<9} exact, max distance {:.4}  {}",
            format!("{:?}", c.kind),
            c.counts[0],
            c.compared,
            c.max,
            c.name
        );
    }

    let (ok, n) = r.server_copy_agreement();
    println!("  server-copy agreement {ok}/{n}");
    println!("  outcome: {:?}", r.outcome);
    for reason in &r.reasons {
        println!("    {reason}");
    }
}

fn cache_path(given: Option<PathBuf>) -> Result<PathBuf> {
    match given.or_else(library_cache_path) {
        Some(p) => Ok(p),
        None => bail!("no platform cache directory, pass --cache"),
    }
}

fn megabytes(bytes: u64) -> f64 {
    bytes as f64 / 1e6
}

fn progress_line(p: &ScanProgress) -> String {
    let rate = megabytes(p.bytes_read) / p.elapsed_seconds.max(0.001);
    let current = p
        .current
        .first()
        .and_then(|c| Path::new(c).file_name())
        .map(|n| format!(", now {}", n.to_string_lossy()))
        .unwrap_or_default();
    format!(
        "{} found{}, {} done ({} cached, {} analysed), {:.0} MB read at {rate:.1} MB/s{current}",
        p.discovered,
        if p.discovery_done { "" } else { " so far" },
        p.completed,
        p.cache_hits,
        p.analyzed,
        megabytes(p.bytes_read)
    )
}

fn scan_cmd(options: &ScanOptions, verbose: bool, json: Option<&Path>) -> Result<ExitCode> {
    let control = ScanControl::new();
    let handler = control.clone();
    ctrlc::set_handler(move || {
        if handler.is_cancelled() {
            std::process::exit(130);
        }
        eprintln!("\nstopping, Ctrl-C again to quit at once");
        handler.cancel();
    })?;

    eprintln!("cache {}", options.cache.display());
    let mut width = 0;
    let clear = |width: &mut usize| {
        if *width > 0 {
            eprint!("\r{:width$}\r", "", width = *width);
            *width = 0;
        }
    };
    let progress = scan(options, &control, |event| match event {
        ScanEvent::Progress(p) => {
            let line = progress_line(p);
            clear(&mut width);
            eprint!("{line}");
            width = line.chars().count();
        }
        ScanEvent::DiscoveryError { path, error } => {
            clear(&mut width);
            eprintln!("cannot read {}: {error}", path.display());
        }
        ScanEvent::File { path, outcome } => {
            let line = match outcome {
                ScanOutcome::CacheHit if verbose => format!("cached     {}", path.display()),
                ScanOutcome::Analyzed { status } if verbose || status != "complete" => {
                    format!("{status:<10} {}", path.display())
                }
                ScanOutcome::Duplicate { of } if verbose => {
                    format!("duplicate  {} (same file as {of})", path.display())
                }
                ScanOutcome::Unreadable { error } => {
                    format!("unreadable {}: {error}", path.display())
                }
                ScanOutcome::SourceChanged => {
                    format!("changed    {} while it was analysed", path.display())
                }
                ScanOutcome::CacheFailed { error } => {
                    format!("not stored {}: {error}", path.display())
                }
                _ => return,
            };
            clear(&mut width);
            eprintln!("{line}");
        }
    })?;
    clear(&mut width);

    match json {
        Some(target) => write_json(target, &progress)?,
        None => {
            eprintln!("{}", progress_line(&progress));
            eprintln!(
                "{} incomplete decodes, {} duplicates, {} unreadable, {} changed during analysis, \
                 {} not stored, {} unreadable directories, {} links not followed, {:.1} s",
                progress.incomplete,
                progress.duplicates,
                progress.unreadable,
                progress.changed,
                progress.cache_failures,
                progress.directory_errors,
                progress.skipped_links,
                progress.elapsed_seconds
            );
        }
    }
    if progress.cancelled {
        eprintln!("cancelled with {} files left", progress.pending());
        return Ok(ExitCode::from(130));
    }

    Ok(ExitCode::SUCCESS)
}

fn print_files(rows: &[FileRow]) {
    for f in rows {
        let spectating = f
            .spectator_fraction
            .filter(|s| *s > 0.)
            .map(|s| format!("  spectating {:.0}%", s * 100.))
            .unwrap_or_default();
        let action = f
            .best_window
            .map(|s| format!("  action {s:.2}"))
            .unwrap_or_default();
        println!(
            "{}  [{}{}{}]  {} kills{spectating}{action}",
            f.path,
            f.map.as_deref().unwrap_or("?"),
            f.duration
                .map(|d| format!(", {}", clock(d)))
                .unwrap_or_default(),
            if f.status == "complete" {
                String::new()
            } else {
                format!(", {}", f.status)
            },
            f.kills.unwrap_or(0),
        );
        for k in &f.matching_kills {
            let motion = |air: Option<bool>, speed: Option<f32>| {
                let state = match air {
                    Some(true) => "air",
                    Some(false) => "ground",
                    None => "unclear",
                };
                speed.map_or_else(|| state.to_string(), |s| format!("{state} {s:.0}u/s"))
            };
            println!(
                "    {} {} killed {} with {}, killer {}, victim {}{}",
                clock(k.time),
                k.killer.as_deref().unwrap_or("?"),
                k.victim.as_deref().unwrap_or("?"),
                k.weapon,
                motion(k.killer_airborne, k.killer_speed),
                motion(k.victim_airborne, k.victim_speed),
                k.distance
                    .map(|d| format!(" at {d:.0}u"))
                    .unwrap_or_default()
            );
        }
    }
}

fn print_windows(rows: &[WindowRow]) {
    for w in rows {
        let sides: Vec<String> = w.sides.iter().map(|(n, k)| format!("{n} {k}")).collect();
        println!(
            "{:6.2}  {}-{}  {}  [{}]  {}",
            w.score,
            clock(w.start),
            clock(w.end),
            sides.join(" vs "),
            w.weapons.join(", "),
            w.path
        );
    }
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
