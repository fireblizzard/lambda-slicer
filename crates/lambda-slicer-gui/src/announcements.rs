//! AG's messages over the view on how the match goes, where its client
//! draws them and in its words (OpenAG's `CHudTimer`, `CHudSuddenDeath`,
//! `CHudNextmap`, `CHudTimeout`, `CHudVote` and `CHudSettings`)
//!
//! The time stands in the middle, one of the game's text lines from the
//! top, with sudden death two lines down, the next map five and a timeout
//! six. A vote stands a twentieth of the width from the left and an eighth
//! of the height from the top. The server's settings stand a fifth of the
//! width from the right, from one line down, AG's version, the game mode,
//! then after half a line more the limits and rules, and while a match is
//! on that says so in the middle two lines down. Everything is in the HUD's
//! colour, sized as on a 640 by 480 screen, where a line is a 32nd of the
//! height, as the countdown is. The text is AG's own, in English as it
//! writes it, but for a pause without a timeout, which AG does not word

use eframe::egui::{self, Align2, Rect};
use lambda_slicer_playback::announcements::{Announced, ServerSettings, Timeout, Vote, VoteStatus};

use crate::i18n::Catalog;
use crate::kill_feed::add_tagged;
use crate::viewer::HUD_COLOR;

/// How many of the game's text lines from the top each message stands
const TIMER_LINE: f32 = 1.;
const SUDDEN_DEATH_LINE: f32 = 2.;
const NEXT_MAP_LINE: f32 = 5.;
const TIMEOUT_LINE: f32 = 6.;
const MATCH_ON_LINE: f32 = 2.;

/// The share of the view's width the settings stand from the right, and
/// the room above their limits, in lines
const SETTINGS_RIGHT: f32 = 1. / 5.;
const SETTINGS_GAP: f32 = 1.5;

/// The share of the view's width and height a vote stands from its top
/// left, and the room after who called it, in lines
const VOTE_LEFT: f32 = 1. / 20.;
const VOTE_TOP: f32 = 1. / 8.;
const VOTE_GAP: f32 = 1.5;

/// The view's height in the game's text lines
const LINES: f32 = 32.;

/// Seconds in a minute, an hour and a day
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

/// The time as AG writes it, seconds alone under a minute or past the
/// time limit, then minutes and seconds, then hours, minutes and seconds
fn clock(seconds: i64) -> String {
    if seconds < MINUTE {
        return seconds.to_string();
    }

    let (days, hours) = (seconds / DAY, seconds % DAY / HOUR);
    let (minutes, rest) = (seconds % HOUR / MINUTE, seconds % MINUTE);
    if days > 0 {
        let plural = if days == 1 { "" } else { "s" };
        format!("{days} day{plural} {hours}h {minutes}m {rest}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {rest}s")
    } else {
        format!("{minutes}:{rest:02}")
    }
}

/// A timeout's line, or a pause's when no timeout is counting
fn timeout_line(catalog: &Catalog, timeout: Option<Timeout>, paused: bool) -> Option<String> {
    match timeout {
        Some(Timeout::Stopping(s)) => Some(format!("Timeout called, stopping in {s} seconds.")),
        Some(Timeout::Starting(s)) => Some(format!("Timeout, starting in {s} seconds.")),
        None if paused => Some(catalog.text("viewer-paused")),
        None => None,
    }
}

/// The lines under a vote's caller, its counts while open or how it ended
fn vote_lines(vote: &Vote) -> Vec<String> {
    match vote.status {
        VoteStatus::Open { yes, no, undecided } => vec![
            format!("For: {yes}"),
            format!("Against: {no}"),
            format!("Undecided: {undecided}"),
        ],
        VoteStatus::Accepted => vec!["Accepted!".to_owned()],
        VoteStatus::Denied => vec!["Denied!".to_owned()],
    }
}

/// The server's settings as AG writes them, each with the line it stands
/// on
fn settings_lines(settings: &ServerSettings) -> Vec<(f32, String)> {
    let on = |value: bool| if value { "On" } else { "Off" };
    let mut lines = vec![
        (1., format!("AG version {}", settings.version)),
        (2., settings.mode.clone()),
    ];

    // The limits and rules after a little more room, then the multipliers
    // that are not the game's own
    let mut rest = vec![
        format!("Time limit: {}", settings.time_limit),
        format!("Frag limit: {}", settings.frag_limit),
        format!("Friendly fire: {}", on(settings.friendly_fire)),
        format!("Weaponstay: {}", on(settings.weapon_stay)),
    ];
    if let Some(wallgauss) = &settings.wallgauss {
        rest.push(format!("Wallgauss: {wallgauss}x (1)"));
    }
    if let Some(headshot) = &settings.headshot {
        rest.push(format!("Headshot: {headshot}x (3)"));
    }
    if let Some(blast_radius) = &settings.blast_radius {
        rest.push(format!("Blast radius: {blast_radius}x (1)"));
    }

    let mut row = 2. + SETTINGS_GAP;
    for text in rest {
        lines.push((row, text));
        row += 1.;
    }
    lines
}

