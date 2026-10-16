//! SQLite cache of per-file analysis summaries (implementation plan §7.2)
//!
//! One row per demo holds its fingerprint, the versions it was analysed with,
//! its status and summary columns. Players, weapon totals, kills with their
//! context, and action windows sit in child tables. Weapon occurrences are
//! not stored, since a selected file is analysed again on demand. A cache
//! whose schema differs is rebuilt, and an entry analysed with other versions
//! is stale

use std::path::{Path, PathBuf};

use rusqlite::functions::FunctionFlags;
use rusqlite::types::ToSql;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;

use super::date::recorded;
use crate::analysis::weapons::{Actor, PROFILE};
use crate::analysis::{ANALYSIS_SCHEMA, AnalysisReport};
use crate::message::Message;
use crate::state::{DECODE_REPORT_SCHEMA, DecodeStatus};

/// Layout of the tables below. A different value rebuilds the cache
pub const CACHE_SCHEMA: i64 = 3;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache database: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("cache directory: {0}")]
    Io(#[from] std::io::Error),
    #[error("a cancelled analysis is not stored")]
    Cancelled,
}

impl crate::message::Coded for CacheError {
    fn code(&self) -> &'static str {
        match self {
            CacheError::Sql(_) => "cache-database",
            CacheError::Io(_) => "cache-directory",
            CacheError::Cancelled => "cache-cancelled",
        }
    }

    fn args(&self) -> Vec<(&'static str, crate::message::Arg)> {
        match self {
            CacheError::Sql(e) => vec![("detail", e.to_string().into())],
            CacheError::Io(e) => vec![("detail", e.to_string().into())],
            CacheError::Cancelled => Vec::new(),
        }
    }
}

/// Everything an entry was produced with. Entries with another version are
/// stale and analysed again
pub fn analysis_version() -> String {
    format!(
        "cache {CACHE_SCHEMA}, analysis {ANALYSIS_SCHEMA}, profile {PROFILE}, decode {DECODE_REPORT_SCHEMA}, parser {}",
        env!("CARGO_PKG_VERSION")
    )
}

/// `library.sqlite` in the platform's cache directory
pub fn default_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
    }?;

    Some(base.join("lambda-slicer").join("library.sqlite"))
}

/// The path's bytes in the platform's own encoding, UTF-16LE on Windows,
/// so a name that is not valid Unicode survives the display text
pub fn native_bytes(path: &Path) -> Vec<u8> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect()
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    }
}

const TABLES: &str = "
CREATE TABLE files(
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    path_blob BLOB NOT NULL,
    size INTEGER NOT NULL,
    mtime_ns INTEGER,
    file_identity TEXT,
    quick_hash TEXT,
    full_hash TEXT,
    version TEXT NOT NULL,
    status TEXT NOT NULL,
    error TEXT,
    error_code TEXT,
    error_args TEXT,
    map TEXT,
    game_dir TEXT,
    demo_protocol INTEGER,
    network_protocol INTEGER,
    duration REAL,
    recording_kind TEXT,
    max_clients INTEGER,
    recorder TEXT,
    playback REAL,
    spectator_fraction REAL,
    shots INTEGER,
    unknown_shots INTEGER,
    kills INTEGER,
    best_window REAL,
    coverage_json TEXT,
    recorded_at TEXT,
    date_source TEXT,
    score TEXT,
    score_json TEXT,
    scanned_at INTEGER NOT NULL
);
CREATE INDEX files_identity ON files(file_identity);
CREATE INDEX files_recorded ON files(recorded_at);
CREATE TABLE players(
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    segment INTEGER NOT NULL,
    slot INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    incarnation INTEGER NOT NULL,
    name TEXT NOT NULL
);
CREATE INDEX players_file ON players(file_id);
CREATE INDEX players_name ON players(name);
CREATE TABLE weapons(
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    weapon TEXT NOT NULL,
    shots INTEGER NOT NULL,
    kills INTEGER NOT NULL,
    records INTEGER NOT NULL,
    intervals INTEGER NOT NULL
);
CREATE INDEX weapons_weapon ON weapons(weapon, file_id);
CREATE INDEX weapons_file ON weapons(file_id);
CREATE TABLE kills(
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    kill_id INTEGER NOT NULL,
    segment INTEGER NOT NULL,
    time REAL NOT NULL,
    weapon TEXT NOT NULL,
    weapon_profile TEXT,
    shot_weapon TEXT,
    cause TEXT NOT NULL,
    killer TEXT,
    victim TEXT,
    killer_airborne INTEGER,
    victim_airborne INTEGER,
    killer_speed REAL,
    victim_speed REAL,
    distance REAL,
    shot_flight REAL
);
CREATE INDEX kills_file ON kills(file_id, time);
CREATE INDEX kills_weapon ON kills(weapon, file_id);
CREATE TABLE windows(
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL,
    segment INTEGER NOT NULL,
    start REAL NOT NULL,
    end REAL NOT NULL,
    score REAL NOT NULL,
    sides TEXT NOT NULL,
    weapons TEXT NOT NULL
);
CREATE INDEX windows_score ON windows(score);
CREATE INDEX windows_file ON windows(file_id);
";

