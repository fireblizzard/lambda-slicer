//! How the match went besides its kills, its votes, timeouts, pauses and
//! sudden death, each change once
//!
//! AG's server repeats its `Vote` message every two seconds and its
//! `Timeout` every second while they last (OpenAG's `CHudVote` and
//! `CHudTimeout`). A vote is a status, 1 open, 2 accepted and 3 denied,
//! the votes for, against and not yet given, what is voted on as two
//! strings and who called it. A change is a vote called, its count changed
//! or its end. A timeout is a status, 1 called and 3 under way, and the
//! seconds left, nought ending it, and the `TextMsg` before it names who
//! called it as `Timeout called by` and the name. `SuddenDeath` turns
//! sudden death on or off, and the engine's `svc_setpause` pauses and
//! resumes the game
//!
//! A new part of the demo, after a level change or a reconnect, names the
//! map it is on. `Spectator` is a player's entity and whether they now
//! spectate. The server sends it again for players already known, so only
//! a change counts, and a player not yet known to spectate plays. A player
//! joining is often switched back and forth at one instant, so a player's
//! switches at one instant count as one change, none when they end as they
//! began. Spectator proxies such as HLTV, whose userinfo has `*hltv`, and
//! players with no name yet are left out

use std::collections::BTreeSet;

use serde::Serialize;

use crate::state::{Decoder, PlayerIdentity, userinfo_value};

/// The userinfo key a spectator proxy carries
const PROXY_KEY: &[u8] = b"*hltv";

/// `Vote`'s statuses, nought hiding it
pub const VOTE_OPEN: u8 = 1;
pub const VOTE_ACCEPTED: u8 = 2;
pub const VOTE_DENIED: u8 = 3;

/// `Timeout`'s statuses, a timeout called and one under way, nought ending
/// it
pub const TIMEOUT_CALLED: u8 = 1;
pub const TIMEOUT_UNDER_WAY: u8 = 3;

/// How AG's server words who called a timeout, before their name
const TIMEOUT_CALLED_BY: &str = "Timeout called by ";

/// The votes for, against and not yet given
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Tally {
    pub yes: u8,
    pub no: u8,
    pub undecided: u8,
}

/// What changed
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum MatchChange {
    VoteCalled {
        subject: String,
        tally: Tally,
    },
    /// The votes for and against grew or shrank by these
    VoteCast {
        subject: String,
        yes: i16,
        no: i16,
        tally: Tally,
    },
    VoteAccepted {
        subject: String,
    },
    VoteDenied {
        subject: String,
    },
    TimeoutCalled,
    TimeoutUnderWay,
    TimeoutOver,
    Paused,
    Resumed,
    SuddenDeath,
    SuddenDeathOver,
    /// A new part of the demo on this map, after a level change or a
    /// reconnect
    MapChanged {
        map: String,
    },
    /// A player started spectating, or stopped and plays again
    Spectating,
    Playing,
}

/// One change, and the player behind it when there is one, who called a
/// vote or a timeout
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MatchEvent {
    pub segment: u32,
    pub time: f32,
    pub player: Option<PlayerIdentity>,
    pub change: MatchChange,
}

/// One `Vote` message, its status, its tally, what is voted on and who
/// called it, colour codes such as `^1` left in their name
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoteMessage {
    pub status: u8,
    pub tally: Tally,
    pub subject: String,
    pub called_by: String,
}

/// A `Vote`'s body, a status, the votes for, against and not yet given,
/// then what is voted on as two strings and who called it
pub fn vote_message(body: &[u8]) -> Option<VoteMessage> {
    let &[status, yes, no, undecided] = body.get(..4)? else {
        return None;
    };
    let (first, rest) = c_string(&body[4..]);
    let (second, rest) = c_string(rest);
    let (called_by, _) = c_string(rest);

    Some(VoteMessage {
        status,
        tally: Tally { yes, no, undecided },
        subject: format!("{first} {second}").trim().to_owned(),
        called_by,
    })
}

/// Follows the messages, keeping only their changes
#[derive(Clone, Debug, Default)]
pub struct MatchEvents {
    pub events: Vec<MatchEvent>,
    /// The open or ended vote as its last message left it
    vote: Option<VoteMessage>,
    /// The timeout's status while one lasts, and who the server last said
    /// called one
    timeout: Option<u8>,
    timeout_caller: Option<String>,
    paused: bool,
    sudden_death: bool,
    /// The entities of the players spectating
    spectating: BTreeSet<u8>,
    /// Switches at the latest instant, kept until time moves on
    switches: Vec<Switch>,
}

