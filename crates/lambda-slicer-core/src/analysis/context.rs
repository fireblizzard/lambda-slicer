//! Kills, the recorder's weapon and what the recording can show
//! (implementation plan §5.2, §5.4)
//!
//! `DeathMsg` is killer entity, victim entity and a weapon string: the
//! killer's active weapon, or the inflictor's class name without its
//! `weapon_`, `monster_` or `func_` prefix (OpenAG
//! `dlls/multiplay_gamerules.cpp`). A kill is not a shot and its time is not
//! a launch time. `CurWeapon` is state, signed weapon ID and signed clip,
//! sent only to the recording player (`cl_dll/ammo.cpp`)

use serde::Serialize;

use super::weapons::Weapon;
use crate::protocol::messages::{Message, ParsedMessage};
use crate::state::{Decoder, FrameLocation, PlayerIdentity};

#[derive(Clone, Debug, Serialize)]
pub struct Kill {
    pub segment: u32,
    pub time: f32,
    pub killer_entity: u8,
    pub victim_entity: u8,
    /// Occupants when the message arrived. A killer entity of 0 is the world
    pub killer: Option<PlayerIdentity>,
    pub victim: Option<PlayerIdentity>,
    /// The weapon string as sent
    pub weapon_name: String,
    /// The weapon it names, when the profile knows the name
    pub weapon: Option<Weapon>,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingKind {
    /// A player's own view in multiplayer
    MultiplayerView,
    Singleplayer,
    /// A spectator proxy recording
    Hltv,
}

#[derive(Clone, Debug, Serialize)]
pub struct Coverage {
    pub kind: RecordingKind,
    /// The recording client's own predicted events
    pub local_events: usize,
    /// Events the server sent this client
    pub network_events: usize,
    pub kills: usize,
    /// What the counts can and cannot show
    pub notes: Vec<String>,
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

/// Follows kills and the recorder's weapon through the decode
#[derive(Default)]
pub struct Context {
    pub kills: Vec<Kill>,
    pub switches: Vec<WeaponSwitch>,
}

impl Context {
    pub fn observe(&mut self, loc: &FrameLocation, d: &Decoder, m: &ParsedMessage) {
        let Message::User { id, body } = &m.message else {
            return;
        };
        let Some(def) = d.user_message(*id) else {
            return;
        };

        match def.name_lossy().as_str() {
            "DeathMsg" => {
                let (Some(&killer), Some(&victim)) = (body.first(), body.get(1)) else {
                    return;
                };
                let rest = body.get(2..).unwrap_or_default();
                let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
                let weapon_name = String::from_utf8_lossy(&rest[..end]).into_owned();

                self.kills.push(Kill {
                    segment: d.segment,
                    time: loc.time(),
                    killer_entity: killer,
                    victim_entity: victim,
                    killer: d.player_for_entity(i64::from(killer)),
                    victim: d.player_for_entity(i64::from(victim)),
                    weapon: kill_weapon(&weapon_name),
                    weapon_name,
                });
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
                    .map(|s| s.id);
                if last == Some(id) {
                    return;
                }

                self.switches.push(WeaponSwitch {
                    segment: d.segment,
                    time: loc.time(),
                    id,
                    weapon: u8::try_from(id).ok().and_then(Weapon::from_id),
                    clip,
                });
            }
            _ => {}
        }
    }

    pub fn coverage(&self, d: &Decoder, local_events: usize, network_events: usize) -> Coverage {
        let kind = if d.hltv {
            RecordingKind::Hltv
        } else if d.max_clients <= 1 {
            RecordingKind::Singleplayer
        } else {
            RecordingKind::MultiplayerView
        };

        let mut notes = Vec::new();
        match kind {
            RecordingKind::MultiplayerView => {
                notes.push(
                    "Other players' events reach this client only while the server counts them \
                     as visible, so their action counts are lower bounds"
                        .into(),
                );
                if local_events == 0 {
                    notes.push(
                        "No local events: the recorder's own actions are missing unless the \
                         server sent them (client weapon prediction may have been off)"
                            .into(),
                    );
                }
            }
            RecordingKind::Singleplayer => notes.push(
                "Singleplayer: actors other than the player are monsters or world entities, and \
                 there is no death feed"
                    .into(),
            ),
            RecordingKind::Hltv => notes.push(
                "Spectator recording: there is no recording player, and events follow the \
                 proxy's view"
                    .into(),
            ),
        }

        Coverage {
            kind,
            local_events,
            network_events,
            kills: self.kills.len(),
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_follows_the_recording_kind() {
        let mut d = Decoder::new();
        let context = Context::default();

        d.max_clients = 1;
        assert_eq!(context.coverage(&d, 5, 0).kind, RecordingKind::Singleplayer);

        d.max_clients = 32;
        let c = context.coverage(&d, 0, 10);
        assert_eq!(c.kind, RecordingKind::MultiplayerView);
        assert_eq!(c.notes.len(), 2, "{:?}", c.notes);

        d.hltv = true;
        assert_eq!(context.coverage(&d, 0, 10).kind, RecordingKind::Hltv);
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