/// What the cache knows about a path, enough to decide whether it is current
#[derive(Clone, Debug, PartialEq)]
pub struct CachedFile {
    pub id: i64,
    pub size: u64,
    pub mtime_ns: Option<i64>,
    pub file_identity: Option<String>,
    pub version: String,
    pub status: String,
}

/// Header facts read without decoding
#[derive(Clone, Debug, Default)]
pub struct HeaderInfo {
    pub map: String,
    pub game_dir: String,
    pub demo_protocol: i32,
    pub network_protocol: i32,
    /// Sum of the directory's track times, in seconds
    pub duration: f32,
}

/// One file's result to store
#[derive(Clone, Debug)]
pub struct Entry<'a> {
    pub path: &'a Path,
    pub size: u64,
    pub mtime_ns: Option<i64>,
    pub file_identity: Option<String>,
    pub quick_hash: Option<String>,
    pub full_hash: Option<String>,
    pub header: Option<HeaderInfo>,
    /// Absent when the file could not be read or analysis failed outright
    pub report: Option<&'a AnalysisReport>,
    /// Why there is no report, or why it is incomplete
    pub error: Option<Message>,
}

impl Entry<'_> {
    /// The decode status, or `failed` when there is no report
    pub fn status(&self) -> &'static str {
        match self.report.map(|r| r.decode) {
            Some(DecodeStatus::Complete) => "complete",
            Some(DecodeStatus::Partial) => "partial",
            Some(DecodeStatus::Failed) => "failed",
            Some(DecodeStatus::Unsupported) => "unsupported",
            Some(DecodeStatus::Cancelled) => "cancelled",
            None => "failed",
        }
    }
}

/// A serialized enum's tag, such as `player_weapon`
fn tag<T: Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(other) => other.to_string(),
        Err(_) => String::new(),
    }
}

/// Library counts for a quick overview
#[derive(Clone, Debug, Default, Serialize)]
pub struct CacheStats {
    pub files: u64,
    /// Files per status
    pub statuses: Vec<(String, u64)>,
    /// Entries analysed with other versions
    pub stale: u64,
    pub kills: u64,
    pub windows: u64,
}

/// Kill conditions. A file matches when one kill meets all of them
#[derive(Clone, Debug, Default)]
pub struct KillQuery {
    /// The death notice's weapon string (`9mmAR`), its profile weapon
    /// (`mp5`), or the linked shot's weapon, ignoring case
    pub weapon: Option<String>,
    pub victim_airborne: bool,
    pub killer_airborne: bool,
    /// Killer or victim moving at least this fast horizontally, in units
    /// per second
    pub min_speed: Option<f32>,
}

impl KillQuery {
    pub fn is_empty(&self) -> bool {
        self.weapon.is_none()
            && !self.victim_airborne
            && !self.killer_airborne
            && self.min_speed.is_none()
    }
}

