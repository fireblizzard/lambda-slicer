//! The parts of the HUD playback shows, AG's countdown before a match, the
//! kill feed, the chat, the messages [`crate::announcements`] keeps, the
//! health, armour and ammo [`crate::status`] keeps, the scores
//! [`crate::scores`] keeps, the damage [`crate::damage`] keeps, the
//! server's own text [`crate::hud_messages`] keeps, and the weapon menu
//! [`crate::weapon_menu`] keeps
//!
//! AG's client counts down when the server sends `Countdown`, a signed byte
//! of seconds left, a byte saying whether to speak it, and the names of the
//! two players a duel is between (OpenAG's `CHudCountdown`). From ten
//! seconds down a spoken number plays as it comes, `fvox/ten.wav` to
//! `fvox/one.wav` and `barney/ba_bring.wav` at nought, and the countdown shows
//! for five seconds from the last one spoken. Fifty seconds left means the
//! match waits for its players, shown until the next message, and less
//! than nought hides it
//!
//! The kill feed is the client's death notices (OpenAG's `CHudDeathNotice`).
//! Each `DeathMsg` adds a line for six seconds, four at most, the oldest
//! dropped for a fifth. A killer of nought or the victim itself is a death
//! with no killer named, and a weapon of `teammate` a team kill. Each name
//! keeps the team `TeamInfo` last gave its player, for its colour
//!
//! The chat is the client's say text (HLSDK's `CHudSayText`). `SayText` is
//! the sending player's entity and a line, which starting with byte 2 names
//! that player, to be drawn in their team's colour. Five lines show at
//! most, the oldest scrolled away for a sixth, and the top line goes five
//! seconds after it reached the top. AG's `Location` is a player's entity
//! and position, which a line's `%l` names (OpenAG's `CHudLocation`)

use std::collections::{BTreeMap, VecDeque};

use lambda_slicer_core::analysis::context::{location, say_text};
use lambda_slicer_core::protocol::messages::{Message, ParsedMessage, unpack_ping};
use lambda_slicer_core::state::Decoder;

use crate::announcements::{Announced, Announcements};
use crate::damage::{Damage, DamageShown};
use crate::hud_messages::{HudMessage, HudMessages, ShownMessage, TE_TEXTMESSAGE};
use crate::scene::Camera;
use crate::scores::{Board, ScoreRow, Scores};
use crate::status::{self, Status};
use crate::time::SourceTime;
use crate::weapon_menu::{MenuShown, WeaponMenu};

/// The user message AG's server counts down with
const MESSAGE: &str = "Countdown";

/// How many death notices show at once, and for how long in seconds
const NOTICES: usize = 4;
const NOTICE_SHOWN: f64 = 6.;

/// How many chat lines show at once, and how long the top one stays, in
/// seconds (`hud_saytext_time`)
const CHAT_LINES: usize = 5;
const CHAT_SHOWN: f64 = 5.;

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

/// A player as a death notice names them
#[derive(Clone, Debug, PartialEq)]
pub struct NoticeName {
    /// Colour codes such as `^1` left in
    pub name: String,
    /// The team `TeamInfo` gave them, empty without one
    pub team: String,
}

/// One line of the kill feed
#[derive(Clone, Debug, PartialEq)]
pub struct DeathNotice {
    /// None for a suicide or a death no player caused
    pub killer: Option<NoticeName>,
    pub victim: NoticeName,
    /// The death message's weapon, such as `9mmAR`, whose icon is `d_` and
    /// the weapon in `sprites/hud.txt`
    pub weapon: String,
    pub teamkill: bool,
    pub time: SourceTime,
}

