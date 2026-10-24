//! The demo's matches, and its free play long enough to list
//!
//! Each match the [roster](super::roster) saw becomes a part, with its map,
//! kills and score, unless it was cut short before anyone was killed. What
//! is left of a session outside its matches is free
//! play, listed when it lasts at least five minutes, as is a map with
//! nothing played on it

use std::collections::BTreeMap;

use serde::Serialize;

use super::context::{
    Kill, LEAST_PLAYED, ScoreUpdate, Scoreboard, SessionMap, TeamScoreUpdate, standing_at,
};
use super::roster::{Counted, Ending, MatchPart, Moment};

/// Whether a part is a match and how much of it was recorded
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PartStatus {
    /// From its countdown to its intermission
    Complete,
    /// Already on when the recording began, and played out
    JoinedLate,
    /// Cut short by another match, by being called off or by a change of
    /// map
    Interrupted,
    /// Still on when the recording ended
    CutOff,
    /// No match was on
    FreePlay,
}

impl PartStatus {
    /// The name the library keeps it by
    pub fn key(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::JoinedLate => "joined-late",
            Self::Interrupted => "interrupted",
            Self::CutOff => "cut-off",
            Self::FreePlay => "free-play",
        }
    }

    /// The status the library kept by `key`
    pub fn from_key(key: &str) -> Option<Self> {
        let all = [
            Self::Complete,
            Self::JoinedLate,
            Self::Interrupted,
            Self::CutOff,
            Self::FreePlay,
        ];
        all.into_iter().find(|s| s.key() == key)
    }
}

/// A match, or a stretch of free play, by the times of its session
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlayedPart {
    pub segment: u32,
    pub status: PartStatus,
    /// Where watching it begins, at its countdown, or where its recording
    /// began
    pub opens_at: f32,
    /// When playing began, and ended
    pub start: f32,
    pub end: f32,
    pub map: String,
    /// Who played as a match started, or the most at once in free play
    pub counted: Counted,
    pub kills: u32,
    /// The score at its last kill
    pub scoreboard: Scoreboard,
}

impl PlayedPart {
    /// How long it was played for, in seconds
    pub fn length(&self) -> f32 {
        (self.end - self.start).max(0.)
    }
}

/// What the parts are made from
pub struct Played<'a> {
    pub matches: &'a [MatchPart],
    pub history: &'a [Moment],
    pub maps: &'a [SessionMap],
    pub kills: &'a [Kill],
    pub updates: &'a [ScoreUpdate],
    pub team_updates: &'a [TeamScoreUpdate],
    /// When each session's recording began and ended
    pub spans: &'a BTreeMap<u32, (f32, f32)>,
}

/// The status of a match by how it began and ended
fn status_of(played: &MatchPart) -> PartStatus {
    match (played.ending, played.from_start) {
        (Ending::Interrupted, _) => PartStatus::Interrupted,
        (Ending::CutOff, _) => PartStatus::CutOff,
        (Ending::Finished, true) => PartStatus::Complete,
        (Ending::Finished, false) => PartStatus::JoinedLate,
    }
}

/// Every part of the demo, in the order played
pub fn played_parts(from: &Played) -> Vec<PlayedPart> {
    let mut parts: Vec<PlayedPart> = Vec::new();
    for played in from.matches {
        let span = from.spans.get(&played.segment).copied();
        let session_end = span.map_or(played.start, |s| s.1);

        // A match already on is watched from where its recording began
        let (opens_at, start) = match (played.from_start, span) {
            (false, Some((began, _))) => (began, began),
            _ => (played.opens_at, played.start),
        };
        let end = played.end.unwrap_or(session_end).max(start);
        let status = status_of(played);
        let part = part(
            from,
            played.segment,
            status,
            (opens_at, start, end),
            played.counted,
        );

        // A match cut short with nobody killed has nothing to watch
        let played_out = matches!(status, PartStatus::Complete | PartStatus::JoinedLate);
        if played_out || part.kills > 0 {
            parts.push(part);
        }
    }

    for (&segment, &(began, ended)) in from.spans {
        for (start, end) in free_stretches(from.matches, segment, began, ended) {
            let counted = most_between(from.history, segment, start, end);
            parts.push(part(
                from,
                segment,
                PartStatus::FreePlay,
                (start, start, end),
                counted,
            ));
        }
    }

    parts.sort_by(|a, b| {
        a.segment
            .cmp(&b.segment)
            .then(a.opens_at.total_cmp(&b.opens_at))
    });
    parts
}