/// Case-insensitive form for searching. Unicode lowercase, with the Turkish
/// dotless and dotted i folded to plain `i` in every language, so `I`, `ı`,
/// `İ` and `i` all match each other
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter(|c| *c != '\u{307}')
        .map(|c| if c == 'ı' { 'i' } else { c })
        .collect()
}

/// Library list order
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Path,
    Map,
    Duration,
    Kills,
    Action,
    Spectating,
    Scanned,
    Gauss,
    Date,
}

impl Sort {
    fn column(self) -> &'static str {
        match self {
            Sort::Path => "f.path",
            Sort::Map => "f.map",
            Sort::Duration => "f.duration",
            Sort::Kills => "f.kills",
            Sort::Action => "f.best_window",
            Sort::Spectating => "f.spectator_fraction",
            Sort::Scanned => "f.scanned_at",
            Sort::Gauss => "gauss",
            Sort::Date => "f.recorded_at",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FileQuery {
    /// Text anywhere in the path, ignoring case as [`fold`] does
    pub text: Option<String>,
    /// Text in the map name, ignoring case
    pub map: Option<String>,
    /// Text in any player's name, ignoring case
    pub player: Option<String>,
    /// Shortest duration in seconds
    pub min_duration: Option<f32>,
    /// Recording dates as `YYYY-MM-DD`, inclusive
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    /// Spectating fraction bounds, inclusive
    pub min_spectating: Option<f64>,
    pub max_spectating: Option<f64>,
    /// A weapon with at least one shot, by its profile name
    pub weapon_used: Option<String>,
    pub kills: KillQuery,
    /// Lowest best action window score
    pub min_action: Option<f64>,
    pub sort: Sort,
    pub descending: bool,
    /// Rows to skip and rows to return, for paging
    pub offset: usize,
    pub limit: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct KillRow {
    pub time: f32,
    pub weapon: String,
    pub cause: String,
    pub killer: Option<String>,
    pub victim: Option<String>,
    pub killer_airborne: Option<bool>,
    pub victim_airborne: Option<bool>,
    pub killer_speed: Option<f32>,
    pub victim_speed: Option<f32>,
    pub distance: Option<f32>,
}

/// A message read back from the cache, with its code and arguments
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StoredMessage {
    pub code: String,
    pub args: serde_json::Value,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileRow {
    pub id: i64,
    pub path: String,
    pub status: String,
    /// Why the file is not complete, if it is not
    pub error: Option<StoredMessage>,
    pub map: Option<String>,
    pub duration: Option<f32>,
    pub recording_kind: Option<String>,
    pub recorder: Option<String>,
    /// `YYYY-MM-DD HH:MM:SS` local time, and `file_name` or `modified` as
    /// its source
    pub recorded_at: Option<String>,
    pub date_source: Option<String>,
    /// The final score, such as `88 – 74`
    pub score: Option<String>,
    pub spectator_fraction: Option<f64>,
    pub shots: Option<i64>,
    /// Gauss shots, the weapon the library highlights
    pub gauss: Option<i64>,
    pub kills: Option<i64>,
    pub best_window: Option<f64>,
    /// The kills that matched the kill conditions, when there were any
    pub matching_kills: Vec<KillRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WindowRow {
    pub path: String,
    pub segment: u32,
    pub start: f32,
    pub end: f32,
    pub score: f64,
    pub sides: Vec<(String, usize)>,
    pub weapons: Vec<String>,
}

pub struct Cache {
    conn: Connection,
}

impl Cache {
    /// Open or create the cache at `path`, rebuilding it when its schema
    /// differs
    pub fn open(path: &Path) -> Result<Self, CacheError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        Self::setup(conn)
    }

    /// An empty cache in memory, for tests
    pub fn in_memory() -> Result<Self, CacheError> {
        Self::setup(Connection::open_in_memory()?)
    }

    fn setup(mut conn: Connection) -> Result<Self, CacheError> {
        conn.busy_timeout(std::time::Duration::from_secs(30))?;
        // Switching the journal mode does not wait for other connections,
        // so retry while another one opening the same cache holds it
        let started = std::time::Instant::now();
        loop {
            match conn.pragma_update(None, "journal_mode", "WAL") {
                Err(rusqlite::Error::SqliteFailure(e, _))
                    if e.code == rusqlite::ErrorCode::DatabaseBusy
                        && started.elapsed() < std::time::Duration::from_secs(30) =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                result => break result?,
            }
        }
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.create_scalar_function(
            "fold",
            1,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
            |ctx| Ok(ctx.get::<Option<String>>(0)?.map(|t| fold(&t))),
        )?;

        // Under a write lock, so connections opening a new cache at once
        // build it once
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let schema: i64 = tx.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if schema != CACHE_SCHEMA {
            tx.execute_batch(
                "DROP TABLE IF EXISTS windows;
                 DROP TABLE IF EXISTS kills;
                 DROP TABLE IF EXISTS weapons;
                 DROP TABLE IF EXISTS players;
                 DROP TABLE IF EXISTS files;",
            )?;
            tx.execute_batch(TABLES)?;
            tx.pragma_update(None, "user_version", CACHE_SCHEMA)?;
        }
        tx.commit()?;

        Ok(Self { conn })
    }

    pub fn lookup(&self, path: &Path) -> Result<Option<CachedFile>, CacheError> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, size, mtime_ns, file_identity, version, status FROM files WHERE path = ?1",
                [path.display().to_string()],
                |r| {
                    Ok(CachedFile {
                        id: r.get(0)?,
                        size: r.get::<_, i64>(1)? as u64,
                        mtime_ns: r.get(2)?,
                        file_identity: r.get(3)?,
                        version: r.get(4)?,
                        status: r.get(5)?,
                    })
                },
            )
            .optional()?)
    }

    /// Replace the path's entry in one transaction. A cancelled analysis is
    /// refused, so it never replaces an earlier result
    pub fn store(&mut self, e: &Entry) -> Result<i64, CacheError> {
        if e.report
            .is_some_and(|r| r.decode == DecodeStatus::Cancelled)
        {
            return Err(CacheError::Cancelled);
        }

        let display = e.path.display().to_string();
        let r = e.report;
        let h = e.header.as_ref();
        let error = e
            .error
            .clone()
            .or_else(|| r.and_then(|r| r.decode_error.clone()));
        let error_args = error
            .as_ref()
            .map(|m| serde_json::to_string(&m.args).unwrap_or_default());
        let shots = r.map(|r| r.weapons.iter().map(|w| w.shots).sum::<usize>() as i64);
        let unknown_shots = r.map(|r| {
            r.occurrences
                .iter()
                .filter(|o| o.shot && !matches!(o.actor, Actor::Player { .. }))
                .count() as i64
        });
        let recorded = recorded(e.path, e.mtime_ns);
        let scanned_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);

        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM files WHERE path = ?1", [&display])?;
        tx.execute(
            "INSERT INTO files(path, path_blob, size, mtime_ns, file_identity, quick_hash,
                full_hash, version, status, error, error_code, error_args, map, game_dir,
                demo_protocol, network_protocol, duration, recording_kind, max_clients, recorder,
                playback, spectator_fraction, shots, unknown_shots, kills, best_window,
                coverage_json, recorded_at, date_source, score, score_json, scanned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31,
                ?32)",
            params![
                display,
                native_bytes(e.path),
                e.size as i64,
                e.mtime_ns,
                e.file_identity,
                e.quick_hash,
                e.full_hash,
                analysis_version(),
                e.status(),
                error.as_ref().map(|m| m.text.clone()),
                error.as_ref().map(|m| m.code),
                error_args,
                h.map(|h| h.map.clone()),
                h.map(|h| h.game_dir.clone()),
                h.map(|h| h.demo_protocol),
                h.map(|h| h.network_protocol),
                h.map(|h| h.duration),
                r.map(|r| tag(&r.coverage.kind)),
                r.map(|r| r.max_clients),
                r.and_then(|r| r.recorder.as_ref().map(|p| p.name.clone())),
                r.map(|r| r.coverage.playback_seconds),
                r.map(|r| r.coverage.spectator_fraction),
                shots,
                unknown_shots,
                r.map(|r| r.kills.len() as i64),
                r.and_then(|r| r.action_windows.first().map(|w| w.score)),
                r.map(|r| serde_json::to_string(&r.coverage).unwrap_or_default()),
                recorded.as_ref().map(|r| r.0.clone()),
                recorded.as_ref().map(|r| r.1.name()),
                r.map(|r| r.scoreboard.summary(3)).filter(|s| !s.is_empty()),
                r.map(|r| serde_json::to_string(&r.scoreboard).unwrap_or_default()),
                scanned_at,
            ],
        )?;
        let id = tx.last_insert_rowid();