/// A player's switches at one instant, from whether they spectated before
/// it to whether they do after it
#[derive(Clone, Debug)]
struct Switch {
    entity: u8,
    segment: u32,
    time: f32,
    before: bool,
    after: bool,
    player: Option<PlayerIdentity>,
}

/// Whether the player at `entity` is a spectator proxy
fn is_proxy(d: &Decoder, entity: u8) -> bool {
    let slot = entity.wrapping_sub(1);
    d.players
        .get(&slot)
        .is_some_and(|p| userinfo_value(&p.info, PROXY_KEY).is_some())
}

/// A string ending at its first nought, and what follows it
fn c_string(bytes: &[u8]) -> (String, &[u8]) {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]).into_owned();
    (text, bytes.get(end + 1..).unwrap_or_default())
}

impl MatchEvents {
    /// A new session, where nothing lasts from the one before
    pub fn new_session(&mut self) {
        self.finish();
        let events = std::mem::take(&mut self.events);
        *self = Self {
            events,
            ..Self::default()
        };
    }

    /// Record `change` at `time` in the decoder's session
    fn push(
        &mut self,
        d: &Decoder,
        time: f32,
        player: Option<PlayerIdentity>,
        change: MatchChange,
    ) {
        self.events.push(MatchEvent {
            segment: d.segment,
            time,
            player,
            change,
        });
    }

    /// Record the switches still kept, once the recording ends
    pub fn finish(&mut self) {
        self.settle(None, f32::INFINITY);
    }

    /// Record the switches kept from before `time`, those that changed
    /// whether their player spectates. A player joining is switched before
    /// their name arrives, so one still unnamed is looked up again in `d`
    fn settle(&mut self, d: Option<&Decoder>, time: f32) {
        let (done, kept): (Vec<Switch>, Vec<Switch>) =
            self.switches.drain(..).partition(|s| s.time < time);
        self.switches = kept;

        for switch in done {
            if switch.before == switch.after {
                continue;
            }
            let named_now = || d.and_then(|d| d.player_for_entity(i64::from(switch.entity)));
            let Some(player) = switch.player.or_else(named_now) else {
                continue;
            };
            let change = if switch.after {
                MatchChange::Spectating
            } else {
                MatchChange::Playing
            };
            self.events.push(MatchEvent {
                segment: switch.segment,
                time: switch.time,
                player: Some(player),
                change,
            });
        }
    }

    /// `svc_setpause`, the game paused or not
    pub fn pause(&mut self, d: &Decoder, time: f32, paused: bool) {
        self.settle(Some(d), time);
        if paused == self.paused {
            return;
        }

        self.paused = paused;
        let change = if paused {
            MatchChange::Paused
        } else {
            MatchChange::Resumed
        };
        self.push(d, time, None, change);
    }

    /// A new part of the demo at `time` on the map `map`
    pub fn map_changed(&mut self, d: &Decoder, time: f32, map: String) {
        self.settle(Some(d), time);
        self.push(d, time, None, MatchChange::MapChanged { map });
    }

    /// The user message `name` with `body` at `time`, if it is one of these
    pub fn message(&mut self, d: &Decoder, time: f32, name: &str, body: &[u8]) {
        self.settle(Some(d), time);
        match name {
            "Spectator" => self.spectator(d, time, body),
            "Vote" => self.vote(d, time, body),
            "Timeout" => self.timeout(d, time, body),
            "SuddenDeath" => {
                let on = body.first().is_some_and(|&b| b != 0);
                if on != self.sudden_death {
                    self.sudden_death = on;
                    let change = if on {
                        MatchChange::SuddenDeath
                    } else {
                        MatchChange::SuddenDeathOver
                    };
                    self.push(d, time, None, change);
                }
            }
            "TextMsg" => {
                let (text, _) = c_string(body.get(1..).unwrap_or_default());
                if let Some(name) = text.trim_end().strip_prefix(TIMEOUT_CALLED_BY) {
                    self.timeout_caller = Some(name.to_owned());
                }
            }
            _ => {}
        }
    }

