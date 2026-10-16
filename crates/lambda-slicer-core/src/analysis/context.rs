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

use serde::{Deserialize, Serialize};

use super::weapons::Weapon;
use crate::protocol::messages::{Message, ParsedMessage};
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
    pub time: f32,
    pub killer_entity: u8,
    pub victim_entity: u8,
    /// Occupants when the message arrived. A killer entity of 0 is the world
    pub killer: Option<PlayerIdentity>,
    pub victim: Option<PlayerIdentity>,
    /// The weapon string as sent
    pub weapon_name: String,
    pub cause: KillCause,
    /// The player weapon used, only when a player killed with one
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

                let (cause, weapon) = kill_cause(killer, victim, &weapon_name, d.max_clients);
                self.kills.push(Kill {
                    segment: d.segment,
                    time: loc.time(),
                    killer_entity: killer,
                    victim_entity: victim,
                    killer: d.player_for_entity(i64::from(killer)),
                    victim: d.player_for_entity(i64::from(victim)),
                    weapon_name,
                    cause,
                    weapon,
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
    fn kill_weapon_names_follow_the_class_names() {
        assert_eq!(kill_weapon("rpg_rocket"), Some(Weapon::Rpg));
        assert_eq!(kill_weapon("bolt"), Some(Weapon::Crossbow));
        assert_eq!(kill_weapon("9mmAR"), Some(Weapon::Mp5));
        // A grenade may come from the mp5, a hand grenade or a satchel
        assert_eq!(kill_weapon("grenade"), None);
        assert_eq!(kill_weapon("world"), None);
    }
}
