//! The recording player's health, armour and ammo over the view, and what
//! they picked up, where the game's client draws them and with its sprites
//! (HLSDK's `CHudHealth`, `CHudBattery`, `CHudAmmo`, `DrawHudNumber` and
//! `HistoryResource`)
//!
//! All three stand on one row, a digit and a half above the bottom. The
//! health has the cross, three digits and a thin bar after them at the left,
//! the armour a fifth of the way across has the suit, filled as far as the
//! armour goes, and three digits, and the ammo at the right has the
//! magazine, a bar and the ammo carried, then the ammo's icon from the
//! weapon's own sprite list. A second kind of ammo stands a row higher.
//! Numbers have three digits' room, leading zeros left blank.
//!
//! The flashlight stands at the top right, half its width from the edge
//! and 16 rows down, filled from the right as far as its battery goes,
//! with the beam after it while it is on. It is brighter while on, and red
//! under a fifth of its battery (`CHudFlashlight`).
//!
//! What was picked up stands at the right, each in its place 50 rows above
//! the one below, a weapon's height and a little more. The game puts the lowest 72 rows above the bottom, where
//! it can cover the ammo, so here it stands a tenth of the height under the
//! middle instead, the rest climbing from there. Ammo is its icon 24
//! from the right with how much before it, a weapon its sprite against the
//! right edge, red when it has no ammo to fire, and an item its icon 10
//! from the right. Each is at full brightness until less than a little
//! over three seconds are left, then fades out.
//!
//! A hit marks the side it came from around the middle of the view with
//! that side's frame of `sprites/640_pain.spr`, in the HUD's colour or red
//! at 25 health or less, as bright as the mark is strong but never under
//! half. Above and below stand three and two of their heights from the
//! middle, and left and right three and two of their widths. Lasting
//! damage shows its icon at the left, the lowest two of its heights above
//! the bottom and an eighth of its width from the edge, the rest stacked
//! above, all flashing together.
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
use lambda_slicer_playback::damage::{DamageShown, Side};
use lambda_slicer_playback::status::{
    Flashlight, Pickup, PickupShown, Reading, Shown, WeaponStatus,
};
use lambda_slicer_playback::time::SourceTime;

use crate::hud_sprites::{HUD_LIST, HudSprites};

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

/// Where the lowest pickup stands, as a share of the height down from the
/// top, how far apart pickups stand, the tallest weapon's 45 rows and 5
/// more, where ammo's icon and an item's stand from the right, and the room
/// before ammo's icon (`AMMO_PICKUP_GAP`, `iHistoryGap`)
const PICKUP_LOWEST: f32 = 0.6;
const PICKUP_STEP: f32 = 50.;
const PICKUP_AMMO_RIGHT: f32 = 24.;
const PICKUP_ITEM_RIGHT: f32 = 10.;
const PICKUP_COUNT_GAP: f32 = 10.;

/// How much brighter a pickup is for each second it has left, the text's
/// size, and the colour of a weapon without ammo (`RGB_REDISH`)
const PICKUP_FADE: f32 = 80.;
const PICKUP_TEXT_SIZE: f32 = 13.;
pub(crate) const NO_AMMO_COLOUR: Color32 = Color32::from_rgb(255, 16, 16);

/// The flashlight's brightness while on, the battery that turns it red,
/// and its rows from the top, the sheet's 32 row icons standing half their
/// height down
const FLASHLIGHT_ON: f32 = 225.;
const FLASHLIGHT_LOW: i32 = 20;
const FLASHLIGHT_TOP: f32 = 16.;

/// The sheet whose four frames mark a hit's side, above, right, below and
/// left (`PAIN_NAME`)
const PAIN_SHEET: &str = "640_pain";

/// The least share of full brightness a mark shows at, and how many of the
/// lasting damage icon's heights its lowest stands above the bottom
const LEAST_MARK_BRIGHTNESS: f32 = 0.5;
const KIND_BOTTOM: f32 = 2.;

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
pub(crate) fn scaled(colour: Color32, brightness: f32) -> Color32 {
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

/// How large the HUD stands against the game's, and its colour, as
/// `hud_color` sets it
#[derive(Clone, Copy)]
pub(crate) struct HudLook {
    pub size: f32,
    pub colour: Color32,
}

/// Draws on the view at the game's scale
pub(crate) struct Painter<'a> {
    painter: egui::Painter,
    ctx: &'a egui::Context,
    library: &'a Library,
    sprites: &'a mut HudSprites,
    rect: Rect,
    scale: f32,
    /// The HUD's colour
    pub(crate) colour: Color32,
}

impl<'a> Painter<'a> {
    /// A painter over the view at `rect`, as large and in the colour `look`
    /// gives
    pub(crate) fn new(
        ui: &'a egui::Ui,
        sprites: &'a mut HudSprites,
        library: &'a Library,
        rect: Rect,
        look: HudLook,
    ) -> Self {
        Painter {
            painter: ui.painter_at(rect),
            ctx: ui.ctx(),
            library,
            sprites,
            rect,
            scale: rect.height() / SCREEN_HEIGHT * look.size,
            colour: look.colour,
        }
    }