    /// A `Spectator`, kept as a switch of its player at this instant
    fn spectator(&mut self, d: &Decoder, time: f32, body: &[u8]) {
        let (Some(&entity), Some(&on)) = (body.first(), body.get(1)) else {
            return;
        };

        let spectating = on != 0;
        let changed = if spectating {
            self.spectating.insert(entity)
        } else {
            self.spectating.remove(&entity)
        };
        if !changed || is_proxy(d, entity) {
            return;
        }

        // The name known last at this instant names the player
        let player = d.player_for_entity(i64::from(entity));
        if let Some(switch) = self.switches.iter_mut().find(|s| s.entity == entity) {
            switch.after = spectating;
            switch.player = player.or(switch.player.take());
            return;
        }
        self.switches.push(Switch {
            entity,
            segment: d.segment,
            time,
            before: !spectating,
            after: spectating,
            player,
        });
    }

    /// A `Vote`, recorded when it is called, when its tally changes and when
    /// it ends
    fn vote(&mut self, d: &Decoder, time: f32, body: &[u8]) {
        let Some(now) = vote_message(body) else {
            return;
        };

        // The vote the last message was about, if this one is still it
        let same = |b: &VoteMessage| b.subject == now.subject && b.called_by == now.called_by;
        let before = self.vote.take().filter(same);
        let subject = now.subject.clone();
        match (now.status, before) {
            (VOTE_OPEN, Some(before)) if before.status == VOTE_OPEN => {
                if before.tally != now.tally {
                    let change = MatchChange::VoteCast {
                        subject,
                        yes: i16::from(now.tally.yes) - i16::from(before.tally.yes),
                        no: i16::from(now.tally.no) - i16::from(before.tally.no),
                        tally: now.tally,
                    };
                    self.push(d, time, None, change);
                }
            }
            (VOTE_OPEN, _) => {
                let caller = d.player_named(&now.called_by);
                let change = MatchChange::VoteCalled {
                    subject,
                    tally: now.tally,
                };
                self.push(d, time, caller, change);
            }
            (VOTE_ACCEPTED | VOTE_DENIED, before) => {
                let already_ended = before.is_some_and(|b| b.status == now.status);
                if !already_ended {
                    let change = if now.status == VOTE_ACCEPTED {
                        MatchChange::VoteAccepted { subject }
                    } else {
                        MatchChange::VoteDenied { subject }
                    };
                    self.push(d, time, None, change);
                }
            }

            // Hidden, so the next open one is a new vote
            _ => return,
        }

        self.vote = Some(now);
    }

