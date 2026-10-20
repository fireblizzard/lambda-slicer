//! Widgets drawn the same way everywhere (GUI redesign plan §4.2)
//!
//! Icons are painted as lines on a 16 point grid, so they need no font or
//! image and look the same on every platform. Every icon button names what
//! it does, shown on hover and given to screen readers, and the keyboard
//! focus is outlined in the accent

use std::f32::consts::TAU;

use eframe::egui::{
    self, Color32, CornerRadius, Popup, Pos2, Rect, Response, Role, Sense, Shape, Stroke,
    StrokeKind, TextStyle, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText,
};

use crate::text;
use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    /// A page, for opening demos
    Open,
    AddFolder,
    /// A circling arrow, for scanning again
    Rescan,
    Close,
    Settings,
    Help,
}

/// The width of an icon's lines
const ICON_LINE: f32 = 1.5;

/// Points along a circle of `radius` around `center`, from `from` to `to`
/// radians, clockwise on screen as y grows downwards
fn arc(center: Pos2, radius: f32, from: f32, to: f32) -> Vec<Pos2> {
    let steps = 24;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let angle = from + (to - from) * i as f32 / steps as f32;
        points.push(center + radius * Vec2::angled(angle));
    }
    points
}

/// Paint `icon` inside `rect` in lines of `color`
pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let stroke = Stroke::new(ICON_LINE, color);

    // A point of the 16 point grid the icons are drawn on
    let unit = rect.width().min(rect.height()) / 16.;
    let origin = rect.center() - Vec2::splat(8. * unit);
    let p = |x: f32, y: f32| origin + Vec2::new(x, y) * unit;

    match icon {
        Icon::Open => {
            // A page with its top right corner cut off
            let page = vec![
                p(4., 1.5),
                p(9.5, 1.5),
                p(12.5, 4.5),
                p(12.5, 14.5),
                p(4., 14.5),
            ];
            painter.add(Shape::closed_line(page, stroke));

            // The corner folded over onto the page
            let fold = vec![p(9.5, 1.5), p(9.5, 4.5), p(12.5, 4.5)];
            painter.add(Shape::line(fold, stroke));
        }
        Icon::AddFolder => {
            let folder = vec![
                p(1.5, 3.),
                p(6., 3.),
                p(7.5, 4.5),
                p(14.5, 4.5),
                p(14.5, 13.5),
                p(1.5, 13.5),
            ];
            painter.add(Shape::closed_line(folder, stroke));

            // A plus inside the folder
            painter.line_segment([p(8., 6.5), p(8., 11.5)], stroke);
            painter.line_segment([p(5.5, 9.), p(10.5, 9.)], stroke);
        }
        Icon::Rescan => {
            // Most of a circle, open at the top right, with its arrowhead
            let center = p(8., 8.);
            let radius = 5.5 * unit;
            let start = -0.15 * TAU;
            let end = start + 0.8 * TAU;
            painter.add(Shape::line(arc(center, radius, start, end), stroke));

            // The arrowhead points back along the circle from its open end
            let tip = center + radius * Vec2::angled(start);
            let outward = Vec2::angled(start);
            let along = Vec2::angled(start - TAU / 4.);
            let head = vec![
                tip + along * 2.5 * unit,
                tip + outward * 2.2 * unit - along * unit,
                tip - outward * 2.2 * unit - along * unit,
            ];
            painter.add(Shape::convex_polygon(head, color, Stroke::NONE));
        }
        Icon::Close => {
            painter.line_segment([p(4., 4.), p(12., 12.)], stroke);
            painter.line_segment([p(12., 4.), p(4., 12.)], stroke);
        }
        Icon::Settings => {
            // A ring around a hole, with six square teeth
            let center = p(8., 8.);
            painter.circle_stroke(center, 4.5 * unit, stroke);
            painter.circle_stroke(center, 1.8 * unit, stroke);

            for tooth in 0..6 {
                let direction = Vec2::angled(tooth as f32 * TAU / 6.);
                let inner = center + direction * 4.8 * unit;
                let outer = center + direction * 6.8 * unit;
                painter.line_segment([inner, outer], Stroke::new(2.4 * unit, color));
            }
        }
        Icon::Help => {
            let center = p(8., 8.);
            painter.circle_stroke(center, 6.5 * unit, stroke);

            // The question mark of the font every platform gets with egui
            let font = egui::FontId::proportional(10. * unit);
            painter.text(center, egui::Align2::CENTER_CENTER, "?", font, color);
        }
    }
}