        if let Some(r) = r {
            let mut players = tx.prepare(
                "INSERT INTO players(file_id, segment, slot, user_id, incarnation, name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for p in &r.players {
                players.execute(params![
                    id,
                    p.segment,
                    p.slot,
                    p.user_id,
                    p.incarnation,
                    p.name
                ])?;
            }

            let mut weapons = tx.prepare(
                "INSERT INTO weapons(file_id, weapon, shots, kills, records, intervals)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for w in &r.weapons {
                weapons.execute(params![
                    id,
                    w.weapon.name(),
                    w.shots as i64,
                    w.kills as i64,
                    w.records as i64,
                    w.intervals as i64
                ])?;
            }

            let mut kills = tx.prepare(
                "INSERT INTO kills(file_id, kill_id, segment, time, weapon, weapon_profile,
                    shot_weapon, cause, killer, victim, killer_airborne, victim_airborne,
                    killer_speed, victim_speed, distance, shot_flight)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )?;
            for (n, k) in r.kills.iter().enumerate() {
                let c = k.context.as_ref();
                kills.execute(params![
                    id,
                    n as i64,
                    k.segment,
                    k.time,
                    k.weapon_name,
                    k.weapon.map(|w| w.name()),
                    c.and_then(|c| c.shot.as_ref().map(|s| s.weapon.name())),
                    tag(&k.cause),
                    k.killer.as_ref().map(|p| p.name.clone()),
                    k.victim.as_ref().map(|p| p.name.clone()),
                    c.and_then(|c| c.killer.airborne),
                    c.and_then(|c| c.victim.airborne),
                    c.and_then(|c| c.killer.horizontal_speed),
                    c.and_then(|c| c.victim.horizontal_speed),
                    c.and_then(|c| c.distance),
                    c.and_then(|c| c.shot.as_ref().map(|s| s.flight)),
                ])?;
            }

