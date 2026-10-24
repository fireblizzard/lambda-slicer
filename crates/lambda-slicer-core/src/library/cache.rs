//! SQLite cache of per-file analysis summaries
//!
//! One row per demo holds its fingerprint, the versions it was analysed with,
//! its status and summary columns. Players, weapon totals, kills with their
//! context, action windows and matches sit in child tables. Weapon occurrences are
//! not stored, since a selected file is analysed again on demand. A cache
//! whose schema differs is rebuilt, and an entry analysed with other versions
//! is stale

use std::path::{Path, PathBuf};

use rusqlite::functions::FunctionFlags;
use rusqlite::types::ToSql;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;

use super::date::recorded;
use crate::analysis::context::{Scoreboard, every_map, map_list, short_map_name};
use crate::analysis::parts::PartStatus;
use crate::analysis::weapons::{Actor, PROFILE};
use crate::analysis::{ANALYSIS_SCHEMA, AnalysisReport};
use crate::message::Message;
use crate::state::{DECODE_REPORT_SCHEMA, DecodeStatus};

/// Layout of the tables below. A different value rebuilds the cache
pub const CACHE_SCHEMA: i64 = 10;

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

/// The path [`native_bytes`] gave, `None` for bytes no path on this
/// platform encodes as, such as an odd length on Windows
pub fn path_from_native(bytes: &[u8]) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let (pairs, rest) = bytes.as_chunks::<2>();
        if !rest.is_empty() {
            return None;
        }
        let wide: Vec<u16> = pairs.iter().map(|b| u16::from_le_bytes(*b)).collect();
        Some(PathBuf::from(std::ffi::OsString::from_wide(&wide)))
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
    }
}

/// Every table, the ones that refer to files first
const DROP_TABLES: &str = "
DROP TABLE IF EXISTS matches;
DROP TABLE IF EXISTS windows;
DROP TABLE IF EXISTS kills;
DROP TABLE IF EXISTS weapons;
DROP TABLE IF EXISTS players;
DROP TABLE IF EXISTS files;
";

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
    floors TEXT,
    status TEXT NOT NULL,
    error TEXT,
    error_code TEXT,
    error_args TEXT,
    map TEXT,
    all_maps TEXT,
    game_dir TEXT,
    clip_of TEXT,
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
    headshots INTEGER,
    players INTEGER,
    teams INTEGER,
    accuracy REAL,
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
    shot_flight REAL,
    headshot INTEGER
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
CREATE TABLE matches(
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL,
    segment INTEGER NOT NULL,
    status TEXT NOT NULL,
    opens_at REAL NOT NULL,
    start REAL NOT NULL,
    end REAL NOT NULL,
    length REAL NOT NULL,
    map TEXT NOT NULL,
    players INTEGER NOT NULL,
    teams INTEGER NOT NULL,
    kills INTEGER NOT NULL,
    score TEXT,
    score_json TEXT
);
CREATE INDEX matches_file ON matches(file_id);
";

/// Indexes for the library's usual list and its figures, which leave out
/// files analysed with other versions. They are made for a cache of the
/// same layout too, as they do not change what it holds
const INDEXES: &str = "
CREATE INDEX IF NOT EXISTS files_version_modified ON files(version, mtime_ns);
CREATE INDEX IF NOT EXISTS files_version_action ON files(version, best_window);
";