/// Outline `rect` in the accent while it has the keyboard's focus
fn focus_outline(ui: &Ui, response: &Response, rect: Rect) {
    if !response.has_focus() {
        return;
    }

    let radius = CornerRadius::same(theme::RADIUS_CONTROL);
    let stroke = Stroke::new(1., theme::ACCENT);
    ui.painter()
        .rect_stroke(rect, radius, stroke, StrokeKind::Inside);
}

/// A button showing only an icon, as large as any control however small
/// its glyph. `label` says what it does, on hover and to screen readers.
/// A selected one is a toggle that is on
pub struct IconButton<'a> {
    icon: Icon,
    label: &'a str,
    selected: bool,
}

impl<'a> IconButton<'a> {
    pub fn new(icon: Icon, label: &'a str) -> Self {
        Self {
            icon,
            label,
            selected: false,
        }
    }

    /// Shown as a toggle that is on when `selected`
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(theme::CONTROL_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let enabled = ui.is_enabled();
        response
            .widget_info(|| WidgetInfo::selected(Role::Button, enabled, self.selected, self.label));
        let response = response.on_hover_text(self.label);

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let (fill, color) = if self.selected {
            (theme::ACCENT_SOFT, theme::ACCENT)
        } else if response.hovered() || response.has_focus() {
            (theme::RAISED, theme::FOREGROUND)
        } else {
            (Color32::TRANSPARENT, theme::MUTED)
        };
        let radius = CornerRadius::same(theme::RADIUS_CONTROL);
        ui.painter().rect_filled(rect, radius, fill);

        let glyph = Rect::from_center_size(rect.center(), Vec2::splat(theme::ICON_SIZE));
        paint_icon(ui.painter(), glyph, self.icon, color);

        focus_outline(ui, &response, rect);
        response
    }
}

/// The icon of a button with text, and the gap between it and the text
const BUTTON_ICON: f32 = 14.;
const BUTTON_ICON_GAP: f32 = 6.;

/// How a button with text stands out
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// A raised button
    Default,
    /// No fill until hovered, for toolbars
    Ghost,
}

/// A button with text and an optional icon before it, which comes on the
/// right in a right-to-left interface
pub struct Button {
    icon: Option<Icon>,
    text: WidgetText,
    variant: Variant,
    rtl: bool,
}

impl Button {
    pub fn new(icon: Option<Icon>, text: WidgetText, variant: Variant, rtl: bool) -> Self {
        Self {
            icon,
            text,
            variant,
            rtl,
        }
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let padding = theme::BUTTON_PADDING;
        let icon_room = match self.icon {
            Some(_) => BUTTON_ICON + BUTTON_ICON_GAP,
            None => 0.,
        };
        let galley = self.text.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Button,
        );
        let width = padding.x * 2. + icon_room + galley.size().x;
        let height = theme::CONTROL_HEIGHT.max(galley.size().y + padding.y * 2.);

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, galley.text()));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let lit = response.hovered() || response.has_focus();
        let (fill, color) = match self.variant {
            Variant::Default if lit => (theme::HOVER, theme::FOREGROUND),
            Variant::Default => (theme::RAISED, theme::FOREGROUND),
            Variant::Ghost if lit => (theme::RAISED, theme::FOREGROUND),
            Variant::Ghost => (Color32::TRANSPARENT, theme::MUTED),
        };
        let radius = CornerRadius::same(theme::RADIUS_CONTROL);
        ui.painter().rect_filled(rect, radius, fill);

        // The icon leads in the interface's direction
        let inner = rect.shrink2(Vec2::new(padding.x, 0.));
        let (icon_x, text_x) = if self.rtl {
            (
                inner.max.x - BUTTON_ICON,
                inner.max.x - icon_room - galley.size().x,
            )
        } else {
            (inner.min.x, inner.min.x + icon_room)
        };
        if let Some(icon) = self.icon {
            let top = rect.center().y - BUTTON_ICON / 2.;
            let at = Rect::from_min_size(Pos2::new(icon_x, top), Vec2::splat(BUTTON_ICON));
            paint_icon(ui.painter(), at, icon, color);
        }

        let text_at = Pos2::new(text_x, rect.center().y - galley.size().y / 2.);
        ui.painter().galley(text_at, galley, color);

        focus_outline(ui, &response, rect);
        response
    }
}

