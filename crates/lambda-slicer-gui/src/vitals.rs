//! The recording player's health, armour and ammo over the view, where the
//! game's client draws them and with its sprites (HLSDK's `CHudHealth`,
//! `CHudBattery`, `CHudAmmo` and `DrawHudNumber`)
//!
//! All three stand on one row, a digit and a half above the bottom. The
//! health has the cross, three digits and a thin bar after them at the left,
//! the armour a fifth of the way across has the suit, filled as far as the
//! armour goes, and three digits, and the ammo at the right has the
//! magazine, a bar and the ammo carried, then the ammo's icon from the
//! weapon's own sprite list. A second kind of ammo stands a row higher.
//! Numbers have three digits' room, leading zeros left blank.
//!
//! Everything is added to the picture in the HUD's colour, brighter for a
//! while after it changes. Health and armour start at 228 of 255 and fade
//! to 100 over five seconds, health is red at 25 or less and full
//! brightness at 15 or less, and the ammo starts at 200 after each word
//! of the weapon and fades to 100. Sizes are as on a 640 by 480 screen, as
//! the kill feed's are, the width being as wide as the view makes it, then
//! made smaller or larger by the size the viewer is set to. The screen then
//! grows or shrinks with them, so they keep to the same corners

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};
use lambda_slicer_audio::library::Library;
use lambda_slicer_playback::status::{Reading, Shown, WeaponStatus};
use lambda_slicer_playback::time::SourceTime;

use crate::hud_sprites::{HUD_LIST, HudSprites};
use crate::viewer::HUD_COLOR;

/// The game's screen height the HUD is laid out on
const SCREEN_HEIGHT: f32 = 480.;

/// The least brightness, and how much more a change starts with and how
/// fast that fades, in brightness a second (`MIN_ALPHA`, `FADE_TIME`)
const LEAST: f32 = 100.;
const CHANGE_FADE: f32 = 100.;
const CHANGE_BOOST: f32 = 128.;
const FADE_PER_SECOND: f32 = 20.;

/// What the ammo starts at after each word of the weapon
const AMMO_FADE: f32 = 200.;

/// Health that turns red, and health that shows at full brightness
const HURT: i32 = 25;
const DYING: i32 = 15;
const HURT_COLOUR: Color32 = Color32::from_rgb(250, 0, 0);

/// How far across the armour stands, as a share of the width
const ARMOUR_ACROSS: f32 = 1. / 5.;

/// Digits a number has room for, and the ammo's room in digits from the
/// right with a magazine and without
const DIGITS: u32 = 3;
const AMMO_ROOM: f32 = 8.;
const RESERVE_ROOM: f32 = 4.;

/// The brightness of a reading at `at`, `start` more than the least at its
/// change, fading by `FADE_PER_SECOND`
fn faded(changed: SourceTime, at: SourceTime, start: f32) -> f32 {
    let since = (at.0 - changed.0).max(0.) as f32;
    (start - since * FADE_PER_SECOND).max(0.)
}

/// The brightness of health or armour at `at` (`MIN_ALPHA + fade / FADE_TIME
/// * 128`)
fn reading_brightness(reading: &Reading, at: SourceTime) -> f32 {
    let fade = faded(reading.changed, at, CHANGE_FADE);
    if fade > 0. {
        LEAST + fade / CHANGE_FADE * CHANGE_BOOST
    } else {
        LEAST
    }
}

/// `colour` at `brightness` of 255, to add to the picture (`ScaleColors`)
fn scaled(colour: Color32, brightness: f32) -> Color32 {
    let k = brightness.clamp(0., 255.) / 255.;
    let [r, g, b, _] = colour.to_array();
    let scale = |c: u8| (f32::from(c) * k).round() as u8;
    Color32::from_rgba_premultiplied(scale(r), scale(g), scale(b), 0)
}

/// The digits of `value` with room for three, leading places left blank
/// and a nought drawn for nought
fn digits(value: i32) -> [Option<u32>; DIGITS as usize] {
    let value = value.clamp(0, 999) as u32;
    let hundreds = (value >= 100).then_some(value / 100);
    let tens = (value >= 10).then_some(value % 100 / 10);
    [hundreds, tens, Some(value % 10)]
}