/// What the cache knows about a path, enough to decide whether it is current
#[derive(Clone, Debug, PartialEq)]
pub struct CachedFile {
    pub id: i64,
    pub size: u64,
    pub mtime_ns: Option<i64>,
    pub file_identity: Option<String>,
    pub version: String,
    /// The game files the floor under players was read from, none without
    pub floors: Option<String>,
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
    /// The demo a clip Lambda Slicer exported came from, by its marked name
    pub clip_of: Option<String>,
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
    /// The game files the floor under players was read from, none without
    pub floors: Option<String>,
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

/// Figures over every current file, whatever a query filters
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LibraryStats {
    pub files: u64,
    /// Their matches and stretches of free play
    pub matches: u64,
    /// The highest best action window score that is a positive number
    pub max_action: Option<f64>,
}

/// Kill conditions. A file matches when one kill meets all of them
#[derive(Clone, Debug, Default)]
pub struct KillQuery {
    /// The death notice's weapon string (`9mmAR`), its profile weapon
    /// (`mp5`), or the linked shot's weapon, ignoring case
    pub weapon: Option<String>,
    pub victim_airborne: bool,
    pub killer_airborne: bool,
    /// The killer or the victim in the air, or both
    pub midair: bool,
    /// Killer or victim moving at least this fast horizontally, in units
    /// per second
    pub min_speed: Option<f32>,
    /// A kill by a hit to the head
    pub headshot: bool,
}

impl KillQuery {
    pub fn is_empty(&self) -> bool {
        self.weapon.is_none()
            && !self.victim_airborne
            && !self.killer_airborne
            && !self.midair
            && self.min_speed.is_none()
            && !self.headshot
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
    Headshots,
    Accuracy,
    /// The frags of the final scoreboard's first side, the highest
    Score,
    /// When the file was last modified, which for an exported clip is
    /// when it was exported
    Modified,
    /// How many players and teams the demo's matches had
    Players,
    Teams,
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
            Sort::Headshots => "f.headshots",
            Sort::Accuracy => "f.accuracy",
            Sort::Modified => "f.mtime_ns",
            Sort::Players => "f.players",
            Sort::Teams => "f.teams",
            // Nothing for a stored score that is not valid JSON, so it sorts
            // with the files that have no score
            Sort::Score => {
                "CASE WHEN json_valid(f.score_json)
                 THEN json_extract(f.score_json, '$.sides[0].frags') END"
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FileQuery {
    /// Text anywhere in the path, ignoring case as [`fold`] does
    pub text: Option<String>,
    /// Text in the map name, ignoring case
    pub map: Option<String>,
    /// Name every map a demo recorded, rather than leaving out those of a
    /// demo of several maps that were barely played
    pub every_map: bool,
    /// Text in any player's name, ignoring case
    pub player: Option<String>,
    /// Shortest duration in seconds, and fewest kills
    pub min_duration: Option<f32>,
    pub min_kills: Option<i64>,
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
    /// Lowest accuracy of the recording player, as a fraction
    pub min_accuracy: Option<f64>,
    /// Only the files at these paths, as [`FileRow::path`] shows them, none
    /// for every file
    pub only_paths: Option<Vec<String>>,
    /// Fewest and most players and teams, inclusive
    pub min_players: Option<i64>,
    pub max_players: Option<i64>,
    pub min_teams: Option<i64>,
    pub max_teams: Option<i64>,
    pub sort: Sort,
    pub descending: bool,
    /// Rows to skip and rows to return, for paging
    pub offset: usize,
    pub limit: usize,
}

/// Matches list order
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MatchSort {
    /// By the demo's date, then in the order played
    #[default]
    Date,
    Path,
    Map,
    Players,
    Teams,
    /// The frags of the first side, the highest
    Score,
    Kills,
    Length,
    Status,
}

impl MatchSort {
    fn column(self) -> &'static str {
        match self {
            MatchSort::Date => "f.recorded_at",
            MatchSort::Path => "f.path",
            MatchSort::Map => "m.map",
            MatchSort::Players => "m.players",
            MatchSort::Teams => "m.teams",
            MatchSort::Kills => "m.kills",
            MatchSort::Length => "m.length",
            MatchSort::Status => "m.status",
            MatchSort::Score => {
                "CASE WHEN json_valid(m.score_json)
                 THEN json_extract(m.score_json, '$.sides[0].frags') END"
            }
        }
    }
}

/// Which matches to list, and how
#[derive(Clone, Debug, Default)]
pub struct MatchQuery {
    /// The demos' conditions. Its map, shortest duration, fewest kills and
    /// players and teams are each match's own, its kill conditions need a
    /// kill in the match, and its paging and order are the matches'
    pub files: FileQuery,
    /// The statuses left out, none for every match
    pub hidden: Vec<PartStatus>,
    pub sort: MatchSort,
}

/// A match, or a stretch of free play, with its demo
#[derive(Clone, Debug, Serialize)]
pub struct MatchRow {
    pub file_id: i64,
    pub path: String,
    pub recorded_at: Option<String>,
    pub date_source: Option<String>,
    /// Its place among the demo's parts, from nought
    pub ordinal: i64,
    pub status: PartStatus,
    /// Where watching it begins, in the demo's playback
    pub opens_at: f32,
    /// How long it was played for, in seconds
    pub length: f32,
    pub map: String,
    pub players: i64,
    pub teams: i64,
    pub kills: i64,
    /// The score at its last kill, such as `88 – 74`
    pub score: Option<String>,
    #[serde(skip)]
    pub scoreboard: Option<Scoreboard>,
    /// The path's own bytes, as for [`FileRow::native`]
    #[serde(skip)]
    pub native: Vec<u8>,
}

impl MatchRow {
    /// The demo this match was recorded in
    pub fn source(&self) -> PathBuf {
        path_from_native(&self.native).unwrap_or_else(|| PathBuf::from(&self.path))
    }
}

/// A kill within the match `m`, by its session and times
const KILL_IN_MATCH: &str = "k.segment = m.segment AND k.time >= m.start AND k.time <= m.end";

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
    pub headshot: Option<bool>,
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
    /// When the file was last modified, `YYYY-MM-DD HH:MM:SS` local time
    pub modified_at: Option<String>,
    /// The demo this file is a clip of, by the name its mark gives
    pub clip_of: Option<String>,
    /// The final score, such as `88 – 74`
    pub score: Option<String>,
    /// The final scoreboard the score was made from, none when the cache
    /// holds none or it cannot be read. Only for display, the command line's
    /// JSON keeps the summary alone
    #[serde(skip)]
    pub scoreboard: Option<Scoreboard>,
    pub spectator_fraction: Option<f64>,
    pub shots: Option<i64>,
    /// Gauss shots, the weapon the library highlights
    pub gauss: Option<i64>,
    pub kills: Option<i64>,
    /// The recording player's kills by a hit to the head
    pub headshots: Option<i64>,
    /// The recording player's accuracy, as a fraction
    pub accuracy: Option<f64>,
    /// How many players and teams the demo's matches had
    pub players: Option<i64>,
    pub teams: Option<i64>,
    pub best_window: Option<f64>,
    /// The kills that matched the kill conditions, when there were any
    pub matching_kills: Vec<KillRow>,
    /// The path's own bytes ([`native_bytes`]), as `path` is display text
    /// and loses names that are not valid Unicode
    #[serde(skip)]
    pub native: Vec<u8>,
}

impl FileRow {
    /// The file this row describes, exactly as the scan found it
    pub fn source(&self) -> PathBuf {
        path_from_native(&self.native).unwrap_or_else(|| PathBuf::from(&self.path))
    }
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
            tx.execute_batch(DROP_TABLES)?;
            tx.execute_batch(TABLES)?;
            tx.pragma_update(None, "user_version", CACHE_SCHEMA)?;
        }
        tx.execute_batch(INDEXES)?;
        tx.commit()?;

