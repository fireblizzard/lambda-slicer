//! Shot accuracy from the blood a hit leaves
//!
//! Demos record no hits. A hit through `TraceAttack` makes the server send a
//! blood sprite (`TE_BLOODSPRITE`) at the impact point to clients that can
//! see it, so a shot counts as a hit when blood appears along its aim line a
//! moment after it. The recording player's aim is exact, from its own events.
//! Other players' events carry the model's pitch, a third of the view pitch,
//! so their aim is rebuilt from the snapshot pose at the shot.
//!
//! Several players shooting one target put the same blood on more than one
//! aim line, so some matches are chance. Shifting every blood sprite 1.5 s
//! later and matching again measures that chance, and accuracy subtracts it.
//! Explosions, gauss reflections and punch-through, and hornets deal damage
//! without blood, so only direct hits count. Unzoomed crossbow bolts fly too
//! long to match and are left out

use std::collections::BTreeMap;

use serde::Serialize;

use super::weapons::{Actor, Weapon, WeaponOccurrence};

pub const TE_BLOODSPRITE: u8 = 115;

/// Shots a player needs with a known aim before their accuracy is shown
pub const MIN_SHOTS: u32 = 50;

/// How far blood is looked for after a shot, and before it for packet order
const AFTER: f32 = 0.25;
const BEFORE: f32 = 0.05;
/// The control shift, well beyond any shot's own blood
const CONTROL_SHIFT: f32 = 1.5;
/// Eye height above a standing and a crouching player's origin
const EYE: f32 = 28.;
const EYE_DUCKED: f32 = 12.;

/// Where blood appeared
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blood {
    pub segment: u32,
    pub time: f32,
    pub origin: [f32; 3],
}

/// A player's position and model angles in one snapshot
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub time: f32,
    pub origin: [f32; 3],
    /// Model pitch, a third of the view pitch with the sign flipped, and yaw
    pub pitch: f32,
    pub yaw: f32,
}

/// The blood sprite of a `TE_BLOODSPRITE` body
pub fn blood_origin(body: &[u8]) -> Option<[f32; 3]> {
    let c = |at: usize| {
        body.get(at..at + 2)
            .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 8.)
    };
    Some([c(0)?, c(2)?, c(4)?])
}

/// One shot's outcome
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ShotOutcome {
    pub segment: u32,
    pub time: f32,
    pub slot: u8,
    pub incarnation: u32,
    /// Blood appeared along the aim
    pub hit: bool,
    /// Blood shifted by the control still matched, a chance match
    pub chance: bool,
    /// The recording player's exact aim rather than a rebuilt one
    pub exact: bool,
}

/// A player's accuracy over a demo
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlayerAccuracy {
    pub name: String,
    pub slot: u8,
    pub incarnation: u32,
    pub segment: u32,
    /// Shots whose aim is known, those with blood along the aim, and chance
    /// matches
    pub shots: u32,
    pub hits: u32,
    pub chance: u32,
    /// The recording player's own aim, exact. Other players' aim is rebuilt
    pub exact: bool,
}

/// Hits less chance matches over shots, as a fraction, when there are
/// enough shots to tell
pub fn accuracy(shots: u32, hits: u32, chance: u32) -> Option<f64> {
    (shots >= MIN_SHOTS)
        .then(|| (f64::from(hits.saturating_sub(chance)) / f64::from(shots)).clamp(0., 1.))
}

impl PlayerAccuracy {
    pub fn value(&self) -> Option<f64> {
        accuracy(self.shots, self.hits, self.chance)
    }
}

fn counts(w: &WeaponOccurrence) -> bool {
    match w.weapon {
        Weapon::Crossbow => w.mode != Some("unzoomed"),
        Weapon::Glock | Weapon::Python | Weapon::Shotgun | Weapon::Gauss => true,
        Weapon::Mp5 => w.mode != Some("grenade"),
        _ => false,
    }
}

/// The widest angle off the aim line a hit lands at, in degrees
fn spread(w: Weapon) -> f32 {
    match w {
        Weapon::Shotgun => 12.,
        Weapon::Mp5 | Weapon::Glock => 6.,
        _ => 3.,
    }
}

fn forward(pitch: f32, yaw: f32) -> [f32; 3] {
    let (p, y) = (pitch.to_radians(), yaw.to_radians());
    [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()]
}

