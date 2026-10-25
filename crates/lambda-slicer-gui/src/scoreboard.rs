//! The full scoreboard over the middle of the view, as the game shows it
//! while its key is held and at intermission (Half-Life's `ScorePanel`)
//!
//! A dark panel of a fixed size stands 104 columns in from each side of the
//! game's 640 by 480 screen and 40 rows from its top and bottom, so it
//! widens with a wide view as the game's does. The server's name tops it,
//! then the columns' titles. Each team has a line of its name, its count of
//! players and its score in its colour, a line under it, and its players
//! under that, best first, their rows in its colour and their names coloured
//! further where AG's colour codes say. The recorder's row is lit. The spectators are
//! listed under everyone else in the same way. A player's SteamID stands
//! between their name and score, as `0:1:23456789`, or BOT for a bot, and
//! their latency and packet loss share a column, as `28/0`
//!
//! Across the panel things stand at shares of its width, and down it at the
//! game's rows, as the game lays it out

use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::roster::steam_id_text;
use lambda_slicer_playback::scores::{Board, BoardPlayer};

use crate::i18n::Catalog;
use crate::kill_feed::add_tagged;
use crate::mini_scores::team_colour;
use crate::viewer::HUD_COLOR;

/// The game's screen the panel is laid out on
const SCREEN_WIDTH: f32 = 640.;
const SCREEN_HEIGHT: f32 = 480.;

/// How far the panel stands in from the screen's sides, and from its top
/// and bottom
const SIDE: f32 = 104.;
const TOP: f32 = 40.;

/// Where the server's name and the columns' titles stand, and where the
/// first team starts, in rows from the panel's top
const TITLE_MIDDLE: f32 = 14.;
const HEADINGS_MIDDLE: f32 = 25.;
const FIRST_SIDE: f32 = 34.;

/// A team's line, the room under it to its players, a player's row and the
/// room between teams, in rows
const SIDE_LINE: f32 = 15.;
const UNDER_LINE: f32 = 2.;
const ROW_HEIGHT: f32 = 12.;
const BETWEEN_SIDES: f32 = 15.;

/// The text's sizes, in rows
const TITLE_SIZE: f32 = 8.;
const HEADING_SIZE: f32 = 6.;
const ROW_SIZE: f32 = 9.5;
const COUNT_SIZE: f32 = 5.5;

/// Where the server's name and players' names start and the names are cut
/// off, where the SteamIDs start, where the score, deaths and latency
/// columns end, and the lit row's and lines' ends, in shares of the
/// panel's width
const SERVER_LEFT: f32 = 0.015;
const NAME_LEFT: f32 = 0.065;
const NAMES_END: f32 = 0.37;
const STEAM_ID_LEFT: f32 = 0.39;
const SCORE_END: f32 = 0.61;
const DEATHS_END: f32 = 0.72;
const PING_END: f32 = 0.826;
const ROW_LEFT: f32 = 0.009;
const ROW_RIGHT: f32 = 0.982;

/// How much of 255 the panel's dark covers, and the lines' strength
const BACKGROUND: u8 = 140;
const LINE: u8 = 200;

/// The spectators' rows, the spectators' title, and the recorder's row
const PLAYER: Color32 = Color32::from_rgb(230, 230, 230);
const SPECTATORS: Color32 = Color32::from_gray(170);
const RECORDER_ROW: Color32 = Color32::from_rgba_unmultiplied_const(110, 130, 150, 120);

/// Where things stand in the panel, and how big a row is in points
struct Layout {
    panel: Rect,
    row: f32,
}

impl Layout {
    /// The x of `share` of the panel's width
    fn x(&self, share: f32) -> f32 {
        self.panel.left() + self.panel.width() * share
    }

    /// `rows` of the game in points
    fn rows(&self, rows: f32) -> f32 {
        rows * self.row
    }

    fn font(&self, size: f32) -> egui::FontId {
        egui::FontId::proportional(self.rows(size))
    }
}