    /// The game's screen as wide and as high as the view makes it
    pub(crate) fn screen(&self) -> Vec2 {
        self.rect.size() / self.scale
    }

    /// The point `x`, `y` of the game's screen
    fn at(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(
            self.rect.left() + x * self.scale,
            self.rect.top() + y * self.scale,
        )
    }

    /// The icon `name` of the list `list`
    pub(crate) fn icon(&mut self, list: &str, name: &str) -> Option<TextureHandle> {
        self.sprites.icon(self.ctx, self.library, list, name)
    }

    /// `icon` added at `x`, `y` in `tint`, the rows from `from` of its height
    /// on, as a share of it, the ones above left out
    pub(crate) fn draw(&self, icon: &TextureHandle, x: f32, y: f32, tint: Color32, from: f32) {
        let size = icon.size_vec2();
        let top = size.y * from;
        let place = Rect::from_min_size(
            self.at(x, y + top),
            Vec2::new(size.x, size.y - top) * self.scale,
        );
        let uv = Rect::from_min_max(Pos2::new(0., from), Pos2::new(1., 1.));
        self.painter.image(icon.id(), place, uv, tint);
    }

    /// `icon` added at `x`, `y` in `tint`, the columns from `from` of its
    /// width on, as a share of it, standing where they would whole, the
    /// ones before left out
    fn draw_right_of(&self, icon: &TextureHandle, x: f32, y: f32, tint: Color32, from: f32) {
        let size = icon.size_vec2();
        let left = size.x * from;
        let place = Rect::from_min_size(
            self.at(x + left, y),
            Vec2::new(size.x - left, size.y) * self.scale,
        );
        let uv = Rect::from_min_max(Pos2::new(from, 0.), Pos2::new(1., 1.));
        self.painter.image(icon.id(), place, uv, tint);
    }

    /// A bar `width` by `height` added at `x`, `y` in `tint` (`FillRGBA`)
    pub(crate) fn bar(&self, x: f32, y: f32, width: f32, height: f32, tint: Color32) {
        let place = Rect::from_min_size(self.at(x, y), Vec2::new(width, height) * self.scale);
        self.painter.rect_filled(place, 0., tint);
    }