        Ok(Self { conn })
    }

    pub fn lookup(&self, path: &Path) -> Result<Option<CachedFile>, CacheError> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, size, mtime_ns, file_identity, version, status, floors FROM files
                 WHERE path = ?1",
                [path.display().to_string()],
                |r| {
                    Ok(CachedFile {
                        id: r.get(0)?,
                        size: r.get::<_, i64>(1)? as u64,
                        mtime_ns: r.get(2)?,
                        file_identity: r.get(3)?,
                        version: r.get(4)?,
                        status: r.get(5)?,
                        floors: r.get(6)?,
                    })
                },
            )
            .optional()?)
    }

    /// Every stored file's id and path, exactly as the scan found it
    pub fn stored_paths(&self) -> Result<Vec<(i64, PathBuf)>, CacheError> {
        let mut stmt = self.conn.prepare("SELECT id, path, path_blob FROM files")?;
        let rows = stmt.query_map([], |r| {
            let path: String = r.get(1)?;
            let native: Vec<u8> = r.get(2)?;
            let path = path_from_native(&native).unwrap_or_else(|| PathBuf::from(path));
            Ok((r.get(0)?, path))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Forget these files and everything stored about them, in one
    /// transaction
    pub fn forget(&mut self, ids: &[i64]) -> Result<(), CacheError> {
        let tx = self.conn.transaction()?;
        for id in ids {
            tx.execute("DELETE FROM files WHERE id = ?1", [id])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Forget every stored file, so the next scan analyses every demo again,
    /// and give the room they took back to the disk
    pub fn clear(&mut self) -> Result<(), CacheError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(DROP_TABLES)?;
        tx.execute_batch(TABLES)?;
        tx.commit()?;

        // Giving the room back only tidies up, so the cleared cache stands
        // when another connection keeps it from happening
        let _ = self.conn.execute_batch("VACUUM");
        Ok(())
    }

    /// Forget the files under `folder`, unless they are also under one of
    /// `kept`, as when a library folder is removed
    pub fn forget_folder(&mut self, folder: &Path, kept: &[PathBuf]) -> Result<(), CacheError> {
        let mut gone = Vec::new();
        for (id, path) in self.stored_paths()? {
            let still_kept = kept.iter().any(|k| path.starts_with(k));
            if path.starts_with(folder) && !still_kept {
                gone.push(id);
            }
        }
        self.forget(&gone)
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

        // Every map played in the demo, by its sessions' kills and lengths, or
        // the header's first one when the analysis found none
        let played: Vec<u32> =
            r.map_or(Vec::new(), |r| r.kills.iter().map(|k| k.segment).collect());
        let maps = match r
            .map(|r| map_list(&r.maps, &played, &r.groups.segment_lengths()))
            .filter(|m| !m.is_empty())
        {
            Some(maps) => Some(maps),
            None => h.map(|h| short_map_name(&h.map)),
        };
        let all_maps = match r.map(|r| every_map(&r.maps)).filter(|m| !m.is_empty()) {
            Some(all) => Some(all),
            None => maps.clone(),
        };
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
                coverage_json, recorded_at, date_source, score, score_json, headshots, accuracy,
                scanned_at, floors, clip_of, all_maps, players, teams)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31,
                ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39)",
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
                maps,
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
                r.and_then(|r| r.recorder_headshots),
                r.and_then(|r| r.recorder_accuracy),
                scanned_at,
                e.floors,
                h.and_then(|h| h.clip_of.clone()),
                all_maps,
                r.map(|r| r.counted.players),
                r.map(|r| r.counted.teams),
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
                    killer_speed, victim_speed, distance, shot_flight, headshot)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                    ?17)",
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
                    k.headshot,
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

            let mut matches = tx.prepare(
                "INSERT INTO matches(file_id, ordinal, segment, status, opens_at, start, end,
                    length, map, players, teams, kills, score, score_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            )?;
            for (n, p) in r.parts.iter().enumerate() {
                let score = Some(p.scoreboard.summary(3)).filter(|s| !s.is_empty());
                matches.execute(params![
                    id,
                    n as i64,
                    p.segment,
                    p.status.key(),
                    p.opens_at,
                    p.start,
                    p.end,
                    p.length(),
                    p.map,
                    p.counted.players,
                    p.counted.teams,
                    p.kills,
                    score,
                    serde_json::to_string(&p.scoreboard).unwrap_or_default(),
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

    /// How many current files there are and their highest action score,
    /// for showing a filtered count against the whole and scaling intensity
    /// the same whatever is filtered
    pub fn library_stats(&self) -> Result<LibraryStats, CacheError> {
        let (files, max_action) = self.conn.query_row(
            "SELECT COUNT(*), MAX(CASE WHEN best_window > 0 THEN best_window END)
             FROM files WHERE version = ?1",
            [analysis_version()],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<f64>>(1)?)),
        )?;
        let matches = self.conn.query_row(
            "SELECT COUNT(*) FROM matches m JOIN files f ON f.id = m.file_id
             WHERE f.version = ?1",
            [analysis_version()],
            |r| r.get::<_, i64>(0),
        )?;

        Ok(LibraryStats {
            files: files as u64,
            matches: matches as u64,
            max_action: max_action.filter(|m| m.is_finite()),
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
        if q.midair {
            c.push("(k.killer_airborne = 1 OR k.victim_airborne = 1)".into());
        }
        if q.headshot {
            c.push("k.headshot = 1".into());
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

    /// The column of `files` aliased `f` that names `q`'s maps
    fn map_column(q: &FileQuery) -> &'static str {
        if q.every_map { "f.all_maps" } else { "f.map" }
    }

    /// The WHERE clause for `q` over `files` aliased `f`, with its arguments
    fn conditions(q: &FileQuery) -> (String, Vec<Box<dyn ToSql>>) {
        Self::conditions_with_kills(q, "1")
    }

    /// [`Self::conditions`], where a kill meeting the kill conditions must
    /// also meet `kill_within`
    fn conditions_with_kills(q: &FileQuery, kill_within: &str) -> (String, Vec<Box<dyn ToSql>>) {
        let mut args: Vec<Box<dyn ToSql>> = vec![Box::new(analysis_version())];
        let mut c = vec!["f.version = ?1".to_string()];
        if let Some(t) = q.text.as_deref().filter(|t| !t.is_empty()) {
            args.push(Box::new(fold(t)));
            c.push(format!("instr(fold(f.path), ?{}) > 0", args.len()));
        }
        if let Some(paths) = &q.only_paths {
            let mut places = Vec::new();
            for path in paths {
                args.push(Box::new(path.clone()));
                places.push(format!("?{}", args.len()));
            }

            // No paths at all lists nothing
            if places.is_empty() {
                c.push("0".to_string());
            } else {
                c.push(format!("f.path IN ({})", places.join(", ")));
            }
        }
        if let Some(m) = q.map.as_deref().filter(|m| !m.is_empty()) {
            args.push(Box::new(fold(m)));
            let column = Self::map_column(q);
            c.push(format!("instr(fold({column}), ?{}) > 0", args.len()));
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
        if let Some(k) = q.min_kills {
            args.push(Box::new(k));
            c.push(format!("f.kills >= ?{}", args.len()));
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
        if let Some(a) = q.min_accuracy {
            args.push(Box::new(a));
            c.push(format!("f.accuracy >= ?{}", args.len()));
        }

        // Players and teams within their bounds
        let bounds = [
            (q.min_players, "f.players >="),
            (q.max_players, "f.players <="),
            (q.min_teams, "f.teams >="),
            (q.max_teams, "f.teams <="),
        ];
        for (bound, test) in bounds {
            if let Some(n) = bound {
                args.push(Box::new(n));
                c.push(format!("{test} ?{}", args.len()));
            }
        }
        if !q.kills.is_empty() {
            let kill_sql = Self::kill_conditions(&q.kills, &mut args);
            c.push(format!(
                "EXISTS (SELECT 1 FROM kills k WHERE k.file_id = f.id AND {kill_within}
                  AND {kill_sql})"
            ));
        }
        (c.join(" AND "), args)
    }

    /// The WHERE clause for `q` over `matches` aliased `m` joined to their
    /// `files` aliased `f`, with its arguments
    fn match_conditions(q: &MatchQuery) -> (String, Vec<Box<dyn ToSql>>) {
        // The demos' own conditions, leaving those of each match
        let mut files = q.files.clone();
        files.map = None;
        files.min_duration = None;
        files.min_kills = None;
        files.min_players = None;
        files.max_players = None;
        files.min_teams = None;
        files.max_teams = None;
        let (conditions, mut args) = Self::conditions_with_kills(&files, KILL_IN_MATCH);
        let mut c = vec![conditions];

        let f = &q.files;
        if let Some(m) = f.map.as_deref().filter(|m| !m.is_empty()) {
            args.push(Box::new(fold(m)));
            c.push(format!("instr(fold(m.map), ?{}) > 0", args.len()));
        }
        let bounds = [
            (f.min_duration.map(f64::from), "m.length >="),
            (f.min_kills.map(|n| n as f64), "m.kills >="),
            (f.min_players.map(|n| n as f64), "m.players >="),
            (f.max_players.map(|n| n as f64), "m.players <="),
            (f.min_teams.map(|n| n as f64), "m.teams >="),
            (f.max_teams.map(|n| n as f64), "m.teams <="),
        ];
        for (bound, test) in bounds {
            if let Some(n) = bound {
                args.push(Box::new(n));
                c.push(format!("{test} ?{}", args.len()));
            }
        }
        for status in &q.hidden {
            args.push(Box::new(status.key()));
            c.push(format!("m.status != ?{}", args.len()));
        }
        (c.join(" AND "), args)
    }

    /// How many matches of current files match `q`, ignoring its paging
    pub fn count_matches(&self, q: &MatchQuery) -> Result<u64, CacheError> {
        let (conditions, args) = Self::match_conditions(q);
        let sql = format!(
            "SELECT COUNT(*) FROM matches m JOIN files f ON f.id = m.file_id WHERE {conditions}"
        );
        let refs: Vec<&dyn ToSql> = args.iter().map(|a| a.as_ref()).collect();
        Ok(self
            .conn
            .query_row(&sql, refs.as_slice(), |r| r.get::<_, i64>(0))? as u64)
    }

    /// One page of matches meeting every condition, in `q`'s order with the
    /// demo and the order played breaking ties
    pub fn query_matches(&self, q: &MatchQuery) -> Result<Vec<MatchRow>, CacheError> {
        let (conditions, mut args) = Self::match_conditions(q);
        args.push(Box::new(q.files.limit.max(1) as i64));
        let limit = args.len();
        args.push(Box::new(q.files.offset as i64));
        let offset = args.len();
        let direction = if q.files.descending { "DESC" } else { "ASC" };
        let order = q.sort.column();

        let sql = format!(
            "SELECT f.id, f.path, f.path_blob, f.recorded_at, f.date_source, m.ordinal,
                m.status, m.opens_at, m.length, m.map, m.players, m.teams, m.kills, m.score,
                m.score_json
             FROM matches m JOIN files f ON f.id = m.file_id WHERE {conditions}
             ORDER BY {order} {direction} NULLS LAST, f.path, m.ordinal
             LIMIT ?{limit} OFFSET ?{offset}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let refs: Vec<&dyn ToSql> = args.iter().map(|a| a.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), |r| {
            let status: String = r.get(6)?;
            Ok(MatchRow {
                file_id: r.get(0)?,
                path: r.get(1)?,
                native: r.get(2)?,
                recorded_at: r.get(3)?,
                date_source: r.get(4)?,
                ordinal: r.get(5)?,
                status: PartStatus::from_key(&status).unwrap_or(PartStatus::FreePlay),
                opens_at: r.get(7)?,
                length: r.get(8)?,
                map: r.get(9)?,
                players: r.get(10)?,
                teams: r.get(11)?,
                kills: r.get(12)?,
                score: r.get(13)?,
                scoreboard: r
                    .get::<_, Option<String>>(14)?
                    .and_then(|json| serde_json::from_str(&json).ok()),
            })
        })?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
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
        let map = Self::map_column(q);
        let order = match q.sort {
            Sort::Map => map,
            other => other.column(),
        };

        let sql = format!(
            "SELECT f.id, f.path, f.status, {map}, f.duration, f.recording_kind,
                f.spectator_fraction, f.kills, f.best_window, f.error_code, f.error_args, f.error,
                f.recorder, f.shots,
                (SELECT w.shots FROM weapons w WHERE w.file_id = f.id AND w.weapon = 'gauss')
                    AS gauss,
                f.recorded_at, f.date_source, f.score, f.headshots, f.accuracy, f.path_blob,
                f.score_json, f.mtime_ns, f.clip_of, f.players, f.teams
             FROM files f WHERE {conditions}
             ORDER BY {order} {direction} NULLS LAST, f.path
             LIMIT ?{limit} OFFSET ?{offset}"
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
                modified_at: r.get::<_, Option<i64>>(22)?.and_then(super::date::local),
                clip_of: r.get(23)?,
                score: r.get(17)?,
                scoreboard: r
                    .get::<_, Option<String>>(21)?
                    .and_then(|json| serde_json::from_str(&json).ok()),
                headshots: r.get(18)?,
                accuracy: r.get(19)?,
                players: r.get(24)?,
                teams: r.get(25)?,
                native: r.get(20)?,
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
                killer_speed, victim_speed, distance, headshot
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
                headshot: r.get(10)?,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A cache of files named `a.dem`, `b.dem` and so on, each with the
    /// stored score JSON and best action score given
    fn cache(files: &[(&str, Option<&str>, Option<f64>)]) -> Cache {
        let mut c = Cache::in_memory().unwrap();
        for (name, score_json, best_window) in files {
            let path = Path::new(name);
            c.store(&Entry {
                path,
                size: 1,
                mtime_ns: None,
                file_identity: None,
                quick_hash: None,
                full_hash: None,
                header: None,
                report: None,
                error: None,
                floors: None,
            })
            .unwrap();

            // Stored as an analysis would have
            c.conn
                .execute(
                    "UPDATE files SET score_json = ?1, best_window = ?2 WHERE path = ?3",
                    params![score_json, best_window, name],
                )
                .unwrap();
        }
        c
    }

    fn side(frags: i32) -> String {
        format!(r#"{{"name":"s{frags}","frags":{frags},"deaths":0}}"#)
    }

    #[test]
    fn newest_files_sort_first_by_when_they_were_modified() {
        let mut c = Cache::in_memory().unwrap();
        for (name, mtime_ns) in [
            ("old.dem", Some(1)),
            ("new.dem", Some(3)),
            ("none.dem", None),
        ] {
            c.store(&Entry {
                path: Path::new(name),
                size: 1,
                mtime_ns,
                file_identity: None,
                quick_hash: None,
                full_hash: None,
                header: None,
                report: None,
                error: None,
                floors: None,
            })
            .unwrap();
        }

        // A file without a time goes last either way
        let q = FileQuery {
            sort: Sort::Modified,
            descending: true,
            limit: 10,
            ..FileQuery::default()
        };
        let rows = c.query(&q).unwrap();
        let names: Vec<&str> = rows.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(names, ["new.dem", "old.dem", "none.dem"]);
        assert!(rows[0].modified_at.is_some() && rows[2].modified_at.is_none());
    }

    #[test]
    fn a_midair_kill_is_either_player_in_the_air() {
        let c = cache(&[
            ("killer.dem", None, None),
            ("victim.dem", None, None),
            ("ground.dem", None, None),
        ]);

        // One kill each, by who was in the air, and nobody where it is not
        // known
        for (name, killer, victim) in [
            ("killer.dem", Some(true), Some(false)),
            ("victim.dem", Some(false), Some(true)),
            ("ground.dem", Some(false), None),
        ] {
            c.conn
                .execute(
                    "INSERT INTO kills(file_id, kill_id, segment, time, weapon, cause,
                        killer_airborne, victim_airborne)
                     SELECT id, 0, 0, 1.0, 'gauss', 'player_weapon', ?2, ?3
                     FROM files WHERE path = ?1",
                    params![name, killer, victim],
                )
                .unwrap();
        }

        let q = FileQuery {
            kills: KillQuery {
                midair: true,
                ..KillQuery::default()
            },
            limit: 10,
            ..FileQuery::default()
        };
        let names: Vec<String> = c.query(&q).unwrap().into_iter().map(|f| f.path).collect();
        assert_eq!(names, ["killer.dem", "victim.dem"]);
    }

    #[test]
    fn score_sorts_by_the_first_side_across_pages() {
        let two = format!(r#"{{"teams":true,"sides":[{},{}]}}"#, side(10), side(5));
        let negative = format!(r#"{{"teams":false,"sides":[{}]}}"#, side(-3));
        let tied = format!(
            r#"{{"teams":false,"sides":[{},{},{}]}}"#,
            side(10),
            side(2),
            side(1)
        );
        let c = cache(&[
            ("e.dem", Some("{not json"), None),
            ("c.dem", Some(&tied), Some(12.5)),
            ("a.dem", Some(&two), Some(5.)),
            ("b.dem", Some(&negative), Some(-1.)),
            ("d.dem", None, None),
        ]);

        let page = |descending: bool, offset: usize| -> Vec<String> {
            let q = FileQuery {
                sort: Sort::Score,
                descending,
                offset,
                limit: 2,
                ..Default::default()
            };
            c.query(&q).unwrap().into_iter().map(|r| r.path).collect()
        };

        // Equal scores fall back to the path, and files without a score that
        // can be read come last either way
        let mut highest = Vec::new();
        for offset in [0, 2, 4] {
            highest.extend(page(true, offset));
        }
        assert_eq!(highest, ["a.dem", "c.dem", "b.dem", "d.dem", "e.dem"]);

        let mut lowest = Vec::new();
        for offset in [0, 2, 4] {
            lowest.extend(page(false, offset));
        }
        assert_eq!(lowest, ["b.dem", "a.dem", "c.dem", "d.dem", "e.dem"]);
    }

    #[test]
    fn stored_scoreboards_read_back_and_bad_ones_are_none() {
        let two = format!(r#"{{"teams":true,"sides":[{},{}]}}"#, side(170), side(155));
        let c = cache(&[
            ("a.dem", Some(&two), None),
            ("b.dem", Some("[1, 2"), None),
            ("c.dem", None, None),
        ]);
        let rows = c
            .query(&FileQuery {
                limit: 10,
                ..Default::default()
            })
            .unwrap();

        let board = rows[0].scoreboard.as_ref().unwrap();
        assert!(board.teams);
        let frags: Vec<i32> = board.sides.iter().map(|s| s.frags).collect();
        assert_eq!(frags, [170, 155]);

        assert_eq!(rows[1].scoreboard, None);
        assert_eq!(rows[2].scoreboard, None);
    }

    #[test]
    fn player_and_team_bounds_keep_the_demos_within_them() {
        let mut c = Cache::in_memory().unwrap();
        for (name, players, teams) in [("duel.dem", 2, 2), ("big.dem", 10, 2), ("ffa.dem", 6, 0)] {
            c.store(&Entry {
                path: Path::new(name),
                size: 1,
                mtime_ns: None,
                file_identity: None,
                quick_hash: None,
                full_hash: None,
                header: None,
                report: None,
                error: None,
                floors: None,
            })
            .unwrap();

            // Stored as an analysis would have
            c.conn
                .execute(
                    "UPDATE files SET players = ?1, teams = ?2 WHERE path = ?3",
                    params![players, teams, name],
                )
                .unwrap();
        }

        let names = |q: FileQuery| -> Vec<String> {
            let q = FileQuery {
                sort: Sort::Players,
                limit: 10,
                ..q
            };
            c.query(&q).unwrap().into_iter().map(|f| f.path).collect()
        };
        let four_or_more = FileQuery {
            min_players: Some(4),
            ..FileQuery::default()
        };
        assert_eq!(names(four_or_more), ["ffa.dem", "big.dem"]);
        let two_teams_up_to_eight = FileQuery {
            min_teams: Some(2),
            max_teams: Some(2),
            max_players: Some(8),
            ..FileQuery::default()
        };
        assert_eq!(names(two_teams_up_to_eight), ["duel.dem"]);
    }

    #[test]
    fn every_map_names_and_finds_the_maps_left_out() {
        let mut c = Cache::in_memory().unwrap();
        c.store(&Entry {
            path: Path::new("two.dem"),
            size: 1,
            mtime_ns: None,
            file_identity: None,
            quick_hash: None,
            full_hash: None,
            header: None,
            report: None,
            error: None,
            floors: None,
        })
        .unwrap();

        // Stored as an analysis of a demo ending on an empty map would have
        c.conn
            .execute(
                "UPDATE files SET map = 'boot_camp', all_maps = 'boot_camp, endcamp'",
                [],
            )
            .unwrap();

        let maps = |every_map: bool, map: &str| -> Vec<Option<String>> {
            let q = FileQuery {
                every_map,
                map: Some(map.into()),
                sort: Sort::Map,
                limit: 10,
                ..FileQuery::default()
            };
            c.query(&q).unwrap().into_iter().map(|f| f.map).collect()
        };
        assert_eq!(maps(false, "boot"), [Some("boot_camp".to_owned())]);
        assert!(maps(false, "endcamp").is_empty());
        assert_eq!(
            maps(true, "endcamp"),
            [Some("boot_camp, endcamp".to_owned())]
        );
    }

    #[test]
    fn the_fewest_kills_leave_out_demos_with_fewer() {
        let c = cache(&[
            ("a.dem", None, None),
            ("b.dem", None, None),
            ("c.dem", None, None),
        ]);
        for (name, kills) in [("a.dem", 3), ("b.dem", 10), ("c.dem", 25)] {
            c.conn
                .execute(
                    "UPDATE files SET kills = ?1 WHERE path = ?2",
                    params![kills, name],
                )
                .unwrap();
        }

        let at_least = |kills| FileQuery {
            min_kills: Some(kills),
            limit: 10,
            ..Default::default()
        };
        assert_eq!(c.count(&at_least(10)).unwrap(), 2);
        assert_eq!(c.count(&at_least(30)).unwrap(), 0);
        assert_eq!(c.count(&FileQuery::default()).unwrap(), 3);
    }

    #[test]
    fn clearing_forgets_every_file_and_keeps_the_cache_usable() {
        let mut c = cache(&[("a.dem", None, None), ("b.dem", None, None)]);
        assert_eq!(c.stored_paths().unwrap().len(), 2);

        c.clear().unwrap();
        assert!(c.stored_paths().unwrap().is_empty());
        assert_eq!(c.count(&FileQuery::default()).unwrap(), 0);
    }

    #[test]
    fn library_stats_cover_every_file_whatever_is_filtered() {
        let c = cache(&[
            ("a.dem", None, Some(5.)),
            ("b.dem", None, Some(-1.)),
            ("c.dem", None, Some(12.5)),
            ("d.dem", None, None),
        ]);
        let stats = c.library_stats().unwrap();
        assert_eq!(stats.files, 4);
        assert_eq!(stats.max_action, Some(12.5));

        // Nothing positive leaves no scale
        let c = cache(&[("a.dem", None, Some(0.)), ("b.dem", None, None)]);
        assert_eq!(c.library_stats().unwrap().max_action, None);
    }

    #[test]
    fn matches_are_found_by_their_own_map_length_and_status() {
        let c = cache(&[("a.dem", None, None), ("b.dem", None, None)]);
        let parts = [
            ("a.dem", 0, "interrupted", "boot_camp", 353.),
            ("a.dem", 1, "complete", "boot_camp", 1260.),
            ("b.dem", 0, "complete", "crossfire", 900.),
            ("b.dem", 1, "free-play", "datacore", 400.),
        ];
        for (path, ordinal, status, map, length) in parts {
            c.conn
                .execute(
                    "INSERT INTO matches(file_id, ordinal, segment, status, opens_at, start, end,
                        length, map, players, teams, kills)
                     SELECT id, ?1, 1, ?2, 0, 0, ?4, ?4, ?3, 4, 2, 10 FROM files
                     WHERE path = ?5",
                    params![ordinal, status, map, length, path],
                )
                .unwrap();
        }
        let found = |q: &MatchQuery| -> Vec<(String, i64)> {
            let rows = c.query_matches(q).unwrap();
            assert_eq!(c.count_matches(q).unwrap(), rows.len() as u64);
            rows.into_iter().map(|r| (r.path, r.ordinal)).collect()
        };
        let all = MatchQuery {
            files: FileQuery {
                limit: 10,
                ..FileQuery::default()
            },
            ..MatchQuery::default()
        };
        assert_eq!(found(&all).len(), 4);
        assert_eq!(c.library_stats().unwrap().matches, 4);

        let complete = MatchQuery {
            hidden: vec![PartStatus::Interrupted, PartStatus::FreePlay],
            ..all.clone()
        };
        assert_eq!(found(&complete), [("a.dem".into(), 1), ("b.dem".into(), 0)]);

        // Only the matches of the files asked for, and none of none
        let mut only = all.clone();
        only.files.only_paths = Some(vec!["b.dem".into()]);
        assert_eq!(found(&only), [("b.dem".into(), 0), ("b.dem".into(), 1)]);
        only.files.only_paths = Some(Vec::new());
        assert_eq!(found(&only), []);

        let mut boot_camp = all.clone();
        boot_camp.files.map = Some("BOOT".into());
        boot_camp.files.min_duration = Some(600.);
        assert_eq!(found(&boot_camp), [("a.dem".into(), 1)]);

        let mut longest = all.clone();
        longest.sort = MatchSort::Length;
        longest.files.descending = true;
        assert_eq!(found(&longest)[0], ("a.dem".into(), 1));
    }
}