/// Whether blood within the time window lies along the aim
fn matches(
    blood: &[Blood],
    segment: u32,
    time: f32,
    shift: f32,
    eye: [f32; 3],
    dir: [f32; 3],
    tan: f32,
) -> bool {
    let from = blood.partition_point(|b| (b.segment, b.time + shift) < (segment, time - BEFORE));
    blood[from..]
        .iter()
        .take_while(|b| b.segment == segment && b.time + shift <= time + AFTER)
        .any(|b| {
            let v = [
                b.origin[0] - eye[0],
                b.origin[1] - eye[1],
                b.origin[2] - eye[2],
            ];
            let along = v[0] * dir[0] + v[1] * dir[1] + v[2] * dir[2];
            if along < 16. {
                return false;
            }
            let perp = [
                v[0] - along * dir[0],
                v[1] - along * dir[1],
                v[2] - along * dir[2],
            ];
            (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt() <= 16. + along * tan
        })
}

/// Every counted shot's outcome, and each player's accuracy. `blood` is in
/// decode order. `poses` holds each player entity's snapshot poses in time
/// order, by segment and entity
pub fn assess(
    occurrences: &[WeaponOccurrence],
    blood: &[Blood],
    poses: &BTreeMap<(u32, u16), Vec<Pose>>,
    recorder_entity: Option<u16>,
) -> (Vec<ShotOutcome>, Vec<PlayerAccuracy>) {
    let mut outcomes = Vec::new();
    let mut players: Vec<PlayerAccuracy> = Vec::new();
    for o in occurrences.iter().filter(|o| o.shot && counts(o)) {
        let Actor::Player { entity, identity } = &o.actor else {
            continue;
        };
        let entity = *entity as u16;
        let exact = Some(entity) == recorder_entity;
        let aim = if exact {
            let a = &o.event.args;
            let eye = if a.ducking != 0 { EYE_DUCKED } else { EYE };
            Some((
                [a.origin[0], a.origin[1], a.origin[2] + eye],
                forward(a.angles[0], a.angles[1]),
            ))
        } else {
            poses.get(&(o.segment, entity)).and_then(|list| {
                let at = list.partition_point(|p| p.time <= o.time);
                let p = list[at.checked_sub(1)?];
                (o.time - p.time <= 0.1).then(|| {
                    (
                        [p.origin[0], p.origin[1], p.origin[2] + EYE],
                        forward(-3. * p.pitch, p.yaw),
                    )
                })
            })
        };
        let Some((eye, dir)) = aim else {
            continue;
        };
        let tan = spread(o.weapon).to_radians().tan();
        let hit = matches(blood, o.segment, o.time, 0., eye, dir, tan);
        let chance = matches(blood, o.segment, o.time, CONTROL_SHIFT, eye, dir, tan);
        outcomes.push(ShotOutcome {
            segment: o.segment,
            time: o.time,
            slot: identity.slot,
            incarnation: identity.incarnation,
            hit,
            chance,
            exact,
        });

        let at = match players.iter().position(|p| {
            (p.segment, p.slot, p.incarnation) == (o.segment, identity.slot, identity.incarnation)
        }) {
            Some(at) => at,
            None => {
                players.push(PlayerAccuracy {
                    name: identity.name.clone(),
                    slot: identity.slot,
                    incarnation: identity.incarnation,
                    segment: o.segment,
                    shots: 0,
                    hits: 0,
                    chance: 0,
                    exact,
                });
                players.len() - 1
            }
        };
        let p = &mut players[at];
        p.shots += 1;
        p.hits += u32::from(hit);
        p.chance += u32::from(chance);
    }
    (outcomes, players)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blood_along_the_aim_matches() {
        let blood = [Blood {
            segment: 1,
            time: 10.05,
            origin: [500., 10., 28.],
        }];
        let eye = [0., 0., 28.];
        let dir = forward(0., 0.);
        let tan = 3f32.to_radians().tan();
        assert!(matches(&blood, 1, 10., 0., eye, dir, tan));
        // Too far off the line, too late, or another session
        assert!(!matches(&blood, 1, 10., 0., eye, forward(0., 20.), tan));
        assert!(!matches(&blood, 1, 9., 0., eye, dir, tan));
        assert!(!matches(&blood, 2, 10., 0., eye, dir, tan));
        // Behind the shooter
        assert!(!matches(&blood, 1, 10., 0., eye, forward(0., 180.), tan));
    }

    #[test]
    fn accuracy_subtracts_chance_and_needs_enough_shots() {
        assert_eq!(accuracy(100, 45, 7), Some(0.38));
        assert_eq!(accuracy(49, 40, 0), None);
        assert_eq!(accuracy(100, 3, 7), Some(0.));
    }
}
