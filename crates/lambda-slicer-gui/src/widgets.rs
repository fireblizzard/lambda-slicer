//! Widgets drawn the same way everywhere (GUI redesign plan §4.2)
//!
//! Icons are painted as lines on a 16 point grid, so they need no font or
//! image and look the same on every platform. Every icon button names what
//! it does, shown on hover and given to screen readers, and the keyboard
//! focus is outlined in the accent

use std::f32::consts::TAU;
use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, Galley, Popup, Pos2, Rect, Response, Role, Sense, Shape, Stroke,
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
    /// A grid, for the full library table
    Table,
    /// Three sliders, for filters
    Filter,
    Columns,
    /// Arrows for an order, highest first or lowest first
    SortDown,
    SortUp,
    Copy,
    /// A folder, for showing a file in its folder
    Folder,
    /// A bar and a triangle, for back to the start
    SkipStart,
    /// A triangle pointing back, for playing in reverse
    PlayBackward,
    Pause,
    /// Chevrons, for a recorded group back or forward
    StepBack,
    StepForward,
    /// Double chevrons, for ten seconds back or forward
    SkipBack,
    SkipForward,
    /// An eye, for seeing through walls
    Eye,
    Volume,
    Mute,
    /// A landscape in a frame, for the picture settings
    Picture,
    /// An i in a circle, for details
    Info,
    /// A pane on the leading side, with an arrow out of it or into it
    PanelOpen,
    PanelClose,
    /// A triangle with an exclamation mark
    Warning,
    /// A magnifying glass with a plus or a minus
    ZoomIn,
    ZoomOut,
    /// Four corners pointing out, for the whole of something, and pointing
    /// in, for leaving the whole screen
    Fit,
    Shrink,
    /// Scissors, for cutting a clip
    Scissors,
    /// Chevrons up and down, for the item before or after
    Previous,
    Next,
    /// An outlined triangle, for playing something once
    Play,
    /// A plus, for adding
    Plus,
    /// An arrow down onto a line, for exporting
    Download,
    /// Two arrows chasing each other round, for looping
    Repeat,
    /// A bin, for removing
    Trash,
    /// A circle with a cross through it, for taking the playhead's time or
    /// keeping to it
    Crosshair,
}

/// The width of an icon's lines
const ICON_LINE: f32 = 1.5;

/// How much lighter the accent turns under the pointer or the keyboard
const ACCENT_LIT: f32 = 0.1;

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
    paint_icon_facing(painter, rect, icon, color, false);
}

