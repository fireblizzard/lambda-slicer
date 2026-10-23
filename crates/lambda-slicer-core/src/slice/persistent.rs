//! Persistent state a restart re-emits
//!
//! Messages before the start that set lasting client state are reduced to
//! the last one per type and key, such as an ammo index or a player slot,
//! and replayed with their original bytes in the order those last ones
//! arrived. Lifecycle messages such as `ResetHUD` are kept the same way, so
//! state they cleared in the source is cleared again in the slice.
//! Transient messages (sounds, effects, chat, pickups, death notices) are
//! not replayed. A message whose effect is not classified blocks the
//! export instead of being guessed.
//!
//! Some HUD messages are timed notices rather than values. The client shows
//! them for a set time after each one arrives and then hides them, so they
//! are replayed only while that time has not run out at the start. Windows
//! and hiding values come from the OpenAG client (`cl_dll/hud_timer.cpp`,
//! `hud_vote.cpp`, `hud_settings.cpp`, `hud_nextmap.cpp`,
//! `hud_countdown.cpp`). The client starts the window when it receives the
//! message, so a replayed notice can show for longer than its remaining
//! time, until the next one replaces it

use crate::protocol::messages::{Message, ParsedMessage, svc};
use crate::state::Decoder;

/// How a message before the start is carried into the slice
enum Treatment {
    /// Replay the last one per key. With a window, the client shows it for
    /// that many seconds after it arrives, and a window of zero hides it
    Keep(Vec<u8>, Option<f32>),
    /// Not lasting state
    Drop,
    /// Lasting state with no tested restart handling
    Block,
}

/// Keyed by the message body's first byte, a player, ammo or flag index
const BY_FIRST_BYTE: &[&str] = &[
    "AmmoX",
    "AuthID",
    "CTFFlag",
    "Location",
    "ScoreInfo",
    "Spectator",
    "StatusText",
    "StatusValue",
    "TeamInfo",
];

/// One value per message type
const SINGLE: &[&str] = &[
    "AllowSpec",
    "Battery",
    "CTF",
    "FlashBat",
    "Flashlight",
    "GameMode",
    "Gametype",
    "Geiger",
    "HideWeapon",
    "Health",
    "InitHUD",
    "InitLoc",
    "ReqState",
    "ResetHUD",
    "ServerName",
    "SetFOV",
    "SuddenDeath",
    "TeamNames",
    "Timeout",
    "Train",
    "VoiceMask",
    "WhString",
];

/// Display windows of timed notices, in seconds
const TIMED: &[(&str, f32)] = &[("Nextmap", 10.), ("Settings", 10.), ("Timer", 5.)];

/// `Countdown` shows the waiting text at 50 until changed, hides at a
/// negative count, and shows other counts for 5 s
fn countdown_window(body: &[u8]) -> Option<f32> {
    match body.first().map(|&b| b as i8) {
        Some(50) => None,
        Some(s) if s < 0 => Some(0.),
        _ => Some(5.),
    }
}

/// `Vote` hides at status 0 and shows other states for 4 s
fn vote_window(body: &[u8]) -> Option<f32> {
    match body.first() {
        Some(0) => Some(0.),
        _ => Some(4.),
    }
}

/// Notifications and effects with no lasting state
const TRANSIENT: &[&str] = &[
    "AmmoPickup",
    "CTFSound",
    "CheatCheck",
    "Damage",
    "DeathMsg",
    "HudText",
    "ItemPickup",
    "MOTD",
    "MapList",
    "PlaySound",
    "PlayerId",
    "SayText",
    "ScreenFade",
    "ScreenShake",
    "ShowMenu",
    "SpikeCheck",
    "Splash",
    "TextMsg",
    "VGUIMenu",
    "WeapPickup",
];

fn user_treatment(name: &str, body: &[u8]) -> Treatment {
    let first = || body.first().map_or_else(Vec::new, |b| vec![*b]);

    if SINGLE.contains(&name) {
        Treatment::Keep(Vec::new(), None)
    } else if let Some(&(_, window)) = TIMED.iter().find(|(n, _)| *n == name) {
        Treatment::Keep(Vec::new(), Some(window))
    } else if name == "Countdown" {
        Treatment::Keep(Vec::new(), countdown_window(body))
    } else if name == "Vote" {
        Treatment::Keep(Vec::new(), vote_window(body))
    } else if BY_FIRST_BYTE.contains(&name) {
        Treatment::Keep(first(), None)
    } else if name == "CurWeapon" {
        // State, weapon id, clip. The HUD keeps a clip per weapon, and the
        // last message with state 1 selects the current weapon
        Treatment::Keep(body.get(1).map_or_else(Vec::new, |b| vec![*b]), None)
    } else if name == "WeaponList" || name == "TeamScore" {
        let end = body.iter().position(|&b| b == 0).unwrap_or(body.len());
        Treatment::Keep(body[..end].to_vec(), None)
    } else if TRANSIENT.contains(&name) {
        Treatment::Drop
    } else {
        Treatment::Block
    }
}