/// Draws on the view at the game's scale
struct Painter<'a> {
    painter: egui::Painter,
    ctx: &'a egui::Context,
    library: &'a Library,
    sprites: &'a mut HudSprites,
    rect: Rect,
    scale: f32,
}

impl Painter<'_> {
    /// The point `x`, `y` of the game's screen
    fn at(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(
            self.rect.left() + x * self.scale,
            self.rect.top() + y * self.scale,
        )
    }

    /// The icon `name` of the list `list`
    fn icon(&mut self, list: &str, name: &str) -> Option<TextureHandle> {
        self.sprites.icon(self.ctx, self.library, list, name)
    }

    /// `icon` added at `x`, `y` in `tint`, the rows from `from` of its height
    /// on, as a share of it, the ones above left out
    fn draw(&self, icon: &TextureHandle, x: f32, y: f32, tint: Color32, from: f32) {
        let size = icon.size_vec2();
        let top = size.y * from;
        let place = Rect::from_min_size(
            self.at(x, y + top),
            Vec2::new(size.x, size.y - top) * self.scale,
        );
        let uv = Rect::from_min_max(Pos2::new(0., from), Pos2::new(1., 1.));
        self.painter.image(icon.id(), place, uv, tint);
    }

    /// A bar `width` by `height` added at `x`, `y` in `tint` (`FillRGBA`)
    fn bar(&self, x: f32, y: f32, width: f32, height: f32, tint: Color32) {
        let place = Rect::from_min_size(self.at(x, y), Vec2::new(width, height) * self.scale);
        self.painter.rect_filled(place, 0., tint);
    }

    /// `value` in digits at `x`, `y` with three digits' room, returning where
    /// the room ends
    fn number(&mut self, x: f32, y: f32, value: i32, tint: Color32) -> f32 {
        let mut x = x;
        for digit in digits(value) {
            let Some(icon) = self.icon(HUD_LIST, "number_0") else {
                return x;
            };
            let width = icon.size_vec2().x;
            if let Some(digit) = digit
                && let Some(icon) = self.icon(HUD_LIST, &format!("number_{digit}"))
            {
                self.draw(&icon, x, y, tint, 0.);
            }
            x += width;
        }
        x
    }
}

/// The recording player's health, armour and ammo in `shown` over the view at
/// `rect` at `at`, `size` times the game's size, their sprites from `library`
pub fn draw(
    ui: &egui::Ui,
    sprites: &mut HudSprites,
    library: &Library,
    rect: Rect,
    size: f32,
    shown: &Shown,
    at: SourceTime,
) {
    let scale = rect.height() / SCREEN_HEIGHT * size;
    let mut p = Painter {
        painter: ui.painter_at(rect),
        ctx: ui.ctx(),
        library,
        sprites,
        rect,
        scale,
    };

    // A digit's size sets the row and the room numbers take (`m_iFontHeight`)
    let Some(digit) = p.icon(HUD_LIST, "number_0") else {
        return;
    };
    let digit = digit.size_vec2();
    let (width, height) = (rect.width() / scale, rect.height() / scale);
    let row = height - digit.y - digit.y / 2.;

    if let Some(health) = &shown.health {
        draw_health(&mut p, health, digit, row, at);
    }
    if let Some(armour) = &shown.armour {
        draw_armour(&mut p, armour, row, width, at);
    }
    if let Some(weapon) = &shown.weapon {
        draw_ammo(&mut p, weapon, digit, row, width, at);
    }
}

/// The cross, the health and the bar after it, at the left
fn draw_health(p: &mut Painter, health: &Reading, digit: Vec2, row: f32, at: SourceTime) {
    let mut brightness = reading_brightness(health, at);
    if health.value <= DYING {
        brightness = 255.;
    }
    let colour = if health.value > HURT {
        HUD_COLOR
    } else {
        HURT_COLOUR
    };
    let tint = scaled(colour, brightness);

    let Some(cross) = p.icon(HUD_LIST, "cross") else {
        return;
    };
    let cross_width = cross.size_vec2().x;
    p.draw(&cross, cross_width / 2., row, tint, 0.);

    let x = cross_width + digit.x / 2.;
    let x = p.number(x, row, health.value, tint) + digit.x / 2.;
    p.bar(
        x,
        row,
        digit.x / 10.,
        digit.y,
        scaled(HUD_COLOR, brightness),
    );
}