/// Paint `icon` inside `rect`, turned left for right when `mirror`, as a
/// pane's icon is in a right-to-left interface. Icons whose direction means
/// time, such as play, are never mirrored
pub fn paint_icon_facing(
    painter: &egui::Painter,
    rect: Rect,
    icon: Icon,
    color: Color32,
    mirror: bool,
) {
    let stroke = Stroke::new(ICON_LINE, color);

    // A point of the 16 point grid the icons are drawn on
    let unit = rect.width().min(rect.height()) / 16.;
    let origin = rect.center() - Vec2::splat(8. * unit);
    let p = |x: f32, y: f32| {
        let x = if mirror { 16. - x } else { x };
        origin + Vec2::new(x, y) * unit
    };

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
        Icon::Table => {
            let frame = Rect::from_min_max(p(2., 3.), p(14., 13.));
            painter.rect_stroke(frame, 1., stroke, StrokeKind::Middle);

            // A heading row and two columns
            painter.line_segment([p(2., 6.5), p(14., 6.5)], stroke);
            painter.line_segment([p(2., 10.), p(14., 10.)], stroke);
            painter.line_segment([p(6., 3.), p(6., 13.)], stroke);
        }
        Icon::Filter => {
            // Three sliders, each knob at a different place
            for (y, knob) in [(4., 10.), (8., 5.), (12., 11.)] {
                painter.line_segment([p(2., y), p(14., y)], stroke);
                painter.circle_filled(p(knob, y), 2. * unit, color);
            }
        }
        Icon::Columns => {
            let frame = Rect::from_min_max(p(2., 3.), p(14., 13.));
            painter.rect_stroke(frame, 1., stroke, StrokeKind::Middle);
            painter.line_segment([p(6., 3.), p(6., 13.)], stroke);
            painter.line_segment([p(10., 3.), p(10., 13.)], stroke);
        }
        Icon::SortDown => {
            painter.line_segment([p(8., 3.), p(8., 13.)], stroke);
            painter.add(Shape::line(vec![p(4., 9.), p(8., 13.), p(12., 9.)], stroke));
        }
        Icon::SortUp => {
            painter.line_segment([p(8., 13.), p(8., 3.)], stroke);
            painter.add(Shape::line(vec![p(4., 7.), p(8., 3.), p(12., 7.)], stroke));
        }
        Icon::Copy => {
            // A sheet in front of another, the one behind showing at the
            // top right
            let front = Rect::from_min_max(p(2.5, 5.5), p(10.5, 13.5));
            painter.rect_stroke(front, 1., stroke, StrokeKind::Middle);

            let behind = vec![
                p(5.5, 5.5),
                p(5.5, 2.5),
                p(13.5, 2.5),
                p(13.5, 10.5),
                p(10.5, 10.5),
            ];
            painter.add(Shape::line(behind, stroke));
        }
        Icon::Folder => {
            let folder = vec![
                p(1.5, 3.),
                p(6., 3.),
                p(7.5, 4.5),
                p(14.5, 4.5),
                p(14.5, 13.5),
                p(1.5, 13.5),
            ];
            painter.add(Shape::closed_line(folder, stroke));
        }
        Icon::SkipStart => {
            painter.line_segment([p(4., 3.), p(4., 13.)], stroke);
            let back = vec![p(12.5, 3.), p(12.5, 13.), p(6., 8.)];
            painter.add(Shape::closed_line(back, stroke));
        }
        Icon::PlayBackward => {
            let back = vec![p(12.5, 3.), p(12.5, 13.), p(3.5, 8.)];
            painter.add(Shape::closed_line(back, stroke));
        }
        Icon::Pause => {
            let bar = Stroke::new(2. * unit, color);
            painter.line_segment([p(5.5, 3.5), p(5.5, 12.5)], bar);
            painter.line_segment([p(10.5, 3.5), p(10.5, 12.5)], bar);
        }
        Icon::StepBack => {
            painter.add(Shape::line(
                vec![p(10., 3.), p(5., 8.), p(10., 13.)],
                stroke,
            ));
        }
        Icon::StepForward => {
            painter.add(Shape::line(vec![p(6., 3.), p(11., 8.), p(6., 13.)], stroke));
        }
        Icon::SkipBack => {
            for x in [3., 8.5] {
                let chevron = vec![p(x + 5., 3.), p(x, 8.), p(x + 5., 13.)];
                painter.add(Shape::line(chevron, stroke));
            }
        }
        Icon::SkipForward => {
            for x in [3., 8.5] {
                let chevron = vec![p(x, 3.), p(x + 5., 8.), p(x, 13.)];
                painter.add(Shape::line(chevron, stroke));
            }
        }
        Icon::Eye => {
            // Two curved lids meeting at the corners, around the pupil
            let steps = 16;
            let mut upper = Vec::with_capacity(steps + 1);
            let mut lower = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let lift = 4.5 * (t * TAU / 2.).sin();
                upper.push(p(1.5 + 13. * t, 8. - lift));
                lower.push(p(1.5 + 13. * t, 8. + lift));
            }
            painter.add(Shape::line(upper, stroke));
            painter.add(Shape::line(lower, stroke));
            painter.circle_stroke(p(8., 8.), 2. * unit, stroke);
        }
        Icon::Volume | Icon::Mute => {
            let speaker = vec![
                p(2., 6.),
                p(5., 6.),
                p(9., 3.),
                p(9., 13.),
                p(5., 10.),
                p(2., 10.),
            ];
            painter.add(Shape::closed_line(speaker, stroke));

            // Sound coming out, or a cross where it would
            if icon == Icon::Volume {
                let center = p(9., 8.);
                let wave = |radius: f32| arc(center, radius * unit, -0.12 * TAU, 0.12 * TAU);
                painter.add(Shape::line(wave(3.), stroke));
                painter.add(Shape::line(wave(5.5), stroke));
            } else {
                painter.line_segment([p(11., 6.), p(15., 10.)], stroke);
                painter.line_segment([p(15., 6.), p(11., 10.)], stroke);
            }
        }
        Icon::Picture => {
            let frame = Rect::from_min_max(p(2., 3.), p(14., 13.));
            painter.rect_stroke(frame, 1., stroke, StrokeKind::Middle);

            // Hills under a sun
            let hills = vec![p(2., 12.), p(6., 8.), p(9., 11.), p(11., 9.), p(14., 12.)];
            painter.add(Shape::line(hills, stroke));
            painter.circle_filled(p(11., 6.), 1.3 * unit, color);
        }
        Icon::Info => {
            painter.circle_stroke(p(8., 8.), 6.5 * unit, stroke);
            painter.circle_filled(p(8., 5.), 1. * unit, color);
            painter.line_segment([p(8., 7.5), p(8., 11.5)], stroke);
        }
        Icon::PanelOpen | Icon::PanelClose => {
            let frame = Rect::from_min_max(p(2., 3.), p(14., 13.));
            painter.rect_stroke(frame, 1., stroke, StrokeKind::Middle);
            painter.line_segment([p(6., 3.), p(6., 13.)], stroke);

            // Out of the pane to open it, into it to close it
            let arrow = if icon == Icon::PanelOpen {
                vec![p(9., 6.), p(11., 8.), p(9., 10.)]
            } else {
                vec![p(11., 6.), p(9., 8.), p(11., 10.)]
            };
            painter.add(Shape::line(arrow, stroke));
        }
        Icon::Warning => {
            let triangle = vec![p(8., 1.5), p(15., 14.), p(1., 14.)];
            painter.add(Shape::closed_line(triangle, stroke));
            painter.line_segment([p(8., 6.), p(8., 10.)], stroke);
            painter.circle_filled(p(8., 12.), 0.9 * unit, color);
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            let lens = p(7., 7.);
            painter.circle_stroke(lens, 4.5 * unit, stroke);
            painter.line_segment([p(10.3, 10.3), p(14., 14.)], stroke);

            painter.line_segment([p(5., 7.), p(9., 7.)], stroke);
            if icon == Icon::ZoomIn {
                painter.line_segment([p(7., 5.), p(7., 9.)], stroke);
            }
        }
        Icon::Fit => {
            // Each corner's two arms, from its tip
            let corners = [
                (p(2.5, 2.5), Vec2::new(1., 1.)),
                (p(13.5, 2.5), Vec2::new(-1., 1.)),
                (p(13.5, 13.5), Vec2::new(-1., -1.)),
                (p(2.5, 13.5), Vec2::new(1., -1.)),
            ];
            for (tip, inward) in corners {
                let arm = 3.5 * unit;
                let across = tip + Vec2::new(inward.x * arm, 0.);
                let down = tip + Vec2::new(0., inward.y * arm);
                painter.add(Shape::line(vec![across, tip, down], stroke));
            }
        }
        Icon::Shrink => {
            // Each corner's two arms, from its bend inside the icon out to
            // its edges
            let corners = [
                (p(6., 6.), Vec2::new(-1., -1.)),
                (p(10., 6.), Vec2::new(1., -1.)),
                (p(10., 10.), Vec2::new(1., 1.)),
                (p(6., 10.), Vec2::new(-1., 1.)),
            ];
            for (bend, outward) in corners {
                let arm = 3.5 * unit;
                let across = bend + Vec2::new(outward.x * arm, 0.);
                let down = bend + Vec2::new(0., outward.y * arm);
                painter.add(Shape::line(vec![across, bend, down], stroke));
            }
        }
        Icon::Scissors => {
            // Two rings for the fingers and the blades crossing above them
            painter.circle_stroke(p(4.5, 12.), 2.2 * unit, stroke);
            painter.circle_stroke(p(11.5, 12.), 2.2 * unit, stroke);
            painter.line_segment([p(6., 10.3), p(12.5, 2.)], stroke);
            painter.line_segment([p(10., 10.3), p(3.5, 2.)], stroke);
        }
        Icon::Previous => {
            painter.add(Shape::line(
                vec![p(3.5, 10.5), p(8., 6.), p(12.5, 10.5)],
                stroke,
            ));
        }
        Icon::Next => {
            painter.add(Shape::line(
                vec![p(3.5, 5.5), p(8., 10.), p(12.5, 5.5)],
                stroke,
            ));
        }
        Icon::Play => {
            let forward = vec![p(4.5, 3.), p(13., 8.), p(4.5, 13.)];
            painter.add(Shape::closed_line(forward, stroke));
        }
        Icon::Plus => {
            painter.line_segment([p(8., 3.), p(8., 13.)], stroke);
            painter.line_segment([p(3., 8.), p(13., 8.)], stroke);
        }
        Icon::Download => {
            painter.line_segment([p(8., 2.), p(8., 10.)], stroke);
            painter.add(Shape::line(
                vec![p(4.5, 6.5), p(8., 10.), p(11.5, 6.5)],
                stroke,
            ));
            painter.line_segment([p(3., 13.5), p(13., 13.5)], stroke);
        }
        Icon::Repeat => {
            // Two rounded arms of a loop, each ending in an arrowhead
            let top = vec![p(3., 9.), p(3., 5.), p(13., 5.)];
            painter.add(Shape::line(top, stroke));
            painter.add(Shape::line(
                vec![p(11., 3.), p(13., 5.), p(11., 7.)],
                stroke,
            ));

            let bottom = vec![p(13., 7.), p(13., 11.), p(3., 11.)];
            painter.add(Shape::line(bottom, stroke));
            painter.add(Shape::line(vec![p(5., 9.), p(3., 11.), p(5., 13.)], stroke));
        }
        Icon::Trash => {
            painter.line_segment([p(2.5, 4.), p(13.5, 4.)], stroke);
            let lid = vec![p(6., 4.), p(6., 2.), p(10., 2.), p(10., 4.)];
            painter.add(Shape::line(lid, stroke));

            let bin = vec![p(4., 4.), p(5., 14.), p(11., 14.), p(12., 4.)];
            painter.add(Shape::line(bin, stroke));
        }
        Icon::Crosshair => {
            painter.circle_stroke(p(8., 8.), 5. * unit, stroke);
            painter.line_segment([p(8., 1.), p(8., 5.)], stroke);
            painter.line_segment([p(8., 11.), p(8., 15.)], stroke);
            painter.line_segment([p(1., 8.), p(5., 8.)], stroke);
            painter.line_segment([p(11., 8.), p(15., 8.)], stroke);
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
    mirror: bool,
}

