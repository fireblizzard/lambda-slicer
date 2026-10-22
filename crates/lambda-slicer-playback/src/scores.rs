//! The scores AG's client keeps for its small scoreboard at the top left
//! (OpenAG's `CHudScores`, with HLSDK's scoreboard sorting)
//!
//! `ScoreInfo` is a player's entity, frags, deaths, class and team number,
//! `TeamScore` a team's name, frags and deaths, `Spectator` a player's
//! entity and whether they spectate, and `GameMode` whether teams play.
//! Outside team play each player who does not spectate has a row, and in
//! team play each team with a player in it, its frags as `TeamScore` gave
//! them or else its players' sum. Rows go highest frags first, fewer deaths
//! first between equal frags, and take the colour of their team number

use std::collections::{BTreeMap, BTreeSet};

use lambda_slicer_core::state::Decoder;

/// A player's frags, deaths and team number, as `ScoreInfo` gave them
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PlayerScore {
    frags: i32,
    deaths: i32,
    team_number: u8,
}

/// One row of the small scoreboard
#[derive(Clone, Debug, PartialEq)]
pub struct ScoreRow {
    /// A player's or a team's, colour codes such as `^1` left in
    pub name: String,
    pub frags: i32,
    /// The team number its colour is chosen by, nought outside team play
    pub team_number: u8,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scores {
    /// By entity
    players: BTreeMap<u8, PlayerScore>,
    /// Frags and deaths by team, lowercase as teams are matched without
    /// case
    teams: BTreeMap<String, (i32, i32)>,
    spectators: BTreeSet<u8>,
    team_play: bool,
}

/// A little-endian short at `at` of `body`
fn short(body: &[u8], at: usize) -> Option<i32> {
    let bytes = body.get(at..at + 2)?;
    Some(i32::from(i16::from_le_bytes([bytes[0], bytes[1]])))
}

impl Scores {
    /// The user message `name` with `body`, true when it was one of the
    /// scores'
    pub fn message(&mut self, name: &str, body: &[u8]) -> bool {
        match name {
            "ScoreInfo" => {
                if let (Some(&player), Some(frags), Some(deaths)) =
                    (body.first(), short(body, 1), short(body, 3))
                {
                    let team_number = short(body, 7).unwrap_or(0).clamp(0, 255) as u8;
                    let score = PlayerScore {
                        frags,
                        deaths,
                        team_number,
                    };
                    self.players.insert(player, score);
                }
            }
            "TeamScore" => {
                let end = body.iter().position(|&b| b == 0).unwrap_or(body.len());
                let team = String::from_utf8_lossy(&body[..end]).to_lowercase();
                if let (Some(frags), Some(deaths)) = (short(body, end + 1), short(body, end + 3)) {
                    self.teams.insert(team, (frags, deaths));
                }
            }
            "Spectator" => {
                if let (Some(&player), Some(&spectating)) = (body.first(), body.get(1)) {
                    if spectating != 0 {
                        self.spectators.insert(player);
                    } else {
                        self.spectators.remove(&player);
                    }
                }
            }
            "GameMode" => {
                if let Some(&mode) = body.first() {
                    self.team_play = mode != 0;
                }
            }
            _ => return false,
        }
        true
    }

    /// The rows, highest first, with the players in the server's slots
    /// `decoder` knows and the teams `teams` gives them by entity
    pub fn rows(&self, decoder: &Decoder, teams: &BTreeMap<u8, String>) -> Vec<ScoreRow> {
        let mut rows = Vec::new();

        // Only players still in a slot under a name count
        let mut players = Vec::new();
        for (&entity, score) in &self.players {
            let Some(player) = decoder.player_for_entity(i64::from(entity)) else {
                continue;
            };
            if player.name.is_empty() {
                continue;
            }
            players.push((entity, player.name, *score));
        }

        if self.team_play {
            // Each team with a player in it, as first named
            let mut sides: Vec<(String, PlayerScore)> = Vec::new();
            for (entity, _, score) in &players {
                let Some(team) = teams.get(entity).filter(|t| !t.is_empty()) else {
                    continue;
                };
                let side = sides.iter_mut().find(|(n, _)| n.eq_ignore_ascii_case(team));
                match side {
                    Some((_, sum)) => {
                        sum.frags += score.frags;
                        sum.deaths += score.deaths;
                    }
                    None => sides.push((team.clone(), *score)),
                }
            }

            for (name, mut score) in sides {
                if let Some(&(frags, deaths)) = self.teams.get(&name.to_lowercase()) {
                    (score.frags, score.deaths) = (frags, deaths);
                }
                rows.push((name, score));
            }
        } else {
            for (entity, name, score) in players {
                if !self.spectators.contains(&entity) {
                    rows.push((name, score));
                }
            }
        }
        ranked(rows)
    }
}

/// `rows` highest frags first, fewer deaths first between equal frags
fn ranked(mut rows: Vec<(String, PlayerScore)>) -> Vec<ScoreRow> {
    rows.sort_by(|a, b| b.1.frags.cmp(&a.1.frags).then(a.1.deaths.cmp(&b.1.deaths)));
    rows.into_iter()
        .map(|(name, score)| ScoreRow {
            name,
            frags: score.frags,
            team_number: score.team_number,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `ScoreInfo` body for `player`
    fn score_info(player: u8, frags: i16, deaths: i16, team_number: i16) -> Vec<u8> {
        let mut body = vec![player];
        for value in [frags, deaths, 0, team_number] {
            body.extend(value.to_le_bytes());
        }
        body
    }

    #[test]
    fn score_messages_are_kept() {
        let mut scores = Scores::default();
        assert!(scores.message("ScoreInfo", &score_info(2, 7, -1, 1)));
        assert!(scores.message("Spectator", &[3, 1]));
        assert!(scores.message("GameMode", &[1]));
        assert!(scores.message("TeamScore", b"Blue\0\x0c\0\x03\0"));
        assert!(!scores.message("Health", &[100]));

        let expected = PlayerScore {
            frags: 7,
            deaths: -1,
            team_number: 1,
        };
        assert_eq!(scores.players[&2], expected);
        assert!(scores.spectators.contains(&3));
        assert!(scores.team_play);
        assert_eq!(scores.teams["blue"], (12, 3));

        scores.message("Spectator", &[3, 0]);
        assert!(scores.spectators.is_empty());
    }

    #[test]
    fn rows_go_highest_frags_then_fewest_deaths_first() {
        let score = |frags, deaths| PlayerScore {
            frags,
            deaths,
            team_number: 0,
        };
        let rows = vec![
            ("a".to_owned(), score(3, 5)),
            ("b".to_owned(), score(9, 2)),
            ("c".to_owned(), score(3, 1)),
        ];
        let names: Vec<String> = ranked(rows).into_iter().map(|r| r.name).collect();
        assert_eq!(names, ["b", "c", "a"]);
    }
}