/// `announced` over the view at `rect`, with the server's settings when
/// `settings` gives their text's size against the game's
pub fn draw(
    painter: &egui::Painter,
    rect: Rect,
    catalog: &Catalog,
    announced: &Announced,
    settings: Option<f32>,
) {
    let line = rect.height() / LINES;
    let font = egui::FontId::proportional(line);
    let centred = |row: f32, text: String| {
        let at = egui::pos2(rect.center().x, rect.top() + row * line);
        painter.text(at, Align2::CENTER_TOP, text, font.clone(), HUD_COLOR);
    };

    if let Some(seconds) = announced.timer {
        centred(TIMER_LINE, clock(seconds));
    }
    if announced.sudden_death {
        centred(SUDDEN_DEATH_LINE, "Sudden death!".to_owned());
    }
    if let Some(map) = &announced.next_map {
        centred(NEXT_MAP_LINE, format!("The next map is {map}."));
    }
    if let Some(text) = timeout_line(catalog, announced.timeout, announced.paused) {
        centred(TIMEOUT_LINE, text);
    }

    if let Some(vote) = &announced.vote {
        draw_vote(painter, rect, line, &font, vote);
    }

    if let Some(size) = settings
        && let Some(server) = &announced.settings
    {
        // The lines stand as far apart as their size makes them
        let left = rect.right() - rect.width() * SETTINGS_RIGHT;
        let sized = line * size;
        let font = egui::FontId::proportional(sized);
        for (row, text) in settings_lines(server) {
            let at = egui::pos2(left, rect.top() + row * sized);
            painter.text(at, Align2::LEFT_TOP, text, font.clone(), HUD_COLOR);
        }
        if server.match_on {
            centred(MATCH_ON_LINE, "A match is on!".to_owned());
        }
    }
}

/// A vote at the top left, what it is on, who called it in their colours,
/// then its counts or how it ended
fn draw_vote(painter: &egui::Painter, rect: Rect, line: f32, font: &egui::FontId, vote: &Vote) {
    let left = rect.left() + rect.width() * VOTE_LEFT;
    let mut top = rect.top() + rect.height() * VOTE_TOP;

    let subject = format!("Vote: {}", vote.subject);
    painter.text(
        egui::pos2(left, top),
        Align2::LEFT_TOP,
        subject,
        font.clone(),
        HUD_COLOR,
    );
    top += line;

    let mut caller = egui::text::LayoutJob::default();
    add_tagged(&mut caller, "Called by: ", HUD_COLOR, font);
    add_tagged(&mut caller, &vote.called_by, HUD_COLOR, font);
    painter.galley(egui::pos2(left, top), painter.layout_job(caller), HUD_COLOR);
    top += line * VOTE_GAP;

    for text in vote_lines(vote) {
        painter.text(
            egui::pos2(left, top),
            Align2::LEFT_TOP,
            text,
            font.clone(),
            HUD_COLOR,
        );
        top += line;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_time_reads_as_ag_writes_it() {
        assert_eq!(clock(45), "45");
        assert_eq!(clock(-3), "-3");
        assert_eq!(clock(1188), "19:48");
        assert_eq!(clock(3 * HOUR + 4 * MINUTE + 5), "3h 4m 5s");
        assert_eq!(clock(DAY + 61), "1 day 0h 1m 1s");
        assert_eq!(clock(2 * DAY), "2 days 0h 0m 0s");
    }

    #[test]
    fn a_vote_shows_its_counts_or_how_it_ended() {
        let open = Vote {
            status: VoteStatus::Open {
                yes: 1,
                no: 0,
                undecided: 7,
            },
            subject: "agstart".into(),
            called_by: "p".into(),
        };
        assert_eq!(vote_lines(&open), ["For: 1", "Against: 0", "Undecided: 7"]);
        let denied = Vote {
            status: VoteStatus::Denied,
            ..open
        };
        assert_eq!(vote_lines(&denied), ["Denied!"]);
    }

    #[test]
    fn the_settings_read_as_ag_writes_them() {
        let settings = ServerSettings {
            match_on: false,
            mode: "Arena".into(),
            time_limit: 10,
            frag_limit: 0,
            friendly_fire: false,
            weapon_stay: true,
            version: "6.6".into(),
            wallgauss: Some("2".into()),
            headshot: None,
            blast_radius: None,
        };
        assert_eq!(
            settings_lines(&settings),
            [
                (1., "AG version 6.6".to_owned()),
                (2., "Arena".to_owned()),
                (3.5, "Time limit: 10".to_owned()),
                (4.5, "Frag limit: 0".to_owned()),
                (5.5, "Friendly fire: Off".to_owned()),
                (6.5, "Weaponstay: On".to_owned()),
                (7.5, "Wallgauss: 2x (1)".to_owned()),
            ]
        );
    }
}
