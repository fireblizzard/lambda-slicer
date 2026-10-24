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
//!
//! `HudText` names a message of the game's `titles.txt` instead, such as a
//! singleplayer chapter's title, which shows in the same list (HLSDK's
//! `CHudMessage::MessageAdd`). A leading `#` is left out of the name, which
//! is found ignoring case. A name the file lacks shows as its own text in
//! the client's custom style, and the client keeps a single custom message,
//! so a second one still listed takes over the first's text and start. A
//! message whose text is already listed is not added again, and at most 16
//! are listed
//!
//! `titles.txt` is read as the engine reads it (`TextMessageParse`). Lines
//! starting with `$` set the place, effect, colours and times of the
//! messages after them, a line on its own names the next message, and its
//! text is every line between a `{` and a `}`. Blank lines and lines
//! starting with `//` are left out, and a stray `{` or `}` makes the engine
//! keep no messages at all

use std::collections::HashMap;

use crate::time::SourceTime;

/// The temporary entity a message comes as
pub const TE_TEXTMESSAGE: u8 = 29;

/// How many channels the engine keeps (`MAX_NETMESSAGE`)
const CHANNELS: usize = 6;

/// The effect that writes the text out a letter at a time
pub const WRITE_OUT: u8 = 2;

/// How many messages the client lists at once (`maxHUDMessages`)
const LISTED: usize = 16;

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

/// A message from `titles.txt`, or a custom one, with its start
#[derive(Clone, Debug, PartialEq)]
struct Titled {
    message: HudMessage,
    start: SourceTime,
    custom: bool,
}

/// The messages by channel, each with its start, and the titled ones
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudMessages {
    channels: [Option<(HudMessage, SourceTime)>; CHANNELS],
    /// The channel a message on none takes next
    next: usize,
    titled: Vec<Titled>,
}

/// A name `titles.txt` lacks, shown as its own text in the client's custom
/// style, written out in the middle a little below the centre
fn custom(name: &str) -> HudMessage {
    HudMessage {
        x: -1.,
        y: 0.7,
        effect: WRITE_OUT,
        colour: [100, 100, 100],
        colour2: [240, 110, 0],
        fade_in: 0.01,
        fade_out: 1.5,
        hold: 5.,
        settle: 0.25,
        text: name.to_owned(),
    }
}