/// The scoreboard `board` over the view at `rect`
pub fn draw(ui: &egui::Ui, rect: Rect, catalog: &Catalog, board: &Board) {
    let row = rect.height() / SCREEN_HEIGHT;
    let side = rect.width() * SIDE / SCREEN_WIDTH;
    let top_left = Pos2::new(rect.left() + side, rect.top() + TOP * row);
    let size = Vec2::new(rect.width() - side * 2., (SCREEN_HEIGHT - TOP * 2.) * row);
    let panel = Rect::from_min_size(top_left, size);
    let at = Layout { panel, row };

    // Everything past the panel's bottom is cut off, and long names before
    // the score
    let painter = ui.painter_at(panel.intersect(rect));
    let names_clip = Rect::from_x_y_ranges(panel.left()..=at.x(NAMES_END), panel.y_range());
    let names = painter.with_clip_rect(names_clip.intersect(painter.clip_rect()));
    painter.rect_filled(panel, 0., Color32::from_black_alpha(BACKGROUND));

    // The server's name at the top left, and the columns' titles
    let mut job = egui::text::LayoutJob::default();
    add_tagged(&mut job, &board.server, HUD_COLOR, &at.font(TITLE_SIZE));
    let job = painter.layout_job(job);
    let middle = panel.top() + at.rows(TITLE_MIDDLE);
    let title = Pos2::new(at.x(SERVER_LEFT), middle - job.size().y / 2.);
    painter.galley(title, job, HUD_COLOR);
    let headings = [
        (SCORE_END, "scores-frags"),
        (DEATHS_END, "scores-deaths"),
        (PING_END, "viewer-scoreboard-ping-loss"),
    ];
    let middle = panel.top() + at.rows(HEADINGS_MIDDLE);
    for (end, id) in headings {
        let heading = Pos2::new(at.x(end), middle);
        let text = catalog.text(id);
        painter.text(
            heading,
            Align2::RIGHT_CENTER,
            text,
            at.font(HEADING_SIZE),
            HUD_COLOR,
        );
    }

    let mut top = panel.top() + at.rows(FIRST_SIDE);
    for side in &board.sides {
        // A team's line, none outside team play
        let colour = team_colour(side.team_number);
        if let Some(name) = &side.name {
            let line = SideLine {
                name,
                players: &side.players,
                frags: side.frags,
                deaths: side.deaths,
                colour,
            };
            top = draw_side_line(&painter, &names, &at, catalog, &line, top);
        }
        for player in &side.players {
            draw_row(&painter, &names, &at, catalog, player, colour, top);
            top += at.rows(ROW_HEIGHT);
        }
        top += at.rows(BETWEEN_SIDES);
    }

    // The spectators under a line of their own
    if board.spectators.is_empty() {
        return;
    }
    let middle = top + at.rows(SIDE_LINE) / 2.;
    let title = catalog.text("scores-spectators");
    let left = Pos2::new(at.x(NAME_LEFT), middle);
    painter.text(
        left,
        Align2::LEFT_CENTER,
        title,
        at.font(ROW_SIZE),
        SPECTATORS,
    );
    top += at.rows(SIDE_LINE);
    let line = Stroke::new(1., SPECTATORS.gamma_multiply(0.6));
    painter.hline(at.x(ROW_LEFT)..=at.x(ROW_RIGHT), top, line);
    top += at.rows(UNDER_LINE);
    for spectator in &board.spectators {
        draw_row(&painter, &names, &at, catalog, spectator, PLAYER, top);
        top += at.rows(ROW_HEIGHT);
    }
}

/// What a team's line shows
struct SideLine<'a> {
    name: &'a str,
    players: &'a [BoardPlayer],
    frags: i32,
    deaths: i32,
    colour: Color32,
}