            let mut windows = tx.prepare(
                "INSERT INTO windows(file_id, rank, segment, start, end, score, sides, weapons)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for (n, w) in r.action_windows.iter().enumerate() {
                windows.execute(params![
                    id,
                    n as i64,
                    w.segment,
                    w.start,
                    w.end,
                    w.score,
                    serde_json::to_string(&w.sides).unwrap_or_default(),
                    serde_json::to_string(&w.weapons).unwrap_or_default(),
                ])?;
            }
        }
        tx.commit()?;

        Ok(id)
    }

    pub fn stats(&self) -> Result<CacheStats, CacheError> {
        let count = |sql: &str| -> Result<u64, CacheError> {
            Ok(self.conn.query_row(sql, [], |r| r.get::<_, i64>(0))? as u64)
        };
        let mut statuses = Vec::new();
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM files GROUP BY status ORDER BY status")?;
        for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
            let (s, n) = row?;
            statuses.push((s, n as u64));
        }

        Ok(CacheStats {
            files: count("SELECT COUNT(*) FROM files")?,
            statuses,
            stale: self.conn.query_row(
                "SELECT COUNT(*) FROM files WHERE version != ?1",
                [analysis_version()],
                |r| r.get::<_, i64>(0),
            )? as u64,
            kills: count("SELECT COUNT(*) FROM kills")?,
            windows: count("SELECT COUNT(*) FROM windows")?,
        })
    }

    /// Kill conditions as SQL over the `kills` table aliased `k`
    fn kill_conditions(q: &KillQuery, args: &mut Vec<Box<dyn ToSql>>) -> String {
        let mut c = vec!["1".to_string()];
        if let Some(w) = &q.weapon {
            args.push(Box::new(w.clone()));
            c.push(format!(
                "(k.weapon = ?{0} COLLATE NOCASE OR k.weapon_profile = ?{0} COLLATE NOCASE
                  OR k.shot_weapon = ?{0} COLLATE NOCASE)",
                args.len()
            ));
        }
        if q.victim_airborne {
            c.push("k.victim_airborne = 1".into());
        }
        if q.killer_airborne {
            c.push("k.killer_airborne = 1".into());
        }
        if let Some(s) = q.min_speed {
            args.push(Box::new(s));
            c.push(format!(
                "(k.killer_speed >= ?{0} OR k.victim_speed >= ?{0})",
                args.len()
            ));
        }
        c.join(" AND ")
    }

    /// The WHERE clause for `q` over `files` aliased `f`, with its arguments
    fn conditions(q: &FileQuery) -> (String, Vec<Box<dyn ToSql>>) {
        let mut args: Vec<Box<dyn ToSql>> = vec![Box::new(analysis_version())];
        let mut c = vec!["f.version = ?1".to_string()];
        if let Some(t) = q.text.as_deref().filter(|t| !t.is_empty()) {
            args.push(Box::new(fold(t)));
            c.push(format!("instr(fold(f.path), ?{}) > 0", args.len()));
        }
        if let Some(m) = q.map.as_deref().filter(|m| !m.is_empty()) {
            args.push(Box::new(fold(m)));
            c.push(format!("instr(fold(f.map), ?{}) > 0", args.len()));
        }
        if let Some(p) = q.player.as_deref().filter(|p| !p.is_empty()) {
            args.push(Box::new(fold(p)));
            c.push(format!(
                "EXISTS (SELECT 1 FROM players p WHERE p.file_id = f.id
                  AND instr(fold(p.name), ?{}) > 0)",
                args.len()
            ));
        }
        if let Some(d) = q.min_duration {
            args.push(Box::new(d));
            c.push(format!("f.duration >= ?{}", args.len()));
        }
        if let Some(d) = &q.from_date {
            args.push(Box::new(d.clone()));
            c.push(format!("substr(f.recorded_at, 1, 10) >= ?{}", args.len()));
        }
        if let Some(d) = &q.to_date {
            args.push(Box::new(d.clone()));
            c.push(format!("substr(f.recorded_at, 1, 10) <= ?{}", args.len()));
        }
        if let Some(s) = q.min_spectating {
            args.push(Box::new(s));
            c.push(format!("f.spectator_fraction >= ?{}", args.len()));
        }
        if let Some(s) = q.max_spectating {
            args.push(Box::new(s));
            c.push(format!("f.spectator_fraction <= ?{}", args.len()));
        }
        if let Some(w) = &q.weapon_used {
            args.push(Box::new(w.clone()));
            c.push(format!(
                "EXISTS (SELECT 1 FROM weapons w WHERE w.file_id = f.id
                  AND w.weapon = ?{} COLLATE NOCASE AND w.shots > 0)",
                args.len()
            ));
        }
        if let Some(s) = q.min_action {
            args.push(Box::new(s));
            c.push(format!("f.best_window >= ?{}", args.len()));
        }
        if !q.kills.is_empty() {
            let kill_sql = Self::kill_conditions(&q.kills, &mut args);
            c.push(format!(
                "EXISTS (SELECT 1 FROM kills k WHERE k.file_id = f.id AND {kill_sql})"
            ));
        }
        (c.join(" AND "), args)
    }

    /// How many current files match `q`, ignoring its paging
    pub fn count(&self, q: &FileQuery) -> Result<u64, CacheError> {
        let (conditions, args) = Self::conditions(q);
        let sql = format!("SELECT COUNT(*) FROM files f WHERE {conditions}");
        let refs: Vec<&dyn ToSql> = args.iter().map(|a| a.as_ref()).collect();
        Ok(self
            .conn
            .query_row(&sql, refs.as_slice(), |r| r.get::<_, i64>(0))? as u64)
    }

    /// One page of current files matching every condition, in `q`'s order
    /// with the path breaking ties
    pub fn query(&self, q: &FileQuery) -> Result<Vec<FileRow>, CacheError> {
        let (conditions, mut args) = Self::conditions(q);
        args.push(Box::new(q.limit.max(1) as i64));
        let limit = args.len();
        args.push(Box::new(q.offset as i64));
        let offset = args.len();
        let direction = if q.descending { "DESC" } else { "ASC" };

        let sql = format!(
            "SELECT f.id, f.path, f.status, f.map, f.duration, f.recording_kind,
                f.spectator_fraction, f.kills, f.best_window, f.error_code, f.error_args, f.error,
                f.recorder, f.shots,
                (SELECT w.shots FROM weapons w WHERE w.file_id = f.id AND w.weapon = 'gauss')
                    AS gauss,
                f.recorded_at, f.date_source, f.score
             FROM files f WHERE {conditions}
             ORDER BY {} {direction} NULLS LAST, f.path
             LIMIT ?{limit} OFFSET ?{offset}",
            q.sort.column()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let refs: Vec<&dyn ToSql> = args.iter().map(|a| a.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), |r| {
            Ok(FileRow {
                id: r.get(0)?,
                path: r.get(1)?,
                status: r.get(2)?,
                map: r.get(3)?,
                duration: r.get(4)?,
                recording_kind: r.get(5)?,
                recorder: r.get(12)?,
                recorded_at: r.get(15)?,
                date_source: r.get(16)?,
                score: r.get(17)?,
                spectator_fraction: r.get(6)?,
                shots: r.get(13)?,
                gauss: r.get(14)?,
                kills: r.get(7)?,
                best_window: r.get(8)?,
                error: match (
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, Option<String>>(11)?,
                ) {
                    (Some(code), Some(text)) => Some(StoredMessage {
                        code,
                        args: r
                            .get::<_, Option<String>>(10)?
                            .and_then(|a| serde_json::from_str(&a).ok())
                            .unwrap_or_default(),
                        text,
                    }),
                    _ => None,
                },
                matching_kills: Vec::new(),
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            let mut file = row?;
            if !q.kills.is_empty() {
                file.matching_kills = self.kills_of(file.id, &q.kills)?;
            }
            out.push(file);
        }

        Ok(out)
    }

    fn kills_of(&self, file_id: i64, q: &KillQuery) -> Result<Vec<KillRow>, CacheError> {
        let mut args: Vec<Box<dyn ToSql>> = vec![Box::new(file_id)];
        let conditions = Self::kill_conditions(q, &mut args);
        let sql = format!(
            "SELECT time, weapon, cause, killer, victim, killer_airborne, victim_airborne,
                killer_speed, victim_speed, distance
             FROM kills k WHERE k.file_id = ?1 AND {conditions} ORDER BY kill_id"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let refs: Vec<&dyn ToSql> = args.iter().map(|a| a.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), |r| {
            Ok(KillRow {
                time: r.get(0)?,
                weapon: r.get(1)?,
                cause: r.get(2)?,
                killer: r.get(3)?,
                victim: r.get(4)?,
                killer_airborne: r.get(5)?,
                victim_airborne: r.get(6)?,
                killer_speed: r.get(7)?,
                victim_speed: r.get(8)?,
                distance: r.get(9)?,
            })
        })?;

        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The best action windows across current files, highest score first
    pub fn top_windows(&self, limit: usize) -> Result<Vec<WindowRow>, CacheError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, w.segment, w.start, w.end, w.score, w.sides, w.weapons
             FROM windows w JOIN files f ON f.id = w.file_id
             WHERE f.version = ?1 ORDER BY w.score DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![analysis_version(), limit as i64], |r| {
            let sides: String = r.get(5)?;
            let weapons: String = r.get(6)?;
            Ok(WindowRow {
                path: r.get(0)?,
                segment: r.get(1)?,
                start: r.get(2)?,
                end: r.get(3)?,
                score: r.get(4)?,
                sides: serde_json::from_str(&sides).unwrap_or_default(),
                weapons: serde_json::from_str(&weapons).unwrap_or_default(),
            })
        })?;

        Ok(rows.collect::<Result<_, _>>()?)
    }
}
