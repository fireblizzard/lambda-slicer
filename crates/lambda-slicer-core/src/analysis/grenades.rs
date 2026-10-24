//! Which weapon a `grenade` kill came from
//!
//! The mp5's grenade and the hand grenade both kill as `grenade`, the
//! class name of either explosive. Three signs tell them apart, the first
//! that tells deciding:
//!
//! 1. The grenade in flight, when it came into view. A thrown hand grenade
//!    is `models/w_grenade.mdl` and the mp5's is `models/grenade.mdl`,
//!    each owned by its player and bouncing. A hand grenade lying as a
//!    pickup has the same model, but no owner and tossed movement
//! 2. The mp5's launch, `events/mp52.sc`, which links the kill to it when
//!    the killer's launch reached the recording
//! 3. What the killer held, the weapon model `models/p_grenade.mdl` or
//!    `models/p_9mmAR.mdl` in their entity's `weaponmodel`. A hand
//!    grenade's fuse lasts three seconds from the pin, so a killer seen
//!    holding one shortly before threw it, and a killer seen holding only
//!    the mp5 all along launched its grenade
//!
//! Anything else, such as a killer and grenade out of view, stays unknown

use std::collections::{BTreeMap, HashMap};

use super::context::{Kill, KillCause};
use super::weapons::Weapon;
use crate::state::Decoder;

/// How long before a kill the grenade behind it was in hand, in seconds:
/// a hand grenade's three second fuse and the throw, or the mp5 grenade's
/// longest flight
const IN_HAND: f32 = 4.;

/// `MOVETYPE_BOUNCE`, how a thrown or launched grenade moves
const BOUNCE: i64 = 10;

/// How long a grenade may go unseen and still be the same one in flight,
/// as an entity's number is used again for something else later
const FLIGHT_GAP: f32 = 1.;

/// A grenade in flight, seen from `first` to `last`
#[derive(Clone, Copy, Debug, PartialEq)]
struct Flight {
    segment: u32,
    /// The player who threw or launched it
    owner: u16,
    weapon: Weapon,
    first: f32,
    last: f32,
}

/// The weapon whose grenade the model at `path` is, none for any other
fn thrown(path: &str) -> Option<Weapon> {
    let name = path.to_ascii_lowercase();
    match name.strip_prefix("models/").unwrap_or(&name) {
        "w_grenade.mdl" => Some(Weapon::Handgrenade),
        "grenade.mdl" => Some(Weapon::Mp5),
        _ => None,
    }
}

/// What a player was seen holding, of the two weapons a grenade comes from
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holding {
    HandGrenade,
    Mp5,
    /// Any other weapon, or none
    Other,
    /// Out of the recording's view, so not known
    Unseen,
}

/// The weapon model at `path` as one of the two a grenade comes from
fn holding(path: &str) -> Holding {
    let name = path.to_ascii_lowercase();
    match name.strip_prefix("models/").unwrap_or(&name) {
        "p_grenade.mdl" => Holding::HandGrenade,
        "p_9mmar.mdl" => Holding::Mp5,
        _ => Holding::Other,
    }
}

/// What each player held, followed through the decode
#[derive(Debug, Default)]
pub struct Held {
    /// Each change by session and entity, in time order
    changes: BTreeMap<(u32, u16), Vec<(f32, Holding)>>,
    /// Weapon models by model index, for the session's models
    models: HashMap<i64, Holding>,
    /// Grenades in flight, and the grenade models by model index
    flights: Vec<Flight>,
    in_flight: HashMap<(u32, u16), usize>,
    grenade_models: HashMap<i64, Option<Weapon>>,
    /// The packet last looked at, once a packet
    last: Option<(u32, f32)>,
}

impl Held {
    /// A new session, whose models are numbered anew
    pub fn new_session(&mut self) {
        self.models.clear();
        self.grenade_models.clear();
    }