/// One line of the chat
#[derive(Clone, Debug, PartialEq)]
pub struct ChatLine {
    /// Colour codes such as `^1` left in, and `%l` for the sender's place
    pub text: String,
    /// Where in the text the sender's name is, and their team, when the
    /// line names them
    pub name: Option<(std::ops::Range<usize>, String)>,
    /// Where the sender was when they sent it, as `Location` last gave it
    pub place: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hud {
    countdown: Option<Countdown>,
    /// When the countdown stops showing, five seconds after the last
    /// spoken number
    until: Option<SourceTime>,
    /// Each player's team by entity, as `TeamInfo` gave it
    teams: BTreeMap<u8, String>,
    /// The kill feed, oldest first
    notices: VecDeque<DeathNotice>,
    /// The chat, oldest first, and when its top line scrolls away
    chat: VecDeque<ChatLine>,
    scroll_at: SourceTime,
    /// Each player's place by entity, as `Location` gave it
    places: BTreeMap<u8, [f32; 3]>,
    /// The time left, sudden death, the next map, timeouts, pauses and votes
    announcements: Announcements,
    /// The recording player's health, armour and ammo
    status: Status,
    /// Every player's and team's frags, for the small scoreboard
    scores: Scores,
    /// Where the recording player is hit from and what hurts them
    damage: Damage,
    /// The text the server places on the HUD
    messages: HudMessages,
    /// The weapon menu the recording player opens with their keys
    menu: WeaponMenu,
    /// Each player's latency in milliseconds and packet loss by entity,
    /// which the server sends while the scoreboard shows, and the server's
    /// name
    pings: BTreeMap<u8, (u32, u32)>,
    server: String,
}

/// A string ending at its first nought, and what follows it
fn c_string(bytes: &[u8]) -> (String, &[u8]) {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]).into_owned();
    (text, bytes.get(end + 1..).unwrap_or_default())
}

impl Hud {
    /// A message the decoder has just applied at `time`, with the recording
    /// player's view at `camera` and `titles.txt`'s messages by name, and
    /// the sound the client plays for it, if any
    pub(crate) fn on_message(
        &mut self,
        decoder: &Decoder,
        m: &ParsedMessage,
        time: SourceTime,
        camera: &Camera,
        titles: &dyn Fn(&str) -> Option<HudMessage>,
    ) -> Option<&'static str> {
        let (id, body) = match &m.message {
            // A new session starts with nothing on the HUD
            Message::ServerInfo(info) => {
                *self = Self::default();
                self.server = c_string(&info.host_name).0;
                return None;
            }
            Message::Pings(pings) => {
                for &packed in pings {
                    let (slot, ping, loss) = unpack_ping(packed);
                    self.pings.insert(slot.wrapping_add(1), (ping, loss));
                }
                return None;
            }
            // A singleplayer recording's pauses do not stop the game it
            // shows, as the player keeps moving through them
            Message::SetPause(paused) => {
                if decoder.max_clients > 1 {
                    self.announcements.pause(*paused != 0);
                }
                return None;
            }
            Message::User { id, body } => (*id, body),
            Message::TempEntity { kind, body } if *kind == TE_TEXTMESSAGE => {
                self.messages.message(body, time);
                return None;
            }
            _ => return None,
        };
        // The time left, sudden death, the next map, timeouts and votes are
        // kept on their own, and so are health, armour and ammo, and the
        // scores
        let name = decoder.user_message(id)?.name_lossy();
        if self.announcements.message(&name, body, time)
            || self.status.message(&name, body, time)
            || self.scores.message(&name, body)
        {
            return None;
        }