    /// A `Timeout`, recorded when it is called, gets under way and is over
    fn timeout(&mut self, d: &Decoder, time: f32, body: &[u8]) {
        let (Some(&status), Some(&seconds)) = (body.first(), body.get(1)) else {
            return;
        };
        match (self.timeout, status) {
            // The server repeats a status every second
            (Some(TIMEOUT_CALLED), TIMEOUT_CALLED)
            | (Some(TIMEOUT_UNDER_WAY), TIMEOUT_UNDER_WAY) => {}
            (_, TIMEOUT_CALLED) => {
                let caller = self
                    .timeout_caller
                    .take()
                    .and_then(|name| d.player_named(&name));
                self.timeout = Some(TIMEOUT_CALLED);
                self.push(d, time, caller, MatchChange::TimeoutCalled);
            }
            // One under way that this recording starts in, but not the
            // server's last word on a timeout already over
            (Some(TIMEOUT_CALLED), TIMEOUT_UNDER_WAY) | (None, TIMEOUT_UNDER_WAY)
                if seconds > 0 =>
            {
                self.timeout = Some(TIMEOUT_UNDER_WAY);
                self.push(d, time, None, MatchChange::TimeoutUnderWay);
            }
            (Some(_), 0) => {
                self.timeout = None;
                self.push(d, time, None, MatchChange::TimeoutOver);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vote(status: u8, yes: u8, no: u8, undecided: u8) -> Vec<u8> {
        let mut body = vec![status, yes, no, undecided];
        body.extend(b"agstart\0\0p\0");
        body
    }

    fn changes(events: &MatchEvents) -> Vec<MatchChange> {
        events.events.iter().map(|e| e.change.clone()).collect()
    }

    /// A decoder with `name` in the first slot, entity 1, and a spectator
    /// proxy in the second
    fn decoder_with(name: &str) -> Decoder {
        let mut d = Decoder::new();
        d.max_clients = 8;
        let player = |slot: u8, name: &str, info: &[u8]| crate::state::PlayerInfo {
            slot,
            user_id: u32::from(slot) + 1,
            info: info.to_vec(),
            name: name.to_owned(),
            incarnation: 0,
        };
        d.players.insert(0, player(0, name, br"\name\p"));
        d.players
            .insert(1, player(1, "HLTV", br"\name\HLTV\*hltv\1"));
        d
    }

    #[test]
    fn switches_at_one_instant_count_once_and_proxies_are_left_out() {
        let d = decoder_with("punk");
        let mut m = MatchEvents::default();

        // A join switches back and forth at once, and ends spectating
        m.message(&d, 1., "Spectator", &[1, 1]);
        m.message(&d, 1., "Spectator", &[1, 0]);
        m.message(&d, 1., "Spectator", &[1, 1]);
        m.message(&d, 1., "Spectator", &[2, 1]);

        // Sent again while known, then back to playing
        m.message(&d, 5., "Spectator", &[1, 1]);
        m.message(&d, 9., "Spectator", &[1, 0]);
        m.finish();

        assert_eq!(changes(&m), [MatchChange::Spectating, MatchChange::Playing]);
        let named: Vec<_> = m
            .events
            .iter()
            .map(|e| e.player.clone().map(|p| p.name))
            .collect();
        assert_eq!(named, [Some("punk".into()), Some("punk".into())]);
    }

    #[test]
    fn a_change_of_map_names_the_new_map() {
        let d = Decoder::new();
        let mut m = MatchEvents::default();
        m.map_changed(&d, 900., "elixir".into());
        assert_eq!(
            changes(&m),
            [MatchChange::MapChanged {
                map: "elixir".into()
            }]
        );
    }

    #[test]
    fn a_vote_is_called_cast_and_ends_once_each() {
        let d = Decoder::new();
        let mut m = MatchEvents::default();
        for (time, body) in [
            (0., vote(1, 1, 0, 7)),
            (2., vote(1, 1, 0, 7)),
            (4., vote(1, 3, 1, 4)),
            (6., vote(2, 3, 1, 4)),
            (8., vote(2, 3, 1, 4)),
        ] {
            m.message(&d, time, "Vote", &body);
        }

        let subject = "agstart".to_owned();
        assert_eq!(
            changes(&m),
            [
                MatchChange::VoteCalled {
                    subject: subject.clone(),
                    tally: Tally {
                        yes: 1,
                        no: 0,
                        undecided: 7
                    },
                },
                MatchChange::VoteCast {
                    subject: subject.clone(),
                    yes: 2,
                    no: 1,
                    tally: Tally {
                        yes: 3,
                        no: 1,
                        undecided: 4
                    },
                },
                MatchChange::VoteAccepted { subject },
            ]
        );
    }

    #[test]
    fn a_timeout_is_called_under_way_and_over_once_each() {
        let d = Decoder::new();
        let mut m = MatchEvents::default();
        m.message(&d, 0., "TextMsg", b"\x02Timeout called by p\n\n\0");
        for (time, status, seconds) in [
            (0., 1, 9),
            (1., 1, 8),
            (10., 3, 59),
            (11., 3, 58),
            (70., 0, 0),
            (70., 3, 0),
            (73., 0, 0),
        ] {
            m.message(&d, time, "Timeout", &[status, seconds]);
        }
        m.pause(&d, 10., true);
        m.pause(&d, 70., false);

        assert_eq!(
            changes(&m),
            [
                MatchChange::TimeoutCalled,
                MatchChange::TimeoutUnderWay,
                MatchChange::TimeoutOver,
                MatchChange::Paused,
                MatchChange::Resumed,
            ]
        );
    }

    #[test]
    fn sudden_death_starts_and_ends_once() {
        let d = Decoder::new();
        let mut m = MatchEvents::default();
        for (time, on) in [(0., 0), (1., 1), (2., 1), (3., 0)] {
            m.message(&d, time, "SuddenDeath", &[on]);
        }
        assert_eq!(
            changes(&m),
            [MatchChange::SuddenDeath, MatchChange::SuddenDeathOver]
        );
    }
}
