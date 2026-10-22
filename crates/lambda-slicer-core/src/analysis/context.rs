//! Kills, the recorder's weapon and what the recording can show
//! (implementation plan §5.2, §5.4)
//!
//! `DeathMsg` is killer entity, victim entity and a weapon string. The
//! string is the killer's active weapon, or the inflictor's class name
//! without its `weapon_`, `monster_` or `func_` prefix (OpenAG
//! `dlls/multiplay_gamerules.cpp`). A kill is not a shot and its time is not
//! a launch time. `CurWeapon` is state, signed weapon ID and signed clip,
//! sent only to the recording player (`cl_dll/ammo.cpp`)
//!
//! The killer slot holds a player only when a player killed. Doors,
//! trains, mounted guns, turrets and monsters leave it at 0 and are told
//! apart by their class names (OpenAG `dlls`). A player operating a mounted
//! gun is the killer, with the gun's name. Singleplayer sends no death
//! messages at all

use std::collections::BTreeMap;

use super::accuracy::{Blood, Pose, TE_BLOODSPRITE, blood_origin};
use crate::message::Message as Note;
use serde::{Deserialize, Serialize};

use super::highlights::KillContext;
use super::match_events::MatchEvents;
use super::motion::{MapName, Motion, MotionState, Sample, gravity, state_at};
use super::weapons::Weapon;
use crate::protocol::messages::{Message, ParsedMessage};
use crate::slice::range::GroupIndex;
use crate::state::{Decoder, FrameLocation, PlayerIdentity};

