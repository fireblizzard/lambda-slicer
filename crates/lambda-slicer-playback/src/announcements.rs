//! AG's messages over the view that tell how the match goes, the time
//! left, sudden death, the next map, a timeout or a pause, and a vote
//! (OpenAG's `CHudTimer`, `CHudSuddenDeath`, `CHudNextmap`, `CHudTimeout`
//! and `CHudVote`)
//!
//! `Timer` is two longs, the match's seconds and the seconds passed, each
//! message showing the time for five seconds, the time left or without a
//! time limit the time passed. `SuddenDeath` is a byte that turns it on or
//! off, and `Nextmap` the next map's name, shown for ten seconds.
//! `Timeout` is a status and seconds left, its stopping and its starting
//! counted down while they last. `Vote` is a status, the votes for, against
//! and not yet given, what is voted on as two strings and who called it,
//! each message showing it for four seconds. The engine's `svc_setpause`
//! says whether the game is paused

use lambda_slicer_core::analysis::match_events::{
    TIMEOUT_CALLED, TIMEOUT_UNDER_WAY, VOTE_ACCEPTED, VOTE_DENIED, VOTE_OPEN, vote_message,
};

use crate::time::SourceTime;

/// How long the time, the next map and a vote show after their message, in
/// seconds
const TIMER_SHOWN: f64 = 5.;
const NEXT_MAP_SHOWN: f64 = 10.;
const VOTE_SHOWN: f64 = 4.;

/// A timeout's countdown
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timeout {
    /// Called, the game stopping in these seconds
    Stopping(u8),
    /// Under way, the game starting again in these seconds
    Starting(u8),
}

/// How a vote stands
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoteStatus {
    /// Open, with the votes for, against and not yet given
    Open {
        yes: u8,
        no: u8,
        undecided: u8,
    },
    Accepted,
    Denied,
}

/// A vote, what it is on and who called it, colour codes such as `^1` left
/// in their name
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vote {
    pub status: VoteStatus,
    pub subject: String,
    pub called_by: String,
}

/// What shows at a moment
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Announced {
    /// The seconds left, or passed when the match has no time limit
    pub timer: Option<i64>,
    pub sudden_death: bool,
    pub next_map: Option<String>,
    pub timeout: Option<Timeout>,
    pub paused: bool,
    pub vote: Option<Vote>,
}

/// What the messages last said, and until when each shows
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Announcements {
    timer: Option<(i64, SourceTime)>,
    sudden_death: bool,
    next_map: Option<(String, SourceTime)>,
    timeout: Option<Timeout>,
    paused: bool,
    vote: Option<(Vote, SourceTime)>,
}

/// A string ending at its first nought, and what follows it
fn c_string(bytes: &[u8]) -> (String, &[u8]) {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]).into_owned();
    (text, bytes.get(end + 1..).unwrap_or_default())
}

/// What `entry` holds while it still shows at `at`, none after
fn while_shown<T: Clone>(entry: &Option<(T, SourceTime)>, at: SourceTime) -> Option<T> {
    let (value, until) = entry.as_ref()?;
    (at < *until).then(|| value.clone())
}

/// The little-endian long at `at` in `body`
fn long(body: &[u8], at: usize) -> Option<i64> {
    let bytes = body.get(at..at + 4)?.try_into().ok()?;
    Some(i64::from(i32::from_le_bytes(bytes)))
}

impl Announcements {
    /// The user message `name` with `body` at `time`. Returns whether it
    /// was one of these
    pub(crate) fn message(&mut self, name: &str, body: &[u8], time: SourceTime) -> bool {
        match name {
            "Timer" => {
                if let (Some(total), Some(passed)) = (long(body, 0), long(body, 4)) {
                    let shown = if total == 0 { passed } else { total - passed };
                    self.timer = Some((shown, SourceTime(time.0 + TIMER_SHOWN)));
                }
            }
            "SuddenDeath" => self.sudden_death = body.first().is_some_and(|&b| b != 0),
            "Nextmap" => {
                let (map, _) = c_string(body);
                self.next_map = Some((map, SourceTime(time.0 + NEXT_MAP_SHOWN)));
            }
            "Timeout" => {
                let (status, seconds) = (body.first().copied(), body.get(1).copied());
                self.timeout = match (status, seconds) {
                    (Some(TIMEOUT_CALLED), Some(s)) => Some(Timeout::Stopping(s)),
                    (Some(TIMEOUT_UNDER_WAY), Some(s)) => Some(Timeout::Starting(s)),
                    _ => None,
                };
            }
            "Vote" => self.vote(body, time),
            _ => return false,
        }
        true
    }

