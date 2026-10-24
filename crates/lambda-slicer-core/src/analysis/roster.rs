//! Who played, and the matches a demo recorded
//!
//! A player is a connected client that is not spectating, by AG's
//! `Spectator`, nor a spectator proxy, whose userinfo has `*hltv`. Teams are
//! the players' different team names from `TeamInfo`, none in a game
//! without teams. A match starts when AG's `Countdown` reaches nought, and
//! is counted then. A match already on when the recording began, which the
//! server's first `Settings` of a session says with no countdown before it,
//! is counted at the session's first kill, once everyone has come in. A
//! match ends at the intermission, or is cut short by another countdown
//! reaching nought, by `Settings` saying no match is on, by a new session or
//! by the end of the recording
//!
//! A recording that begins mid-game may have no `Settings` at all, as the
//! server sent them before. Such a session's game is taken for a match
//! already on from its first kill, and kept only when it plays out to the
//! intermission

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::state::{Decoder, userinfo_value};

/// The userinfo key a spectator proxy carries
const PROXY_KEY: &[u8] = b"*hltv";

/// The count `Countdown` shows while it waits for players
const WAITING: i8 = 50;

/// How many players and teams there were
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Counted {
    pub players: u32,
    pub teams: u32,
}

impl Counted {
    /// The most of each of `self` and `other`
    pub fn most(self, other: Self) -> Self {
        Self {
            players: self.players.max(other.players),
            teams: self.teams.max(other.teams),
        }
    }
}

/// How a match's recording ended
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Ending {
    /// At the intermission, the match played out
    Finished,
    /// By another match, by the match being called off or by a change of
    /// map
    Interrupted,
    /// By the end of the recording
    CutOff,
}

/// A match the recording saw, by the times of its session
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct MatchPart {
    pub segment: u32,
    /// When its countdown began, or its session for a match already on
    pub opens_at: f32,
    /// When its countdown reached nought, or its session began
    pub start: f32,
    /// None while the match lasted to the end of its session's recording
    pub end: Option<f32>,
    /// Its countdown was recorded
    pub from_start: bool,
    pub ending: Ending,
    /// Who played as it started
    pub counted: Counted,
}

/// Who played at one moment, from then until the next
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Moment {
    pub segment: u32,
    pub time: f32,
    pub counted: Counted,
}

/// Who plays and which matches were played, followed through the decode
#[derive(Debug, Default)]
pub struct Roster {
    /// Spectating players by entity
    spectators: BTreeSet<u8>,
    /// When this session began
    session_start: f32,
    /// This session counted down to a match, or the server said whether
    /// one was on
    counted_down: bool,
    settings_seen: bool,
    /// When the countdown under way began
    countdown_from: Option<f32>,
    /// A match on before the recording began, to count at the first kill
    count_at_kill: bool,
    /// The match under way is only taken for one, as nothing said whether
    /// a match was on, and is kept only when it plays out
    unconfirmed: bool,
    /// The match under way, and those already over
    open: Option<MatchPart>,
    pub matches: Vec<MatchPart>,
    /// Who played, each time it changed
    pub history: Vec<Moment>,
    /// The most players and teams at any moment
    pub any_moment: Counted,
}

/// The players and teams in `d`'s slots now, with `teams` by entity. In a
/// game with teams only those on one play, as a player still coming in or
/// left out of the teams has none
fn count(d: &Decoder, spectators: &BTreeSet<u8>, teams: &BTreeMap<u8, String>) -> Counted {
    let (mut players, mut on_teams) = (0, 0);
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (&slot, player) in &d.players {
        let entity = slot.wrapping_add(1);
        let proxy = userinfo_value(&player.info, PROXY_KEY).is_some();
        if spectators.contains(&entity) || proxy || player.name.is_empty() {
            continue;
        }

        players += 1;
        if let Some(team) = teams.get(&entity).filter(|t| !t.is_empty()) {
            on_teams += 1;
            names.insert(team.to_lowercase());
        }
    }

    Counted {
        players: if names.is_empty() { players } else { on_teams },
        teams: names.len() as u32,
    }
}

