//! The weapon menu over the view, where and how the game's client draws it
//! (HLSDK's `CHudAmmo::DrawWList`)
//!
//! A row of the five columns' numbers stands 10 of the game's columns and
//! rows from the top left, 5 apart. The open column is as wide as its
//! first weapon with ammo's picture, and under it each weapon shows its
//! picture with its ammo's bars, the one picked lit and framed, the rest
//! dimmer and red without ammo. Every other column shows a small box for
//! each weapon, red without ammo. Everything is added to the picture, sized
//! as the health, armour and ammo are

use eframe::egui::{self, Color32, Rect, Vec2};
use lambda_slicer_audio::library::Library;
use lambda_slicer_playback::weapon_menu::{MenuShown, MenuWeapon};

use crate::hud_sprites::{HUD_LIST, HudSprites};
use crate::viewer::HUD_COLOR;
use crate::vitals::{NO_AMMO_COLOUR, Painter, scaled};

/// Where the menu starts from the top left, and the room between columns
/// and between weapons, in the game's columns and rows
const MARGIN: f32 = 10.;
const GAP: f32 = 5.;

/// How bright a weapon not picked is, with ammo and without, and a box
/// with ammo and without, of 255
const UNPICKED: f32 = 192.;
const UNPICKED_EMPTY: f32 = 128.;
const BOX: f32 = 128.;
const BOX_EMPTY: f32 = 96.;

/// An ammo bar's size, where it starts on the picture, and its full part's
/// colour (`giABWidth`, `giABHeight`, `RGB_GREENISH`)
const BAR: Vec2 = Vec2::new(20., 4.);
const BAR_LEFT: f32 = BAR.x / 2.;
const BAR_FULL: Color32 = Color32::from_rgb(0, 160, 0);

/// How bright an ammo bar's empty part is, of 255
const BAR_EMPTY: f32 = 128.;

/// A weapon's list of pictures
fn list(weapon: &MenuWeapon) -> String {
    format!("sprites/{}.txt", weapon.name)
}

/// `menu` over the view at `rect`, `size` times the game's size, its
/// sprites from `library`
pub fn draw(
    ui: &egui::Ui,
    sprites: &mut HudSprites,
    library: &Library,
    rect: Rect,
    size: f32,
    menu: &MenuShown,
) {
    let mut p = Painter::new(ui, sprites, library, rect, size);
    let Some(bucket) = p.icon(HUD_LIST, "bucket1") else {
        return;
    };
    let bucket = bucket.size_vec2();

    // The open column is as wide as its first weapon with ammo's picture
    let mut open_width = bucket.x;
    if let Some(column) = menu.open_column.and_then(|c| menu.columns.get(c))
        && let Some(first) = column.iter().find(|w| w.has_ammo)
        && let Some(picture) = p.icon(&list(first), "weapon_s")
    {
        open_width = picture.size_vec2().x;
    }
    let width_of = |column: usize| {
        if menu.open_column == Some(column) {
            open_width
        } else {
            bucket.x
        }
    };

    // The columns' numbers along the top
    let mut x = MARGIN;
    for column in 0..menu.columns.len() {
        if let Some(number) = p.icon(HUD_LIST, &format!("bucket{}", column + 1)) {
            p.draw(&number, x, MARGIN, scaled(HUD_COLOR, 255.), 0.);
        }
        x += width_of(column) + GAP;
    }

    // Each column's weapons under its number
    let mut x = MARGIN;
    for (column, weapons) in menu.columns.iter().enumerate() {
        let mut y = bucket.y + MARGIN;
        for weapon in weapons {
            if menu.open_column == Some(column) {
                y += draw_weapon(&mut p, weapon, x, y) + GAP;
            } else {
                let (colour, brightness) = if weapon.has_ammo {
                    (HUD_COLOR, BOX)
                } else {
                    (NO_AMMO_COLOUR, BOX_EMPTY)
                };
                p.bar(x, y, bucket.x, bucket.y, scaled(colour, brightness));
                y += bucket.y + GAP;
            }
        }
        x += width_of(column) + GAP;
    }
}

/// `weapon`'s picture and ammo bars at `x`, `y` in the open column,
/// returning the height it takes
fn draw_weapon(p: &mut Painter, weapon: &MenuWeapon, x: f32, y: f32) -> f32 {
    let list = list(weapon);
    let Some(lit) = p.icon(&list, "weapon_s") else {
        return 0.;
    };

    if weapon.picked {
        p.draw(&lit, x, y, scaled(HUD_COLOR, 255.), 0.);
        if let Some(frame) = p.icon(HUD_LIST, "selection") {
            p.draw(&frame, x, y, scaled(HUD_COLOR, 255.), 0.);
        }
    } else if let Some(dim) = p.icon(&list, "weapon") {
        let tint = if weapon.has_ammo {
            scaled(HUD_COLOR, UNPICKED)
        } else {
            scaled(NO_AMMO_COLOUR, UNPICKED_EMPTY)
        };
        p.draw(&dim, x, y, tint, 0.);
    }

    // Each ammo's bar, full as far as it is, the second a little after
    let mut bar_x = x + BAR_LEFT;
    for &share in &weapon.bars {
        let mut full = share * BAR.x;
        if share > 0. {
            full = full.max(1.);
            p.bar(bar_x, y, full, BAR.y, scaled(BAR_FULL, 255.));
        }
        p.bar(
            bar_x + full,
            y,
            BAR.x - full,
            BAR.y,
            scaled(HUD_COLOR, BAR_EMPTY),
        );
        bar_x += BAR.x + GAP;
    }

    lit.size_vec2().y
}
