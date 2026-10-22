//! Refining the kills shown by how the players moved and how the kill was
//! made (GUI redesign plan §10.1.1 to §10.1.4)
//!
//! A movement the analysis could not see is unknown, never grounded or
//! standing still, so a refinement asks whether a kill is known to match,
//! known not to, or cannot be told. Only kills known to match are shown.
//! Speeds are horizontal, as the library's kill queries measure them. A
//! kill is through a wall only once reconstructed as one

use lambda_slicer_playback::kill_features::WallGauss;

use crate::timeline::{KillFeatures, KillMotion};

/// Whether a kill meets a refinement, as far as can be told
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Match,
    NoMatch,
    /// What would decide it is not known
    Unknown,
}

impl Verdict {
    /// Both refinements together. A known miss decides it, and otherwise
    /// anything unknown leaves the whole unknown
    fn and(self, other: Verdict) -> Verdict {
        match (self, other) {
            (Verdict::NoMatch, _) | (_, Verdict::NoMatch) => Verdict::NoMatch,
            (Verdict::Unknown, _) | (_, Verdict::Unknown) => Verdict::Unknown,
            (Verdict::Match, Verdict::Match) => Verdict::Match,
        }
    }

    /// Either refinement. A known match decides it, and otherwise anything
    /// unknown leaves the whole unknown
    fn or(self, other: Verdict) -> Verdict {
        match (self, other) {
            (Verdict::Match, _) | (_, Verdict::Match) => Verdict::Match,
            (Verdict::Unknown, _) | (_, Verdict::Unknown) => Verdict::Unknown,
            (Verdict::NoMatch, Verdict::NoMatch) => Verdict::NoMatch,
        }
    }
}

/// Which players a midair kill needs in the air
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MidairMode {
    #[default]
    Off,
    /// At least one of them, so both count too
    Either,
    Both,
    /// Exactly one of them, the other known to be on the ground
    ExactlyOne,
}

/// Whose speed a fast kill is about
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpeedSubject {
    Killer,
    Victim,
    #[default]
    Either,
}

/// The speed a fast kill needs at first, in units per second
pub const DEFAULT_MIN_UPS: u32 = 800;

/// The refinements of the kills shown, all of them on the same kill
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KillFilters {
    pub midair: MidairMode,
    pub speed: bool,
    pub min_ups: u32,
    pub speed_subject: SpeedSubject,
    pub wallgauss: bool,
}

impl Default for KillFilters {
    fn default() -> Self {
        Self {
            midair: MidairMode::Off,
            speed: false,
            min_ups: DEFAULT_MIN_UPS,
            speed_subject: SpeedSubject::Either,
            wallgauss: false,
        }
    }
}

impl KillFilters {
    /// Whether any refinement is on
    pub fn any(&self) -> bool {
        self.midair != MidairMode::Off || self.speed || self.wallgauss
    }

    /// Every refinement off, keeping the speed typed in for when it is
    /// turned on again
    pub fn clear(&mut self) {
        *self = Self {
            min_ups: self.min_ups,
            ..Self::default()
        };
    }

    /// Whether `kill` meets every refinement that is on, `wall` being what
    /// is told of it going through a wall so far. A kill meets no
    /// refinements at all when none is on
    pub fn judge(&self, kill: &KillFeatures, wall: Option<&WallGauss>) -> Verdict {
        let killer = kill.killer_motion.as_ref();
        let victim = kill.victim_motion.as_ref();
        let mut verdict = Verdict::Match;

        if self.midair != MidairMode::Off {
            let airborne = |m: Option<&KillMotion>| m.and_then(|m| m.airborne);
            let midair = midair(self.midair, airborne(killer), airborne(victim));
            verdict = verdict.and(midair);
        }

        if self.speed {
            let speed_of = |m: Option<&KillMotion>| m.and_then(|m| m.horizontal_speed);
            let fast = speed(
                self.speed_subject,
                self.min_ups,
                speed_of(killer),
                speed_of(victim),
            );
            verdict = verdict.and(fast);
        }

        // A Gauss kill not told yet, or not surely, cannot be told, and any
        // other kill is not one
        if self.wallgauss {
            let wall = match (kill.gauss, wall) {
                (false, _) => Verdict::NoMatch,
                (true, Some(WallGauss::Yes(_))) => Verdict::Match,
                (true, Some(WallGauss::No(_))) => Verdict::NoMatch,
                (true, _) => Verdict::Unknown,
            };
            verdict = verdict.and(wall);
        }

        verdict
    }
}

