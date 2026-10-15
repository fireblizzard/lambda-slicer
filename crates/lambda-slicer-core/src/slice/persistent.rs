//! Persistent state a restart re-emits (implementation plan §6.2)
//!
//! Messages before the start that set lasting client state are reduced to
//! the last one per type and key, such as an ammo index or a player slot,
//! and replayed with their original bytes in the order those last ones
//! arrived. Lifecycle messages such as `ResetHUD` are kept the same way, so
//! state they cleared in the source is cleared again in the slice.
//! Transient messages (sounds, effects, chat, pickups, death notices) are
//! not replayed. A message whose effect is not classified blocks the
//! export instead of being guessed. The AG messages are classified from
//! their names and observed use, not from the AG client source

use crate::protocol::messages::{Message, ParsedMessage, svc};
use crate::state::Decoder;

/// How a message before the start is carried into the slice
enum Treatment {
    /// Replay the last one per key
    Keep(Vec<u8>),
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
    "Countdown",
    "FlashBat",
    "Flashlight",
    "GameMode",
    "Gametype",
    "Geiger",
    "HideWeapon",
    "Health",
    "InitHUD",
    "InitLoc",
    "Nextmap",
    "ReqState",
    "ResetHUD",
    "ServerName",
    "SetFOV",
    "Settings",
    "SuddenDeath",
    "TeamNames",
    "Timeout",
    "Timer",
    "Train",
    "VoiceMask",
    "Vote",
    "WhString",
];

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
        Treatment::Keep(Vec::new())
    } else if BY_FIRST_BYTE.contains(&name) {
        Treatment::Keep(first())
    } else if name == "CurWeapon" {
        // State, weapon id, clip. The HUD keeps a clip per weapon, and the
        // last message with state 1 selects the current weapon
        Treatment::Keep(body.get(1).map_or_else(Vec::new, |b| vec![*b]))
    } else if name == "WeaponList" || name == "TeamScore" {
        let end = body.iter().position(|&b| b == 0).unwrap_or(body.len());
        Treatment::Keep(body[..end].to_vec())
    } else if TRANSIENT.contains(&name) {
        Treatment::Drop
    } else {
        Treatment::Block
    }
}

fn engine_treatment(m: &Message) -> Treatment {
    use Message as M;

    let single = || Treatment::Keep(Vec::new());
    match m {
        M::LightStyle { index, .. } => Treatment::Keep(vec![*index]),
        M::UpdateUserInfo { slot, .. } => Treatment::Keep(vec![*slot]),
        M::Customization(body) => Treatment::Keep(body.first().map_or_else(Vec::new, |b| vec![*b])),
        M::NewUserMessage { id, .. } => Treatment::Keep(vec![*id]),
        M::DecalName { index, .. } => Treatment::Keep(vec![*index]),
        // Each one adds a lasting static entity or ambient sound
        M::SpawnStatic(body) | M::SpawnStaticSound(body) => Treatment::Keep(body.clone()),
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

struct Entry {
    opcode: u8,
    key: Vec<u8>,
    /// Arrival order of the latest message for this key
    order: u64,
    bytes: Vec<u8>,
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
    /// is the network record's message bytes
    pub fn observe(&mut self, d: &Decoder, parsed: &ParsedMessage, payload: &[u8]) {
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

        let key = match treatment {
            Treatment::Keep(key) => key,
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
                e.bytes = bytes.to_vec();
            }
            None => self.entries.push(Entry {
                opcode: parsed.opcode,
                key,
                order,
                bytes: bytes.to_vec(),
            }),
        }
    }

    /// The kept messages in the order their latest versions arrived
    pub fn messages(&self) -> Vec<u8> {
        let mut entries: Vec<&Entry> = self.entries.iter().collect();
        entries.sort_by_key(|e| e.order);

        entries
            .iter()
            .flat_map(|e| e.bytes.iter().copied())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
