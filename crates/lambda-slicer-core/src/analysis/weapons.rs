//! Weapon actions from recorded events (implementation plan §5.3, §5.4)
//!
//! A versioned profile maps each event script of Half-Life and AG to a
//! weapon and an action. Only firing and launching count as shots. Charge
//! and egon beam activity become intervals per actor, with endpoints marked
//! unknown where the recording does not show them. Nothing is merged across
//! records, so every action has exactly one recorded event behind it
//!
//! Gauss follows the server (OpenAG `dlls/gauss.cpp`). A shot is
//! `gauss.sc` with `bparam2 = 0`, and `bparam1` tells primary from charged
//! fire. `bparam2 != 0` is the stop sent 10 ms after every shot and on
//! holster, which is not a shot. `gaussspin.sc` updates an ongoing charge

use serde::Serialize;

use super::events::RecordedEvent;
use crate::state::{ActorResolution, EventSource, PlayerIdentity};

/// Version of the rules below, carried in every report
pub const PROFILE: &str = "hl-ag-1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Weapon {
    Crowbar,
    Glock,
    Python,
    Mp5,
    Crossbow,
    Shotgun,
    Rpg,
    Gauss,
    Egon,
    Hivehand,
    Tripmine,
    Snark,
}

impl Weapon {
    /// The name used in reports and fixture manifests
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Crowbar => "crowbar",
            Weapon::Glock => "glock",
            Weapon::Python => "python",
            Weapon::Mp5 => "mp5",
            Weapon::Crossbow => "crossbow",
            Weapon::Shotgun => "shotgun",
            Weapon::Rpg => "rpg",
            Weapon::Gauss => "gauss",
            Weapon::Egon => "egon",
            Weapon::Hivehand => "hivehand",
            Weapon::Tripmine => "tripmine",
            Weapon::Snark => "snark",
        }
    }

    /// Standard Half-Life weapon ID (`dlls/weapons.h`)
    pub fn id(self) -> u8 {
        match self {
            Weapon::Crowbar => 1,
            Weapon::Glock => 2,
            Weapon::Python => 3,
            Weapon::Mp5 => 4,
            Weapon::Crossbow => 6,
            Weapon::Shotgun => 7,
            Weapon::Rpg => 8,
            Weapon::Gauss => 9,
            Weapon::Egon => 10,
            Weapon::Hivehand => 11,
            Weapon::Tripmine => 13,
            Weapon::Snark => 15,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// A swing, not a confirmed hit
    Swing,
    /// A shot
    Fire,
    /// A projectile launched
    Launch,
    /// Gauss charging, followed into intervals
    ChargeActivity,
    /// Gauss charge or spin ended without a shot in this record
    ChargeStop,
    /// Egon beam started, kept going, and ended
    BeamStart,
    BeamSustain,
    BeamStop,
    /// Sent before the server checks the placement or release worked
    PlacementAttempt,
    ReleaseAttempt,
}

impl Action {
    /// Whether this action adds to a shot count
    pub fn is_shot(self) -> bool {
        matches!(self, Action::Fire | Action::Launch)
    }

    /// Whether the action is followed into intervals rather than listed
    /// one record at a time
    pub fn is_activity(self) -> bool {
        matches!(self, Action::ChargeActivity | Action::BeamSustain)
    }
}

/// Event script names compared without case or separator differences.
/// The recorded bytes stay unchanged in the evidence
fn normalized(resource: &str) -> String {
    resource.replace('\\', "/").to_ascii_lowercase()
}

/// Weapon, action and mode of an event, when the profile knows it
pub fn classify(e: &RecordedEvent) -> Option<(Weapon, Action, Option<&'static str>)> {
    use Action::*;
    use Weapon::*;

    let name = normalized(e.resource.as_deref()?);
    let script = name.strip_prefix("events/")?;
    let a = &e.args;

    Some(match script {
        "crowbar.sc" => (Crowbar, Swing, None),
        "glock1.sc" => (Glock, Fire, Some("primary")),
        "glock2.sc" => (Glock, Fire, Some("rapid")),
        "python.sc" => (Python, Fire, None),
        "mp5.sc" => (Mp5, Fire, Some("bullets")),
        "mp52.sc" => (Mp5, Launch, Some("grenade")),
        "crossbow1.sc" => (Crossbow, Fire, Some("unzoomed")),
        "crossbow2.sc" => (Crossbow, Fire, Some("zoomed")),
        "shotgun1.sc" => (Shotgun, Fire, Some("single")),
        "shotgun2.sc" => (Shotgun, Fire, Some("double")),
        "rpg.sc" => (Rpg, Launch, None),
        "gauss.sc" if a.bparam2 != 0 => (Gauss, ChargeStop, None),
        "gauss.sc" if a.bparam1 != 0 => (Gauss, Fire, Some("primary")),
        "gauss.sc" => (Gauss, Fire, Some("charged")),
        "gaussspin.sc" => (Gauss, ChargeActivity, None),
        "egon_fire.sc" if a.bparam1 != 0 => (Egon, BeamStart, None),
        "egon_fire.sc" => (Egon, BeamSustain, None),
        "egon_stop.sc" => (Egon, BeamStop, None),
        "firehornet.sc" => (Hivehand, Fire, None),
        "tripfire.sc" => (Tripmine, PlacementAttempt, None),
        "snarkfire.sc" => (Snark, ReleaseAttempt, None),
        _ => return None,
    })
}

/// Who performed an action, as far as the recording shows
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Actor {
    /// A player slot with its occupant when the event arrived
    Player {
        entity: i32,
        identity: PlayerIdentity,
    },
    /// A player entity whose occupant is not known at that time
    UnknownPlayer { entity: i32 },
    /// An entity outside the player range, such as a monster
    Entity { entity: i32 },
    /// No entity resolved
    Unknown,
}

/// How the actor was resolved
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Attribution {
    /// Explicit `entindex` in the event arguments
    ExplicitEntity,
    /// Position in the packet's complete entity array
    PacketEntity,
    /// No entity in the record, or the packet index could not be resolved
    Unresolved,
}

/// How the action was recorded
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    /// The recording client's own predicted event, a demo record
    LocalEvent,
    /// Sent by the server to this client
    NetworkEvent,
    ReliableNetworkEvent,
}