    /// A `Vote`, its status and counts, then what it is on and who called it
    fn vote(&mut self, body: &[u8], time: SourceTime) {
        let Some(message) = vote_message(body) else {
            return;
        };

        let tally = message.tally;
        let status = match message.status {
            VOTE_OPEN => VoteStatus::Open {
                yes: tally.yes,
                no: tally.no,
                undecided: tally.undecided,
            },
            VOTE_ACCEPTED => VoteStatus::Accepted,
            VOTE_DENIED => VoteStatus::Denied,

            // Any other status hides it
            _ => {
                self.vote = None;
                return;
            }
        };

        let vote = Vote {
            status,
            subject: message.subject,
            called_by: message.called_by,
        };
        self.vote = Some((vote, SourceTime(time.0 + VOTE_SHOWN)));
    }

    /// Whether the game is paused, as `svc_setpause` says
    pub(crate) fn pause(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// What shows at `at`
    pub fn at(&self, at: SourceTime) -> Announced {
        Announced {
            timer: while_shown(&self.timer, at),
            sudden_death: self.sudden_death,
            next_map: while_shown(&self.next_map, at),
            timeout: self.timeout,
            paused: self.paused,
            vote: while_shown(&self.vote, at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timer_shows_the_time_left_for_five_seconds() {
        let mut a = Announcements::default();
        // 20 minutes, 12 seconds passed, as a recorded one came
        let body = [0xb0, 0x04, 0, 0, 0x0c, 0, 0, 0];
        assert!(a.message("Timer", &body, SourceTime(12.)));
        assert_eq!(a.at(SourceTime(13.)).timer, Some(1188));
        assert_eq!(a.at(SourceTime(17.)).timer, None);

        // Without a time limit, the time passed
        a.message("Timer", &[0, 0, 0, 0, 0x3c, 0, 0, 0], SourceTime(20.));
        assert_eq!(a.at(SourceTime(20.)).timer, Some(60));
    }

    #[test]
    fn a_timeout_counts_its_stopping_then_its_starting() {
        let mut a = Announcements::default();
        a.message("Timeout", &[1, 9], SourceTime(0.));
        assert_eq!(a.at(SourceTime(0.)).timeout, Some(Timeout::Stopping(9)));
        a.message("Timeout", &[3, 59], SourceTime(10.));
        assert_eq!(a.at(SourceTime(10.)).timeout, Some(Timeout::Starting(59)));
        a.message("Timeout", &[0, 0], SourceTime(70.));
        assert_eq!(a.at(SourceTime(70.)).timeout, None);
    }

    #[test]
    fn a_vote_shows_its_counts_for_four_seconds() {
        let mut a = Announcements::default();
        a.message("Vote", b"\x01\x01\x00\x07agstart\0\0p\0", SourceTime(0.));
        let vote = a.at(SourceTime(1.)).vote.unwrap();
        assert_eq!(
            vote.status,
            VoteStatus::Open {
                yes: 1,
                no: 0,
                undecided: 7
            }
        );
        assert_eq!(
            (vote.subject.as_str(), vote.called_by.as_str()),
            ("agstart", "p")
        );
        assert_eq!(a.at(SourceTime(4.)).vote, None);
    }

    #[test]
    fn the_next_map_shows_for_ten_seconds_and_sudden_death_until_off() {
        let mut a = Announcements::default();
        a.message("Nextmap", b"endcamp\0", SourceTime(0.));
        a.message("SuddenDeath", &[1], SourceTime(0.));
        let now = a.at(SourceTime(9.));
        assert_eq!(now.next_map.as_deref(), Some("endcamp"));
        assert!(now.sudden_death);
        assert_eq!(a.at(SourceTime(10.)).next_map, None);

        a.message("SuddenDeath", &[0], SourceTime(11.));
        assert!(!a.at(SourceTime(11.)).sudden_death);
    }
}