/// How far a library file's analysis got, shown as a symbol whose shape as
/// well as colour tells the states apart
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisState {
    Complete,
    /// Analysed through, with no weapon use found
    NoWeaponUse,
    /// Decoding stopped part way, so later events are missing
    Partial,
    Unsupported,
    Failed,
}

impl AnalysisState {
    /// The state of a cached file by its stored status and shot count
    pub fn of(status: &str, shots: Option<i64>) -> Self {
        match status {
            "complete" if shots.unwrap_or(0) == 0 => Self::NoWeaponUse,
            "complete" => Self::Complete,
            "partial" => Self::Partial,
            "unsupported" => Self::Unsupported,
            _ => Self::Failed,
        }
    }

    /// The message naming the state
    pub fn message_id(self) -> &'static str {
        match self {
            Self::Complete => "status-complete",
            Self::NoWeaponUse => "status-no-use",
            Self::Partial => "status-partial",
            Self::Unsupported => "status-unsupported",
            Self::Failed => "status-failed",
        }
    }
}

/// The symbol of an analysis state. `label` names the state on hover and to
/// screen readers
pub fn status_symbol(ui: &mut Ui, state: AnalysisState, label: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(theme::ICON_SIZE), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, label));

    let painter = ui.painter();
    let center = rect.center();
    let radius = 3.5;
    let stroke = |color| Stroke::new(ICON_LINE, color);

    match state {
        AnalysisState::Complete => {
            painter.circle_filled(center, radius, theme::SUCCESS);
        }
        AnalysisState::NoWeaponUse => {
            painter.circle_stroke(center, radius, stroke(theme::SUCCESS));
        }
        AnalysisState::Partial => {
            // Half filled, as half of it is known
            let left_half = arc(center, radius, 0.25 * TAU, 0.75 * TAU);
            painter.add(Shape::convex_polygon(
                left_half,
                theme::WARNING,
                Stroke::NONE,
            ));
            painter.circle_stroke(center, radius, stroke(theme::WARNING));
        }
        AnalysisState::Unsupported => {
            let diamond = vec![
                center + Vec2::new(0., -radius - 0.5),
                center + Vec2::new(radius + 0.5, 0.),
                center + Vec2::new(0., radius + 0.5),
                center + Vec2::new(-radius - 0.5, 0.),
            ];
            painter.add(Shape::closed_line(diamond, stroke(theme::MUTED)));
        }
        AnalysisState::Failed => {
            let triangle = vec![
                center + Vec2::new(0., -radius - 1.),
                center + Vec2::new(radius + 1., radius),
                center + Vec2::new(-radius - 1., radius),
            ];
            painter.add(Shape::convex_polygon(triangle, theme::ERROR, Stroke::NONE));
        }
    }

    response.on_hover_text(label)
}

/// The glyph of a help question mark, a little smaller than other icons as it
/// sits beside text
const HELP_ICON: f32 = 14.;

/// A question mark explaining what is beside it. Hovering shows the
/// explanation, and a click or the keyboard opens it to stay, so it can be
/// read without a pointer too
pub fn help(ui: &mut Ui, hint: &str, rtl: bool) -> Response {
    let size = Vec2::splat(theme::CONTROL_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let enabled = ui.is_enabled();
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, hint));

    let lit = response.hovered() || response.has_focus();
    let color = if lit { theme::MUTED } else { theme::FAINT };
    let glyph = Rect::from_center_size(rect.center(), Vec2::splat(HELP_ICON));
    paint_icon(ui.painter(), glyph, Icon::Help, color);
    focus_outline(ui, &response, rect);

    // The hover text would only repeat the open explanation
    let popup_id = Popup::default_response_id(&response);
    let response = if Popup::is_id_open(ui.ctx(), popup_id) {
        response
    } else {
        response
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text(text::help(ui.ctx(), hint, rtl))
    };

    Popup::from_toggle_button_response(&response).show(|ui| {
        ui.set_max_width(text::HELP_WIDTH);
        text::label(ui, hint, rtl);
    });

    response
}