    /// `text` added with its right end at `x` and its top at `y`
    fn text_before(&self, x: f32, y: f32, text: String, tint: Color32) {
        let font = egui::FontId::proportional(PICKUP_TEXT_SIZE * self.scale);
        let align = egui::Align2::RIGHT_TOP;
        self.painter.text(self.at(x, y), align, text, font, tint);
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
/// `rect` at `at`, as `look` says, their sprites from `library`
pub fn draw(
    ui: &egui::Ui,
    sprites: &mut HudSprites,
    library: &Library,
    rect: Rect,
    look: HudLook,
    shown: &Shown,
    at: SourceTime,
) {
    let mut p = Painter::new(ui, sprites, library, rect, look);

    // A digit's size sets the row and the room numbers take (`m_iFontHeight`)
    let Some(digit) = p.icon(HUD_LIST, "number_0") else {
        return;
    };
    let digit = digit.size_vec2();
    let Vec2 {
        x: width,
        y: height,
    } = p.screen();
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
    if let Some(flashlight) = &shown.flashlight {
        draw_flashlight(&mut p, flashlight, width);
    }
}

/// What the recording player picked up in `pickups` over the view at `rect`
/// at `at`, as `look` says, their sprites from `library`
pub fn draw_pickups(
    ui: &egui::Ui,
    sprites: &mut HudSprites,
    library: &Library,
    rect: Rect,
    look: HudLook,
    pickups: &[Pickup],
    at: SourceTime,
) {
    let mut p = Painter::new(ui, sprites, library, rect, look);
    let Vec2 {
        x: width,
        y: height,
    } = p.screen();

    for pickup in pickups {
        let left = (pickup.until.0 - at.0).max(0.) as f32;
        let brightness = (left * PICKUP_FADE).min(255.);
        let y = height * PICKUP_LOWEST - PICKUP_STEP * pickup.place as f32;

        match &pickup.shown {
            PickupShown::Ammo {
                weapon,
                icon,
                count,
            } => {
                let tint = scaled(p.colour, brightness);
                let x = width - PICKUP_AMMO_RIGHT;
                let list = format!("sprites/{weapon}.txt");
                if let Some(icon) = p.icon(&list, icon) {
                    p.draw(&icon, x, y, tint, 0.);
                }
                p.text_before(x - PICKUP_COUNT_GAP, y, count.to_string(), tint);
            }
            PickupShown::Weapon { name, empty } => {
                let colour = if *empty { NO_AMMO_COLOUR } else { p.colour };
                let list = format!("sprites/{name}.txt");
                if let Some(icon) = p.icon(&list, "weapon") {
                    let x = width - icon.size_vec2().x;
                    p.draw(&icon, x, y, scaled(colour, brightness), 0.);
                }
            }
            PickupShown::Item(name) => {
                if let Some(icon) = p.icon(HUD_LIST, name) {
                    let x = width - icon.size_vec2().x - PICKUP_ITEM_RIGHT;
                    p.draw(&icon, x, y, scaled(p.colour, brightness), 0.);
                }
            }
        }
    }
}

/// Where the recording player was hit from and what hurts them in
/// `damage` over the view at `rect`, as `look` says, the marks red for
/// `health` of 25 or less
pub fn draw_damage(
    ui: &egui::Ui,
    sprites: &mut HudSprites,
    library: &Library,
    rect: Rect,
    look: HudLook,
    damage: &DamageShown,
    health: Option<i32>,
) {
    let mut p = Painter::new(ui, sprites, library, rect, look);
    let Vec2 {
        x: width,
        y: height,
    } = p.screen();
    let middle = Vec2::new(width / 2., height / 2.);

    let hurt = health.is_some_and(|h| h <= HURT);
    let colour = if hurt { HURT_COLOUR } else { p.colour };
    for &(side, mark) in &damage.marks {
        let frame = side as usize;
        let Some(sprite) = p.sprites.frame(p.ctx, p.library, PAIN_SHEET, frame) else {
            continue;
        };

        // Each side stands its own way from the middle (`DrawPain`)
        let Vec2 { x: w, y: h } = sprite.size_vec2();
        let (x, y) = match side {
            Side::Front => (middle.x - w / 2., middle.y - h * 3.),
            Side::Right => (middle.x + w * 2., middle.y - h / 2.),
            Side::Rear => (middle.x - w / 2., middle.y + h * 2.),
            Side::Left => (middle.x - w * 3., middle.y - h / 2.),
        };
        let brightness = 255. * mark.max(LEAST_MARK_BRIGHTNESS);
        p.draw(&sprite, x, y, scaled(colour, brightness), 0.);
    }

    // Every icon flashes together (`DrawDamage`)
    let tint = scaled(p.colour, damage.flash);
    for &(name, place) in &damage.kinds {
        let Some(icon) = p.icon(HUD_LIST, name) else {
            continue;
        };
        let Vec2 { x: w, y: h } = icon.size_vec2();
        let y = height - h * (KIND_BOTTOM + place as f32);
        p.draw(&icon, w / 8., y, tint, 0.);
    }
}

/// The cross, the health and the bar after it, at the left
fn draw_health(p: &mut Painter, health: &Reading, digit: Vec2, row: f32, at: SourceTime) {
    let mut brightness = reading_brightness(health, at);
    if health.value <= DYING {
        brightness = 255.;
    }
    let colour = if health.value > HURT {
        p.colour
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
    p.bar(x, row, digit.x / 10., digit.y, scaled(p.colour, brightness));
}

/// The suit filled as far as the armour goes and the armour, a fifth of the
/// way across
fn draw_armour(p: &mut Painter, armour: &Reading, row: f32, width: f32, at: SourceTime) {
    let tint = scaled(p.colour, reading_brightness(armour, at));
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
    let tint = scaled(p.colour, brightness);
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

/// The flashlight filled as far as its battery goes, and the beam while
/// it is on, at the top right
fn draw_flashlight(p: &mut Painter, flashlight: &Flashlight, width: f32) {
    let brightness = if flashlight.on { FLASHLIGHT_ON } else { LEAST };
    let colour = if flashlight.battery < FLASHLIGHT_LOW {
        NO_AMMO_COLOUR
    } else {
        p.colour
    };
    let tint = scaled(colour, brightness);
    let (Some(empty), Some(full)) = (
        p.icon(HUD_LIST, "flash_empty"),
        p.icon(HUD_LIST, "flash_full"),
    ) else {
        return;
    };

    // The beam stands right after the casing
    let icon_width = full.size_vec2().x;
    let x = width - icon_width - icon_width / 2.;
    p.draw(&empty, x, FLASHLIGHT_TOP, tint, 0.);
    if flashlight.on
        && let Some(beam) = p.icon(HUD_LIST, "flash_beam")
    {
        p.draw(&beam, width - icon_width / 2., FLASHLIGHT_TOP, tint, 0.);
    }

    // The full casing cut from the left by the battery used, in whole
    // columns as the game cuts it
    let used = battery_used(icon_width, flashlight.battery);
    if used < icon_width {
        p.draw_right_of(&full, x, FLASHLIGHT_TOP, tint, used / icon_width);
    }
}

/// How many of the `width` columns of the full casing the `battery` used
/// leaves out, in whole columns
fn battery_used(width: f32, battery: i32) -> f32 {
    let left = battery.clamp(0, 100) as f32 / 100.;
    (width * (1. - left)).floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flashlight_empties_from_the_left_in_whole_columns() {
        assert_eq!(battery_used(32., 100), 0.);
        assert_eq!(battery_used(32., 50), 16.);
        assert_eq!(battery_used(32., 99), 0.);
        assert_eq!(battery_used(32., 0), 32.);
    }

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
