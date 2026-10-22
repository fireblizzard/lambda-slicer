//! Text the server places on the HUD, such as a bhop or KZ server's timer
//! and stats (the engine's `TE_TEXTMESSAGE`, which HLSDK's `CHudMessage`
//! draws)
//!
//! A message is a channel, where it stands as shares of the screen, an
//! effect, two colours, how long it fades in, fades out and holds, for
//! the writing out effect how long a letter takes to settle, and its text.
//! The engine keeps one message a channel, so a message on a channel still
//! showing takes its place and keeps its start, as the client finds the
//! channel already listed. A message on a channel past the last, or on
//! none, takes the next in turn. A new message also clears any other
//! standing at exactly the same place

use crate::time::SourceTime;

/// The temporary entity a message comes as
pub const TE_TEXTMESSAGE: u8 = 29;

/// How many channels the engine keeps (`MAX_NETMESSAGE`)
const CHANNELS: usize = 6;

/// The effect that writes the text out a letter at a time
pub const WRITE_OUT: u8 = 2;

/// One message as the server sent it
#[derive(Clone, Debug, PartialEq)]
pub struct HudMessage {
    /// Shares of the screen's width and height, -1 for the middle and
    /// negative from the right or the bottom
    pub x: f32,
    pub y: f32,
    /// 0 fades in and out, 1 also flickers, `WRITE_OUT` writes out
    pub effect: u8,
    pub colour: [u8; 3],
    /// The colour a written out letter starts in, and a flicker's
    pub colour2: [u8; 3],
    /// Seconds to fade in, or for each letter to appear when written out,
    /// to fade out, to hold, and for a written out letter to settle
    pub fade_in: f32,
    pub fade_out: f32,
    pub hold: f32,
    pub settle: f32,
    pub text: String,
}

impl HudMessage {
    /// How long it shows from its start, in seconds
    pub fn length(&self) -> f32 {
        let fade_in = if self.effect == WRITE_OUT {
            self.fade_in * self.text.len() as f32
        } else {
            self.fade_in
        };
        fade_in + self.hold + self.fade_out
    }
}

/// A message showing, and the seconds since it started
#[derive(Clone, Debug, PartialEq)]
pub struct ShownMessage {
    pub message: HudMessage,
    pub since: f32,
}

/// The messages by channel, each with its start
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudMessages {
    channels: [Option<(HudMessage, SourceTime)>; CHANNELS],
    /// The channel a message on none takes next
    next: usize,
}

/// The two little-endian bytes at `at` of `body`
fn short(body: &[u8], at: usize) -> Option<[u8; 2]> {
    let bytes = body.get(at..at + 2)?;
    Some([bytes[0], bytes[1]])
}

/// The message in a `TE_TEXTMESSAGE`'s `body`, and its channel. Places are
/// signed and times unsigned, so a server's longest hold, such as 999999
/// seconds, arrives as a little over four minutes rather than as less
/// than nought (`FixedSigned16`, `FixedUnsigned16`)
fn parse(body: &[u8]) -> Option<(u8, HudMessage)> {
    let channel = *body.first()?;
    let share = |at| short(body, at).map(|b| f32::from(i16::from_le_bytes(b)) / 8192.);
    let seconds = |at| short(body, at).map(|b| f32::from(u16::from_le_bytes(b)) / 256.);
    let effect = *body.get(5)?;
    let colours = body.get(6..14)?;

    // The writing out effect has its settling time before the text
    let (settle, text_at) = if effect == WRITE_OUT {
        (seconds(20)?, 22)
    } else {
        (0., 20)
    };
    let text = body.get(text_at..)?;
    let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());

    let message = HudMessage {
        x: share(1)?,
        y: share(3)?,
        effect,
        colour: [colours[0], colours[1], colours[2]],
        colour2: [colours[4], colours[5], colours[6]],
        fade_in: seconds(14)?,
        fade_out: seconds(16)?,
        hold: seconds(18)?,
        settle,
        text: String::from_utf8_lossy(&text[..end]).into_owned(),
    };
    Some((channel, message))
}

/// Whether `a` and `b` stand at the same place
fn same_place(a: &HudMessage, b: &HudMessage) -> bool {
    (a.x - b.x).abs() < 0.0001 && (a.y - b.y).abs() < 0.0001
}

