//! AG's messages over the view on how the match goes, where its client
//! draws them and in its words (OpenAG's `CHudTimer`, `CHudSuddenDeath`,
//! `CHudNextmap`, `CHudTimeout` and `CHudVote`)
//!
//! The time stands in the middle, one of the game's text lines from the
//! top, with sudden death two lines down, the next map five and a timeout
//! six. A vote stands a twentieth of the width from the left and an eighth
//! of the height from the top. Everything is in the HUD's colour, sized as on a
//! 640 by 480 screen, where a line is a 32nd of the height, as the
//! countdown is. The text is AG's own, in English as it writes it, but for
//! a pause without a timeout, which AG does not word

use eframe::egui::{self, Align2, Rect};
use lambda_slicer_playback::announcements::{Announced, Timeout, Vote, VoteStatus};

use crate::i18n::Catalog;
use crate::kill_feed::add_tagged;
use crate::viewer::HUD_COLOR;

/// How many of the game's text lines from the top each message stands
const TIMER_LINE: f32 = 1.;
const SUDDEN_DEATH_LINE: f32 = 2.;
const NEXT_MAP_LINE: f32 = 5.;
const TIMEOUT_LINE: f32 = 6.;

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

/// `announced` over the view at `rect`
pub fn draw(painter: &egui::Painter, rect: Rect, catalog: &Catalog, announced: &Announced) {
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
}
