//! The chat over the view, as the game's client draws its say text
//! (HLSDK's `CHudSayText`, with OpenAG's colour codes and locations)
//!
//! Lines start 10 of the game's columns from the left and stand one under
//! another from as many rows above the bottom as seven lines and 60 more
//! take, wrapping 40 short of the screen's width. The sender's name takes
//! their team's colour and the rest the console's, each coloured further
//! where AG's `^1` to `^8` say. A line's `%l`, `%L`, `%d` and `%D` become the
//! place nearest the sender in the map's `locs/<map>.loc`, a list of names
//! and their positions split by `#` (OpenAG's `CHudLocation`). Everything is
//! sized to the view as the game's 640 by 480 screen would be, as the kill
//! feed is

use std::collections::HashMap;

use eframe::egui::{self, Color32, Pos2, Rect};
use lambda_slicer_audio::library::Library;
use lambda_slicer_playback::hud::ChatLine;

use crate::kill_feed::{add_tagged, team_colour};

/// The game's screen height the HUD is laid out on
const SCREEN_HEIGHT: f32 = 480.;

/// Where lines start from the left, the room left on the right, the room
/// under the lines besides two more lines, and a line's height and text
/// size, in the game's rows
const LEFT: f32 = 10.;
const RIGHT_ROOM: f32 = 30.;
const BOTTOM_ROOM: f32 = 60.;
const LINE_HEIGHT: f32 = 16.;
const TEXT_SIZE: f32 = 13.;

/// Lines the chat makes room for, its five and two more
const ROOM_LINES: f32 = 7.;

/// The console's text colour, which chat takes (`con_color`)
const CONSOLE: Color32 = Color32::from_rgb(255, 180, 30);

/// The codes a sender's place stands in for
const PLACE_CODES: [&str; 4] = ["%l", "%L", "%d", "%D"];

/// A map's places, by name and position
type Places = Vec<(String, [f32; 3])>;

/// The places of a `.loc` file, a name and three coordinates at a time
fn places(text: &str) -> Places {
    let fields: Vec<&str> = text.split('#').collect();
    let mut out = Vec::new();
    let (entries, _) = fields.as_chunks::<4>();
    for [name, x, y, z] in entries {
        let number = |s: &str| s.trim().parse::<f32>().ok();
        if let (Some(x), Some(y), Some(z)) = (number(x), number(y), number(z)) {
            out.push((name.trim().to_owned(), [x, y, z]));
        }
    }
    out
}

/// The place in `places` nearest `at`, the first of any as near
fn nearest(places: &Places, at: [f32; 3]) -> Option<&str> {
    let mut best: Option<(&str, f32)> = None;
    for (name, position) in places {
        let d = [0, 1, 2].map(|k| position[k] - at[k]);
        let distance = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if best.is_none_or(|(_, least)| distance < least) {
            best = Some((name, distance));
        }
    }
    best.map(|(name, _)| name)
}

/// `text` with every code for a place in it as `place`
pub fn with_place(text: &str, place: &str) -> String {
    let mut out = text.to_owned();
    for code in PLACE_CODES {
        out = out.replace(code, place);
    }
    out
}

/// The map's name in a frame's map path, such as `boot_camp`
fn map_name(map: &str) -> &str {
    let name = map.strip_prefix("maps/").unwrap_or(map);
    name.strip_suffix(".bsp").unwrap_or(name)
}

/// The chat's places, read from the game's files as each map first needs
/// them
#[derive(Default)]
pub struct Chat {
    places: HashMap<String, Places>,
}

impl Chat {
    /// The places of `map`, read from `library` the first time
    fn places(&mut self, library: &Library, map: &str) -> &Places {
        self.places.entry(map.to_owned()).or_insert_with(|| {
            let text = library.read(&format!("locs/{map}.loc")).unwrap_or_default();
            places(&String::from_utf8_lossy(&text))
        })
    }

    /// The name of the place nearest `at` on `map`, from `library`
    pub fn place_name(&mut self, library: &Library, map: &str, at: [f32; 3]) -> Option<String> {
        nearest(self.places(library, map), at).map(str::to_owned)
    }

    /// One line laid out to wrap within `width`, its place codes filled in
    /// from the places of `map`
    fn line_job(
        &mut self,
        line: &ChatLine,
        map: Option<&str>,
        library: &Library,
        font: &egui::FontId,
        width: f32,
    ) -> egui::text::LayoutJob {
        let place = match (map, line.place) {
            (Some(map), Some(at)) => nearest(self.places(library, map), at).unwrap_or(""),
            _ => "",
        };

        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = width;
        match &line.name {
            Some((range, team)) => {
                let (before, name, after) = (
                    &line.text[..range.start],
                    &line.text[range.clone()],
                    &line.text[range.end..],
                );
                add_tagged(&mut job, &with_place(before, place), CONSOLE, font);
                add_tagged(&mut job, name, team_colour(team), font);
                add_tagged(&mut job, &with_place(after, place), CONSOLE, font);
            }
            None => add_tagged(&mut job, &with_place(&line.text, place), CONSOLE, font),
        }
        job
    }

    /// The chat's `lines` over the view at `rect`, its places from the map
    /// at `map` in `library`, its text `size` times the game's
    pub fn draw(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        lines: &[ChatLine],
        map: Option<&str>,
        library: &Library,
        size: f32,
    ) {
        let scale = rect.height() / SCREEN_HEIGHT;
        let font = egui::FontId::proportional(TEXT_SIZE * size * scale);
        let width = rect.width() - (LEFT + RIGHT_ROOM) * scale;
        let painter = ui.painter_at(rect);
        let map = map.map(map_name);

        let mut top = rect.bottom() - (BOTTOM_ROOM + LINE_HEIGHT * ROOM_LINES) * scale;
        for line in lines {
            let job = self.line_job(line, map, library, &font, width);
            let galley = painter.layout_job(job);
            let height = galley.size().y.max(LINE_HEIGHT * scale);
            painter.galley(Pos2::new(rect.left() + LEFT * scale, top), galley, CONSOLE);
            top += height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_loc_file_names_places_by_their_positions() {
        let text = "Bridge#2105.5#-113.25#46.5#Court#-933#384#47#broken#x#1#2#";
        let found = places(text);
        assert_eq!(
            found,
            [
                ("Bridge".to_owned(), [2105.5, -113.25, 46.5]),
                ("Court".to_owned(), [-933., 384., 47.]),
            ]
        );
        assert_eq!(nearest(&found, [-900., 380., 40.]), Some("Court"));
        assert_eq!(nearest(&Vec::new(), [0.; 3]), None);
    }

    #[test]
    fn every_place_code_takes_the_place() {
        assert_eq!(with_place("at %l, %D", "Bridge"), "at Bridge, Bridge");
        assert_eq!(map_name("maps/boot_camp.bsp"), "boot_camp");
    }
}