/// The suit filled as far as the armour goes and the armour, a fifth of the
/// way across
fn draw_armour(p: &mut Painter, armour: &Reading, row: f32, width: f32, at: SourceTime) {
    let tint = scaled(HUD_COLOR, reading_brightness(armour, at));
    let (Some(empty), Some(full)) = (
        p.icon(HUD_LIST, "suit_empty"),
        p.icon(HUD_LIST, "suit_full"),
    ) else {
        return;
    };

    // The full suit cut from the top by the armour missing
    let suit = empty.size_vec2();
    let x = width * ARMOUR_ACROSS;
    let y = row - suit.y / 6.;
    let missing = (100 - armour.value.min(100)).max(0) as f32 / 100.;
    p.draw(&empty, x, y, tint, 0.);
    if missing < 1. {
        p.draw(&full, x, y, tint, missing);
    }

    p.number(x + suit.x, row, armour.value, tint);
}

/// The magazine, a bar and the ammo carried with the ammo's icon at the
/// right, and the second ammo a row higher
fn draw_ammo(
    p: &mut Painter,
    weapon: &WeaponStatus,
    digit: Vec2,
    row: f32,
    width: f32,
    at: SourceTime,
) {
    let brightness = faded(weapon.changed, at, AMMO_FADE).max(LEAST);
    let tint = scaled(HUD_COLOR, brightness);
    let list = format!("sprites/{}.txt", weapon.name);

    if let Some(ammo) = weapon.ammo {
        let icon = p.icon(&list, "ammo");
        let icon_width = icon.as_ref().map_or(0., |i| i.size_vec2().x);
        let x = match weapon.magazine {
            Some(magazine) => {
                let x = width - AMMO_ROOM * digit.x - icon_width;
                let x = p.number(x, row, magazine, tint) + digit.x / 2.;
                let bar = digit.x / 10.;
                p.bar(x, row, bar, digit.y, tint);
                p.number(x + bar + digit.x / 2., row, ammo, tint)
            }
            None => {
                let x = width - RESERVE_ROOM * digit.x - icon_width;
                p.number(x, row, ammo, tint)
            }
        };
        if let Some(icon) = icon {
            let lift = icon.size_vec2().y / 8.;
            p.draw(&icon, x, row - lift, tint, 0.);
        }
    }

    if let Some(ammo2) = weapon.ammo2 {
        let icon = p.icon(&list, "ammo2");
        let icon_width = icon.as_ref().map_or(0., |i| i.size_vec2().x);
        let y = row - digit.y - digit.y / 4.;
        let x = width - RESERVE_ROOM * digit.x - icon_width;
        let x = p.number(x, y, ammo2, tint);
        if let Some(icon) = icon {
            let lift = icon.size_vec2().y / 8.;
            p.draw(&icon, x, y - lift, tint, 0.);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_leave_leading_places_blank_and_draw_nought() {
        assert_eq!(digits(100), [Some(1), Some(0), Some(0)]);
        assert_eq!(digits(37), [None, Some(3), Some(7)]);
        assert_eq!(digits(0), [None, None, Some(0)]);
        assert_eq!(digits(-5), [None, None, Some(0)]);
    }

    #[test]
    fn a_change_brightens_then_fades_over_five_seconds() {
        let reading = Reading {
            value: 100,
            changed: SourceTime(10.),
        };
        assert_eq!(reading_brightness(&reading, SourceTime(10.)), 228.);
        assert_eq!(reading_brightness(&reading, SourceTime(12.5)), 164.);
        assert_eq!(reading_brightness(&reading, SourceTime(15.)), LEAST);
        assert_eq!(
            faded(SourceTime(0.), SourceTime(2.), AMMO_FADE).max(LEAST),
            160.
        );
    }
}