fn actor(e: &RecordedEvent, max_clients: u8) -> (Actor, Attribution) {
    let (entity, attribution) = match e.actor {
        ActorResolution::Explicit { entity } => (entity, Attribution::ExplicitEntity),
        ActorResolution::PacketEntity { entity, .. } => {
            (i32::from(entity), Attribution::PacketEntity)
        }
        _ => return (Actor::Unknown, Attribution::Unresolved),
    };
    if entity <= 0 {
        return (Actor::Unknown, Attribution::Unresolved);
    }

    let actor = match &e.player {
        Some(identity) => Actor::Player {
            entity,
            identity: identity.clone(),
        },
        None if entity <= i32::from(max_clients) => Actor::UnknownPlayer { entity },
        None => Actor::Entity { entity },
    };

    (actor, attribution)
}

fn actor_entity(a: &Actor) -> Option<i32> {
    match a {
        Actor::Player { entity, .. }
        | Actor::UnknownPlayer { entity }
        | Actor::Entity { entity } => Some(*entity),
        Actor::Unknown => None,
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct WeaponOccurrence {
    pub id: usize,
    pub segment: u32,
    pub time: f32,
    pub server_time: f32,
    pub weapon: Weapon,
    pub action: Action,
    pub mode: Option<&'static str>,
    pub shot: bool,
    pub actor: Actor,
    pub attribution: Attribution,
    pub evidence: Evidence,
    /// The recorded event behind it
    pub event: RecordedEvent,
}

/// Why an interval ended
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntervalEnd {
    /// A shot ended a charge
    Fire,
    /// A stop event ended it
    Stop,
    /// The updates stopped arriving without an end in the recording
    Unknown,
}

/// A gauss charge or egon beam of one actor
#[derive(Clone, Debug, Serialize)]
pub struct ActivityInterval {
    pub weapon: Weapon,
    pub segment: u32,
    pub actor: Actor,
    pub start: f32,
    /// Whether the record that starts it was seen, rather than a later update
    pub start_known: bool,
    pub end: f32,
    pub end_reason: IntervalEnd,
    /// First and last recorded event, and how many it spans
    pub first_event: usize,
    pub last_event: usize,
    pub events: usize,
}

/// Updates of an ongoing charge or beam come every frame or so. A longer
/// silence means the recording lost sight of it
const ACTIVITY_GAP: f32 = 1.0;

/// Weapon actions and activity intervals of a recording
pub struct Derived {
    pub occurrences: Vec<WeaponOccurrence>,
    pub intervals: Vec<ActivityInterval>,
    /// Event scripts the profile does not know, with their record counts
    pub unclassified: Vec<(String, usize)>,
}

pub fn derive(events: &[RecordedEvent], max_clients: u8) -> Derived {
    let mut occurrences = Vec::new();
    let mut intervals: Vec<ActivityInterval> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    let mut unclassified: Vec<(String, usize)> = Vec::new();

    for e in events {
        let Some((weapon, action, mode)) = classify(e) else {
            let name = e
                .resource
                .clone()
                .unwrap_or_else(|| format!("#{}", e.index));
            match unclassified.iter_mut().find(|(n, _)| *n == name) {
                Some(entry) => entry.1 += 1,
                None => unclassified.push((name, 1)),
            }
            continue;
        };

        let (who, attribution) = actor(e, max_clients);
        let key = (weapon, e.segment, actor_entity(&who));
        let same = |i: &usize| {
            let iv = &intervals[*i];
            (iv.weapon, iv.segment, actor_entity(&iv.actor)) == key
        };

        // A silence ends the open interval with an unknown end
        if let Some(pos) = open.iter().position(same) {
            let iv = &intervals[open[pos]];
            if e.time - iv.end > ACTIVITY_GAP {
                open.remove(pos);
            }
        }
        let current = open.iter().position(same);

        let starts = matches!(
            (weapon, action),
            (Weapon::Gauss, Action::ChargeActivity) | (Weapon::Egon, Action::BeamStart)
        ) && (weapon != Weapon::Gauss || e.args.bparam1 == 0);
        let extends = action.is_activity() || action == Action::BeamStart;
        let ends = match (weapon, action) {
            (Weapon::Gauss, Action::Fire) | (Weapon::Egon, Action::BeamStop) => {
                Some(if action == Action::Fire {
                    IntervalEnd::Fire
                } else {
                    IntervalEnd::Stop
                })
            }
            (Weapon::Gauss, Action::ChargeStop) => Some(IntervalEnd::Stop),
            _ => None,
        };

        if extends {
            let fresh = starts || current.is_none();
            if fresh {
                if let Some(pos) = current {
                    open.remove(pos);
                }
                intervals.push(ActivityInterval {
                    weapon,
                    segment: e.segment,
                    actor: who.clone(),
                    start: e.time,
                    start_known: starts,
                    end: e.time,
                    end_reason: IntervalEnd::Unknown,
                    first_event: e.id,
                    last_event: e.id,
                    events: 1,
                });
                open.push(intervals.len() - 1);
            } else if let Some(pos) = current {
                let iv = &mut intervals[open[pos]];
                iv.end = e.time;
                iv.last_event = e.id;
                iv.events += 1;
            }
        } else if let (Some(reason), Some(pos)) = (ends, current) {
            let iv = &mut intervals[open[pos]];
            iv.end = e.time;
            iv.end_reason = reason;
            iv.last_event = e.id;
            iv.events += 1;
            open.remove(pos);
        }

        if action.is_activity() {
            continue;
        }

        occurrences.push(WeaponOccurrence {
            id: occurrences.len(),
            segment: e.segment,
            time: e.time,
            server_time: e.server_time,
            weapon,
            action,
            mode,
            shot: action.is_shot(),
            actor: who,
            attribution,
            evidence: match e.source {
                EventSource::Demo => Evidence::LocalEvent,
                EventSource::Svc => Evidence::NetworkEvent,
                EventSource::SvcReliable => Evidence::ReliableNetworkEvent,
            },
            event: e.clone(),
        });
    }

    Derived {
        occurrences,
        intervals,
        unclassified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::events::EventArgs;

    fn event(id: usize, time: f32, script: &str, bparam1: i32, bparam2: i32) -> RecordedEvent {
        RecordedEvent {
            id,
            segment: 1,
            entry: 1,
            ordinal: id as u64,
            time,
            server_time: time,
            source: EventSource::Demo,
            message_ordinal: None,
            index: 0,
            resource: Some(format!("events/{script}")),
            dispatch_flags: Some(0),
            args: EventArgs {
                entindex: 1,
                bparam1,
                bparam2,
                ..Default::default()
            },
            delay: Some(0.),
            actor: ActorResolution::Explicit { entity: 1 },
            player: None,
        }
    }

    #[test]
    fn gauss_stops_are_not_shots() {
        // A charged shot, its stop 10 ms later, a primary shot and a
        // holster stop without a shot
        let events = [
            event(0, 1.0, "gauss.sc", 0, 0),
            event(1, 1.01, "gauss.sc", 0, 1),
            event(2, 2.0, "gauss.sc", 1, 0),
            event(3, 3.0, "gauss.sc", 0, 1),
        ];
        let d = derive(&events, 2);

        let actions: Vec<_> = d
            .occurrences
            .iter()
            .map(|o| (o.action, o.mode, o.shot))
            .collect();
        assert_eq!(
            actions,
            [
                (Action::Fire, Some("charged"), true),
                (Action::ChargeStop, None, false),
                (Action::Fire, Some("primary"), true),
                (Action::ChargeStop, None, false),
            ]
        );
    }

    #[test]
    fn a_charge_runs_from_its_first_spin_to_the_shot() {
        let mut events = vec![event(0, 1.0, "gaussspin.sc", 0, 0)];
        for i in 1..10 {
            events.push(event(i, 1.0 + i as f32 * 0.05, "gaussspin.sc", 1, 0));
        }
        events.push(event(10, 1.6, "gauss.sc", 0, 0));
        let d = derive(&events, 2);

        assert_eq!(d.intervals.len(), 1);
        let iv = &d.intervals[0];
        assert!(iv.start_known);
        assert_eq!(
            (iv.start, iv.end, iv.end_reason),
            (1.0, 1.6, IntervalEnd::Fire)
        );
        assert_eq!((iv.first_event, iv.last_event, iv.events), (0, 10, 11));

        // Spin updates are intervals, not listed actions
        assert_eq!(d.occurrences.len(), 1);
    }

    #[test]
    fn unseen_charge_ends_and_starts_stay_unknown() {
        // Updates without the starting spin, then a silence, then a new
        // charge that the recording ends during
        let events = [
            event(0, 1.0, "gaussspin.sc", 1, 0),
            event(1, 1.1, "gaussspin.sc", 1, 0),
            event(2, 5.0, "gaussspin.sc", 0, 0),
            event(3, 5.1, "gaussspin.sc", 1, 0),
        ];
        let d = derive(&events, 2);

        let shape: Vec<_> = d
            .intervals
            .iter()
            .map(|i| (i.start, i.start_known, i.end, i.end_reason))
            .collect();
        assert_eq!(
            shape,
            [
                (1.0, false, 1.1, IntervalEnd::Unknown),
                (5.0, true, 5.1, IntervalEnd::Unknown),
            ]
        );
    }

    #[test]
    fn actors_without_a_known_occupant_are_labelled() {
        let mut unknown = event(0, 1.0, "rpg.sc", 0, 0);
        unknown.actor = ActorResolution::Unknown;
        let mut monster = event(1, 1.0, "rpg.sc", 0, 0);
        monster.actor = ActorResolution::Explicit { entity: 40 };
        let player = event(2, 1.0, "rpg.sc", 0, 0);

        let d = derive(&[unknown, monster, player], 2);
        let actors: Vec<_> = d.occurrences.iter().map(|o| o.actor.clone()).collect();
        assert_eq!(
            actors,
            [
                Actor::Unknown,
                Actor::Entity { entity: 40 },
                Actor::UnknownPlayer { entity: 1 },
            ]
        );
    }

    #[test]
    fn attempts_and_swings_are_not_shots() {
        let events = [
            event(0, 1.0, "crowbar.sc", 0, 0),
            event(1, 2.0, "tripfire.sc", 0, 0),
            event(2, 3.0, "snarkfire.sc", 0, 0),
            event(3, 4.0, "rpg.sc", 0, 0),
            event(4, 5.0, "unknown.sc", 0, 0),
        ];
        let d = derive(&events, 2);

        let shots: Vec<_> = d.occurrences.iter().map(|o| (o.weapon, o.shot)).collect();
        assert_eq!(
            shots,
            [
                (Weapon::Crowbar, false),
                (Weapon::Tripmine, false),
                (Weapon::Snark, false),
                (Weapon::Rpg, true),
            ]
        );
        assert_eq!(d.unclassified, [("events/unknown.sc".to_string(), 1)]);
    }
}