    /// The players in the latest snapshot at `time`, those missing from it
    /// as unseen, and the grenades in flight in it
    pub fn observe(&mut self, d: &Decoder, time: f32) {
        let now = (d.segment, time);
        if self.last == Some(now) {
            return;
        }
        self.last = Some(now);
        let (Some(snap), Some(schema)) = (d.latest_snapshot(), d.schema("entity_state_player_t"))
        else {
            return;
        };
        let Some(entities) = &snap.entities else {
            return;
        };

        let mut seen = Vec::new();
        for e in entities.iter() {
            if e.number < 1 || e.number > u16::from(d.max_clients) {
                continue;
            }
            let index = e
                .fields
                .value(&schema, "weaponmodel")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let now_holding = *self.models.entry(index).or_insert_with(|| {
                let resource = d
                    .resources()
                    .iter()
                    .find(|r| r.kind == 2 && i64::from(r.index) == index);
                resource.map_or(Holding::Other, |r| {
                    holding(&String::from_utf8_lossy(&r.name))
                })
            });
            self.change(d.segment, e.number, time, now_holding);
            seen.push(e.number);
        }

        // Players of this session gone from view
        let mut gone = Vec::new();
        for (&(segment, entity), list) in &self.changes {
            let shown = list.last().is_some_and(|c| c.1 != Holding::Unseen);
            if segment == d.segment && shown && !seen.contains(&entity) {
                gone.push(entity);
            }
        }
        for entity in gone {
            self.change(d.segment, entity, time, Holding::Unseen);
        }

        self.grenades(d, time);
    }

    /// The grenades in flight in the latest snapshot at `time`, each with
    /// the player who threw or launched it
    fn grenades(&mut self, d: &Decoder, time: f32) {
        let Some(entities) = d.latest_snapshot().and_then(|s| s.entities.as_ref()) else {
            return;
        };
        for e in entities.iter() {
            if e.number <= u16::from(d.max_clients) {
                continue;
            }
            let Some(schema) = d.entity_schema(e.number, e.custom) else {
                continue;
            };
            let field = |name: &str| e.fields.value(&schema, name).and_then(|v| v.as_i64());
            let index = field("modelindex").unwrap_or(0);
            let weapon = *self.grenade_models.entry(index).or_insert_with(|| {
                let resource = d
                    .resources()
                    .iter()
                    .find(|r| r.kind == 2 && i64::from(r.index) == index);
                resource.and_then(|r| thrown(&String::from_utf8_lossy(&r.name)))
            });
            let Some(weapon) = weapon else {
                continue;
            };

            // A pickup lying still has no owner
            let owner = field("owner").unwrap_or(0);
            let owned = owner >= 1 && owner <= i64::from(d.max_clients);
            if !owned || field("movetype") != Some(BOUNCE) {
                continue;
            }
            self.fly(d.segment, e.number, owner as u16, weapon, time);
        }
    }

    /// The grenade `entity` seen at `time`, the one seen before under that
    /// number when it is the same, and a new one otherwise
    fn fly(&mut self, segment: u32, entity: u16, owner: u16, weapon: Weapon, time: f32) {
        if let Some(&k) = self.in_flight.get(&(segment, entity)) {
            let f = &mut self.flights[k];
            let same = f.owner == owner && f.weapon == weapon && time - f.last <= FLIGHT_GAP;
            if same && time >= f.last {
                f.last = time;
                return;
            }
        }
        self.in_flight.insert((segment, entity), self.flights.len());
        self.flights.push(Flight {
            segment,
            owner,
            weapon,
            first: time,
            last: time,
        });
    }

    /// The weapons of the grenades `owner` had in flight from `from` to
    /// `to` in `segment`, each once
    fn flying(&self, segment: u32, owner: u16, from: f32, to: f32) -> Vec<Weapon> {
        let mut weapons = Vec::new();
        for f in &self.flights {
            let during = f.first <= to && f.last >= from;
            if f.segment == segment && f.owner == owner && during && !weapons.contains(&f.weapon) {
                weapons.push(f.weapon);
            }
        }
        weapons
    }

    /// `entity` held `now` from `time`, kept when it differs from before
    fn change(&mut self, segment: u32, entity: u16, time: f32, now: Holding) {
        let list = self.changes.entry((segment, entity)).or_default();
        if list.last().is_none_or(|c| c.1 != now) {
            list.push((time, now));
        }
    }

