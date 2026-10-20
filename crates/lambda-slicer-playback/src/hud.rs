//! The parts of AG's HUD playback shows, its countdown before a match
//!
//! AG's client counts down when the server sends `Countdown`, a signed byte
//! of seconds left, a byte saying whether to speak it, and the names of the
//! two players a duel is between (OpenAG's `CHudCountdown`). From ten
//! seconds down a spoken number plays as it comes, `fvox/ten.wav` to
//! `fvox/one.wav` and `barney/ba_bring.wav` at nought, and the countdown shows
//! for five seconds from the last one spoken. Fifty seconds left means the
//! match waits for its players, shown until the next message, and less
//! than nought hides it

use lambda_slicer_core::protocol::messages::{Message, ParsedMessage};
use lambda_slicer_core::state::Decoder;

use crate::time::SourceTime;

/// The user message AG's server counts down with
const MESSAGE: &str = "Countdown";

/// The seconds that mean the match waits for its players
pub const WAITING: i8 = 50;

/// How long the countdown shows after a spoken number, in seconds
const SHOWN: f64 = 5.;

/// The sound each number from nought to ten is spoken with
const SPOKEN: [&str; 11] = [
    "barney/ba_bring.wav",
    "fvox/one.wav",
    "fvox/two.wav",
    "fvox/three.wav",
    "fvox/four.wav",
    "fvox/five.wav",
    "fvox/six.wav",
    "fvox/seven.wav",
    "fvox/eight.wav",
    "fvox/nine.wav",
    "fvox/ten.wav",
];

/// What the countdown shows
#[derive(Clone, Debug, PartialEq)]
pub struct Countdown {
    /// Seconds left, or [`WAITING`] while the match waits for its players
    pub seconds: i8,
    /// The two players of a duel, or while waiting the last round's
    /// winner first. Empty when there are none, colour codes such as `^1`
    /// left in
    pub names: [String; 2],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hud {
    countdown: Option<Countdown>,
    /// When the countdown stops showing, five seconds after the last
    /// spoken number
    until: Option<SourceTime>,
}

/// A string ending at its first nought, and what follows it
fn c_string(bytes: &[u8]) -> (String, &[u8]) {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]).into_owned();
    (text, bytes.get(end + 1..).unwrap_or_default())
}

impl Hud {
    /// A message the decoder has just applied at `time`, and the sound the
    /// client plays for it, if any
    pub(crate) fn on_message(
        &mut self,
        decoder: &Decoder,
        m: &ParsedMessage,
        time: SourceTime,
    ) -> Option<&'static str> {
        let (id, body) = match &m.message {
            // A new session starts with nothing on the HUD
            Message::ServerInfo(_) => {
                *self = Self::default();
                return None;
            }
            Message::User { id, body } => (*id, body),
            _ => return None,
        };
        let name = decoder.user_message(id)?.name_lossy();
        if name != MESSAGE {
            return None;
        }

        let (&seconds, &spoken) = (body.first()?, body.get(1)?);
        let seconds = seconds as i8;
        if seconds < 0 {
            self.countdown = None;
            return None;
        }
        let (first, rest) = c_string(body.get(2..).unwrap_or_default());
        let (second, _) = c_string(rest);
        self.countdown = Some(Countdown {
            seconds,
            names: [first, second],
        });

        let sample = SPOKEN.get(usize::try_from(seconds).ok()?)?;
        if spoken == 0 {
            return None;
        }
        self.until = Some(SourceTime(time.0 + SHOWN));
        Some(sample)
    }

    /// The countdown showing at `at`, none when there is none
    pub fn countdown(&self, at: SourceTime) -> Option<&Countdown> {
        let countdown = self.countdown.as_ref()?;
        let shown = countdown.seconds == WAITING || self.until.is_some_and(|until| at < until);
        shown.then_some(countdown)
    }
}
