//! How many players and teams a demo's games had, counted when a match
//! starts
//!
//! A player is a connected client that is not spectating, by AG's
//! `Spectator`, nor a spectator proxy, whose userinfo has `*hltv`. Teams are
//! the players' different team names from `TeamInfo`, none in a game
//! without teams. A match starts when AG's `Countdown` reaches nought, and
//! a match already on when the recording began, which the server's first
//! `Settings` of a session says with no countdown before it, is counted at
//! the session's first kill, once everyone has come in. A demo of several
//! matches keeps the most of each. A demo with no match keeps the most
//! seen at any moment instead

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::state::{Decoder, userinfo_value};

/// The userinfo key a spectator proxy carries
const PROXY_KEY: &[u8] = b"*hltv";

/// How many players and teams there were
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Counted {
    pub players: u32,
    pub teams: u32,
}

impl Counted {
    /// The most of each of `self` and `other`
    fn most(self, other: Self) -> Self {
        Self {
            players: self.players.max(other.players),
            teams: self.teams.max(other.teams),
        }
    }
}

/// Who plays, followed through the decode
#[derive(Debug, Default)]
pub struct Roster {
    /// Spectating players by entity
    spectators: BTreeSet<u8>,
    /// This session counted down to a match, or the server said whether
    /// one was on
    counted_down: bool,
    settings_seen: bool,
    /// A match on before the recording began, to count at the first kill
    count_at_kill: bool,
    /// The most at a match's start, and at any moment
    at_match: Option<Counted>,
    any_moment: Counted,
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
    /// A new session, where nobody spectates yet and no match has started
    pub fn new_session(&mut self) {
        self.spectators.clear();
        self.counted_down = false;
        self.settings_seen = false;
        self.count_at_kill = false;
    }

    /// The user message `name` with `body`, after the analysis has taken
    /// it in, with each player's team in `teams`
    pub fn message(&mut self, d: &Decoder, name: &str, body: &[u8], teams: &BTreeMap<u8, String>) {
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
                if seconds == 0 {
                    self.match_started(d, teams);
                }
            }
            "Settings" => {
                let match_on = body.first().is_some_and(|&on| on != 0);
                if !self.settings_seen && match_on && !self.counted_down {
                    self.count_at_kill = true;
                }
                self.settings_seen = true;
            }
            "DeathMsg" if self.count_at_kill => {
                self.count_at_kill = false;
                self.match_started(d, teams);
            }
            _ => {}
        }

        // A spectator joining or leaving the game changes who plays
        self.changed(d, teams);
    }

    /// Who plays may have changed, as a player came, went or was renamed
    pub fn changed(&mut self, d: &Decoder, teams: &BTreeMap<u8, String>) {
        let now = count(d, &self.spectators, teams);
        self.any_moment = self.any_moment.most(now);
    }

    /// A match started, so who plays now counts for it
    fn match_started(&mut self, d: &Decoder, teams: &BTreeMap<u8, String>) {
        let now = count(d, &self.spectators, teams);
        self.at_match = Some(self.at_match.map_or(now, |most| most.most(now)));
    }

    /// The players and teams of the demo's matches, or of any moment
    /// without one
    pub fn counted(&self) -> Counted {
        self.at_match.unwrap_or(self.any_moment)
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
    fn a_match_counts_over_any_moment() {
        let d = Decoder::new();
        let mut roster = Roster {
            any_moment: Counted {
                players: 10,
                teams: 2,
            },
            ..Roster::default()
        };
        assert_eq!(roster.counted().players, 10);

        // An empty server at the countdown's end
        let teams = BTreeMap::new();
        roster.message(&d, "Countdown", &[0, 1], &teams);
        assert_eq!(roster.counted(), Counted::default());

        // A match on before the recording began counts at the first kill
        let mut late = Roster::default();
        late.message(&d, "Settings", &[1, b'x', 0], &teams);
        assert_eq!(late.at_match, None);
        late.message(&d, "DeathMsg", &[1, 2, 0], &teams);
        assert_eq!(late.at_match, Some(Counted::default()));
    }
}
