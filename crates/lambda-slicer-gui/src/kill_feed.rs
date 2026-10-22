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

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::sprite::{self, Sprite};
use lambda_slicer_audio::library::Library;
use lambda_slicer_playback::hud::{DeathNotice, NoticeName};

/// The game's screen height the HUD is laid out on
const SCREEN_HEIGHT: f32 = 480.;

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

/// The screen width whose sprites `hud.txt` names for the HUD
pub(crate) const HUD_RESOLUTION: &str = "640";

/// The colours `^1` to `^8` give a name
const TAG_COLOURS: [Color32; 8] = [
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

/// Where an icon lies in its sheet, as `hud.txt` gives it
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Place {
    pub sheet: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The kill feed's icons, read from the game's files as they are first
/// needed
#[derive(Default)]
pub struct KillFeed {
    /// `hud.txt`'s icons by name, none until read
    places: Option<HashMap<String, Place>>,
    sheets: HashMap<String, Option<Arc<Sprite>>>,
    icons: HashMap<String, Option<TextureHandle>>,
}

/// The icons `hud.txt` places for the HUD's screen width. Each line is a
/// name, a screen width, a sheet in `sprites/` and the icon's place in it.
/// A weapon's own file, such as `sprites/weapon_crossbow.txt`, is laid out
/// the same
pub(crate) fn places(text: &str) -> HashMap<String, Place> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let [name, resolution, sheet, x, y, width, height] = words[..] else {
            continue;
        };
        if resolution != HUD_RESOLUTION {
            continue;
        }

        let number = |s: &str| s.parse::<u32>().ok();
        if let (Some(x), Some(y), Some(width), Some(height)) =
            (number(x), number(y), number(width), number(height))
        {
            let sheet = sheet.to_owned();
            out.insert(
                name.to_owned(),
                Place {
                    sheet,
                    x,
                    y,
                    width,
                    height,
                },
            );
        }
    }
    out
}

/// A team's colour for a name
fn team_colour(team: &str) -> Color32 {
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
    let team = team_colour(&name.team);
    let mut job = egui::text::LayoutJob::default();
    let mut colour = team;
    let mut run = String::new();
    let mut chars = name.name.chars().peekable();
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
            _ => team,
        };
    }
    job.append(&run, 0., egui::TextFormat::simple(font, colour));
    job
}

impl KillFeed {
    /// The icon named `name`, read from `library` the first time it is
    /// asked for
    fn icon(
        &mut self,
        ctx: &egui::Context,
        library: &Library,
        name: &str,
    ) -> Option<TextureHandle> {
        if let Some(icon) = self.icons.get(name) {
            return icon.clone();
        }

        let places = self.places.get_or_insert_with(|| {
            let text = library.read("sprites/hud.txt").unwrap_or_default();
            places(&String::from_utf8_lossy(&text))
        });
        let place = places.get(name).cloned();
        let icon = place.and_then(|place| {
            let sheet = self
                .sheets
                .entry(place.sheet.clone())
                .or_insert_with(|| {
                    let bytes = library.read(&format!("sprites/{}.spr", place.sheet))?;
                    sprite::parse(&bytes, AssetLimits::default())
                        .ok()
                        .map(Arc::new)
                })
                .clone()?;
            Some(ctx.load_texture(name, cut(&sheet, &place)?, egui::TextureOptions::LINEAR))
        });
        self.icons.insert(name.to_owned(), icon.clone());
        icon
    }

    /// The kill feed's `notices` over the view at `rect`, its icons from
    /// `library`
    pub fn draw(&mut self, ui: &egui::Ui, rect: Rect, notices: &[DeathNotice], library: &Library) {
        let scale = rect.height() / SCREEN_HEIGHT;
        let font = egui::FontId::proportional(TEXT_SIZE * scale);
        let painter = ui.painter_at(rect);

        for (i, notice) in notices.iter().enumerate() {
            let top = rect.top() + (FIRST_LINE + LINE_STEP * i as f32) * scale;
            let name = format!("d_{}", notice.weapon);
            let icon = self
                .icon(ui.ctx(), library, &name)
                .or_else(|| self.icon(ui.ctx(), library, FALLBACK_ICON));
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

/// The icon at `place` in `sheet`'s first frame, its colours added to what
/// is under it as the game adds a HUD sprite, so it carries no opacity
fn cut(sheet: &Sprite, place: &Place) -> Option<egui::ColorImage> {
    let frame = sheet.frames.first()?;
    let (width, height) = (place.width as usize, place.height as usize);
    let mut pixels = Vec::with_capacity(width * height);
    for row in 0..height {
        for column in 0..width {
            let (x, y) = (place.x as usize + column, place.y as usize + row);
            if x >= frame.width as usize || y >= frame.height as usize {
                pixels.push(Color32::TRANSPARENT);
                continue;
            }
            let index = frame.indices[y * frame.width as usize + x];
            let [r, g, b] = sheet
                .palette
                .get(usize::from(index))
                .copied()
                .unwrap_or([0; 3]);
            pixels.push(Color32::from_rgba_premultiplied(r, g, b, 0));
        }
    }
    Some(egui::ColorImage::new([width, height], pixels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_txt_places_the_hud_widths_icons() {
        let text = "137\nd_9mmAR\t\t320\t320hud1\t96\t48\t24\t8\nd_9mmAR\t\t\t640\t640hud1\t192\t48\t48\t16\nbroken 640 x\n";
        let found = places(text);
        assert_eq!(found.len(), 1);
        assert_eq!(
            found["d_9mmAR"],
            Place {
                sheet: "640hud1".into(),
                x: 192,
                y: 48,
                width: 48,
                height: 16,
            }
        );
    }

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