impl<'a> IconButton<'a> {
    pub fn new(icon: Icon, label: &'a str) -> Self {
        Self {
            icon,
            label,
            selected: false,
            mirror: false,
        }
    }

    /// Shown as a toggle that is on when `selected`
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Its icon turned left for right, as a pane's is in a right-to-left
    /// interface
    pub fn mirrored(mut self, mirror: bool) -> Self {
        self.mirror = mirror;
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
        paint_icon_facing(ui.painter(), glyph, self.icon, color, self.mirror);

        focus_outline(ui, &response, rect);
        response
    }
}

/// The main play button, round and filled in the accent, showing pause
/// while playing. `label` says what a click does
pub struct PlayButton<'a> {
    playing: bool,
    label: &'a str,
}

impl<'a> PlayButton<'a> {
    pub fn new(playing: bool, label: &'a str) -> Self {
        Self { playing, label }
    }
}

impl Widget for PlayButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(theme::PLAY_BUTTON);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let enabled = ui.is_enabled();
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, self.label));
        let response = response.on_hover_text(self.label);

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let lit = response.hovered() || response.has_focus();
        let fill = if lit {
            theme::ACCENT.lerp_to_gamma(Color32::WHITE, ACCENT_LIT)
        } else {
            theme::ACCENT
        };
        let painter = ui.painter();
        let center = rect.center();
        painter.circle_filled(center, rect.width() / 2., fill);

        // A filled triangle a little right of the middle, as it looks
        // centred there, or the pause bars
        let glyph = Rect::from_center_size(center, Vec2::splat(theme::ICON_SIZE));
        if self.playing {
            paint_icon(painter, glyph, Icon::Pause, theme::ACCENT_FOREGROUND);
        } else {
            let unit = glyph.width() / 16.;
            let at = |x: f32, y: f32| glyph.min + Vec2::new(x, y) * unit;
            let triangle = vec![at(5., 3.), at(13.5, 8.), at(5., 13.)];
            painter.add(Shape::convex_polygon(
                triangle,
                theme::ACCENT_FOREGROUND,
                Stroke::NONE,
            ));
        }

        if response.has_focus() {
            let outline = Stroke::new(1., theme::FOREGROUND);
            painter.circle_stroke(center, rect.width() / 2. + 1., outline);
        }

        response
    }
}

