//! Highlights for finding interesting moments, such as midair or fast kills
//! and stretches packed with kills on both sides
//!
//! Each kill by a player is linked to the shot that most likely caused it,
//! the killer's latest shot with that weapon inside the weapon's flight
//! time. The killer's movement is taken at that shot and the victim's at
//! the kill. Action windows score each stretch of play with the formula
//! `a^(3/4) + b^(3/4) - max(|a - b|^(2/5), 1)` over the frags of the two
//! busiest sides, so even exchanges outscore one-sided runs, with a bonus
//! for using more weapons

use serde::Serialize;

use super::context::{Kill, KillCause};
use super::motion::{MotionState, state_at};
use super::weapons::{Action, Actor, Weapon, WeaponOccurrence};

/// The shot a kill was linked to
#[derive(Clone, Debug, Serialize)]
pub struct LinkedShot {
    /// Index into the report's occurrences
    pub occurrence: usize,
    pub weapon: Weapon,
    pub action: Action,
    pub mode: Option<&'static str>,
    pub time: f32,
    /// Seconds from the shot to the kill, by server time
    pub flight: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct KillContext {
    /// The killer at the linked shot, or at the kill when no shot was found
    pub killer: MotionState,
    /// The victim at the kill
    pub victim: MotionState,
    pub shot: Option<LinkedShot>,
    /// Killer at the shot to victim at the kill, in units
    pub distance: Option<f32>,
}

/// Longest plausible time from a shot to the kill it caused
fn flight_limit(weapon: Weapon, action: Action, mode: Option<&str>) -> f32 {
    match (weapon, action, mode) {
        (Weapon::Crossbow, _, Some("unzoomed")) => 3.,
        (Weapon::Rpg, _, _) => 6.,
        (Weapon::Mp5, Action::Launch, _) => 4.,
        (Weapon::Hivehand, _, _) => 3.,
        _ => 0.3,
    }
}

/// Whether an action can be the shot behind a kill credited to `name`
fn may_cause(o: &WeaponOccurrence, kill: &Kill) -> bool {
    match (kill.cause, kill.weapon) {
        // `grenade` from a player is the mp5's when a launch fits, while a
        // hand grenade leaves no event
        (KillCause::PlayerOther | KillCause::Suicide, None) if kill.weapon_name == "grenade" => {
            o.weapon == Weapon::Mp5 && o.action == Action::Launch
        }
        (_, Some(Weapon::Mp5)) => o.weapon == Weapon::Mp5 && o.action == Action::Fire,
        (_, Some(Weapon::Crowbar)) => o.weapon == Weapon::Crowbar,
        (_, Some(w)) => o.weapon == w && o.shot,
        _ => false,
    }
}

fn same_player(o: &WeaponOccurrence, kill: &Kill) -> bool {
    let (Actor::Player { identity, .. }, Some(killer)) = (&o.actor, &kill.killer) else {
        return false;
    };

    (identity.segment, identity.slot, identity.incarnation)
        == (killer.segment, killer.slot, killer.incarnation)
}

fn distance(a: Option<[f32; 3]>, b: Option<[f32; 3]>) -> Option<f32> {
    let (a, b) = (a?, b?);

    Some(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt())
}

/// Fill each kill's context. `gravity` is the server's
pub fn link_kills(kills: &mut [Kill], occurrences: &[WeaponOccurrence], gravity: f32) {
    for kill in kills.iter_mut() {
        let shot = occurrences
            .iter()
            .filter(|o| o.segment == kill.segment && o.server_time <= kill.server_time)
            .filter(|o| same_player(o, kill) && may_cause(o, kill))
            .filter(|o| {
                kill.server_time - o.server_time <= flight_limit(o.weapon, o.action, o.mode)
            })
            .max_by(|a, b| a.server_time.total_cmp(&b.server_time))
            .map(|o| LinkedShot {
                occurrence: o.id,
                weapon: o.weapon,
                action: o.action,
                mode: o.mode,
                time: o.time,
                flight: kill.server_time - o.server_time,
            });

        let at = shot
            .as_ref()
            .map_or(kill.server_time, |s| kill.server_time - s.flight);
        let killer = state_at(&kill.killer_samples, at, gravity);
        let victim = kill.victim_state;

        kill.context = Some(KillContext {
            distance: distance(killer.origin, victim.origin),
            killer,
            victim,
            shot,
        });
    }
}

/// The action score of two sides' frags, as in the original formula
pub fn action_score(a: usize, b: usize) -> f64 {
    let (a, b) = (a as f64, b as f64);

    a.powf(0.75) + b.powf(0.75) - (a - b).abs().powf(0.4).max(1.)
}

/// How much each weapon beyond the first raises a window's score
pub const VARIETY_BONUS: f64 = 0.1;

/// A stretch of play and its action score
#[derive(Clone, Debug, Serialize)]
pub struct ActionWindow {
    pub segment: u32,
    /// Playback times of the first and last frag
    pub start: f32,
    pub end: f32,
    pub score: f64,
    /// Frags per side, busiest first. Sides are teams when both players
    /// have one, otherwise players
    pub sides: Vec<(String, usize)>,
    /// Weapon strings of the frags
    pub weapons: Vec<String>,
}

/// Kills that count as frags, by another player and not a suicide or a
/// teammate
fn is_frag(k: &Kill) -> bool {
    matches!(
        k.cause,
        KillCause::PlayerWeapon | KillCause::PlayerOther | KillCause::MountedGun
    ) && k.killer.is_some()
}

fn side(k: &Kill) -> String {
    match (&k.killer_team, &k.victim_team) {
        (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() && a != b => a.clone(),
        _ => k
            .killer
            .as_ref()
            .map_or_else(String::new, |p| p.name.clone()),
    }
}

/// The best non-overlapping windows of `seconds`, highest score first
pub fn action_windows(kills: &[Kill], seconds: f32, top: usize) -> Vec<ActionWindow> {
    let frags: Vec<&Kill> = kills.iter().filter(|k| is_frag(k)).collect();

    let mut windows: Vec<ActionWindow> = Vec::new();
    for (i, first) in frags.iter().enumerate() {
        let inside: Vec<&&Kill> = frags[i..]
            .iter()
            .take_while(|k| k.segment == first.segment && k.time - first.time < seconds)
            .collect();

        let mut sides: Vec<(String, usize)> = Vec::new();
        let mut weapons: Vec<String> = Vec::new();
        for k in &inside {
            let s = side(k);
            match sides.iter_mut().find(|(n, _)| *n == s) {
                Some(e) => e.1 += 1,
                None => sides.push((s, 1)),
            }
            if !weapons.contains(&k.weapon_name) {
                weapons.push(k.weapon_name.clone());
            }
        }
        sides.sort_by_key(|s| std::cmp::Reverse(s.1));

        let a = sides.first().map_or(0, |s| s.1);
        let b = sides.get(1).map_or(0, |s| s.1);
        let variety = 1. + VARIETY_BONUS * weapons.len().saturating_sub(1) as f64;
        windows.push(ActionWindow {
            segment: first.segment,
            start: first.time,
            end: inside.last().map_or(first.time, |k| k.time),
            score: action_score(a, b) * variety,
            sides,
            weapons,
        });
    }

    windows.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut chosen: Vec<ActionWindow> = Vec::new();
    for w in windows {
        if chosen.len() >= top {
            break;
        }
        let overlaps = chosen
            .iter()
            .any(|c| c.segment == w.segment && w.start <= c.end && c.start <= w.end);
        if !overlaps {
            chosen.push(w);
        }
    }

    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_action_score_matches_the_original_formula() {
        // The original JavaScript's values for its example rows
        for (a, b, want) in [
            (5, 5, 5.6874),
            (8, 2, 4.3909),
            (15, 0, 4.6678),
            (10, 3, 5.7250),
            (9, 4, 6.1209),
            (7, 5, 6.3277),
            (9, 6, 7.4780),
            (12, 2, 5.6173),
            (15, 1, 5.7482),
            (6, 6, 6.6673),
            (8, 7, 8.0603),
        ] {
            let got = action_score(a, b);
            assert!((got - want).abs() < 0.0001, "{a}-{b}: {got}");
        }
    }

    #[test]
    fn flight_limits_follow_the_weapon() {
        assert_eq!(
            flight_limit(Weapon::Mp5, Action::Launch, Some("grenade")),
            4.
        );
        assert_eq!(
            flight_limit(Weapon::Crossbow, Action::Fire, Some("zoomed")),
            0.3
        );
        assert_eq!(
            flight_limit(Weapon::Crossbow, Action::Fire, Some("unzoomed")),
            3.
        );
    }
}
