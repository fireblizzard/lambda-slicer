//! AG's messages over the view that tell how the match goes, the time
//! left, sudden death, the next map, a timeout or a pause, a vote and the
//! server's settings (OpenAG's `CHudTimer`, `CHudSuddenDeath`,
//! `CHudNextmap`, `CHudTimeout`, `CHudVote` and `CHudSettings`)
//!
//! `Timer` is two longs, the match's seconds and the seconds passed, each
//! message showing the time for five seconds, the time left or without a
//! time limit the time passed. `SuddenDeath` is a byte that turns it on or
//! off, and `Nextmap` the next map's name, shown for ten seconds.
//! `Timeout` is a status and seconds left, its stopping and its starting
//! counted down while they last. `Vote` is a status, the votes for, against
//! and not yet given, what is voted on as two strings and who called it,
//! each message showing it for four seconds. `Settings` is whether a match
//! is on, the game mode, the time and frag limits, friendly fire and weapon
//! stay, AG's version, then the wallgauss, headshot and blast radius
//! multipliers, each shown only when not the game's own, all for ten
//! seconds. The engine's `svc_setpause` says whether the game is paused

use lambda_slicer_core::analysis::match_events::{
    TIMEOUT_CALLED, TIMEOUT_UNDER_WAY, VOTE_ACCEPTED, VOTE_DENIED, VOTE_OPEN, vote_message,
};

use crate::time::SourceTime;

/// How long the time, the next map and a vote show after their message, in
/// seconds
const TIMER_SHOWN: f64 = 5.;
const NEXT_MAP_SHOWN: f64 = 10.;
const VOTE_SHOWN: f64 = 4.;
const SETTINGS_SHOWN: f64 = 10.;

/// The game's own wallgauss, headshot and blast radius multipliers, which
/// the settings leave out
const USUAL_WALLGAUSS: &str = "1";
const USUAL_HEADSHOT: &str = "3";
const USUAL_BLAST_RADIUS: &str = "1";

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

/// The server's settings, as `Settings` gave them
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerSettings {
    pub match_on: bool,
    pub mode: String,
    /// In minutes
    pub time_limit: i8,
    pub frag_limit: i8,
    pub friendly_fire: bool,
    pub weapon_stay: bool,
    pub version: String,
    /// Each multiplier as written, none when it is the game's own
    pub wallgauss: Option<String>,
    pub headshot: Option<String>,
    pub blast_radius: Option<String>,
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
    pub settings: Option<ServerSettings>,
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
    settings: Option<(ServerSettings, SourceTime)>,
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

/// `multiplier` unless it is empty or `usual`
fn unusual(multiplier: String, usual: &str) -> Option<String> {
    let unusual = !multiplier.is_empty() && multiplier != usual;
    unusual.then_some(multiplier)
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
            "Settings" => self.settings(body, time),
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

    /// A `Settings`, shown for ten seconds
    fn settings(&mut self, body: &[u8], time: SourceTime) {
        let Some((&match_on, rest)) = body.split_first() else {
            return;
        };
        let (mode, rest) = c_string(rest);
        let [time_limit, frag_limit, friendly_fire, weapon_stay] =
            rest.get(..4).unwrap_or_default()
        else {
            return;
        };
        let (version, rest) = c_string(&rest[4..]);
        let (wallgauss, rest) = c_string(rest);
        let (headshot, rest) = c_string(rest);
        let (blast_radius, _) = c_string(rest);

        let settings = ServerSettings {
            match_on: match_on != 0,
            mode,
            time_limit: *time_limit as i8,
            frag_limit: *frag_limit as i8,
            friendly_fire: *friendly_fire != 0,
            weapon_stay: *weapon_stay != 0,
            version,
            wallgauss: unusual(wallgauss, USUAL_WALLGAUSS),
            headshot: unusual(headshot, USUAL_HEADSHOT),
            blast_radius: unusual(blast_radius, USUAL_BLAST_RADIUS),
        };
        self.settings = Some((settings, SourceTime(time.0 + SETTINGS_SHOWN)));
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
            settings: while_shown(&self.settings, at),
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

    #[test]
    fn the_settings_show_for_ten_seconds_leaving_out_the_usual_multipliers() {
        let mut a = Announcements::default();
        let body = b"\x01Arena\x00\x0a\x00\x00\x01 6.6\x002\x003\x00\x00";
        assert!(a.message("Settings", body, SourceTime(0.)));
        let settings = a.at(SourceTime(9.)).settings.unwrap();
        assert!(settings.match_on);
        assert_eq!(
            (settings.mode.as_str(), settings.version.as_str()),
            ("Arena", " 6.6")
        );
        assert_eq!((settings.time_limit, settings.frag_limit), (10, 0));
        assert!(!settings.friendly_fire && settings.weapon_stay);
        assert_eq!(settings.wallgauss.as_deref(), Some("2"));
        assert_eq!((settings.headshot, settings.blast_radius), (None, None));
        assert_eq!(a.at(SourceTime(10.)).settings, None);
    }
}