    /// What `entity` held from `from` to `to` in `segment`, starting with
    /// what it held as `from` began, unseen before it was first seen
    fn between(&self, segment: u32, entity: u16, from: f32, to: f32) -> Vec<Holding> {
        let Some(list) = self.changes.get(&(segment, entity)) else {
            return vec![Holding::Unseen];
        };
        let mut out = Vec::new();
        let mut before = Holding::Unseen;
        for &(time, holding) in list {
            if time > to {
                break;
            }
            if time <= from {
                before = holding;
            } else {
                out.push(holding);
            }
        }
        out.insert(0, before);
        out
    }
}

/// The weapon a `grenade` kill came from, by the killer's grenades in
/// flight in the seconds before when only one kind was, then by a launch
/// of the mp5 linked to the kill, then by what the killer held, none when
/// none of these tells
fn source(kill: &Kill, held: &Held) -> Option<Weapon> {
    let killer = u16::from(kill.killer_entity);
    let from = kill.time - IN_HAND;
    let flying = held.flying(kill.segment, killer, from, kill.time);
    if let [weapon] = flying.as_slice() {
        return Some(*weapon);
    }

    let launched = kill
        .context
        .as_ref()
        .and_then(|c| c.shot.as_ref())
        .is_some_and(|s| s.weapon == Weapon::Mp5);
    if launched {
        return Some(Weapon::Mp5);
    }

    let held = held.between(kill.segment, killer, from, kill.time);
    if held.contains(&Holding::HandGrenade) {
        return Some(Weapon::Handgrenade);
    }
    let seen_all_along = !held.contains(&Holding::Unseen);
    (seen_all_along && held.contains(&Holding::Mp5)).then_some(Weapon::Mp5)
}

/// Give each `grenade` kill by a player the weapon it came from, when the
/// recording tells. Run once kills are linked to their shots
pub fn tell_grenades(kills: &mut [Kill], held: &Held) {
    for kill in kills.iter_mut() {
        let by_player = matches!(kill.cause, KillCause::PlayerOther | KillCause::Suicide);
        if !by_player || kill.weapon_name != "grenade" || kill.weapon.is_some() {
            continue;
        }
        kill.weapon = source(kill, held);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(changes: &[(f32, Holding)]) -> Held {
        let mut held = Held::default();
        for &(time, holding) in changes {
            held.change(1, 2, time, holding);
        }
        held
    }

    #[test]
    fn weapon_models_name_the_two_grenade_weapons() {
        assert_eq!(holding("models/p_grenade.mdl"), Holding::HandGrenade);
        assert_eq!(holding("models/p_9mmAR.mdl"), Holding::Mp5);
        assert_eq!(holding("models/p_crowbar.mdl"), Holding::Other);
    }

    #[test]
    fn a_grenade_in_flight_is_one_until_its_number_goes_unseen_too_long() {
        let mut h = Held::default();
        h.fly(1, 300, 2, Weapon::Handgrenade, 10.);
        h.fly(1, 300, 2, Weapon::Handgrenade, 10.5);
        h.fly(1, 300, 2, Weapon::Handgrenade, 12.);
        assert_eq!(h.flights.len(), 2);

        // Flying while the window lasts, and only the thrower's
        assert_eq!(h.flying(1, 2, 10.4, 11.), [Weapon::Handgrenade]);
        assert_eq!(h.flying(1, 2, 10.6, 11.9), []);
        assert_eq!(h.flying(1, 3, 9., 13.), []);

        // Both kinds at once tell nothing on their own
        h.fly(1, 301, 2, Weapon::Mp5, 12.2);
        assert_eq!(h.flying(1, 2, 11.5, 13.).len(), 2);
    }

    #[test]
    fn grenade_models_name_their_weapon() {
        assert_eq!(thrown("models/w_grenade.mdl"), Some(Weapon::Handgrenade));
        assert_eq!(thrown("models/grenade.mdl"), Some(Weapon::Mp5));
        assert_eq!(thrown("models/w_battery.mdl"), None);
    }

    #[test]
    fn what_was_held_counts_from_before_the_window() {
        let h = held(&[
            (10., Holding::Mp5),
            (20., Holding::HandGrenade),
            (21., Holding::Unseen),
        ]);
        assert_eq!(h.between(1, 2, 15., 19.), [Holding::Mp5]);
        assert_eq!(
            h.between(1, 2, 18., 22.),
            [Holding::Mp5, Holding::HandGrenade, Holding::Unseen]
        );
        assert_eq!(h.between(1, 3, 18., 22.), [Holding::Unseen]);
    }
}
