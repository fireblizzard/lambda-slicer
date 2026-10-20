//! The app's colours, sizes and type, installed once at start (GUI redesign
//! plan §4)
//!
//! Sizes are egui points. Team and error colours keep their meaning, and the
//! weapon colours of the timeline are not part of this palette

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};

/// Behind the library, the lists and the demo view
pub const BACKGROUND: Color32 = Color32::from_rgb(0x17, 0x19, 0x1c);
/// Toolbars, windows, cards and text fields
pub const SURFACE: Color32 = Color32::from_rgb(0x1e, 0x21, 0x25);
/// Buttons and other controls at rest
pub const RAISED: Color32 = Color32::from_rgb(0x27, 0x2b, 0x30);
/// A control under the pointer
pub const HOVER: Color32 = Color32::from_rgb(0x31, 0x36, 0x3c);
/// Thin separators
pub const BORDER: Color32 = Color32::from_rgb(0x2b, 0x2f, 0x35);

/// Text that is read, secondary text, and only decoration such as grid lines
pub const FOREGROUND: Color32 = Color32::from_rgb(0xd8, 0xd5, 0xcf);
pub const MUTED: Color32 = Color32::from_rgb(0x8b, 0x90, 0x97);
pub const FAINT: Color32 = Color32::from_rgb(0x5e, 0x63, 0x6a);

/// Primary actions, what is chosen and the playhead
pub const ACCENT: Color32 = Color32::from_rgb(0xe8, 0x91, 0x3a);
/// The accent behind chosen rows and toggles
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0xe8, 0x91, 0x3a, 0x24);

/// A side known to be blue or red
pub const TEAM_BLUE: Color32 = Color32::from_rgb(0x74, 0xa7, 0xde);
pub const TEAM_RED: Color32 = Color32::from_rgb(0xe0, 0x72, 0x6a);

/// Analysis or an export that went through
pub const SUCCESS: Color32 = Color32::from_rgb(0x86, 0xb4, 0x6e);
/// Partial coverage and warnings that ask for something to be done
pub const WARNING: Color32 = Color32::from_rgb(0xd9, 0xb6, 0x4f);
/// Failures
pub const ERROR: Color32 = Color32::from_rgb(0xd9, 0x67, 0x5f);

/// Side panes such as the library's filters, a shade between the background
/// and the surface so the fields in them stand out. egui's faint fills, such
/// as a grid's alternate rows, use it too
pub const PANE: Color32 = Color32::from_rgb(0x1b, 0x1d, 0x21);

/// Main text, secondary and numeric text, small headings, and the titles of
/// windows and notices
pub const TEXT_BODY: f32 = 13.;
pub const TEXT_SECONDARY: f32 = 12.;
pub const TEXT_SMALL: f32 = 11.;
const TEXT_TITLE: f32 = 15.;

/// The height of a button, field or other control, and the least an icon
/// button takes in both directions however small its glyph
pub const CONTROL_HEIGHT: f32 = 28.;
/// The room inside a button around its text, and the least width egui gives
/// a small control such as a number field
pub const BUTTON_PADDING: Vec2 = Vec2::new(10., 4.);
const CONTROL_MIN_WIDTH: f32 = 40.;
/// The glyph inside an icon button
pub const ICON_SIZE: f32 = 16.;
/// The accent edge on a chosen row's leading side
pub const SELECTED_EDGE: f32 = 2.;

/// The main toolbar and the status strip under everything
pub const TOOLBAR_HEIGHT: f32 = 44.;
pub const STATUS_HEIGHT: f32 = 24.;

/// Space around and between things, smallest first
pub const GAP_XS: f32 = 4.;
pub const GAP_S: f32 = 8.;
pub const GAP_M: f32 = 12.;

/// Corners of small widgets, of controls, and of cards and windows
pub const RADIUS_SMALL: u8 = 4;
pub const RADIUS_CONTROL: u8 = 6;
pub const RADIUS_CARD: u8 = 8;

/// The text style of secondary text, such as a row's second line of
/// metadata
pub fn secondary() -> TextStyle {
    TextStyle::Name("secondary".into())
}