        match name.as_str() {
            MESSAGE => {}
            "TeamInfo" => {
                let (&player, rest) = body.split_first()?;
                self.teams.insert(player, c_string(rest).0);
                return None;
            }
            "DeathMsg" => {
                self.death(decoder, body, time);
                return None;
            }
            "SayText" => {
                self.say(decoder, body, time);
                return None;
            }
            "Location" => {
                let (player, place) = location(body)?;
                self.places.insert(player, place);
                return None;
            }
            "Damage" => {
                let (origin, angles) = (camera.origin, camera.angles);
                self.damage.message(body, time, origin, angles);
                return None;
            }
            "HudText" => {
                // The name is looked up without a leading `#`, and shown
                // with it when the file lacks it
                let (name, _) = c_string(body);
                let found = titles(name.strip_prefix('#').unwrap_or(&name));
                self.messages.title(&name, found, time);
                return None;
            }
            _ => return None,
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

    /// A `DeathMsg`, the killer's and the victim's entities and the weapon
    fn death(&mut self, decoder: &Decoder, body: &[u8], time: SourceTime) {
        let (Some(&killer), Some(&victim)) = (body.first(), body.get(1)) else {
            return;
        };
        let weapon = c_string(body.get(2..).unwrap_or_default()).0;
        let named = |entity: u8| NoticeName {
            name: decoder
                .player_for_entity(i64::from(entity))
                .map(|p| p.name)
                .unwrap_or_default(),
            team: self.teams.get(&entity).cloned().unwrap_or_default(),
        };

        let suicide = killer == 0 || killer == victim;
        self.notices.push_back(DeathNotice {
            killer: (!suicide).then(|| named(killer)),
            victim: named(victim),
            teamkill: weapon == "teammate",
            weapon,
            time,
        });
        while self.notices.len() > NOTICES {
            self.notices.pop_front();
        }
    }

    /// A `SayText`, the sender's entity and the line, added at `time`
    fn say(&mut self, decoder: &Decoder, body: &[u8], time: SourceTime) {
        let Some((sender, named, text)) = say_text(body) else {
            return;
        };

        // The sender's name where the line has it
        let mut name = None;
        if named
            && let Some(player) = decoder.player_for_entity(i64::from(sender))
            && !player.name.is_empty()
            && let Some(start) = text.find(&player.name)
        {
            let team = self.teams.get(&sender).cloned().unwrap_or_default();
            name = Some((start..start + player.name.len(), team));
        }

        self.scroll(time);
        if self.chat.is_empty() {
            self.scroll_at = SourceTime(time.0 + CHAT_SHOWN);
        }
        self.chat.push_back(ChatLine {
            text,
            name,
            place: self.places.get(&sender).copied(),
        });
        while self.chat.len() > CHAT_LINES {
            self.chat.pop_front();
        }
    }

    /// Scroll the chat's top lines away until `at`, each five seconds after
    /// the one before it
    fn scroll(&mut self, at: SourceTime) {
        while !self.chat.is_empty() && at >= self.scroll_at {
            self.chat.pop_front();
            self.scroll_at = SourceTime(self.scroll_at.0 + CHAT_SHOWN);
        }
    }

    /// The chat's lines at `at`, oldest first
    pub fn chat(&self, at: SourceTime) -> Vec<ChatLine> {
        let mut shown = self.clone();
        shown.scroll(at);
        shown.chat.into()
    }

    /// The recording player's health, armour, ammo and flashlight as the
    /// HUD shows them, with the weapons of `camera` and the flashlight on
    /// too when `lit`
    pub fn status(&self, camera: &Camera, lit: bool) -> status::Shown {
        self.status.shown(camera.weapons, lit)
    }

    /// Where the recording player was hit from and what hurts them at `at`,
    /// as the HUD shows it
    pub fn damage(&self, at: SourceTime) -> DamageShown {
        if self.status.hides_health() {
            return DamageShown::default();
        }
        self.damage.at(at)
    }

    /// A command the recording player gave, with the weapons of `camera`,
    /// and the sound the client plays for it, if any
    pub(crate) fn on_command(&mut self, command: &str, camera: &Camera) -> Option<&'static str> {
        self.menu.command(command, &self.status, camera.weapons)
    }

    /// The weapon menu with the weapons of `camera`, while it is open
    pub fn weapon_menu(&self, camera: &Camera) -> Option<MenuShown> {
        self.menu.shown(&self.status, camera.weapons)
    }

    /// The full scoreboard as the game shows it while its key is held, with
    /// the players `decoder` knows
    pub fn board(&self, decoder: &Decoder) -> Board {
        self.scores
            .board(decoder, &self.teams, &self.pings, &self.server)
    }

    /// The text the server placed on the HUD that shows at `at`
    pub fn messages(&self, at: SourceTime) -> Vec<ShownMessage> {
        self.messages.at(at)
    }

    /// What the recording player picked up and still shows at `at`
    pub fn pickups(&self, at: SourceTime) -> Vec<status::Pickup> {
        self.status.pickups(at)
    }

    /// The small scoreboard's rows, highest first, with the players
    /// `decoder` knows
    pub fn scores(&self, decoder: &Decoder) -> Vec<ScoreRow> {
        self.scores.rows(decoder, &self.teams)
    }

    /// AG's messages on how the match goes at `at`
    pub fn announced(&self, at: SourceTime) -> Announced {
        self.announcements.at(at)
    }

    /// The kill feed's lines at `at`, oldest first
    pub fn death_notices(&self, at: SourceTime) -> Vec<DeathNotice> {
        self.notices
            .iter()
            .filter(|n| n.time <= at && at.0 < n.time.0 + NOTICE_SHOWN)
            .cloned()
            .collect()
    }