/// A team's line at `top`, its name, count of players, score, deaths and
/// its players' average latency and loss, and a line under it in its
/// colour. Returns where its first player's row starts
fn draw_side_line(
    painter: &egui::Painter,
    names: &egui::Painter,
    at: &Layout,
    catalog: &Catalog,
    side: &SideLine,
    top: f32,
) -> f32 {
    let middle = top + at.rows(SIDE_LINE) / 2.;
    let font = at.font(ROW_SIZE);
    let mut job = egui::text::LayoutJob::default();
    add_tagged(&mut job, side.name, side.colour, &font);
    let job = names.layout_job(job);
    let name_at = Pos2::new(at.x(NAME_LEFT), middle - job.size().y / 2.);
    let name_end = name_at.x + job.size().x;
    names.galley(name_at, job, side.colour);

    // The count of players stands small after the name, on its baseline
    let count = FluentValue::from(side.players.len() as i64);
    let count = catalog.text_with("viewer-scoreboard-players", &[("count", count)]);
    let count_at = Pos2::new(name_end + at.rows(4.), middle + at.rows(ROW_SIZE) * 0.3);
    let small = at.font(COUNT_SIZE);
    names.text(count_at, Align2::LEFT_BOTTOM, count, small, side.colour);

    let ping = average(side.players.iter().filter_map(|p| p.ping));
    let loss = average(side.players.iter().filter_map(|p| p.loss));
    let numbers = [
        (SCORE_END, side.frags.to_string()),
        (DEATHS_END, side.deaths.to_string()),
        (PING_END, ping_loss(ping, loss)),
    ];
    for (end, number) in numbers {
        let number_at = Pos2::new(at.x(end), middle);
        painter.text(
            number_at,
            Align2::RIGHT_CENTER,
            number,
            font.clone(),
            side.colour,
        );
    }

    let bottom = top + at.rows(SIDE_LINE);
    let line = Stroke::new(1., side.colour.gamma_multiply(f32::from(LINE) / 255.));
    painter.hline(at.x(ROW_LEFT)..=at.x(ROW_RIGHT), bottom, line);
    bottom + at.rows(UNDER_LINE)
}

/// `player`'s row from `top` in `colour`, their name coloured further where
/// AG's colour codes say, lit for the recorder
fn draw_row(
    painter: &egui::Painter,
    names: &egui::Painter,
    at: &Layout,
    catalog: &Catalog,
    player: &BoardPlayer,
    colour: Color32,
    top: f32,
) {
    let height = at.rows(ROW_HEIGHT);
    if player.recorder {
        let lit = Rect::from_x_y_ranges(at.x(ROW_LEFT)..=at.x(ROW_RIGHT), top..=top + height);
        painter.rect_filled(lit, 0., RECORDER_ROW);
    }

    let middle = top + height / 2.;
    let font = at.font(ROW_SIZE);
    let mut job = egui::text::LayoutJob::default();
    add_tagged(&mut job, &player.name, colour, &font);
    let job = names.layout_job(job);
    let name_at = Pos2::new(at.x(NAME_LEFT), middle - job.size().y / 2.);
    names.galley(name_at, job, colour);

    // Who they are to Steam, or that they are a bot
    let id = if player.bot {
        catalog.text("viewer-scoreboard-bot")
    } else {
        player.steam_id.map_or(String::new(), steam_id_text)
    };
    let id_at = Pos2::new(at.x(STEAM_ID_LEFT), middle);
    painter.text(id_at, Align2::LEFT_CENTER, id, font.clone(), colour);

    let numbers = [
        (SCORE_END, player.frags.to_string()),
        (DEATHS_END, player.deaths.to_string()),
        (PING_END, ping_loss(player.ping, player.loss)),
    ];
    for (end, number) in numbers {
        let number_at = Pos2::new(at.x(end), middle);
        painter.text(
            number_at,
            Align2::RIGHT_CENTER,
            number,
            font.clone(),
            colour,
        );
    }
}

/// A latency and loss as the game writes them, `28/0`, or nothing before
/// the server has sent them
fn ping_loss(ping: Option<u32>, loss: Option<u32>) -> String {
    match (ping, loss) {
        (Some(ping), Some(loss)) => format!("{ping}/{loss}"),
        _ => String::new(),
    }
}

/// The average of `values`, rounded down, none without any
fn average(values: impl Iterator<Item = u32>) -> Option<u32> {
    let (mut sum, mut count) = (0, 0);
    for value in values {
        sum += value;
        count += 1;
    }
    if count == 0 {
        return None;
    }
    Some(sum / count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_and_loss_show_together_once_both_are_known() {
        assert_eq!(ping_loss(Some(28), Some(0)), "28/0");
        assert_eq!(ping_loss(None, None), "");
        assert_eq!(average([20, 31].into_iter()), Some(25));
        assert_eq!(average(std::iter::empty()), None);
    }
}