/// A small heading over a group of controls
pub fn section_title(ui: &mut Ui, title: &str, rtl: bool) -> Response {
    let look = text::Look::Color(theme::MUTED);
    let title = text::styled_text(ui.ctx(), title, rtl, look, &TextStyle::Small);
    ui.label(title)
}

/// A count in the accent, such as how many filters are on, beside what it
/// counts
pub fn badge(ui: &mut Ui, count: usize) -> Response {
    let text = egui::RichText::new(count.to_string())
        .text_style(TextStyle::Small)
        .color(theme::BACKGROUND);
    let frame = egui::Frame::new()
        .fill(theme::ACCENT)
        .corner_radius(CornerRadius::same(theme::RADIUS_SMALL))
        .inner_margin(egui::Margin::symmetric(4, 0));

    frame.show(ui, |ui| ui.label(text)).response
}

/// The room around a tag's text, as around the badges of events
const TAG_PADDING: Vec2 = Vec2::new(4., 1.);

/// A tag's text in its small style
pub fn tag_galley(ui: &Ui, text: &str) -> Arc<Galley> {
    let text = WidgetText::from(egui::RichText::new(text));
    text.into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Small,
    )
}

/// The room a tag takes, its text and the chip around it
pub fn tag_size(galley: &Galley) -> Vec2 {
    galley.size() + TAG_PADDING * 2.
}