/// The dark theme, whatever the system's preference, for every style egui
/// keeps
pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(apply);
}

/// The app's type, spacing and colours on top of egui's own style
fn apply(style: &mut egui::Style) {
    use FontFamily::{Monospace, Proportional};

    style.text_styles = [
        (TextStyle::Small, FontId::new(TEXT_SMALL, Proportional)),
        (TextStyle::Body, FontId::new(TEXT_BODY, Proportional)),
        (TextStyle::Button, FontId::new(TEXT_BODY, Proportional)),
        (TextStyle::Heading, FontId::new(TEXT_TITLE, Proportional)),
        (TextStyle::Monospace, FontId::new(TEXT_SECONDARY, Monospace)),
        (secondary(), FontId::new(TEXT_SECONDARY, Proportional)),
    ]
    .into();

    let spacing = &mut style.spacing;
    spacing.item_spacing = Vec2::new(GAP_S, GAP_XS);
    spacing.button_padding = BUTTON_PADDING;
    spacing.interact_size = Vec2::new(CONTROL_MIN_WIDTH, CONTROL_HEIGHT);
    spacing.window_margin = egui::Margin::same(GAP_M as i8);
    spacing.menu_margin = egui::Margin::same(GAP_S as i8);

    let visuals = &mut style.visuals;
    visuals.dark_mode = true;
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SURFACE;
    visuals.window_stroke = Stroke::new(1., BORDER);
    visuals.window_corner_radius = CornerRadius::same(RADIUS_CARD);
    visuals.menu_corner_radius = CornerRadius::same(RADIUS_CONTROL);
    visuals.extreme_bg_color = SURFACE;
    visuals.text_edit_bg_color = Some(SURFACE);
    visuals.faint_bg_color = PANE;
    visuals.code_bg_color = RAISED;
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = WARNING;
    visuals.error_fg_color = ERROR;
    visuals.weak_text_color = Some(MUTED);
    visuals.striped = false;

    // Chosen rows, toggles and selected text in the soft accent, with the
    // text and a text field's focus outline in the accent itself
    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::new(1., ACCENT);
    visuals.text_cursor.stroke = Stroke::new(2., ACCENT);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = SURFACE;
    widgets.noninteractive.weak_bg_fill = SURFACE;
    widgets.noninteractive.bg_stroke = Stroke::new(1., BORDER);
    widgets.noninteractive.fg_stroke = Stroke::new(1., FOREGROUND);
    widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_SMALL);

    widgets.inactive.bg_fill = RAISED;
    widgets.inactive.weak_bg_fill = RAISED;
    widgets.inactive.bg_stroke = Stroke::NONE;
    widgets.inactive.fg_stroke = Stroke::new(1., FOREGROUND);
    widgets.inactive.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    widgets.hovered.bg_fill = HOVER;
    widgets.hovered.weak_bg_fill = HOVER;
    widgets.hovered.bg_stroke = Stroke::NONE;
    widgets.hovered.fg_stroke = Stroke::new(1.5, FOREGROUND);
    widgets.hovered.corner_radius = CornerRadius::same(RADIUS_CONTROL);
    widgets.hovered.expansion = 0.;

    // egui draws a focused widget in its active look, so the accent outline
    // is what shows where the keyboard is
    widgets.active.bg_fill = HOVER;
    widgets.active.weak_bg_fill = HOVER;
    widgets.active.bg_stroke = Stroke::new(1., ACCENT);
    widgets.active.fg_stroke = Stroke::new(1.5, FOREGROUND);
    widgets.active.corner_radius = CornerRadius::same(RADIUS_CONTROL);
    widgets.active.expansion = 0.;

    widgets.open.bg_fill = RAISED;
    widgets.open.weak_bg_fill = RAISED;
    widgets.open.bg_stroke = Stroke::new(1., BORDER);
    widgets.open.fg_stroke = Stroke::new(1., FOREGROUND);
    widgets.open.corner_radius = CornerRadius::same(RADIUS_CONTROL);
}