impl Roster {
    /// A new session beginning at `time`, where nobody spectates yet and no
    /// match has started. A match still on was cut short by it
    pub fn new_session(&mut self, time: f32) {
        self.close(None, Ending::Interrupted);
        self.spectators.clear();
        self.session_start = time;
        self.counted_down = false;
        self.settings_seen = false;
        self.countdown_from = None;
        self.count_at_kill = false;
    }

    /// The user message `name` with `body` at `time`, after the analysis
    /// has taken it in, with each player's team in `teams`
    pub fn message(
        &mut self,
        d: &Decoder,
        time: f32,
        name: &str,
        body: &[u8],
        teams: &BTreeMap<u8, String>,
    ) {
        match name {
            "Spectator" => {
                if let (Some(&player), Some(&spectating)) = (body.first(), body.get(1)) {
                    if spectating != 0 {
                        self.spectators.insert(player);
                    } else {
                        self.spectators.remove(&player);
                    }
                }
            }
            "Countdown" => {
                let Some(&seconds) = body.first() else {
                    return;
                };
                self.counted_down = true;
                self.countdown(d, time, seconds as i8, teams);
            }
            "Settings" => {
                let match_on = body.first().is_some_and(|&on| on != 0);
                if self.unconfirmed {
                    // Now known, a game on taken for a match is one, or was
                    // never one
                    self.unconfirmed = false;
                    if !match_on {
                        self.open = None;
                    }
                } else if !self.settings_seen && match_on && !self.counted_down {
                    self.already_on(d, teams);
                }
                if !match_on {
                    self.close(Some(time), Ending::Interrupted);
                }
                self.settings_seen = true;
            }
            "DeathMsg" if self.count_at_kill => {
                self.count_at_kill = false;
                let now = count(d, &self.spectators, teams);
                if let Some(open) = &mut self.open {
                    open.counted = now;
                }
            }
            "DeathMsg" if self.open.is_none() && !self.settings_seen && !self.counted_down => {
                // Nothing said whether a match was on, so the game may be one
                self.already_on(d, teams);
                self.count_at_kill = false;
                self.unconfirmed = true;
            }
            _ => {}
        }

        // A spectator joining or leaving the game changes who plays
        self.changed(d, time, teams);
    }

    /// The countdown showing `seconds` at `time`. Its first count opens the
    /// match to come, nought starts it, and a negative count only hides it,
    /// or calls it off when it never reached nought. While it waits for
    /// players it has not begun yet
    fn countdown(&mut self, d: &Decoder, time: f32, seconds: i8, teams: &BTreeMap<u8, String>) {
        if seconds == WAITING {
            return;
        }
        if seconds < 0 {
            self.countdown_from = None;
            return;
        }
        if seconds > 0 {
            self.countdown_from.get_or_insert(time);
            return;
        }

        let opens_at = self.countdown_from.take().unwrap_or(time);
        self.close(Some(opens_at), Ending::Interrupted);
        self.open = Some(MatchPart {
            segment: d.segment,
            opens_at,
            start: time,
            end: None,
            from_start: true,
            ending: Ending::CutOff,
            counted: count(d, &self.spectators, teams),
        });
    }

    /// A match on since before the recording began
    fn already_on(&mut self, d: &Decoder, teams: &BTreeMap<u8, String>) {
        self.count_at_kill = true;
        self.open = Some(MatchPart {
            segment: d.segment,
            opens_at: self.session_start,
            start: self.session_start,
            end: None,
            from_start: false,
            ending: Ending::CutOff,
            counted: count(d, &self.spectators, teams),
        });
    }

    /// The intermission began at `time`, so the match under way played out
    pub fn intermission(&mut self, time: f32) {
        self.close(Some(time), Ending::Finished);
    }

    /// The match under way ended at `end`, or with its session's recording.
    /// A game only taken for a match is kept when it played out
    fn close(&mut self, end: Option<f32>, ending: Ending) {
        let unconfirmed = std::mem::take(&mut self.unconfirmed);
        if let Some(mut open) = self.open.take()
            && (!unconfirmed || ending == Ending::Finished)
        {
            open.end = end;
            open.ending = ending;
            self.matches.push(open);
        }
        self.count_at_kill = false;
    }