/// A quiet tag at `place`, muted text on a raised chip
pub fn paint_tag(painter: &egui::Painter, place: Rect, galley: Arc<Galley>) {
    painter.rect_filled(place, theme::RADIUS_SMALL, theme::RAISED);
    let at = place.center() - galley.size() / 2.;
    painter.galley(at, galley, theme::MUTED);
}

/// A quiet tag in a row, such as the one marking a clip
pub fn tag(ui: &mut Ui, text: &str) -> Response {
    let galley = tag_galley(ui, text);
    let (place, response) = ui.allocate_exact_size(tag_size(&galley), Sense::hover());
    paint_tag(ui.painter(), place, galley);
    response
}

/// The icon of a button with text, and the gap between it and the text
const BUTTON_ICON: f32 = 14.;
const BUTTON_ICON_GAP: f32 = 6.;

/// How a button with text stands out
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// A raised button
    Default,
    /// Filled in the accent, for the one action a place is for
    Primary,
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
            Variant::Primary if lit => (
                theme::ACCENT.lerp_to_gamma(Color32::WHITE, ACCENT_LIT),
                theme::ACCENT_FOREGROUND,
            ),
            Variant::Primary => (theme::ACCENT, theme::ACCENT_FOREGROUND),
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

    paint_status(ui.painter(), rect.center(), state);
    response.on_hover_text(label)
}

/// Paint the symbol of an analysis state around `center`
pub fn paint_status(painter: &egui::Painter, center: Pos2, state: AnalysisState) {
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