/// Whether a known airborne state is in the air, and unknown when not known
fn in_air(airborne: Option<bool>) -> Verdict {
    match airborne {
        Some(true) => Verdict::Match,
        Some(false) => Verdict::NoMatch,
        None => Verdict::Unknown,
    }
}

/// Whether the killer and victim were in the air as `mode` asks
pub fn midair(mode: MidairMode, killer: Option<bool>, victim: Option<bool>) -> Verdict {
    match mode {
        MidairMode::Off => Verdict::Match,
        MidairMode::Either => in_air(killer).or(in_air(victim)),
        MidairMode::Both => in_air(killer).and(in_air(victim)),
        // One known in the air with the other unknown might be both, so it
        // takes both known to tell
        MidairMode::ExactlyOne => match (killer, victim) {
            (Some(k), Some(v)) if k != v => Verdict::Match,
            (Some(_), Some(_)) => Verdict::NoMatch,
            _ => Verdict::Unknown,
        },
    }
}

/// Whether a known speed is at least `min_ups`, unknown when there is no
/// finite speed, even for a threshold of zero
fn fast(speed: Option<f32>, min_ups: u32) -> Verdict {
    match speed.filter(|s| s.is_finite()) {
        Some(speed) if f64::from(speed) >= f64::from(min_ups) => Verdict::Match,
        Some(_) => Verdict::NoMatch,
        None => Verdict::Unknown,
    }
}

/// Whether `subject` moved at least `min_ups` across the ground
pub fn speed(
    subject: SpeedSubject,
    min_ups: u32,
    killer: Option<f32>,
    victim: Option<f32>,
) -> Verdict {
    match subject {
        SpeedSubject::Killer => fast(killer, min_ups),
        SpeedSubject::Victim => fast(victim, min_ups),
        SpeedSubject::Either => fast(killer, min_ups).or(fast(victim, min_ups)),
    }
}

/// How a kill's airborne badge reads
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AirBadge {
    /// Both known in the air
    Both,
    /// At least one known in the air, which does not say the other was not
    One,
}

/// The airborne badge a kill shows, none unless someone is known in the air
pub fn air_badge(kill: &KillFeatures) -> Option<AirBadge> {
    let killer = kill.killer_motion.and_then(|m| m.airborne);
    let victim = kill.victim_motion.and_then(|m| m.airborne);
    match (killer, victim) {
        (Some(true), Some(true)) => Some(AirBadge::Both),
        (Some(true), _) | (_, Some(true)) => Some(AirBadge::One),
        _ => None,
    }
}

/// How a kill's speed badge reads
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpeedBadge {
    /// The higher of the players' known speeds
    Known(f32),
    /// The speed refinement is on and neither speed is known
    Unknown,
}