    /// The recording ended, cutting off a match still on
    pub fn finish(&mut self) {
        self.close(None, Ending::CutOff);
    }

    /// Who plays may have changed at `time`, as a player came, went or was
    /// renamed
    pub fn changed(&mut self, d: &Decoder, time: f32, teams: &BTreeMap<u8, String>) {
        let now = count(d, &self.spectators, teams);
        self.any_moment = self.any_moment.most(now);

        let last = self.history.last();
        if last.is_none_or(|m| m.segment != d.segment || m.counted != now) {
            self.history.push(Moment {
                segment: d.segment,
                time,
                counted: now,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_most_of_each_is_kept_on_its_own() {
        let a = Counted {
            players: 8,
            teams: 2,
        };
        let b = Counted {
            players: 4,
            teams: 4,
        };
        assert_eq!(
            a.most(b),
            Counted {
                players: 8,
                teams: 4
            }
        );
    }

    #[test]
    fn a_countdown_opens_a_match_and_the_intermission_ends_it() {
        let d = Decoder::new();
        let teams = BTreeMap::new();
        let mut roster = Roster::default();
        roster.new_session(0.);
        roster.message(&d, 10., "Countdown", &[3, 1], &teams);
        roster.message(&d, 12., "Countdown", &[1, 1], &teams);
        roster.message(&d, 13., "Countdown", &[0, 1], &teams);
        roster.message(&d, 14., "Countdown", &[255, 1], &teams);
        roster.intermission(900.);

        let [played] = roster.matches.as_slice() else {
            panic!("one match: {:?}", roster.matches);
        };
        assert_eq!((played.opens_at, played.start), (10., 13.));
        assert_eq!(played.end, Some(900.));
        assert!(played.from_start);
        assert_eq!(played.ending, Ending::Finished);
    }

    #[test]
    fn a_match_already_on_is_interrupted_by_the_next_countdown() {
        let d = Decoder::new();
        let teams = BTreeMap::new();
        let mut roster = Roster::default();
        roster.new_session(0.);
        roster.message(&d, 2., "Settings", &[1, b'x', 0], &teams);
        roster.message(&d, 353., "Countdown", &[9, 1], &teams);
        roster.message(&d, 362., "Countdown", &[0, 1], &teams);
        roster.finish();

        let [late, next] = roster.matches.as_slice() else {
            panic!("two matches: {:?}", roster.matches);
        };
        assert!(!late.from_start);
        assert_eq!((late.opens_at, late.end), (0., Some(353.)));
        assert_eq!(late.ending, Ending::Interrupted);
        assert_eq!((next.start, next.end), (362., None));
        assert_eq!(next.ending, Ending::CutOff);
    }

    #[test]
    fn a_game_recorded_from_mid_game_counts_when_it_plays_out() {
        let d = Decoder::new();
        let teams = BTreeMap::new();
        let mut roster = Roster::default();

        // No word of a match, then a kill and the intermission
        roster.new_session(0.);
        roster.message(&d, 10., "DeathMsg", &[1, 2, 0], &teams);
        roster.intermission(60.);
        let [played] = roster.matches.as_slice() else {
            panic!("one match: {:?}", roster.matches);
        };
        assert!(!played.from_start);
        assert_eq!((played.opens_at, played.end), (0., Some(60.)));
        assert_eq!(played.ending, Ending::Finished);

        // Cut off, or said to be no match, it was free play
        roster.new_session(70.);
        roster.message(&d, 80., "DeathMsg", &[1, 2, 0], &teams);
        roster.new_session(90.);
        roster.message(&d, 95., "DeathMsg", &[1, 2, 0], &teams);
        roster.message(&d, 99., "Settings", &[0, b'x', 0], &teams);
        roster.intermission(120.);
        roster.finish();
        assert_eq!(roster.matches.len(), 1);
    }

    #[test]
    fn a_countdown_hidden_before_nought_starts_nothing() {
        let d = Decoder::new();
        let teams = BTreeMap::new();
        let mut roster = Roster::default();
        roster.message(&d, 1., "Countdown", &[5, 1], &teams);
        roster.message(&d, 2., "Countdown", &[255, 1], &teams);
        roster.finish();
        assert!(roster.matches.is_empty());
    }
}
