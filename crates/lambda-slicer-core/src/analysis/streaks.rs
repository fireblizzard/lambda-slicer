//! Kills made together and kills made in a row
//!
//! Kills of other players by one killer with one weapon in the same server
//! frame are one shot, as a blast kills everyone it reaches at once and a
//! Gauss beam everyone it goes through. A player's killing spree is their
//! kills of other players in one session without dying, so any death ends
//! it, a suicide too, and so does the session's end or their leaving. Team
//! kills neither add to a spree nor end it, and a single kill is no spree

use std::collections::HashMap;

use super::context::{Kill, KillCause};

/// The fewest kills a spree has to be counted
const SPREE_LEAST: u32 = 2;

/// Whether `kill` is a player's kill of another player
fn by_player(kill: &Kill) -> bool {
    matches!(
        kill.cause,
        KillCause::PlayerWeapon | KillCause::PlayerOther | KillCause::MountedGun
    )
}

/// The shot that made `kill`, by session, killer, weapon and server frame,
/// none for a kill no player made of another
fn shot_of(kill: &Kill) -> Option<(u32, u8, u32, &str, u32)> {
    let killer = kill.killer.as_ref().filter(|_| by_player(kill))?;
    let frame = kill.server_time.to_bits();
    Some((
        kill.segment,
        killer.slot,
        killer.incarnation,
        &kill.weapon_name,
        frame,
    ))
}

/// How many players the shot that made each of `kills` killed, in their
/// order, 1 for a kill on its own
pub fn kills_at_once(kills: &[Kill]) -> Vec<u32> {
    let mut counts: HashMap<_, u32> = HashMap::new();
    for kill in kills {
        if let Some(shot) = shot_of(kill) {
            *counts.entry(shot).or_default() += 1;
        }
    }

    let mut out = Vec::new();
    for kill in kills {
        let count = shot_of(kill).map_or(1, |shot| counts[&shot]);
        out.push(count);
    }
    out
}

/// How many kills the killing spree each of `kills` ends had, in their
/// order, the spree's last kill, and 0 for every other kill. `kills` are in
/// the order they happened, a session after another
pub fn spree_ends(kills: &[Kill]) -> Vec<u32> {
    let mut out = vec![0; kills.len()];
    let mut keep = |(count, last): (u32, usize)| {
        if count >= SPREE_LEAST {
            out[last] = count;
        }
    };

    // Each player's kills so far and the place of the last, by slot and
    // connection, until their session ends
    let mut going: HashMap<(u8, u32), (u32, usize)> = HashMap::new();
    let mut segment = None;
    for (index, kill) in kills.iter().enumerate() {
        if segment != Some(kill.segment) {
            for spree in going.drain().map(|(_, spree)| spree) {
                keep(spree);
            }
            segment = Some(kill.segment);
        }

        if by_player(kill)
            && let Some(killer) = &kill.killer
        {
            let spree = going
                .entry((killer.slot, killer.incarnation))
                .or_insert((0, index));
            spree.0 += 1;
            spree.1 = index;
        }
        let victim = kill.victim.as_ref().map(|v| (v.slot, v.incarnation));
        if let Some(spree) = victim.and_then(|v| going.remove(&v)) {
            keep(spree);
        }
    }

    // Sprees still going when the last session ends, or their player left
    for spree in going.into_values() {
        keep(spree);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::motion::MotionState;
    use crate::state::PlayerIdentity;

    fn player(slot: u8) -> PlayerIdentity {
        PlayerIdentity {
            segment: 0,
            slot,
            user_id: u32::from(slot),
            incarnation: 0,
            name: format!("p{slot}"),
        }
    }

    /// A kill in `segment` at `time` of `victim` by `killer`, by slot, with
    /// `weapon`
    fn kill(segment: u32, time: f32, (killer, victim): (u8, u8), weapon: &str) -> Kill {
        Kill {
            segment,
            time,
            server_time: time,
            killer_entity: killer + 1,
            victim_entity: victim + 1,
            killer: Some(player(killer)),
            victim: Some(player(victim)),
            killer_team: None,
            victim_team: None,
            weapon_name: weapon.into(),
            cause: KillCause::PlayerWeapon,
            weapon: None,
            victim_state: MotionState::not_visible(),
            killer_samples: Vec::new(),
            map: None,
            context: None,
            headshot: None,
        }
    }

    #[test]
    fn kills_of_one_shot_count_together() {
        let suicide = Kill {
            cause: KillCause::Suicide,
            ..kill(0, 5., (0, 0), "rpg_rocket")
        };
        let kills = [
            kill(0, 5., (0, 1), "rpg_rocket"),
            kill(0, 5., (0, 2), "rpg_rocket"),
            suicide,
            kill(0, 5., (0, 3), "gauss"),
            kill(0, 5., (4, 1), "rpg_rocket"),
            kill(0, 6., (0, 1), "rpg_rocket"),
            kill(0, 6., (0, 2), "rpg_rocket"),
            kill(0, 6., (0, 3), "rpg_rocket"),
        ];

        // The rocket's two kills at 5 seconds, not the killer's own death,
        // another weapon's or another killer's, and three at 6 seconds
        assert_eq!(kills_at_once(&kills), [2, 2, 1, 1, 1, 3, 3, 3]);
    }

    #[test]
    fn a_spree_ends_with_a_death_and_counts_kills_of_others() {
        let teamkill = Kill {
            cause: KillCause::Teammate,
            ..kill(0, 3., (1, 4), "crossbow")
        };
        let suicide = Kill {
            cause: KillCause::Suicide,
            ..kill(0, 7., (1, 1), "crossbow")
        };
        let kills = [
            kill(0, 1., (1, 2), "crossbow"),
            kill(0, 2., (1, 3), "crossbow"),
            teamkill,
            kill(0, 4., (1, 2), "crossbow"),
            kill(0, 5., (2, 1), "crossbow"),
            kill(0, 6., (1, 3), "crossbow"),
            suicide,
            kill(0, 8., (3, 2), "crossbow"),
            kill(0, 9., (3, 4), "crossbow"),
            kill(1, 1., (3, 2), "crossbow"),
        ];

        // Three kills ended by a death, the team kill not counted, one
        // kill ended by a suicide and too short, and two kills still going
        // as the session ends, the next session's starting afresh
        assert_eq!(spree_ends(&kills), [0, 0, 0, 3, 0, 0, 0, 0, 2, 0]);
    }
}