/// The messages of a `titles.txt`, by their names in lowercase. The first
/// of a name wins, as the engine looks them up in order
pub fn parse_titles(file: &[u8]) -> HashMap<String, HudMessage> {
    let mut titles = HashMap::new();

    // The settings start at nought and carry on to every later message
    let mut style = HudMessage {
        x: 0.,
        y: 0.,
        effect: 0,
        colour: [0; 3],
        colour2: [0; 3],
        fade_in: 0.,
        fade_out: 0.,
        hold: 0.,
        settle: 0.,
        text: String::new(),
    };
    let mut name = String::new();
    let mut text: Option<Vec<&[u8]>> = None;

    for raw in file.split(|&b| b == b'\n') {
        let line = String::from_utf8_lossy(raw);
        let trimmed = line.trim();

        if let Some(lines) = &mut text {
            if trimmed.starts_with('}') {
                let joined = lines.join(&b'\n');
                let mut message = style.clone();
                message.text = String::from_utf8_lossy(&joined).replace('\r', "");
                titles.entry(name.to_ascii_lowercase()).or_insert(message);
                text = None;
            } else if trimmed.starts_with('{') {
                return HashMap::new();
            } else {
                lines.push(raw);
            }
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(directive) = trimmed.strip_prefix('$') {
            set(&mut style, directive);
        } else if trimmed.starts_with('{') {
            text = Some(Vec::new());
        } else if trimmed.starts_with('}') {
            return HashMap::new();
        } else {
            name = trimmed.to_owned();
        }
    }

    titles
}

/// The numbers after a directive's word, nought for any missing or not a
/// number, as C's `atof` reads them
fn numbers<const N: usize>(directive: &str) -> [f32; N] {
    let mut words = directive.split_whitespace().skip(1);
    std::array::from_fn(|_| words.next().map_or(0., leading_number))
}

/// The number `word` starts with, nought when it starts with none
fn leading_number(word: &str) -> f32 {
    let mut end = 0;
    for (i, c) in word.char_indices() {
        let sign = i == 0 && (c == '-' || c == '+');
        if !(sign || c.is_ascii_digit() || c == '.') {
            break;
        }
        end = i + c.len_utf8();
    }
    word[..end].parse().unwrap_or(0.)
}

/// `style` with the directive applied, its word matched as a prefix
/// ignoring case. `color2` is tried before `color`, which it starts with,
/// and an unknown word changes nothing
fn set(style: &mut HudMessage, directive: &str) {
    let word = directive.to_ascii_lowercase();
    let colour = |[r, g, b]: [f32; 3]| [r as u8, g as u8, b as u8];

    if word.starts_with("position") {
        [style.x, style.y] = numbers(directive);
    } else if word.starts_with("effect") {
        style.effect = numbers::<1>(directive)[0] as u8;
    } else if word.starts_with("fxtime") {
        style.settle = numbers::<1>(directive)[0];
    } else if word.starts_with("color2") {
        style.colour2 = colour(numbers(directive));
    } else if word.starts_with("color") {
        style.colour = colour(numbers(directive));
    } else if word.starts_with("fadein") {
        style.fade_in = numbers::<1>(directive)[0];
    } else if word.starts_with("fadeout") {
        style.fade_out = numbers::<1>(directive)[0];
    } else if word.starts_with("holdtime") {
        style.hold = numbers::<1>(directive)[0];
    }
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
        self.titled.retain(|t| !same_place(&t.message, &message));
        self.channels[channel] = Some((message, start));
    }

    /// A `HudText` naming `name` at `time`, with the message `titles.txt`
    /// has for it, if any
    pub(crate) fn title(&mut self, name: &str, found: Option<HudMessage>, time: SourceTime) {
        // The client drops a message from its list once it is over
        let over = |message: &HudMessage, start: SourceTime| {
            time.0 > start.0 + f64::from(message.length())
        };
        self.titled.retain(|t| !over(&t.message, t.start));
        for slot in &mut self.channels {
            if slot.as_ref().is_some_and(|(m, start)| over(m, *start)) {
                *slot = None;
            }
        }

        if self.listed().count() >= LISTED {
            return;
        }

        // The one custom message takes the new text, so a custom message
        // still listed shows it from its own start
        let is_custom = found.is_none();
        let message = found.unwrap_or_else(|| custom(name));
        if is_custom && let Some(shown) = self.titled.iter_mut().find(|t| t.custom) {
            shown.message.text = message.text;
            return;
        }

        // Already listed by its text, it is not added again
        for (listed, _) in self.listed() {
            if listed.text == message.text {
                return;
            }
        }

        for slot in &mut self.channels {
            if slot.as_ref().is_some_and(|(m, _)| same_place(m, &message)) {
                *slot = None;
            }
        }
        self.titled.retain(|t| !same_place(&t.message, &message));
        self.titled.push(Titled {
            message,
            start: time,
            custom: is_custom,
        });
    }

    /// Every message listed, on its channel or titled, with its start
    fn listed(&self) -> impl Iterator<Item = (&HudMessage, SourceTime)> {
        let channels = self.channels.iter().flatten().map(|(m, start)| (m, *start));
        let titled = self.titled.iter().map(|t| (&t.message, t.start));
        channels.chain(titled)
    }

    /// The messages showing at `at`
    pub fn at(&self, at: SourceTime) -> Vec<ShownMessage> {
        let mut out = Vec::new();
        for (message, start) in self.listed() {
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

    /// A `titles.txt` in the game's own style, with Windows line ends
    const TITLES: &str = "//TITLES FOR HALF-LIFE\r\n\
        $position -1 -1\r\n\
        $effect 2\r\n\
        // The final colour, then the highlight\r\n\
        $color 100 100 100\r\n\
        $color2 240 110 0\r\n\
        $fadein 0.01\r\n\
        $fxtime 0.25\r\n\
        $holdtime 3.5\r\n\
        $fadeout 1.5\r\n\
        \r\n\
        CR27\r\n\
        {\r\n\
        Black Mesa Research Facility\r\n\
        Black Mesa, New Mexico\r\n\
        }\r\n\
        \r\n\
        $position 0.1 0.8\r\n\
        $effect 1\r\n\
        CR1\r\n\
        {\r\n\
        Ted Backman\r\n\
        }\r\n";

    #[test]
    fn titles_read_with_their_settings_carried_to_later_messages() {
        let titles = parse_titles(TITLES.as_bytes());
        assert_eq!(titles.len(), 2);

        let intro = &titles["cr27"];
        assert_eq!(
            intro.text,
            "Black Mesa Research Facility\nBlack Mesa, New Mexico"
        );
        assert_eq!((intro.x, intro.y, intro.effect), (-1., -1., WRITE_OUT));
        assert_eq!((intro.colour, intro.colour2), ([100; 3], [240, 110, 0]));
        assert_eq!((intro.fade_in, intro.settle), (0.01, 0.25));
        assert_eq!((intro.hold, intro.fade_out), (3.5, 1.5));

        // Only the place and effect change, the rest carries on
        let credit = &titles["cr1"];
        assert_eq!(credit.text, "Ted Backman");
        assert_eq!((credit.x, credit.y, credit.effect), (0.1, 0.8, 1));
        assert_eq!((credit.colour, credit.hold), ([100; 3], 3.5));
    }

    #[test]
    fn a_stray_brace_leaves_no_titles_as_in_the_engine() {
        assert!(parse_titles(b"A\n{\ntext\n{\n}\n").is_empty());
        assert!(parse_titles(b"A\n}\n").is_empty());
    }

    #[test]
    fn a_title_shows_once_and_clears_its_place() {
        let titles = parse_titles(TITLES.as_bytes());
        let mut messages = HudMessages::default();
        messages.message(&body(1, -1., -1., 5., "server"), SourceTime(0.));

        // At the same place as the server's text, it takes over
        messages.title("CR27", titles.get("cr27").cloned(), SourceTime(1.));
        let shown = messages.at(SourceTime(1.));
        assert_eq!(shown.len(), 1);
        assert!(shown[0].message.text.starts_with("Black Mesa"));

        // Sent again while listed, it keeps its start
        messages.title("CR27", titles.get("cr27").cloned(), SourceTime(2.));
        assert_eq!(messages.at(SourceTime(2.))[0].since, 1.);
    }

    #[test]
    fn a_name_the_file_lacks_shows_as_the_one_custom_message() {
        let mut messages = HudMessages::default();
        messages.title("#Hello", None, SourceTime(0.));
        let shown = messages.at(SourceTime(0.));
        assert_eq!(shown[0].message.text, "#Hello");
        assert_eq!((shown[0].message.x, shown[0].message.y), (-1., 0.7));

        // A second custom message takes over the first's text and start
        messages.title("World", None, SourceTime(1.));
        let shown = messages.at(SourceTime(1.));
        assert_eq!(shown.len(), 1);
        assert_eq!(
            (shown[0].message.text.as_str(), shown[0].since),
            ("World", 1.)
        );
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