impl HudMessages {
    /// A `TE_TEXTMESSAGE`'s `body` at `time`
    pub(crate) fn message(&mut self, body: &[u8], time: SourceTime) {
        let Some((channel, message)) = parse(body) else {
            return;
        };
        let mut channel = usize::from(channel);
        if channel == 0 || channel >= CHANNELS {
            channel = self.next;
            self.next = (self.next + 1) % CHANNELS;
        }

        // Still showing on its channel, it keeps its start
        let showing = self.channels[channel]
            .as_ref()
            .filter(|(old, start)| time.0 <= start.0 + f64::from(old.length()));
        let start = showing.map_or(time, |(_, start)| *start);

        for (other, slot) in self.channels.iter_mut().enumerate() {
            let at_place = slot.as_ref().is_some_and(|(m, _)| same_place(m, &message));
            if other != channel && at_place {
                *slot = None;
            }
        }
        self.channels[channel] = Some((message, start));
    }

    /// The messages showing at `at`
    pub fn at(&self, at: SourceTime) -> Vec<ShownMessage> {
        let mut out = Vec::new();
        for (message, start) in self.channels.iter().flatten() {
            let since = (at.0 - start.0) as f32;
            if since >= 0. && since <= message.length() {
                let message = message.clone();
                out.push(ShownMessage { message, since });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `TE_TEXTMESSAGE` body on `channel` at `x`, `y` holding `hold`
    /// seconds
    fn body(channel: u8, x: f32, y: f32, hold: f32, text: &str) -> Vec<u8> {
        let mut b = vec![channel];
        b.extend(((x * 8192.) as i16).to_le_bytes());
        b.extend(((y * 8192.) as i16).to_le_bytes());
        b.push(0);
        b.extend([255, 160, 0, 255, 0, 0, 0, 0]);
        for seconds in [0.25, 0.5, hold] {
            b.extend(((seconds * 256.) as u16).to_le_bytes());
        }
        b.extend(text.as_bytes());
        b.push(0);
        b
    }

    #[test]
    fn a_message_reads_and_shows_for_its_fades_and_hold() {
        let (channel, m) = parse(&body(3, 0.75, -1., 1., "Speed: 300")).unwrap();
        assert_eq!(channel, 3);
        assert_eq!((m.x, m.y, m.text.as_str()), (0.75, -1., "Speed: 300"));
        assert_eq!((m.fade_in, m.fade_out, m.hold), (0.25, 0.5, 1.));

        let mut messages = HudMessages::default();
        messages.message(&body(3, 0.75, -1., 1., "a"), SourceTime(10.));
        assert_eq!(messages.at(SourceTime(11.7)).len(), 1);
        assert!(messages.at(SourceTime(11.8)).is_empty());
    }

    #[test]
    fn a_channel_still_showing_keeps_its_start_and_a_place_holds_one() {
        let mut messages = HudMessages::default();
        messages.message(&body(1, 0.1, 0.1, 1., "first"), SourceTime(0.));
        messages.message(&body(1, 0.1, 0.1, 1., "second"), SourceTime(1.));
        let shown = messages.at(SourceTime(1.));
        assert_eq!(shown.len(), 1);
        assert_eq!(
            (shown[0].message.text.as_str(), shown[0].since),
            ("second", 1.)
        );

        // Another channel at the same place clears the first
        messages.message(&body(2, 0.1, 0.1, 1., "third"), SourceTime(1.2));
        let texts: Vec<String> = messages
            .at(SourceTime(1.2))
            .into_iter()
            .map(|s| s.message.text)
            .collect();
        assert_eq!(texts, ["third"]);
    }

    #[test]
    fn the_longest_hold_stays_up_while_the_server_resends_it() {
        // 999999 seconds comes as the most a hold can be
        let (_, m) = parse(&body(1, 0.7, 0.2, 999999., "Sync: 0.00%")).unwrap();
        assert_eq!(m.hold, 65535. / 256.);

        let mut messages = HudMessages::default();
        messages.message(&body(1, 0.7, 0.2, 999999., "a"), SourceTime(0.));
        messages.message(&body(1, 0.7, 0.2, 999999., "b"), SourceTime(30.));
        let shown = messages.at(SourceTime(30.));
        assert_eq!((shown[0].message.text.as_str(), shown[0].since), ("b", 30.));
    }
}