/// The speed badge a kill shows, when the faster player is known to be at
/// least `DEFAULT_MIN_UPS` or the speed refinement is on
pub fn speed_badge(kill: &KillFeatures, filters: &KillFilters) -> Option<SpeedBadge> {
    let mut highest: Option<f32> = None;
    for motion in [kill.killer_motion, kill.victim_motion]
        .into_iter()
        .flatten()
    {
        let Some(speed) = motion.horizontal_speed.filter(|s| s.is_finite()) else {
            continue;
        };
        highest = Some(match highest {
            Some(h) => h.max(speed),
            None => speed,
        });
    }

    match highest {
        Some(speed) if filters.speed || f64::from(speed) >= f64::from(DEFAULT_MIN_UPS) => {
            Some(SpeedBadge::Known(speed))
        }
        Some(_) => None,
        None if filters.speed => Some(SpeedBadge::Unknown),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use lambda_slicer_core::analysis::motion::MotionEvidence;
    use lambda_slicer_playback::kill_features::{Doubt, OtherRoute, WallDetails, WallRoute};

    use super::*;
    use crate::timeline::{KillKey, SampledAt};

    const STATES: [Option<bool>; 3] = [Some(true), Some(false), None];

    fn motion(airborne: Option<bool>, speed: Option<f32>) -> KillMotion {
        KillMotion {
            airborne,
            horizontal_speed: speed,
            evidence: MotionEvidence::Trajectory,
            sampled_at: SampledAt::Kill,
        }
    }

    fn kill(killer: KillMotion, victim: KillMotion, gauss: bool) -> KillFeatures {
        KillFeatures {
            key: KillKey {
                segment: 0,
                ordinal: 0,
            },
            killer_motion: Some(killer),
            victim_motion: Some(victim),
            shot: None,
            victim: None,
            killer_team: None,
            victim_team: None,
            gauss,
            gauss_kill: None,
        }
    }

    #[test]
    fn midair_either_needs_one_known_in_the_air() {
        use Verdict::*;
        let expected = [
            // Killer in the air, on the ground, unknown, against each victim
            [Match, Match, Match],
            [Match, NoMatch, Unknown],
            [Match, Unknown, Unknown],
        ];
        for (k, row) in STATES.iter().zip(expected) {
            for (v, want) in STATES.iter().zip(row) {
                assert_eq!(midair(MidairMode::Either, *k, *v), want, "{k:?} {v:?}");
            }
        }
    }

    #[test]
    fn midair_both_needs_both_known_in_the_air() {
        use Verdict::*;
        let expected = [
            [Match, NoMatch, Unknown],
            [NoMatch, NoMatch, NoMatch],
            [Unknown, NoMatch, Unknown],
        ];
        for (k, row) in STATES.iter().zip(expected) {
            for (v, want) in STATES.iter().zip(row) {
                assert_eq!(midair(MidairMode::Both, *k, *v), want, "{k:?} {v:?}");
            }
        }
    }

    #[test]
    fn midair_exactly_one_needs_the_other_known_on_the_ground() {
        use Verdict::*;
        // One in the air with the other unknown does not prove exactly one
        let expected = [
            [NoMatch, Match, Unknown],
            [Match, NoMatch, Unknown],
            [Unknown, Unknown, Unknown],
        ];
        for (k, row) in STATES.iter().zip(expected) {
            for (v, want) in STATES.iter().zip(row) {
                assert_eq!(midair(MidairMode::ExactlyOne, *k, *v), want, "{k:?} {v:?}");
            }
        }
    }

    #[test]
    fn speed_compares_the_unrounded_speed_at_the_threshold() {
        let killer = |s| speed(SpeedSubject::Killer, 1000, Some(s), None);
        assert_eq!(killer(999.9), Verdict::NoMatch);
        assert_eq!(killer(1000.), Verdict::Match);
        assert_eq!(killer(1000.1), Verdict::Match);

        // A speed that is not a number is unknown
        assert_eq!(killer(f32::NAN), Verdict::Unknown);
        assert_eq!(killer(f32::INFINITY), Verdict::Unknown);

        // A large threshold typed in compares as it is
        assert_eq!(
            speed(SpeedSubject::Killer, u32::MAX, Some(5000.), None),
            Verdict::NoMatch
        );
    }

    #[test]
    fn speed_subjects_test_their_own_player() {
        let fast = Some(1200.);
        let slow = Some(300.);
        assert_eq!(
            speed(SpeedSubject::Killer, 1000, fast, slow),
            Verdict::Match
        );
        assert_eq!(
            speed(SpeedSubject::Victim, 1000, fast, slow),
            Verdict::NoMatch
        );
        assert_eq!(
            speed(SpeedSubject::Victim, 1000, slow, fast),
            Verdict::Match
        );
        assert_eq!(
            speed(SpeedSubject::Either, 1000, slow, fast),
            Verdict::Match
        );

        // Either with one fast is a match whatever the other, and with
        // neither fast unknown when one could be
        assert_eq!(
            speed(SpeedSubject::Either, 1000, fast, None),
            Verdict::Match
        );
        assert_eq!(
            speed(SpeedSubject::Either, 1000, slow, None),
            Verdict::Unknown
        );
        assert_eq!(
            speed(SpeedSubject::Either, 1000, slow, slow),
            Verdict::NoMatch
        );
    }

    #[test]
    fn an_unknown_speed_never_meets_a_threshold_of_zero() {
        assert_eq!(speed(SpeedSubject::Killer, 0, None, None), Verdict::Unknown);
        assert_eq!(speed(SpeedSubject::Either, 0, None, None), Verdict::Unknown);
        assert_eq!(
            speed(SpeedSubject::Killer, 0, Some(0.), None),
            Verdict::Match
        );
    }

    #[test]
    fn refinements_combine_on_the_same_kill() {
        let air_fast = kill(
            motion(Some(true), Some(1200.)),
            motion(Some(false), None),
            false,
        );
        let mut filters = KillFilters {
            midair: MidairMode::Either,
            speed: true,
            speed_subject: SpeedSubject::Killer,
            ..Default::default()
        };
        assert_eq!(filters.judge(&air_fast, None), Verdict::Match);

        // A known miss of one decides it whatever the other
        filters.speed_subject = SpeedSubject::Victim;
        let ground_fast = kill(
            motion(Some(false), Some(1200.)),
            motion(Some(false), Some(10.)),
            false,
        );
        assert_eq!(filters.judge(&ground_fast, None), Verdict::NoMatch);

        // A match and an unknown leave it unknown
        assert_eq!(filters.judge(&air_fast, None), Verdict::Unknown);

        // No refinement on matches every kill
        assert_eq!(
            KillFilters::default().judge(&ground_fast, None),
            Verdict::Match
        );
    }

    #[test]
    fn only_a_gauss_kill_reconstructed_through_a_wall_is_one() {
        let filters = KillFilters {
            wallgauss: true,
            ..Default::default()
        };
        let still = motion(Some(false), Some(0.));
        let gauss = kill(still, still, true);
        let yes = WallGauss::Yes(WallDetails {
            route: WallRoute::ExitBlast,
            thickness: 16.,
            shot_time: 1.,
        });
        assert_eq!(filters.judge(&gauss, Some(&yes)), Verdict::Match);
        let no = WallGauss::No(OtherRoute::Direct);
        assert_eq!(filters.judge(&gauss, Some(&no)), Verdict::NoMatch);

        // Not told yet, or not surely, is unknown rather than no
        assert_eq!(filters.judge(&gauss, None), Verdict::Unknown);
        let unsure = WallGauss::Unknown(Doubt::Ambiguous);
        assert_eq!(filters.judge(&gauss, Some(&unsure)), Verdict::Unknown);

        // Another weapon's kill never is, whatever it was told
        let other = kill(still, still, false);
        assert_eq!(filters.judge(&other, Some(&yes)), Verdict::NoMatch);
    }

    #[test]
    fn clearing_turns_everything_off_but_keeps_the_speed() {
        let mut filters = KillFilters {
            midair: MidairMode::Both,
            speed: true,
            min_ups: 1500,
            speed_subject: SpeedSubject::Victim,
            wallgauss: true,
        };
        filters.clear();
        assert!(!filters.any());
        assert_eq!(filters.min_ups, 1500);
        assert_eq!(filters.speed_subject, SpeedSubject::Either);
    }

    #[test]
    fn badges_say_only_what_is_known() {
        let off = KillFilters::default();
        let on = KillFilters {
            speed: true,
            ..Default::default()
        };

        let both = kill(
            motion(Some(true), Some(700.)),
            motion(Some(true), None),
            false,
        );
        assert_eq!(air_badge(&both), Some(AirBadge::Both));
        // Below the badge's speed it shows only while the refinement is on
        assert_eq!(speed_badge(&both, &off), None);
        assert_eq!(speed_badge(&both, &on), Some(SpeedBadge::Known(700.)));

        let one = kill(
            motion(Some(true), Some(1500.)),
            motion(None, Some(2000.)),
            false,
        );
        assert_eq!(air_badge(&one), Some(AirBadge::One));
        assert_eq!(speed_badge(&one, &off), Some(SpeedBadge::Known(2000.)));

        let none = kill(motion(None, None), motion(Some(false), None), false);
        assert_eq!(air_badge(&none), None);
        assert_eq!(speed_badge(&none, &off), None);
        assert_eq!(speed_badge(&none, &on), Some(SpeedBadge::Unknown));
    }
}