/// A part of `segment` watched from the first of `times`, played from the
/// second to the third
fn part(
    from: &Played,
    segment: u32,
    status: PartStatus,
    times: (f32, f32, f32),
    counted: Counted,
) -> PlayedPart {
    let (opens_at, start, end) = times;
    let mut kills = 0;
    let mut last_kill = None;
    for kill in from.kills {
        if kill.segment == segment && kill.time >= start && kill.time <= end {
            kills += 1;
            last_kill = Some(kill.time);
        }
    }

    let decided = last_kill.unwrap_or(end);
    let standing = standing_at(from.updates, from.team_updates, segment, decided);
    let map = from.maps.iter().find(|m| m.segment == segment);
    PlayedPart {
        segment,
        status,
        opens_at,
        start,
        end,
        map: map.map(|m| m.name.clone()).unwrap_or_default(),
        counted,
        kills,
        scoreboard: Scoreboard::from(&standing),
    }
}

/// The stretches of `segment`, recorded from `began` to `ended`, with no
/// match on that last five minutes or more
fn free_stretches(matches: &[MatchPart], segment: u32, began: f32, ended: f32) -> Vec<(f32, f32)> {
    let mut stretches = Vec::new();
    let mut from = began;
    for played in matches.iter().filter(|m| m.segment == segment) {
        let until = played.opens_at.max(from);
        if until - from >= LEAST_PLAYED {
            stretches.push((from, until));
        }
        from = played.end.unwrap_or(ended).max(from);
    }

    if ended - from >= LEAST_PLAYED {
        stretches.push((from, ended));
    }
    stretches
}

/// The most players and teams in `segment` at once from `start` to `end`,
/// starting with who played as it began
fn most_between(history: &[Moment], segment: u32, start: f32, end: f32) -> Counted {
    let mut most = Counted::default();
    for moment in history.iter().filter(|m| m.segment == segment) {
        if moment.time > end {
            break;
        }
        if moment.time <= start {
            most = moment.counted;
        } else {
            most = most.most(moment.counted);
        }
    }
    most
}

/// The players and teams to list a demo by, the most in any complete
/// match, or at any moment with none
pub fn demo_counted(parts: &[PlayedPart], any_moment: Counted) -> Counted {
    let mut most: Option<Counted> = None;
    for part in parts.iter().filter(|p| p.status == PartStatus::Complete) {
        most = Some(most.map_or(part.counted, |m| m.most(part.counted)));
    }
    most.unwrap_or(any_moment)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn played(segment: u32, opens_at: f32, end: Option<f32>, ending: Ending) -> MatchPart {
        MatchPart {
            segment,
            opens_at,
            start: opens_at + 10.,
            end,
            from_start: true,
            ending,
            counted: Counted::default(),
        }
    }

    #[test]
    fn free_play_is_what_lasts_five_minutes_between_matches() {
        let matches = [
            played(1, 400., Some(1300.), Ending::Finished),
            played(2, 100., None, Ending::CutOff),
        ];
        assert_eq!(
            free_stretches(&matches, 1, 0., 1700.),
            [(0., 400.), (1300., 1700.)]
        );
        assert_eq!(free_stretches(&matches, 2, 0., 2000.), []);
        assert_eq!(free_stretches(&matches, 3, 0., 299.), []);
    }

    #[test]
    fn free_play_counts_the_most_at_once() {
        let at = |time: f32, players: u32| Moment {
            segment: 1,
            time,
            counted: Counted { players, teams: 0 },
        };
        let history = [at(0., 2), at(50., 6), at(200., 3), at(900., 9)];
        assert_eq!(most_between(&history, 1, 100., 800.).players, 6);
        assert_eq!(most_between(&history, 1, 300., 800.).players, 3);
    }

    #[test]
    fn a_demo_counts_its_complete_matches_only() {
        let part = |status: PartStatus, players: u32| PlayedPart {
            segment: 1,
            status,
            opens_at: 0.,
            start: 0.,
            end: 0.,
            map: String::new(),
            counted: Counted { players, teams: 0 },
            kills: 0,
            scoreboard: Scoreboard::default(),
        };
        let any = Counted {
            players: 8,
            teams: 0,
        };
        let parts = [
            part(PartStatus::Interrupted, 6),
            part(PartStatus::Complete, 4),
        ];
        assert_eq!(demo_counted(&parts, any).players, 4);
        assert_eq!(demo_counted(&parts[..1], any).players, 8);
    }
}
