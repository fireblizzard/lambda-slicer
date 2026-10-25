//! The kill feed over the view, as AG's client draws its death notices
//! (OpenAG's `CHudDeathNotice`)
//!
//! Each line stands at the top right, the killer's name, the weapon's icon
//! and the victim's name, the victim's ending a little short of the view's
//! edge. A death
//! no player caused shows the icon and the victim alone. Lines start 34 of
//! the game's 480 rows down and stand 20 apart, and everything is sized to
//! the view as the game's 640 by 480 screen would be.
//!
//! The icons are `d_` and the weapon in `sprites/hud.txt`, the skull for a
//! weapon with none, drawn as the game draws them, added to the picture
//! below, in orange or for a team kill in green. A name takes its team's
//! colour (`GetClientColor`), and AG's `^1` to `^8` colour what follows
//! them until `^0` or `^9` (OpenAG's colour tags)

use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use lambda_slicer_audio::library::Library;
use lambda_slicer_playback::hud::{DeathNotice, NoticeName};

use crate::hud_sprites::{HUD_LIST, HudSprites};

/// The game's screen height the HUD is laid out on
pub(crate) const SCREEN_HEIGHT: f32 = 480.;

/// Where the first line's top stands, how far apart lines stand, the room
/// between the killer and the icon, and the text's size, in the game's rows
const FIRST_LINE: f32 = 34.;
const LINE_STEP: f32 = 20.;
const KILLER_GAP: f32 = 5.;
const TEXT_SIZE: f32 = 13.;

/// The room between the victim's name and the view's right edge, in the
/// app's own points
const RIGHT_MARGIN: f32 = 8.;

/// The icon's colour, and a team kill's
const ICON: Color32 = Color32::from_rgb(255, 80, 0);
const TEAMKILL_ICON: Color32 = Color32::from_rgb(10, 240, 10);

/// The icon a weapon without one of its own takes
const FALLBACK_ICON: &str = "d_skull";

/// The colours `^1` to `^8` give a name
pub(crate) const TAG_COLOURS: [Color32; 8] = [
    Color32::from_rgb(255, 0, 0),
    Color32::from_rgb(0, 255, 0),
    Color32::from_rgb(255, 255, 0),
    Color32::from_rgb(0, 0, 255),
    Color32::from_rgb(0, 255, 255),
    Color32::from_rgb(255, 0, 255),
    Color32::from_rgb(136, 136, 136),
    Color32::from_rgb(255, 255, 255),
];

/// The colours a player's team gives their name, blue, red, yellow without
/// a team and grey for any other (`g_ColorBlue` and the rest)
const TEAM_BLUE: Color32 = Color32::from_rgb(153, 204, 255);
const TEAM_RED: Color32 = Color32::from_rgb(255, 64, 64);
const NO_TEAM: Color32 = Color32::from_rgb(255, 179, 0);
const OTHER_TEAM: Color32 = Color32::from_rgb(204, 204, 204);

/// The kill feed's icons, read from the game's files as they are first
/// needed
#[derive(Default)]
pub struct KillFeed {
    sprites: HudSprites,
}

/// A team's colour for a name
pub(crate) fn team_colour(team: &str) -> Color32 {
    match team.trim().to_lowercase().as_str() {
        "blue" => TEAM_BLUE,
        "red" => TEAM_RED,
        "" => NO_TEAM,
        _ => OTHER_TEAM,
    }
}

/// A name laid out in its team's colour, coloured further where AG's
/// colour codes say, the codes themselves left out
fn name_job(name: &NoticeName, font: egui::FontId) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    add_tagged(&mut job, &name.name, team_colour(&name.team), &font);
    job
}

/// `text` added to `job` in `base`, coloured otherwise where AG's colour
/// codes say until `^0` or `^9` go back to `base`, the codes left out
pub(crate) fn add_tagged(
    job: &mut egui::text::LayoutJob,
    text: &str,
    base: Color32,
    font: &egui::FontId,
) {
    let mut colour = base;
    let mut run = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let code = chars
            .peek()
            .and_then(|d| d.to_digit(10))
            .filter(|_| c == '^');
        let Some(digit) = code else {
            run.push(c);
            continue;
        };
        chars.next();

        let format = egui::TextFormat::simple(font.clone(), colour);
        job.append(&std::mem::take(&mut run), 0., format);
        colour = match digit {
            1..=8 => TAG_COLOURS[digit as usize - 1],
            _ => base,
        };
    }
    job.append(&run, 0., egui::TextFormat::simple(font.clone(), colour));
}

impl KillFeed {
    /// The kill feed's `notices` over the view at `rect`, its icons from
    /// `library` and its names `size` times the game's
    pub fn draw(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        notices: &[DeathNotice],
        library: &Library,
        size: f32,
    ) {
        let scale = rect.height() / SCREEN_HEIGHT;
        let font = egui::FontId::proportional(TEXT_SIZE * size * scale);
        let painter = ui.painter_at(rect);

        for (i, notice) in notices.iter().enumerate() {
            let top = rect.top() + (FIRST_LINE + LINE_STEP * i as f32) * scale;
            let name = format!("d_{}", notice.weapon);
            let mut icon = |name: &str| self.sprites.icon(ui.ctx(), library, HUD_LIST, name);
            let icon = icon(&name).or_else(|| icon(FALLBACK_ICON));
            let icon_size = icon.as_ref().map_or(Vec2::ZERO, |i| i.size_vec2() * scale);

            // From the right edge, the victim, the icon and then the killer
            let victim = painter.layout_job(name_job(&notice.victim, font.clone()));
            let victim_x = rect.right() - RIGHT_MARGIN - victim.size().x;
            let icon_x = victim_x - icon_size.x;
            painter.galley(Pos2::new(victim_x, top), victim, Color32::WHITE);

            if let Some(icon) = &icon {
                let tint = if notice.teamkill { TEAMKILL_ICON } else { ICON };
                let place = Rect::from_min_size(Pos2::new(icon_x, top), icon_size);
                let whole = Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.));
                painter.image(icon.id(), place, whole, tint);
            }

            if let Some(killer) = &notice.killer {
                let killer = painter.layout_job(name_job(killer, font.clone()));
                let killer_x = icon_x - KILLER_GAP * scale - killer.size().x;
                painter.galley(Pos2::new(killer_x, top), killer, Color32::WHITE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_codes_colour_what_follows_until_reset() {
        let name = NoticeName {
            name: "^7[L4^1TT^03R]".into(),
            team: "red".into(),
        };
        let job = name_job(&name, egui::FontId::proportional(13.));
        let runs: Vec<(String, Color32)> = job
            .sections
            .iter()
            .map(|s| {
                let text = &job.text[s.byte_range.start.0..s.byte_range.end.0];
                (text.to_owned(), s.format.color)
            })
            .filter(|(text, _)| !text.is_empty())
            .collect();
        assert_eq!(
            runs,
            [
                ("[L4".to_owned(), TAG_COLOURS[6]),
                ("TT".to_owned(), TAG_COLOURS[0]),
                ("3R]".to_owned(), TEAM_RED),
            ]
        );
    }
}