fn engine_treatment(m: &Message) -> Treatment {
    use Message as M;

    let single = || Treatment::Keep(Vec::new(), None);
    let keyed = |key: Vec<u8>| Treatment::Keep(key, None);
    match m {
        M::LightStyle { index, .. } => keyed(vec![*index]),
        M::UpdateUserInfo { slot, .. } => keyed(vec![*slot]),
        M::Customization(body) => keyed(body.first().map_or_else(Vec::new, |b| vec![*b])),
        M::NewUserMessage { id, .. } => keyed(vec![*id]),
        M::DecalName { index, .. } => keyed(vec![*index]),
        // Each one adds a lasting static entity or ambient sound
        M::SpawnStatic(body) | M::SpawnStaticSound(body) => keyed(body.clone()),
        M::SetView(_)
        | M::NewMoveVars(_)
        | M::CdTrack(..)
        | M::RoomType(_)
        | M::SetPause(_)
        | M::Pings(_)
        | M::Intermission
        | M::Finale(_)
        | M::Cutscene(_)
        | M::CrosshairAngle(..)
        | M::VoiceInit { .. }
        | M::SendExtraInfo { .. }
        | M::ResourceLocation(_)
        | M::TimeScale(_) => single(),
        M::Nop
        | M::Event(_)
        | M::Sound(_)
        | M::Time(_)
        | M::Print(_)
        | M::StuffText(_)
        | M::SetAngle(_)
        | M::AddAngle(_)
        | M::ClientData(_)
        | M::PacketEntities(_)
        | M::DeltaPacketEntities(_)
        | M::EventReliable(_)
        | M::TempEntity { .. }
        | M::Particle(_)
        | M::CenterPrint(_)
        | M::KilledMonster
        | M::FoundSecret
        | M::WeaponAnim { .. }
        | M::Choke
        | M::StopSound(_)
        | M::SoundFade(_)
        | M::VoiceData { .. }
        | M::ResourceRequest(..)
        | M::FileTxferFailed(_)
        | M::SendCvarValue(_)
        | M::SendCvarValue2 { .. }
        | M::Director(_)
        | M::Hltv(_)
        | M::SignOnNum(_)
        | M::Version(_) => Treatment::Drop,
        M::Disconnect(_)
        | M::ServerInfo(_)
        | M::DeltaDescription(_)
        | M::SpawnBaseline(_)
        | M::ResourceList { .. }
        | M::Restore { .. }
        | M::User { .. } => Treatment::Block,
    }
}

/// User messages take the opcodes from 64 up
const FIRST_USER_MESSAGE: u8 = 64;

struct Entry {
    opcode: u8,
    key: Vec<u8>,
    /// Arrival order of the latest message for this key
    order: u64,
    /// Its outer time and display window
    time: f32,
    window: Option<f32>,
    bytes: Vec<u8>,
}

/// The kept messages, split by kind
pub struct Kept {
    /// Engine messages, for the bootstrap group
    pub engine: Vec<u8>,
    pub engine_count: usize,
    /// User messages (HUD state), for the record after it
    pub hud: Vec<u8>,
    pub hud_count: usize,
    /// Timed notices left out because they were no longer showing
    pub expired: usize,
}

#[derive(Default)]
pub struct Persistent {
    entries: Vec<Entry>,
    arrivals: u64,
    /// The first message with no tested restart handling
    pub blocker: Option<String>,
}

impl Persistent {
    /// Record a message of the playback entry before the start. `payload`
    /// is the network record's message bytes and `time` its outer time
    pub fn observe(&mut self, d: &Decoder, parsed: &ParsedMessage, payload: &[u8], time: f32) {
        let treatment = match &parsed.message {
            Message::User { id, body } => {
                let name = d.user_message(*id).map(|u| u.name_lossy());
                match name {
                    Some(n) => match user_treatment(&n, body) {
                        Treatment::Block => {
                            self.blocker.get_or_insert(format!("user message {n}"));
                            return;
                        }
                        t => t,
                    },
                    None => Treatment::Block,
                }
            }
            m => engine_treatment(m),
        };

        let (key, window) = match treatment {
            Treatment::Keep(key, window) => (key, window),
            Treatment::Drop => return,
            Treatment::Block => {
                self.blocker
                    .get_or_insert(svc::name(parsed.opcode).to_string());
                return;
            }
        };

        let (start, end) = (parsed.start as usize, parsed.end as usize);
        let Some(bytes) = payload.get(start..end) else {
            self.blocker
                .get_or_insert("message span outside its payload".into());
            return;
        };

        self.arrivals += 1;
        let order = self.arrivals;
        match self
            .entries
            .iter_mut()
            .find(|e| e.opcode == parsed.opcode && e.key == key)
        {
            Some(e) => {
                e.order = order;
                e.time = time;
                e.window = window;
                e.bytes = bytes.to_vec();
            }
            None => self.entries.push(Entry {
                opcode: parsed.opcode,
                key,
                order,
                time,
                window,
                bytes: bytes.to_vec(),
            }),
        }
    }

    /// The messages to replay at `start` (outer time), engine and user
    /// messages apart, each in the order their latest versions arrived.
    /// Timed notices whose window has run out are left out
    pub fn split(&self, start: f32) -> Kept {
        let showing = |e: &&Entry| e.window.is_none_or(|w| start - e.time < w);
        let mut entries: Vec<&Entry> = self.entries.iter().filter(showing).collect();
        let expired = self.entries.len() - entries.len();
        entries.sort_by_key(|e| e.order);

        let (user, engine): (Vec<&Entry>, Vec<&Entry>) = entries
            .into_iter()
            .partition(|e| e.opcode >= FIRST_USER_MESSAGE);
        let bytes = |list: &[&Entry]| -> Vec<u8> {
            list.iter().flat_map(|e| e.bytes.iter().copied()).collect()
        };

        Kept {
            engine: bytes(&engine),
            engine_count: engine.len(),
            hud: bytes(&user),
            hud_count: user.len(),
            expired,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
