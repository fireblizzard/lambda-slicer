//! The HUD's sprites, read from the game's files as they are first needed
//!
//! A sprite list such as `sprites/hud.txt` or a weapon's own
//! `sprites/weapon_crossbow.txt` names each icon with a screen width, a
//! sheet in `sprites/` and the icon's place in it. The HUD takes those for
//! a 640 wide screen. Most icons are drawn as the game draws them, added to
//! the picture below in the colour the HUD tints them

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{self, Color32, TextureHandle};
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::sprite::{self, Sprite};
use lambda_slicer_audio::library::Library;

/// The HUD's own sprite list
pub const HUD_LIST: &str = "sprites/hud.txt";

/// The screen width whose sprites a list names for the HUD
const HUD_RESOLUTION: &str = "640";

/// Where an icon lies in its sheet, as a sprite list gives it
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub sheet: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The icons a sprite list places for the HUD's screen width. Each line is
/// a name, a screen width, a sheet in `sprites/` and the icon's place in it
pub fn places(text: &str) -> HashMap<String, Place> {
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

/// The sprite lists, sheets and icons read so far
#[derive(Default)]
pub struct HudSprites {
    lists: HashMap<String, HashMap<String, Place>>,
    sheets: HashMap<String, Option<Arc<Sprite>>>,
    /// Each list's icons by name, for adding to the picture
    icons: HashMap<(String, String), Option<TextureHandle>>,
    /// Whole frames of sheets by sheet and frame, for adding to the picture
    frames: HashMap<(String, usize), Option<TextureHandle>>,
}

impl HudSprites {
    /// Where the list at `list` places `name`, read from `library` the first
    /// time the list is asked for
    pub fn place(&mut self, library: &Library, list: &str, name: &str) -> Option<Place> {
        let places = self.lists.entry(list.to_owned()).or_insert_with(|| {
            let text = library.read(list).unwrap_or_default();
            places(&String::from_utf8_lossy(&text))
        });
        places.get(name).cloned()
    }

    /// The sheet `sprites/<name>.spr`, read from `library` the first time
    pub fn sheet(&mut self, library: &Library, name: &str) -> Option<Arc<Sprite>> {
        self.sheets
            .entry(name.to_owned())
            .or_insert_with(|| {
                let bytes = library.read(&format!("sprites/{name}.spr"))?;
                sprite::parse(&bytes, AssetLimits::default())
                    .ok()
                    .map(Arc::new)
            })
            .clone()
    }

    /// The icon `name` of the list at `list`, to add to the picture, read
    /// from `library` the first time it is asked for
    pub fn icon(
        &mut self,
        ctx: &egui::Context,
        library: &Library,
        list: &str,
        name: &str,
    ) -> Option<TextureHandle> {
        let key = (list.to_owned(), name.to_owned());
        if let Some(icon) = self.icons.get(&key) {
            return icon.clone();
        }

        let icon = self.place(library, list, name).and_then(|place| {
            let sheet = self.sheet(library, &place.sheet)?;
            let id = format!("{list} {name}");
            Some(ctx.load_texture(id, added(&sheet, 0, &place)?, egui::TextureOptions::LINEAR))
        });
        self.icons.insert(key, icon.clone());
        icon
    }

    /// The whole frame `frame` of the sheet `sprites/<sheet>.spr`, to add to
    /// the picture, read from `library` the first time it is asked for
    pub fn frame(
        &mut self,
        ctx: &egui::Context,
        library: &Library,
        sheet: &str,
        frame: usize,
    ) -> Option<TextureHandle> {
        let key = (sheet.to_owned(), frame);
        if let Some(texture) = self.frames.get(&key) {
            return texture.clone();
        }

        let texture = self.sheet(library, sheet).and_then(|sprite| {
            let whole = sprite.frames.get(frame)?;
            let place = Place {
                sheet: sheet.to_owned(),
                x: 0,
                y: 0,
                width: whole.width,
                height: whole.height,
            };
            let id = format!("{sheet} frame {frame}");
            let image = added(&sprite, frame, &place)?;
            Some(ctx.load_texture(id, image, egui::TextureOptions::LINEAR))
        });
        self.frames.insert(key, texture.clone());
        texture
    }
}

/// The icon at `place` in `sheet`'s frame `frame`, its colours added to
/// what is under it as the game adds a HUD sprite, so it carries no opacity
fn added(sheet: &Sprite, frame: usize, place: &Place) -> Option<egui::ColorImage> {
    let frame = sheet.frames.get(frame)?;
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
    fn a_list_places_the_hud_widths_icons() {
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
}