    /// The countdown showing at `at`, none when there is none
    pub fn countdown(&self, at: SourceTime) -> Option<&Countdown> {
        let countdown = self.countdown.as_ref()?;
        let shown = countdown.seconds == WAITING || self.until.is_some_and(|until| at < until);
        shown.then_some(countdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `DeathMsg` body for `killer`, `victim` and `weapon`
    fn death(killer: u8, victim: u8, weapon: &str) -> Vec<u8> {
        let mut body = vec![killer, victim];
        body.extend(weapon.as_bytes());
        body.push(0);
        body
    }

    #[test]
    fn the_feed_keeps_four_lines_for_six_seconds() {
        let decoder = Decoder::new();
        let mut hud = Hud::default();
        hud.teams.insert(1, "red".into());
        for (k, weapon) in ["gauss", "crossbow", "357", "rpg_rocket", "egon"]
            .iter()
            .enumerate()
        {
            hud.death(&decoder, &death(1, 2, weapon), SourceTime(k as f64));
        }

        // The first dropped for the fifth, each gone six seconds after it came
        let weapons = |at: f64| -> Vec<String> {
            hud.death_notices(SourceTime(at))
                .into_iter()
                .map(|n| n.weapon)
                .collect()
        };
        assert_eq!(weapons(4.), ["crossbow", "357", "rpg_rocket", "egon"]);
        assert_eq!(weapons(8.5), ["rpg_rocket", "egon"]);
        assert!(weapons(10.).is_empty());
        assert_eq!(
            hud.death_notices(SourceTime(4.))[0]
                .killer
                .as_ref()
                .unwrap()
                .team,
            "red"
        );
    }

    #[test]
    fn the_chat_keeps_five_lines_each_top_one_five_seconds() {
        let decoder = Decoder::new();
        let mut hud = Hud::default();
        // Six lines half a second apart
        for k in 0..6 {
            let mut body = vec![3];
            body.extend(format!("line {k}\n").as_bytes());
            body.push(0);
            hud.say(&decoder, &body, SourceTime(f64::from(k) * 0.5));
        }

        // The first scrolled away for the sixth, then the top one five
        // seconds after the first line came, and every five seconds after
        let texts = |at: f64| -> Vec<String> {
            hud.chat(SourceTime(at))
                .into_iter()
                .map(|l| l.text)
                .collect()
        };
        assert_eq!(
            texts(3.),
            ["line 1", "line 2", "line 3", "line 4", "line 5"]
        );
        assert_eq!(texts(5.5), ["line 2", "line 3", "line 4", "line 5"]);
        assert_eq!(texts(10.5), ["line 3", "line 4", "line 5"]);
        assert!(texts(25.).is_empty());
    }

    #[test]
    fn a_location_places_the_next_lines_sender() {
        let decoder = Decoder::new();
        let mut hud = Hud::default();
        // As a recorded one came, a sixteenth of the units in each short
        let body = [4, 0xf3, 0x3a, 0x80, 0xea, 0x20, 0x01];
        let (player, place) = location(&body).unwrap();
        assert_eq!((player, place), (4, [1886.375, -688., 36.]));

        hud.places.insert(player, place);
        hud.say(&decoder, b"\x04\x02(T) naz: at %l\n\0", SourceTime(1.));
        let line = &hud.chat(SourceTime(1.))[0];
        assert_eq!(line.text, "(T) naz: at %l");
        assert_eq!(line.place, Some(place));
    }

    #[test]
    fn a_suicide_names_no_killer_and_a_teammate_is_a_team_kill() {
        let decoder = Decoder::new();
        let mut hud = Hud::default();
        hud.death(&decoder, &death(3, 3, "rpg_rocket"), SourceTime(1.));
        hud.death(&decoder, &death(0, 3, "world"), SourceTime(1.));
        hud.death(&decoder, &death(1, 3, "teammate"), SourceTime(1.));

        let notices = hud.death_notices(SourceTime(1.));
        let killed: Vec<(bool, bool)> = notices
            .iter()
            .map(|n| (n.killer.is_some(), n.teamkill))
            .collect();
        assert_eq!(killed, [(false, false), (false, false), (true, true)]);
    }
}
