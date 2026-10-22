//! The small scoreboard at the top left of the view, as AG's client draws
//! it (OpenAG's `CHudScores`)
//!
//! Each row is a player's frags and name, or in team play a team's, highest
//! first, starting 20 of the game's columns from the left and 50 rows down
//! and standing one under another. A row's strip is its team's colour,
//! faint, as wide as the widest row, with the frags right aligned before
//! the names. The text is added to the picture in the team's colour at a
//! little over half its brightness, and AG's `^1` to `^8` colour names
//! further. Everything is sized to the view as the game's 640 by 480 screen
//! would be, as the kill feed is

use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use lambda_slicer_playback::scores::ScoreRow;

use crate::kill_feed::add_tagged;

/// The game's screen height the HUD is laid out on
const SCREEN_HEIGHT: f32 = 480.;

/// Where the first row's top left stands, a row's height, the text's size,
/// the room at either end of a strip and between the frags and the name, in
/// the game's rows (`cl_scores_pos`, `PADDING`, `GAP`)
const LEFT: f32 = 20.;
const TOP: f32 = 50.;
const ROW_HEIGHT: f32 = 16.;
const TEXT_SIZE: f32 = 13.;
const PADDING: f32 = 10.;
const GAP: f32 = 10.;

/// How much of 255 a strip covers, and how bright its text is
/// (`cl_scores_alpha`, `ScaleColors`)
const STRIP: u8 = 20;
const TEXT_BRIGHTNESS: u16 = 135;

/// The colours of team numbers nought to four, orange without a team then
/// blue, red, yellow and green (`iTeamColors`)
const TEAM_COLOURS: [Color32; 5] = [
    Color32::from_rgb(255, 170, 0),
    Color32::from_rgb(125, 165, 210),
    Color32::from_rgb(200, 90, 70),
    Color32::from_rgb(225, 205, 45),
    Color32::from_rgb(145, 215, 140),
];

/// The colour of team number `number`, the numbers past four starting over
fn team_colour(number: u8) -> Color32 {
    TEAM_COLOURS[usize::from(number) % TEAM_COLOURS.len()]
}

/// `colour` at `TEXT_BRIGHTNESS` of 255
fn dimmed(colour: Color32) -> Color32 {
    let [r, g, b, _] = colour.to_array();
    let scale = |c: u8| (u16::from(c) * TEXT_BRIGHTNESS / 255) as u8;
    Color32::from_rgb(scale(r), scale(g), scale(b))
}

/// `job` added to the picture rather than laid over it
fn added(mut job: egui::text::LayoutJob) -> egui::text::LayoutJob {
    for section in &mut job.sections {
        let [r, g, b, _] = section.format.color.to_array();
        section.format.color = Color32::from_rgba_premultiplied(r, g, b, 0);
    }
    job
}

/// The first `most` of `rows` over the view at `rect`
pub fn draw(ui: &egui::Ui, rect: Rect, rows: &[ScoreRow], most: usize) {
    let rows = &rows[..rows.len().min(most)];
    if rows.is_empty() {
        return;
    }
    let scale = rect.height() / SCREEN_HEIGHT;
    let font = egui::FontId::proportional(TEXT_SIZE * scale);
    let painter = ui.painter_at(rect);

    // Each row's frags and name laid out, to find the widest of each
    let mut laid = Vec::new();
    for row in rows {
        let colour = dimmed(team_colour(row.team_number));
        let mut frags = egui::text::LayoutJob::default();
        frags.append(
            &row.frags.to_string(),
            0.,
            egui::TextFormat::simple(font.clone(), colour),
        );
        let mut name = egui::text::LayoutJob::default();
        add_tagged(&mut name, &row.name, colour, &font);
        laid.push((
            row,
            painter.layout_job(added(frags)),
            painter.layout_job(added(name)),
        ));
    }
    let (mut frags_width, mut name_width) = (0_f32, 0_f32);
    for (_, frags, name) in &laid {
        frags_width = frags_width.max(frags.size().x);
        name_width = name_width.max(name.size().x);
    }

    let height = ROW_HEIGHT * scale;
    let strip_width = (PADDING * 2. + GAP) * scale + frags_width + name_width;
    let left = rect.left() + LEFT * scale;
    let mut top = rect.top() + TOP * scale;
    for (row, frags, name) in laid {
        let [r, g, b, _] = team_colour(row.team_number).to_array();
        let strip = Rect::from_min_size(Pos2::new(left, top), Vec2::new(strip_width, height));
        painter.rect_filled(strip, 0., Color32::from_rgba_unmultiplied(r, g, b, STRIP));

        // The frags end where the widest do, and the name starts a gap on
        let frags_end = left + PADDING * scale + frags_width;
        let middle = |galley: &egui::Galley| top + (height - galley.size().y) / 2.;
        let frags_at = Pos2::new(frags_end - frags.size().x, middle(&frags));
        let name_at = Pos2::new(frags_end + GAP * scale, middle(&name));
        painter.galley(frags_at, frags, Color32::WHITE);
        painter.galley(name_at, name, Color32::WHITE);
        top += height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_numbers_take_their_colours_and_start_over_past_four() {
        assert_eq!(team_colour(0), Color32::from_rgb(255, 170, 0));
        assert_eq!(team_colour(2), Color32::from_rgb(200, 90, 70));
        assert_eq!(team_colour(6), team_colour(1));
        assert_eq!(
            dimmed(Color32::from_rgb(255, 170, 0)),
            Color32::from_rgb(135, 90, 0)
        );
    }
}