/// What a kill came from
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KillCause {
    /// Another player, with a player weapon or its projectile
    PlayerWeapon,
    /// Another player, by a name that is not one weapon (such as
    /// `grenade`, which the mp5 and the hand grenade both send)
    PlayerOther,
    /// A player operating a mounted gun (`func_tank` and its kinds)
    MountedGun,
    /// The victim killed itself
    Suicide,
    /// A teammate, which the server reports without the weapon
    Teammate,
    /// World damage such as a fall
    World,
    /// Doors, platforms, trains, rotating parts, unmanned guns, hurt
    /// triggers and environment effects
    Environment,
    /// A monster or an unowned weapon
    Monster,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct Kill {
    pub segment: u32,
    /// Playback time, and the server time of the packet that carried it
    pub time: f32,
    pub server_time: f32,
    pub killer_entity: u8,
    pub victim_entity: u8,
    /// Occupants when the message arrived. A killer entity of 0 is the world
    pub killer: Option<PlayerIdentity>,
    pub victim: Option<PlayerIdentity>,
    /// Teams from `TeamInfo` when the message arrived
    pub killer_team: Option<String>,
    pub victim_team: Option<String>,
    /// The weapon string as sent
    pub weapon_name: String,
    pub cause: KillCause,
    /// The player weapon used, only when a player killed with one
    pub weapon: Option<Weapon>,
    /// The victim's movement when last seen alive, as the hit's push, such
    /// as a blast's, moves them as they die, and the death message can come
    /// a few frames after
    #[serde(skip)]
    pub victim_state: MotionState,
    /// The killer's recent samples, to judge its movement at the shot
    #[serde(skip)]
    pub killer_samples: Vec<Sample>,
    /// The session's map, where the floor under the players is
    pub map: Option<MapName>,
    /// Movement, linked shot and distance, filled after the decode
    pub context: Option<KillContext>,
    /// Whether the killing hit was to the head, from the victim's death
    /// animation. None when it could not be seen, such as a victim out of
    /// view or gibbed, or when the kill was not a player's direct hit
    pub headshot: Option<bool>,
}

/// Death animations of the standard `player.mdl`, which the server picks by
/// the last body part hit (`GetDeathActivity`). Sequences 22 `headshot` and
/// 23 `die_spin` both have `ACT_DIE_HEADSHOT`
const DEATH_SEQUENCES: std::ops::RangeInclusive<i64> = 18..=24;
const HEADSHOT_SEQUENCES: [i64; 2] = [22, 23];

/// Weapons whose hits go through `TraceAttack`, which records the body part.
/// Explosions and hornets damage without it, so the death animation would
/// come from an earlier hit
fn hits_body_parts(w: Weapon) -> bool {
    matches!(
        w,
        Weapon::Crowbar
            | Weapon::Glock
            | Weapon::Python
            | Weapon::Mp5
            | Weapon::Shotgun
            | Weapon::Crossbow
            | Weapon::Gauss
            | Weapon::Egon
    )
}

/// Whether an entity sequence is a headshot death. Other deaths give false,
/// and anything else gives None, such as a gibbed player still in an aiming
/// pose
pub fn headshot_sequence(sequence: i64) -> Option<bool> {
    DEATH_SEQUENCES
        .contains(&sequence)
        .then(|| HEADSHOT_SEQUENCES.contains(&sequence))
}

/// The recorder's active weapon from this time on
#[derive(Clone, Debug, Serialize)]
pub struct WeaponSwitch {
    pub segment: u32,
    pub time: f32,
    /// Signed weapon ID, below 1 when no weapon is shown
    pub id: i8,
    pub weapon: Option<Weapon>,
    pub clip: i8,
}

/// What kind of recording this is, which bounds what it can show
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingKind {
    /// A player's own view in multiplayer
    MultiplayerView,
    Singleplayer,
    /// A spectator proxy recording
    Hltv,
    /// Decoding stopped before the server said how many players it holds
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct Coverage {
    pub kind: RecordingKind,
    /// The recording client's own predicted events
    pub local_events: usize,
    /// Events the server sent this client
    pub network_events: usize,
    pub kills: usize,
    /// Playback time, and the part the recording client spent spectating
    pub playback_seconds: f64,
    pub spectator_seconds: f64,
    /// `spectator_seconds` over `playback_seconds`
    pub spectator_fraction: f64,
    /// What the counts can and cannot show
    pub notes: Vec<Note>,
}

/// `func_` classes without their prefix, from OpenAG `dlls`
const ENVIRONMENT: &[&str] = &[
    "breakable",
    "button",
    "conveyor",
    "door",
    "door_rotating",
    "friction",
    "guntarget",
    "healthcharger",
    "illusionary",
    "ladder",
    "monsterclip",
    "mortar_field",
    "pendulum",
    "plat",
    "platrot",
    "pushable",
    "recharge",
    "rot_button",
    "rotating",
    "tank",
    "tankcontrols",
    "tanklaser",
    "tankmortar",
    "tankrocket",
    "trackautochange",
    "trackchange",
    "tracktrain",
    "train",
    "traincontrols",
    "wall",
    "wall_toggle",
    "water",
    // The shell of a `func_mortar_field`, a `monster_mortar`
    "mortar",
];

/// Mounted guns, which a player can operate
const MOUNTED_GUNS: &[&str] = &["tank", "tanklaser", "tankmortar", "tankrocket"];

/// `monster_` classes without their prefix, and projectiles monsters fire
const MONSTERS: &[&str] = &[
    "alien_controller",
    "alien_grunt",
    "alien_slave",
    "apache",
    "babycrab",
    "barnacle",
    "barney",
    "bigmomma",
    "bloater",
    "bullchicken",
    "cockroach",
    "flyer",
    "gargantua",
    "generic",
    "gman",
    "grunt_repel",
    "headcrab",
    "houndeye",
    "human_assassin",
    "human_grunt",
    "ichthyosaur",
    "leech",
    "miniturret",
    "nihilanth",
    "osprey",
    "rat",
    "scientist",
    "sentry",
    "tentacle",
    "turret",
    "vortigaunt",
    "zombie",
    // Alien grunt hornets and wild snarks, when no player owns them
    "hornet",
    "snark",
];

/// Cause and player weapon of a death message
fn kill_cause(killer: u8, victim: u8, name: &str, max_clients: u8) -> (KillCause, Option<Weapon>) {
    let player = killer >= 1 && killer <= max_clients;
    if name == "teammate" {
        return (KillCause::Teammate, None);
    }
    if player && killer == victim {
        return (KillCause::Suicide, kill_weapon(name));
    }
    if player {
        if MOUNTED_GUNS.contains(&name) {
            return (KillCause::MountedGun, None);
        }
        return match kill_weapon(name) {
            Some(w) => (KillCause::PlayerWeapon, Some(w)),
            None => (KillCause::PlayerOther, None),
        };
    }

    let cause = if name == "world" || name == "worldspawn" {
        KillCause::World
    } else if ENVIRONMENT.contains(&name)
        || name.starts_with("trigger_")
        || name.starts_with("env_")
    {
        KillCause::Environment
    } else if MONSTERS.contains(&name) {
        KillCause::Monster
    } else {
        KillCause::Unknown
    };

    (cause, None)
}

/// Weapon names of `DeathMsg`, from Half-Life weapon and projectile class
/// names
fn kill_weapon(name: &str) -> Option<Weapon> {
    Some(match name {
        "crowbar" => Weapon::Crowbar,
        "9mmhandgun" | "glock" => Weapon::Glock,
        "357" | "python" => Weapon::Python,
        "9mmAR" | "mp5" => Weapon::Mp5,
        "crossbow" | "bolt" => Weapon::Crossbow,
        "shotgun" => Weapon::Shotgun,
        "rpg" | "rpg_rocket" => Weapon::Rpg,
        "gauss" | "tau_cannon" => Weapon::Gauss,
        "egon" | "gluon gun" => Weapon::Egon,
        "hornetgun" | "hornet" => Weapon::Hivehand,
        "handgrenade" => Weapon::Handgrenade,
        "tripmine" => Weapon::Tripmine,
        "satchel" => Weapon::Satchel,
        "snark" => Weapon::Snark,
        _ => return None,
    })
}

/// One side of the final score, a team or a player
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreSide {
    pub name: String,
    pub frags: i32,
    pub deaths: i32,
}

/// The final score of the recording's last session, as the scores stood at
/// its last kill. Players often leave between the end of a game and the end
/// of the recording, and their frags still belong to the result
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scoreboard {
    /// Whether sides are teams rather than players
    pub teams: bool,
    /// Highest frags first. Players who left before the last kill, and
    /// players with neither a frag nor a death such as spectators, are left
    /// out
    pub sides: Vec<ScoreSide>,
}

impl From<&Standing> for Scoreboard {
    fn from(s: &Standing) -> Self {
        Scoreboard {
            teams: s.teams,
            sides: s
                .sides
                .iter()
                .map(|side| ScoreSide {
                    name: side.name.clone(),
                    frags: side.frags,
                    deaths: side.deaths,
                })
                .collect(),
        }
    }
}

/// When the result was decided, in the session played longest, since a
/// demo may end with a few moments of the next map. The time is that
/// session's last kill, or its end when nobody was killed there. Without
/// frame groups the session is the last one with a kill or scores
pub fn final_moment(kills: &[Kill], updates: &[ScoreUpdate], groups: &GroupIndex) -> (u32, f32) {
    let segment = match (groups.longest_segment(), kills.last()) {
        (Some(segment), _) => segment,
        (None, Some(k)) => k.segment,
        (None, None) => updates.last().map_or(0, |u| u.segment),
    };

    match kills.iter().rev().find(|k| k.segment == segment) {
        Some(k) => (segment, k.time),
        None => (segment, f32::INFINITY),
    }
}

impl Scoreboard {
    /// Frags of the best sides, such as `88 – 74` or `50 – 44 – 38 – …`. The
    /// en dash keeps a negative score readable, as in `150 – -1`
    pub fn summary(&self, shown: usize) -> String {
        let mut parts: Vec<String> = self
            .sides
            .iter()
            .take(shown)
            .map(|s| s.frags.to_string())
            .collect();
        if self.sides.len() > shown {
            parts.push("…".into());
        }
        parts.join(" – ")
    }
}

/// A player's frags, deaths or team as the server changed them
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ScoreUpdate {
    pub segment: u32,
    pub time: f32,
    pub player: PlayerIdentity,
    /// Empty outside team play
    pub team: String,
    pub frags: i32,
    pub deaths: i32,
    /// The player left the server here, and no longer counts
    pub left: bool,
}

/// A team's frags as `TeamScore` set them, overriding its players' sum
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TeamScoreUpdate {
    pub segment: u32,
    pub time: f32,
    /// Lowercase, as teams are matched without case
    pub team: String,
    pub frags: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerScore {
    pub name: String,
    pub frags: i32,
    pub deaths: i32,
    /// The connection, so other per-player counts can be matched to it
    pub slot: u8,
    pub incarnation: u32,
}

/// A team, or a player outside team play, with its players
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StandingSide {
    pub name: String,
    pub frags: i32,
    pub deaths: i32,
    /// Highest frags first
    pub players: Vec<PlayerScore>,
}

/// The scores at one moment
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Standing {
    pub teams: bool,
    /// Highest frags first
    pub sides: Vec<StandingSide>,
}

/// The scores as they stood at `time` in `segment`. Times restart with each
/// session, and a recording's loading part has times of its own, so the
/// session is given rather than guessed from the time. Players count once
/// they score or die at any point of that session, so a player still on
/// nothing early on is shown with zeros, and spectators who never score are
/// left out. A player who left no longer counts, as on the game's own
/// scoreboard
pub fn standing_at(
    updates: &[ScoreUpdate],
    team_updates: &[TeamScoreUpdate],
    segment: u32,
    time: f32,
) -> Standing {
    let in_segment = || updates.iter().filter(|u| u.segment == segment);
    let key = |u: &ScoreUpdate| (u.player.slot, u.player.incarnation);

    let mut keys: Vec<(u8, u32)> = Vec::new();
    for u in in_segment().filter(|u| u.frags != 0 || u.deaths != 0) {
        if !keys.contains(&key(u)) {
            keys.push(key(u));
        }
    }

    // Each counted player with their team, as of `time` or as first seen
    // when not yet
    let score = |u: &ScoreUpdate, now: bool| PlayerScore {
        name: u.player.name.clone(),
        frags: if now { u.frags } else { 0 },
        deaths: if now { u.deaths } else { 0 },
        slot: u.player.slot,
        incarnation: u.player.incarnation,
    };
    let mut players: Vec<(PlayerScore, String)> = keys
        .iter()
        .filter_map(|k| {
            let mut mine = in_segment().filter(|u| key(u) == *k);
            let first = mine.clone().next()?;
            match mine.rfind(|u| u.time <= time) {
                Some(u) if u.left => None,
                Some(u) => Some((score(u, true), u.team.clone())),
                None => Some((score(first, false), first.team.clone())),
            }
        })
        .collect();
    players.sort_by(|a, b| b.0.frags.cmp(&a.0.frags).then(a.0.deaths.cmp(&b.0.deaths)));

    let teams: Vec<String> = {
        let mut t: Vec<String> = Vec::new();
        for p in &players {
            if !p.1.is_empty() && !t.iter().any(|x| x.eq_ignore_ascii_case(&p.1)) {
                t.push(p.1.clone());
            }
        }
        t
    };

    let mut sides: Vec<StandingSide> = if teams.len() >= 2 {
        teams
            .iter()
            .map(|team| {
                let members: Vec<PlayerScore> = players
                    .iter()
                    .filter(|p| p.1.eq_ignore_ascii_case(team))
                    .map(|p| p.0.clone())
                    .collect();
                let set = team_updates
                    .iter()
                    .filter(|u| u.segment == segment && u.time <= time)
                    .rfind(|u| u.team == team.to_lowercase())
                    .map(|u| u.frags);
                StandingSide {
                    name: team.clone(),
                    frags: set.unwrap_or_else(|| members.iter().map(|m| m.frags).sum()),
                    deaths: members.iter().map(|m| m.deaths).sum(),
                    players: members,
                }
            })
            .collect()
    } else {
        players
            .iter()
            .map(|p| StandingSide {
                name: p.0.name.clone(),
                frags: p.0.frags,
                deaths: p.0.deaths,
                players: vec![p.0.clone()],
            })
            .collect()
    };
    sides.sort_by(|a, b| b.frags.cmp(&a.frags).then(a.deaths.cmp(&b.deaths)));

    Standing {
        teams: teams.len() >= 2,
        sides,
    }
}

/// The map a session was played on, when it began, and when the server's
/// intermission began, the match over and the next map to come
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SessionMap {
    pub segment: u32,
    pub time: f32,
    /// As players name it, such as `crossfire`
    pub name: String,
    pub intermission: Option<f32>,
}

/// How long before its death message a victim must have been seen alive
/// for their movement then to count as their movement at the death
const DEATH_DELAY: f32 = 0.25;

/// A victim's movement when last seen alive in `samples`, before what
/// killed them pushed them, unless that was well before the death message
/// at `server_time`
fn last_alive(samples: &[Sample], server_time: f32, d: &Decoder) -> MotionState {
    let mut alive = samples.to_vec();
    while alive.last().is_some_and(|s| !s.alive) {
        alive.pop();
    }

    match alive.last() {
        Some(s) if server_time - s.time <= DEATH_DELAY => state_at(&alive, s.time, gravity(d)),
        _ => MotionState::not_visible(),
    }
}

/// A map as players name it, without the `maps/` folder and `.bsp`
pub fn short_map_name(path: &str) -> String {
    let name = path.strip_prefix("maps/").unwrap_or(path);
    name.strip_suffix(".bsp").unwrap_or(name).to_owned()
}

/// How long a map must be recorded for, in seconds, to be listed with
/// nobody killed on it
const LEAST_PLAYED: f32 = 5. * 60.;

/// Every map in `maps` once, in the order they were first played, as the
/// library lists them, such as `ag_hidden_lab, elixir`. A demo of several
/// maps may end shortly after the next one loads, with nothing played
/// there, so it leaves out a map nobody was killed on and recorded for less
/// than five minutes. `played` gives the sessions someone was killed in and
/// `lengths` how long each session was recorded for. With no map left, the
/// first
pub fn map_list(maps: &[SessionMap], played: &[u32], lengths: &BTreeMap<u32, f32>) -> String {
    let names = map_names(maps);
    if names.len() < 2 {
        return names.join(", ");
    }

    // A map counts as played with a kill in any of its sessions, or with
    // five minutes recorded over them
    let mut kept: Vec<&str> = Vec::new();
    for name in names {
        let mut killed_on = false;
        let mut recorded = 0.;
        for map in maps.iter().filter(|m| m.name == name) {
            killed_on |= played.contains(&map.segment);
            recorded += lengths.get(&map.segment).copied().unwrap_or(0.);
        }
        if killed_on || recorded >= LEAST_PLAYED {
            kept.push(name);
        }
    }

    if kept.is_empty()
        && let Some(first) = maps.first()
    {
        kept.push(&first.name);
    }
    kept.join(", ")
}

/// Every map in `maps` once, in the order they were first played, whether
/// anything was played on them or not
pub fn every_map(maps: &[SessionMap]) -> String {
    map_names(maps).join(", ")
}

/// The names of the maps in `maps`, each once, in the order they were first
/// played
fn map_names(maps: &[SessionMap]) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for map in maps {
        if !names.contains(&map.name.as_str()) {
            names.push(&map.name);
        }
    }
    names
}

/// A line of chat, as `SayText` sent it
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ChatMessage {
    pub segment: u32,
    pub time: f32,
    /// The player the line names as sending it, none for the server's own
    pub sender: Option<PlayerIdentity>,
    /// Colour codes such as `^1` left in, and `%l` for the sender's place
    pub text: String,
    /// Where the sender was, as AG's `Location` last gave it
    pub place: Option<[f32; 3]>,
}

/// The byte a chat line starts with when it names its sender
const NAMED: u8 = 2;

/// How many units a coordinate's step is, as `WRITE_COORD` sends them
const COORD_STEP: f32 = 8.;

/// A `SayText`, the sending player's entity, whether the line names that
/// player, and the line without its leading control bytes and trailing
/// line break
pub fn say_text(body: &[u8]) -> Option<(u8, bool, String)> {
    let (&sender, rest) = body.split_first()?;
    let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    let line = String::from_utf8_lossy(&rest[..end]);
    let named = rest.first() == Some(&NAMED);
    let text = line
        .trim_start_matches(|c: char| c.is_ascii_control())
        .trim_end()
        .to_owned();
    Some((sender, named, text))
}

/// AG's `Location`, a player's entity and three coordinates
pub fn location(body: &[u8]) -> Option<(u8, [f32; 3])> {
    let (&player, rest) = body.split_first()?;
    let coord = |k: usize| {
        let bytes = [*rest.get(k * 2)?, *rest.get(k * 2 + 1)?];
        Some(f32::from(i16::from_le_bytes(bytes)) / COORD_STEP)
    };
    Some((player, [coord(0)?, coord(1)?, coord(2)?]))
}

/// Follows kills and the recorder's weapon through the decode
#[derive(Default)]
pub struct Context {
    pub kills: Vec<Kill>,
    pub switches: Vec<WeaponSwitch>,
    /// The session's map, and every session's in decode order
    map: Option<MapName>,
    pub maps: Vec<SessionMap>,
    /// The weapon the LOADING entry left in hand, per session, which
    /// playback starts with rather than switches to
    loading_weapon: Option<(u32, i8)>,
    pub motion: Motion,
    /// Team names by player entity, from `TeamInfo`
    teams: std::collections::BTreeMap<u8, String>,
    /// Frags and deaths by player entity, from `ScoreInfo`
    scores: std::collections::BTreeMap<u8, (i32, i32)>,
    /// Kills whose victim's death animation is still to be seen, as kill
    /// index, victim entity and time
    pending_deaths: Vec<(usize, u8, f32)>,
    /// Blood sprites in decode order, the trace of hits
    pub blood: Vec<Blood>,
    /// Player poses per snapshot, by segment and entity, for rebuilding aim
    pub poses: std::collections::BTreeMap<(u32, u16), Vec<Pose>>,
    last_pose: Option<(u32, f32)>,
    /// Every score and team change in order, for the scores at any moment
    pub score_updates: Vec<ScoreUpdate>,
    pub team_score_updates: Vec<TeamScoreUpdate>,
    /// The chat in order, and each player's place by entity as `Location`
    /// last gave it
    pub chat: Vec<ChatMessage>,
    places: std::collections::BTreeMap<u8, [f32; 3]>,
    /// Votes, timeouts, pauses and sudden death, each change once
    pub match_events: MatchEvents,
}

impl Context {
    /// Log a player's current frags, deaths and team
    fn log_score(&mut self, loc: &FrameLocation, d: &Decoder, entity: u8) {
        let Some(player) = d.player_for_entity(i64::from(entity)) else {
            return;
        };
        let (frags, deaths) = self.scores.get(&entity).copied().unwrap_or_default();
        self.score_updates.push(ScoreUpdate {
            segment: d.segment,
            time: loc.playback_time(),
            player,
            team: self.teams.get(&entity).cloned().unwrap_or_default(),
            frags,
            deaths,
            left: false,
        });
    }

    /// Each visible player's position and model angles, once per packet
    fn record_poses(&mut self, loc: &FrameLocation, d: &Decoder) {
        let now = (d.segment, loc.playback_time());
        if self.last_pose == Some(now) {
            return;
        }
        self.last_pose = Some(now);
        let (Some(snap), Some(schema)) = (d.latest_snapshot(), d.schema("entity_state_player_t"))
        else {
            return;
        };
        let Some(entities) = &snap.entities else {
            return;
        };
        for e in entities
            .iter()
            .filter(|e| e.number >= 1 && e.number <= u16::from(d.max_clients))
        {
            let f = |n: &str| {
                e.fields
                    .value(&schema, n)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.) as f32
            };
            self.poses
                .entry((d.segment, e.number))
                .or_default()
                .push(Pose {
                    time: loc.playback_time(),
                    origin: [f("origin[0]"), f("origin[1]"), f("origin[2]")],
                    pitch: f("angles[0]"),
                    yaw: f("angles[1]"),
                });
        }
    }

    /// Read the death animation of victims killed in earlier packets, from
    /// the first packet after the death notice that shows them
    fn resolve_deaths(&mut self, loc: &FrameLocation, d: &Decoder) {
        if self.pending_deaths.is_empty() {
            return;
        }
        let now = loc.playback_time();
        let snapshot = d.latest_snapshot();
        let entities = snapshot.and_then(|s| s.entities.as_ref());
        let schema = d.schema("entity_state_player_t");
        let kills = &mut self.kills;
        self.pending_deaths.retain(|&(index, victim, time)| {
            // Given up after a second, or when time restarted with a new
            // session, leaving the headshot unknown
            if now < time || now - time > 1. {
                return false;
            }
            // The death notice's own packet still shows the living player
            if now == time {
                return true;
            }
            let (Some(entities), Some(schema)) = (entities, &schema) else {
                return true;
            };
            let Some(e) = entities.iter().find(|e| e.number == u16::from(victim)) else {
                return true;
            };
            let Some(sequence) = e.fields.value(schema, "sequence").and_then(|v| v.as_i64()) else {
                return true;
            };
            kills[index].headshot = headshot_sequence(sequence);
            false
        });
    }

    pub fn observe(&mut self, loc: &FrameLocation, d: &Decoder, m: &ParsedMessage) {
        self.motion.observe(d);
        self.resolve_deaths(loc, d);
        self.record_poses(loc, d);
        if let Message::TempEntity { kind, body } = &m.message
            && *kind == TE_BLOODSPRITE
            && let Some(origin) = blood_origin(body)
        {
            self.blood.push(Blood {
                segment: d.segment,
                time: loc.playback_time(),
                origin,
            });
        }
        if let Message::ServerInfo(info) = &m.message {
            let path = String::from_utf8_lossy(&info.map_name).into_owned();
            self.maps.push(SessionMap {
                segment: d.segment,
                time: loc.playback_time(),
                name: short_map_name(&path),
                intermission: None,
            });
            self.map = Some(MapName {
                game_dir: String::from_utf8_lossy(&info.game_dir).into_owned(),
                path,
            });
            self.pending_deaths.clear();
            self.teams.clear();
            self.places.clear();
            self.match_events.new_session();
            self.scores.clear();
        }

        if let Message::Intermission = &m.message
            && let Some(session) = self.maps.last_mut()
        {
            session.intermission = Some(loc.playback_time());
        }

        // A singleplayer recording's pauses do not stop the game it shows,
        // as the player keeps moving through them
        if let Message::SetPause(paused) = &m.message
            && d.max_clients > 1
        {
            let paused = *paused != 0;
            self.match_events.pause(d, loc.playback_time(), paused);
        }

        let Message::User { id, body } = &m.message else {
            return;
        };
        let Some(def) = d.user_message(*id) else {
            return;
        };

        // Votes, timeouts and sudden death are followed on their own
        let name = def.name_lossy();
        self.match_events
            .message(d, loc.playback_time(), &name, body);

        match name.as_str() {
            "DeathMsg" => {
                let (Some(&killer), Some(&victim)) = (body.first(), body.get(1)) else {
                    return;
                };
                let rest = body.get(2..).unwrap_or_default();
                let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
                let weapon_name = String::from_utf8_lossy(&rest[..end]).into_owned();

                let (cause, weapon) = kill_cause(killer, victim, &weapon_name, d.max_clients);
                let server_time = f32::from_bits(d.time_bits);
                let victim_state = last_alive(&self.motion.samples(victim.into()), server_time, d);
                self.kills.push(Kill {
                    segment: d.segment,
                    time: loc.playback_time(),
                    server_time,
                    killer_entity: killer,
                    victim_entity: victim,
                    killer: d.player_for_entity(i64::from(killer)),
                    victim: d.player_for_entity(i64::from(victim)),
                    killer_team: self.teams.get(&killer).cloned(),
                    victim_team: self.teams.get(&victim).cloned(),
                    weapon_name,
                    cause,
                    weapon,
                    victim_state,
                    killer_samples: self.motion.samples(killer.into()),
                    map: self.map.clone(),
                    context: None,
                    headshot: None,
                });
                // Only a player's direct hit leaves the body part to read
                let direct =
                    cause == KillCause::PlayerWeapon && weapon.is_some_and(hits_body_parts);
                let index = self.kills.len() - 1;
                if direct {
                    self.pending_deaths
                        .push((index, victim, loc.playback_time()));
                } else if cause == KillCause::PlayerWeapon {
                    self.kills[index].headshot = Some(false);
                }
            }
            "ScoreInfo" => {
                let short = |at: usize| {
                    body.get(at..at + 2)
                        .map(|b| i32::from(i16::from_le_bytes([b[0], b[1]])))
                };
                if let (Some(&player), Some(frags), Some(deaths)) =
                    (body.first(), short(1), short(3))
                {
                    self.scores.insert(player, (frags, deaths));
                    self.log_score(loc, d, player);
                }
            }
            "TeamScore" => {
                let end = body.iter().position(|&b| b == 0).unwrap_or(body.len());
                let name = String::from_utf8_lossy(&body[..end]).into_owned();
                if let Some(b) = body.get(end + 1..end + 3) {
                    let frags = i32::from(i16::from_le_bytes([b[0], b[1]]));
                    self.team_score_updates.push(TeamScoreUpdate {
                        segment: d.segment,
                        time: loc.playback_time(),
                        team: name.to_lowercase(),
                        frags,
                    });
                }
            }
            "SayText" => {
                let Some((sender, named, text)) = say_text(body) else {
                    return;
                };
                let player = named
                    .then(|| d.player_for_entity(i64::from(sender)))
                    .flatten();
                self.chat.push(ChatMessage {
                    segment: d.segment,
                    time: loc.playback_time(),
                    place: player.as_ref().and(self.places.get(&sender).copied()),
                    sender: player,
                    text,
                });
            }
            "Location" => {
                if let Some((player, place)) = location(body) {
                    self.places.insert(player, place);
                }
            }
            "TeamInfo" => {
                let Some(&player) = body.first() else {
                    return;
                };
                let rest = body.get(1..).unwrap_or_default();
                let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
                self.teams
                    .insert(player, String::from_utf8_lossy(&rest[..end]).into_owned());
                if self.scores.contains_key(&player) {
                    self.log_score(loc, d, player);
                }
            }
            "CurWeapon" => {
                let (Some(&state), Some(&id), Some(&clip)) =
                    (body.first(), body.get(1), body.get(2))
                else {
                    return;
                };
                // Only the current weapon, and only when it changes
                let (id, clip) = (id as i8, clip as i8);
                if state == 0 {
                    return;
                }
                let last = self
                    .switches
                    .last()
                    .filter(|s| s.segment == d.segment)
                    .map(|s| s.id)
                    .or_else(|| {
                        self.loading_weapon
                            .filter(|(segment, _)| *segment == d.segment)
                            .map(|(_, id)| id)
                    });
                if last == Some(id) {
                    return;
                }
                // The weapon held when playback starts is not a switch
                if loc.entry == 0 {
                    self.loading_weapon = Some((d.segment, id));
                    return;
                }

                self.switches.push(WeaponSwitch {
                    segment: d.segment,
                    time: loc.playback_time(),
                    id,
                    weapon: u8::try_from(id).ok().and_then(Weapon::from_id),
                    clip,
                });
            }
            _ => {}
        }
    }

    pub fn coverage(
        &self,
        d: &Decoder,
        local_events: usize,
        network_events: usize,
        playback_seconds: f64,
        spectator_seconds: f64,
    ) -> Coverage {
        let kind = if d.hltv {
            RecordingKind::Hltv
        } else if d.max_clients == 0 {
            RecordingKind::Unknown
        } else if d.max_clients == 1 {
            RecordingKind::Singleplayer
        } else {
            RecordingKind::MultiplayerView
        };

        let mut notes = Vec::new();
        match kind {
            RecordingKind::MultiplayerView => {
                notes.push(Note::new(
                    "coverage-visible-only",
                    "Other players' events reach this client only while the server counts them \
                     as visible, so their action counts are lower bounds",
                ));
                if local_events == 0 {
                    notes.push(Note::new(
                        "coverage-no-local-events",
                        "No local events: the recorder's own actions are missing unless the \
                         server sent them (client weapon prediction may have been off)",
                    ));
                }
            }
            RecordingKind::Singleplayer => notes.push(Note::new(
                "coverage-singleplayer",
                "Singleplayer: actors other than the player are monsters or world entities, and \
                 there is no death feed",
            )),
            RecordingKind::Unknown => notes.push(Note::new(
                "coverage-unknown",
                "The recording did not decode far enough to tell what kind it is, so nothing \
                 can be said about its events",
            )),
            RecordingKind::Hltv => notes.push(Note::new(
                "coverage-hltv",
                "Spectator recording: there is no recording player, and events follow the \
                 proxy's view",
            )),
        }

        let spectator_fraction = if playback_seconds > 0. {
            spectator_seconds / playback_seconds
        } else {
            0.
        };
        if kind == RecordingKind::MultiplayerView && spectator_fraction > 0.5 {
            let percent = (spectator_fraction * 100.).round();
            notes.push(
                Note::new(
                    "coverage-mostly-spectating",
                    format!(
                        "The recorder spectated for {percent:.0}% of the playback, so most events \
                         are other players' as the spectated view showed them"
                    ),
                )
                .arg("percent", percent),
            );
        }

        Coverage {
            kind,
            local_events,
            network_events,
            kills: self.kills.len(),
            playback_seconds,
            spectator_seconds,
            spectator_fraction,
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_map_list_names_each_map_once_in_the_order_played() {
        let map = |segment: u32, path: &str| SessionMap {
            segment,
            time: 0.,
            name: short_map_name(path),
            intermission: None,
        };
        let maps = [
            map(1, "maps/ag_hidden_lab.bsp"),
            map(2, "maps/elixir.bsp"),
            map(3, "maps/ag_hidden_lab.bsp"),
            map(4, "maps/endcamp.bsp"),
        ];
        assert_eq!(
            map_list(&maps, &[1, 2, 3, 4], &BTreeMap::new()),
            "ag_hidden_lab, elixir, endcamp"
        );

        // A map nobody was killed on and recorded for less than five
        // minutes is left out, unless no map is left
        let short = BTreeMap::from([(1, 600.), (2, 600.), (3, 30.), (4, 30.)]);
        assert_eq!(map_list(&maps, &[1, 2, 3], &short), "ag_hidden_lab, elixir");
        assert_eq!(map_list(&maps, &[2], &short), "ag_hidden_lab, elixir");
        assert_eq!(map_list(&maps, &[], &BTreeMap::new()), "ag_hidden_lab");
        assert_eq!(map_list(&[], &[], &BTreeMap::new()), "");

        // Five minutes or more counts as played, kills or not, over all of a
        // map's sessions
        let long = BTreeMap::from([(1, 200.), (2, 10.), (3, 100.), (4, 300.)]);
        assert_eq!(map_list(&maps, &[], &long), "ag_hidden_lab, endcamp");

        // A demo of one map always names it
        let one = [map(1, "maps/endcamp.bsp"), map(2, "maps/endcamp.bsp")];
        assert_eq!(map_list(&one, &[], &BTreeMap::new()), "endcamp");

        // Every map, played or not
        assert_eq!(every_map(&maps), "ag_hidden_lab, elixir, endcamp");
        assert_eq!(short_map_name("crossfire"), "crossfire");
    }

    #[test]
    fn coverage_follows_the_recording_kind() {
        let mut d = Decoder::new();
        let context = Context::default();

        let c = context.coverage(&d, 0, 0, 0., 0.);
        assert_eq!(c.kind, RecordingKind::Unknown);
        assert_eq!(c.notes[0].code, "coverage-unknown");

        d.max_clients = 1;
        let c = context.coverage(&d, 5, 0, 10., 0.);
        assert_eq!(c.kind, RecordingKind::Singleplayer);

        d.max_clients = 32;
        let c = context.coverage(&d, 0, 10, 10., 0.);
        assert_eq!(c.kind, RecordingKind::MultiplayerView);
        let codes: Vec<&str> = c.notes.iter().map(|n| n.code).collect();
        assert_eq!(codes, ["coverage-visible-only", "coverage-no-local-events"]);

        // Mostly spectating adds a note
        let c = context.coverage(&d, 0, 10, 10., 6.);
        assert_eq!(c.spectator_fraction, 0.6);
        assert_eq!(c.notes[2].code, "coverage-mostly-spectating");
        assert_eq!(
            c.notes[2].args.get("percent"),
            Some(&crate::message::Arg::Number(60.))
        );

        d.hltv = true;
        let c = context.coverage(&d, 0, 10, 10., 0.);
        assert_eq!(c.kind, RecordingKind::Hltv);
    }

    fn update(time: f32, slot: u8, team: &str, frags: i32, deaths: i32) -> ScoreUpdate {
        ScoreUpdate {
            segment: 0,
            time,
            player: PlayerIdentity {
                segment: 0,
                slot,
                user_id: u32::from(slot),
                incarnation: 0,
                name: format!("p{slot}"),
            },
            team: team.into(),
            frags,
            deaths,
            left: false,
        }
    }

    #[test]
    fn standings_follow_the_moment() {
        let updates = [
            update(1., 0, "red", 0, 0),
            update(1., 1, "blue", 0, 0),
            update(1., 2, "", 0, 0),
            update(5., 0, "red", 1, 0),
            update(5., 1, "blue", 0, 1),
            update(9., 1, "blue", 1, 1),
            update(9., 0, "red", 1, 1),
        ];
        let at = |t| standing_at(&updates, &[], 0, t);
        // Before anything, both players show with zeros and the spectator
        // who never scores is left out
        let start = at(2.);
        assert!(start.teams);
        assert_eq!(start.sides.len(), 2);
        assert!(start.sides.iter().all(|s| s.frags == 0));
        let mid = at(6.);
        assert_eq!((mid.sides[0].name.as_str(), mid.sides[0].frags), ("red", 1));
        assert_eq!(mid.sides[1].players[0].deaths, 1);
        let end = at(100.);
        assert_eq!(
            end.sides.iter().map(|s| s.frags).collect::<Vec<_>>(),
            [1, 1]
        );

        // A player who left drops out from then on
        let mut gone = updates.to_vec();
        gone.push(ScoreUpdate {
            left: true,
            ..update(20., 1, "blue", 1, 1)
        });
        assert_eq!(standing_at(&gone, &[], 0, 10.).sides.len(), 2);
        let after = standing_at(&gone, &[], 0, 30.);
        assert_eq!(after.sides.len(), 1);
        assert!(!after.teams);

        // TeamScore overrides the players' sum from its moment on
        let set = [TeamScoreUpdate {
            segment: 0,
            time: 7.,
            team: "blue".into(),
            frags: 5,
        }];
        assert_eq!(standing_at(&updates, &set, 0, 6.).sides[0].name, "red");
        assert_eq!(standing_at(&updates, &set, 0, 8.).sides[0].frags, 5);
        // Another session has nothing to show
        assert!(standing_at(&updates, &set, 3, 8.).sides.is_empty());
    }

    #[test]
    fn players_are_the_sides_outside_team_play() {
        let updates = [update(1., 0, "", 3, 1), update(1., 1, "", 5, 2)];
        let s = standing_at(&updates, &[], 0, 2.);
        assert!(!s.teams);
        assert_eq!(s.sides[0].name, "p1");
        assert_eq!(s.sides[0].players.len(), 1);
    }

    #[test]
    fn score_summaries_show_the_best_sides() {
        let side = |frags| ScoreSide {
            name: String::new(),
            frags,
            deaths: 0,
        };
        let board = |frags: &[i32]| Scoreboard {
            teams: false,
            sides: frags.iter().map(|f| side(*f)).collect(),
        };
        assert_eq!(board(&[88, 74]).summary(3), "88 – 74");
        assert_eq!(board(&[50, 44, 38]).summary(3), "50 – 44 – 38");
        assert_eq!(board(&[50, 44, 38, 2]).summary(3), "50 – 44 – 38 – …");
        assert_eq!(board(&[150, -1]).summary(3), "150 – -1");
        assert_eq!(board(&[]).summary(3), "");
    }

    #[test]
    fn kill_causes_follow_the_killer_slot_and_class_name() {
        let cases = [
            (1, 2, "gauss", KillCause::PlayerWeapon, Some(Weapon::Gauss)),
            (1, 2, "grenade", KillCause::PlayerOther, None),
            (1, 2, "tank", KillCause::MountedGun, None),
            (1, 1, "rpg_rocket", KillCause::Suicide, Some(Weapon::Rpg)),
            (1, 1, "world", KillCause::Suicide, None),
            (1, 2, "teammate", KillCause::Teammate, None),
            (0, 2, "worldspawn", KillCause::World, None),
            (0, 2, "door", KillCause::Environment, None),
            (0, 2, "tracktrain", KillCause::Environment, None),
            (0, 2, "tank", KillCause::Environment, None),
            (0, 2, "trigger_hurt", KillCause::Environment, None),
            (0, 2, "human_grunt", KillCause::Monster, None),
            (0, 2, "turret", KillCause::Monster, None),
            (0, 2, "snark", KillCause::Monster, None),
            (0, 2, "something", KillCause::Unknown, None),
        ];
        for (killer, victim, name, cause, weapon) in cases {
            assert_eq!(
                kill_cause(killer, victim, name, 16),
                (cause, weapon),
                "{name}"
            );
        }
    }

    #[test]
    fn death_animations_tell_headshots() {
        assert_eq!(headshot_sequence(22), Some(true));
        assert_eq!(headshot_sequence(23), Some(true));
        assert_eq!(headshot_sequence(18), Some(false));
        assert_eq!(headshot_sequence(24), Some(false));
        // An aiming pose, as a gibbed player keeps
        assert_eq!(headshot_sequence(45), None);
        assert!(hits_body_parts(Weapon::Mp5));
        assert!(!hits_body_parts(Weapon::Rpg));
    }

    #[test]
    fn kill_weapon_names_follow_the_class_names() {
        assert_eq!(kill_weapon("rpg_rocket"), Some(Weapon::Rpg));
        assert_eq!(kill_weapon("bolt"), Some(Weapon::Crossbow));
        assert_eq!(kill_weapon("9mmAR"), Some(Weapon::Mp5));
        // A grenade may come from the mp5, a hand grenade or a satchel
        assert_eq!(kill_weapon("grenade"), None);
        assert_eq!(kill_weapon("world"), None);
    }
}
