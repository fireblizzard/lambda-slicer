//! The crosshair over the view's middle, as the game's client sets it for
//! the weapon in hand (HLSDK's `CHudAmmo`)
//!
//! Each weapon's `sprites/weapon_*.txt` places its `crosshair` in a sheet,
//! `crosshairs` for every weapon of the game, and the crossbow and the 357
//! a `zoom` one, drawn while the player's field of view is under 90. The
//! sheet's own format says how it is drawn, the game's `crosshairs.spr`
//! with index 255 left out. It is sized to the view as the game's 640 by
//! 480 screen would be, as the kill feed is

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle};
use lambda_slicer_assets::Limits as AssetLimits;
use lambda_slicer_assets::sprite::{self, Sprite};
use lambda_slicer_audio::library::Library;

use crate::kill_feed::{Place, places};

/// The game's screen height the HUD is laid out on
const SCREEN_HEIGHT: f32 = 480.;

/// Under this field of view the zoomed crosshair shows
const ZOOMED_UNDER: f32 = 90.;

/// The file naming the crosshairs of each weapon in view
const WEAPON_FILES: [(&str, &str); 15] = [
    ("models/v_crossbow.mdl", "weapon_crossbow"),
    ("models/v_crowbar.mdl", "weapon_crowbar"),
    ("models/v_egon.mdl", "weapon_egon"),
    ("models/v_gauss.mdl", "weapon_gauss"),
    ("models/v_9mmhandgun.mdl", "weapon_9mmhandgun"),
    ("models/v_grenade.mdl", "weapon_handgrenade"),
    ("models/v_hgun.mdl", "weapon_hornetgun"),
    ("models/v_9mmAR.mdl", "weapon_9mmar"),
    ("models/v_357.mdl", "weapon_357"),
    ("models/v_rpg.mdl", "weapon_rpg"),
    ("models/v_shotgun.mdl", "weapon_shotgun"),
    ("models/v_squeak.mdl", "weapon_snark"),
    ("models/v_tripmine.mdl", "weapon_tripmine"),
    ("models/v_satchel_radio.mdl", "weapon_satchel"),
    ("models/v_satchel.mdl", "weapon_satchel"),
];

/// The weapon file of the weapon whose view model is `viewmodel`
fn weapon_file(viewmodel: &str) -> Option<&'static str> {
    WEAPON_FILES
        .iter()
        .find(|(model, _)| model.eq_ignore_ascii_case(viewmodel))
        .map(|(_, file)| *file)
}

/// The crosshairs, read from the game's files as they are first needed
#[derive(Default)]
pub struct Crosshairs {
    /// Each weapon file's places by name
    files: HashMap<String, HashMap<String, Place>>,
    sheets: HashMap<String, Option<Arc<Sprite>>>,
    /// By weapon file and whether zoomed
    textures: HashMap<(String, bool), Option<TextureHandle>>,
}

impl Crosshairs {
    /// The crosshair of the weapon file `file`, zoomed or not, read from
    /// `library` the first time it is asked for. A weapon with no zoomed
    /// one keeps its own while zoomed, as the game does
    fn texture(
        &mut self,
        ctx: &egui::Context,
        library: &Library,
        file: &str,
        zoomed: bool,
    ) -> Option<TextureHandle> {
        let key = (file.to_owned(), zoomed);
        if let Some(texture) = self.textures.get(&key) {
            return texture.clone();
        }

        let places = self.files.entry(file.to_owned()).or_insert_with(|| {
            let text = library
                .read(&format!("sprites/{file}.txt"))
                .unwrap_or_default();
            places(&String::from_utf8_lossy(&text))
        });
        let mut place = places.get("crosshair").cloned();
        if zoomed && let Some(zoom) = places.get("zoom") {
            place = Some(zoom.clone());
        }

        let texture = place.and_then(|place| {
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
            let name = format!("crosshair {file} {zoomed}");
            let options = egui::TextureOptions::NEAREST;
            Some(ctx.load_texture(name, cut(&sheet, &place)?, options))
        });
        self.textures.insert(key, texture.clone());
        texture
    }

    /// The crosshair of the weapon whose view model is `viewmodel` in the
    /// middle of the view at `rect`, the zoomed one under a field of view
    /// of 90, from `library`
    pub fn draw(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        viewmodel: &str,
        fov: Option<f32>,
        library: &Library,
    ) {
        let Some(file) = weapon_file(viewmodel) else {
            return;
        };
        let zoomed = fov.is_some_and(|f| f < ZOOMED_UNDER);
        let Some(texture) = self.texture(ui.ctx(), library, file, zoomed) else {
            return;
        };

        // Whole points, so the sprite's pixels stay sharp
        let scale = rect.height() / SCREEN_HEIGHT;
        let size = (texture.size_vec2() * scale).round();
        let corner = (rect.center() - size / 2.).round();
        let place = Rect::from_min_size(corner, size);
        let whole = Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.));
        ui.painter_at(rect)
            .image(texture.id(), place, whole, Color32::WHITE);
    }
}

/// The crosshair at `place` in `sheet`'s first frame, coloured as the
/// sheet's format says
fn cut(sheet: &Sprite, place: &Place) -> Option<egui::ColorImage> {
    let frame = sheet.frames.first()?;
    let rgba = sheet.rgba(frame);
    let (width, height) = (place.width as usize, place.height as usize);
    let mut pixels = Vec::with_capacity(width * height);
    for row in 0..height {
        for column in 0..width {
            let (x, y) = (place.x as usize + column, place.y as usize + row);
            if x >= frame.width as usize || y >= frame.height as usize {
                pixels.push(Color32::TRANSPARENT);
                continue;
            }
            let at = (y * frame.width as usize + x) * 4;
            let [r, g, b, a] = [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]];
            pixels.push(Color32::from_rgba_unmultiplied(r, g, b, a));
        }
    }
    Some(egui::ColorImage::new([width, height], pixels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_weapon_in_view_has_its_file() {
        assert_eq!(weapon_file("models/v_9mmAR.mdl"), Some("weapon_9mmar"));
        assert_eq!(weapon_file("models/V_SQUEAK.MDL"), Some("weapon_snark"));
        assert_eq!(weapon_file("models/v_unknown.mdl"), None);
    }

    #[test]
    fn a_weapon_file_places_its_crosshairs() {
        let text = "crosshair\t\t320 crosshairs\t72\t0\t24\t24\n\
                    crosshair\t\t640 crosshairs\t72\t0\t24\t24\n\
                    zoom\t\t\t640 crosshairs\t24\t96\t104\t16\n";
        let found = places(text);
        assert_eq!(found["crosshair"].x, 72);
        assert_eq!(found["zoom"].width, 104);
    }
}
