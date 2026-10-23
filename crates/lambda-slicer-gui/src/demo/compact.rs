//! The timeline under the transport, with its heading and overview, and the
//! player lanes it opens into
//!
//! The track, the overview and the lanes share one view. The track and the
//! lanes share one hit test, and a press decides what its drag does until
//! the drag is let go

use eframe::egui::{
    self, Color32, CursorIcon, Painter, Pos2, Rangef, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, Ui, Vec2,
};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::weapons::Weapon;
use lambda_slicer_core::timeline::time::DemoTime;

use super::track::{self, Side, Surface, Target, Transform};
use super::{
    Chooser, DemoAction, DemoView, Drag, Edge, KILL_COLOR, MarkerCache, WINDOW_TINT, secs,
    weapon_color,
};
use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::timeline::{Lane, LaneKey, Mark, MarkKind, Model, SpanKind, bins, nearest, tick_step};
use crate::widgets::{self, Icon, IconButton};

/// The track's ruler, density strip, markers and clip strip from top to
/// bottom, and the overview under the track
const RULER_HEIGHT: f32 = 18.;
const DENSITY_HEIGHT: f32 = 12.;
const MARKERS_HEIGHT: f32 = 30.;
const STRIP_HEIGHT: f32 = 16.;
const TRACK_HEIGHT: f32 = RULER_HEIGHT + DENSITY_HEIGHT + MARKERS_HEIGHT + STRIP_HEIGHT;
const OVERVIEW_HEIGHT: f32 = 12.;

/// How much a zoom button zooms, and how much a point of wheel scrolling
const ZOOM_STEP: f32 = 1.6;
const WHEEL_ZOOM: f32 = 0.003;

/// The least width of a marker's bin and of a density bar, in points
const MARKER_BIN: f32 = 3.;
const DENSITY_BAR: f32 = 2.;

/// How close a marker must be to the pointer to be taken, in points
const MARKER_REACH: f32 = 5.;

/// Time labels on the ruler at least this far apart, placed this far right
/// of their tick and down from the top
const TICK_GAP: f32 = 90.;
const TICK_LABEL_INSET: Vec2 = Vec2::new(3., 2.);

/// Views shorter than this show their times to the millisecond
const PRECISE_VIEW: f32 = 10.;

/// A heading this wide has room for the legend beside its title
const LEGEND_ROOM: f32 = 760.;

/// A legend swatch, and the most kill colours it shows
const SWATCH: f32 = 8.;
const SWATCH_RADIUS: f32 = 2.;
const LEGEND_SIDES: usize = 4;

/// Markers hang from the top of their strip, a kill's longer and a
/// headshot's longest, and shots tick near the bottom
const MARK_TOP: f32 = 3.;
const KILL_MARK: f32 = 16.;
const HEADSHOT_MARK: f32 = 24.;
const OTHER_MARK: f32 = 8.;
const SHOT_TICK: Rangef = Rangef { min: 20., max: 26. };
const MARK_WIDTH: f32 = 2.;
const CROWDED_WIDTH: f32 = 3.;

/// A hairline across the timeline, for the playhead, the grid and edges
const HAIRLINE: f32 = 1.;

/// The corners of a marker or a handle
const MARK_RADIUS: f32 = 1.;

/// The outline around the selected event's marker, half its width, its
/// line, its corners and its inset from the strip's edges
const SELECTED_HALF: f32 = 4.;
const SELECTED_STROKE: f32 = 1.5;
const SELECTED_RADIUS: f32 = 2.;
const SELECTED_INSET: f32 = 1.;

/// The least height of a density bar, so any stretch with events shows
const BAR_LEAST: f32 = 1.;

/// The diamond heading the playhead on the track, half its size
const PLAYHEAD_HEAD: f32 = 4.;

/// A chosen clip's handles on its edges, its name's room from the band's
/// edge, and the least room a name is drawn in
const HANDLE: Vec2 = Vec2::new(4., 10.);
const CLIP_LABEL_INSET: f32 = 3.;
const CLIP_LABEL_LEAST: f32 = 24.;

/// The overview's window at its narrowest
const WINDOW_LEAST: f32 = 3.;

/// The most events a chooser offers
const CHOOSER_MOST: usize = 12;

/// A player lane, its label's widest, and the room around a label
const LANE_HEIGHT: f32 = 22.;
const LABEL_WIDTH: f32 = 170.;
const LABEL_PADDING: f32 = 4.;

/// Half the height of a lane's shot, action, kill and span marks, a shot
/// taller by up to `SHOT_GROWTH` as its bin fills to `CROWD_FULL` marks,
/// the radius of a death and the size of a switch
const SHOT_HALF: f32 = 5.;
const SHOT_GROWTH: f32 = 5.;
const CROWD_FULL: f32 = 5.;
const ACTION_HALF: f32 = 3.;
const KILL_HALF: f32 = 6.;
const SPAN_HALF: f32 = 5.;
const DEATH_RADIUS: f32 = 5.;
const SWITCH_SIZE: f32 = 4.;

/// The lines of a lane's shot, action and death marks, a span's corners
/// and least width, how far up a kill's triangle sits, and the charge's
/// swatch in the legend
const SHOT_STROKE: f32 = 2.;
const ACTION_STROKE: f32 = 1.;
const DEATH_STROKE: f32 = 1.5;
const SPAN_RADIUS: f32 = 2.;
const SPAN_LEAST: f32 = 1.;
const KILL_LIFT: f32 = 1.;
const CHARGE_SWATCH: Vec2 = Vec2::new(14., 8.);

/// A legend entry of the lanes, its swatch's size and the gap after it
const LANE_SWATCH: Vec2 = Vec2::new(16., 14.);
const LANE_LEGEND_GAP: f32 = 8.;

/// How strongly a charge and a beam span show their weapon's colour
const CHARGE_ALPHA: f32 = 0.35;
const BEAM_ALPHA: f32 = 0.5;

/// Colours of teams other than red and blue, by [`Side::Other`]
const OTHER_TEAMS: [Color32; track::OTHER_SIDES as usize] = [
    Color32::from_rgb(0x86, 0xb4, 0x6e),
    Color32::from_rgb(0xb0, 0x8c, 0xe0),
    Color32::from_rgb(0x5f, 0xc0, 0xb5),
    Color32::from_rgb(0xd9, 0xb6, 0x4f),
];

/// Density bars on the track and on the overview
const DENSITY_FILL: Color32 = Color32::from_rgba_unmultiplied_const(0xe8, 0x91, 0x3a, 0x40);
const OVERVIEW_FILL: Color32 = Color32::from_rgba_unmultiplied_const(0xe8, 0x91, 0x3a, 0x4d);

/// A clip's band, stronger for the chosen clip, and its edges
const CLIP_FILL: Color32 = Color32::from_rgba_unmultiplied_const(0xe8, 0x91, 0x3a, 0x18);
const CLIP_CHOSEN_FILL: Color32 = Color32::from_rgba_unmultiplied_const(0xe8, 0x91, 0x3a, 0x30);
const CLIP_EDGE: Color32 = Color32::from_rgba_unmultiplied_const(0xe8, 0x91, 0x3a, 0xb0);
const CLIP_EDGE_WIDTH: f32 = 1.;
const CLIP_CHOSEN_EDGE_WIDTH: f32 = 2.;

/// The dashes of the range clipping the selected event would take
const PROPOSAL_DASH: f32 = 6.;
const PROPOSAL_GAP: f32 = 4.;

/// The overview's window over the stretch the track shows
const WINDOW_FILL: Color32 = Color32::from_rgba_unmultiplied_const(0xd8, 0xd5, 0xcf, 0x0d);
const WINDOW_EDGE: Color32 = Color32::from_rgba_unmultiplied_const(0xd8, 0xd5, 0xcf, 0x66);

/// Behind a lane's name over its marks
const LABEL_FILL: Color32 = Color32::from_rgba_unmultiplied_const(0x1b, 0x1d, 0x21, 0xd9);

/// Every other lane's stripe
const LANE_STRIPE: Color32 = Color32::from_rgba_unmultiplied_const(0x1e, 0x21, 0x25, 0x80);

/// A mark of the player lanes, painted alike in the lanes and their legend
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LaneMark {
    Shot,
    Charge,
    Action,
    Kill,
    Death,
    Switch,
}

impl DemoView {
    /// The timeline under the transport with its heading and overview, then
    /// the player lanes while they are open, taking at most `most_lanes` of
    /// height
    pub(super) fn timeline_ui(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        most_lanes: f32,
    ) {
        // Escape gives up a drag before anything acts on the drag
        self.refresh_rows();
        self.cancel_drag(ui);

        self.timeline_heading(ui, catalog, rtl);

        // A recording of no known length has nowhere to show or seek
        let extent = self.extent();
        if track::narrowest(extent).is_none() {
            let size = Vec2::new(ui.available_width(), TRACK_HEIGHT);
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter()
                .rect_filled(rect, theme::RADIUS_SMALL, theme::SURFACE);
            return;
        }

        self.follow_playhead(extent);
        self.view = track::clamp_view(self.view, extent);

        let across = self.track(ui, catalog, rtl, extent);
        ui.add_space(theme::GAP_XS);
        self.overview(ui, extent);

        if self.lanes_open {
            self.lanes(ui, catalog, rtl, most_lanes, across);
        }

        self.chooser_ui(ui, catalog, rtl);
    }

    /// The title and legend, then from the trailing end the buttons to fit
    /// and zoom, the lanes toggle, help and the stretch in view
    fn timeline_heading(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let room = ui.available_width();
        layout::row(ui, rtl, |ui| {
            widgets::section_title(ui, &catalog.text("timeline-title"), rtl);
            if room >= LEGEND_ROOM {
                self.legend_inline(ui, catalog, rtl);
            }

            layout::trailing(ui, rtl, |ui| {
                let fit = catalog.text("timeline-fit-all");
                if ui.add(IconButton::new(Icon::Fit, &fit)).clicked() {
                    self.actions.push(DemoAction::Fit);
                }
                let zoom_in = catalog.text("timeline-zoom-in");
                if ui.add(IconButton::new(Icon::ZoomIn, &zoom_in)).clicked() {
                    self.actions.push(DemoAction::Zoom(1. / ZOOM_STEP));
                }
                let zoom_out = catalog.text("timeline-zoom-out");
                if ui.add(IconButton::new(Icon::ZoomOut, &zoom_out)).clicked() {
                    self.actions.push(DemoAction::Zoom(ZOOM_STEP));
                }

                let lanes = catalog.text("timeline-lanes");
                let lanes = text::widget_text(ui.ctx(), &lanes, rtl, None, Look::Plain);
                let toggle = egui::Button::selectable(self.lanes_open, lanes);
                if ui.add_enabled(self.model.is_some(), toggle).clicked() {
                    self.lanes_open = !self.lanes_open;
                }

                let gestures = catalog.text("timeline-gestures");
                let density = catalog.text("timeline-density");
                widgets::help(ui, &format!("{gestures}\n\n{density}"), rtl);

                let range = egui::RichText::new(range_text(self.view))
                    .monospace()
                    .color(theme::FAINT);
                ui.label(range);
            });
        });
    }

    /// What the track's marks and bands stand for, for what it shows. Kills
    /// show the colours of the sides they have
    fn legend_inline(&self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let mut items: Vec<(Vec<Color32>, &str)> = Vec::new();

        if self.shown.kills {
            let mut colours = Vec::new();
            for mark in &self.marks {
                let colour = side_colour(mark.side);
                if mark.kind == MarkKind::Kill && !colours.contains(&colour) {
                    colours.push(colour);
                }
                if colours.len() == LEGEND_SIDES {
                    break;
                }
            }
            if colours.is_empty() {
                colours.push(side_colour(Side::Unknown));
            }
            items.push((colours, "legend-kill"));
        }
        if self.shown.shots {
            items.push((vec![theme::MUTED], "legend-shot"));
        }
        if !self.clips.is_empty() {
            items.push((vec![CLIP_EDGE], "legend-clip"));
        }

        for (colours, id) in items {
            for colour in colours {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(SWATCH), Sense::hover());
                ui.painter().rect_filled(rect, SWATCH_RADIUS, colour);
            }

            let look = Look::Color(theme::MUTED);
            let label =
                text::styled_text(ui.ctx(), &catalog.text(id), rtl, look, &theme::secondary());
            ui.label(label);
        }
    }

    /// The track, painted in layers from its ruler down to its clip strip,
    /// acting on presses, drags, the wheel and clicks through the shared
    /// hit test. Returns where across the window it shows the view, for the
    /// lanes to share
    fn track(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, extent: (f32, f32)) -> Rangef {
        let size = Vec2::new(ui.available_width(), TRACK_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        // Read out as a slider at the playhead's time
        let label = catalog.text("timeline-title");
        let playhead = self.viewer.as_ref().map(|v| v.time());
        response.widget_info(|| {
            let value = f64::from(playhead.unwrap_or(0.));
            egui::WidgetInfo::slider(true, value, label.as_str())
        });

        let ruler = Rangef::new(rect.min.y, rect.min.y + RULER_HEIGHT);
        let density = Rangef::new(ruler.max, ruler.max + DENSITY_HEIGHT);
        let markers = Rangef::new(density.max, density.max + MARKERS_HEIGHT);
        let strip = Rangef::new(markers.max, rect.max.y);

        // What the pointer is over, worked out before acting on it
        let transform = Transform {
            view: self.view,
            left: rect.min.x,
            width: rect.width(),
        };
        let under = match response.hover_pos() {
            Some(pointer) if markers.contains(pointer.y) => self.marks_under(transform, pointer.x),
            _ => Vec::new(),
        };
        let clips = self.clip_times();
        let surface = Surface {
            transform,
            ruler: Some(ruler),
            markers,
            strip: Some(strip),
            clips: &clips,
            chosen_clip: self.chosen_clip,
            playhead,
        };
        self.surface_input(ui, &response, &surface, &under, extent);

        // Painted with the view the input left
        let transform = Transform {
            view: self.view,
            ..transform
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, theme::RADIUS_SMALL, theme::SURFACE);
        paint_ruler(&painter, ruler, rect.max.y, transform);
        self.paint_density(&painter, density, transform);
        self.paint_bands(&painter, Rangef::new(density.min, rect.max.y), transform);
        self.paint_markers(&painter, markers, transform);
        self.paint_selection(&painter, markers, transform);
        self.paint_playhead(&painter, rect.y_range(), transform, true);
        self.paint_clip_strip(ui, &painter, strip, transform);

        self.track_tooltip(&response, catalog, rtl, &under, density, transform);

        rect.x_range()
    }

    /// The shown events whose markers are within reach of `x`, nearest first
    fn marks_under(&self, transform: Transform, x: f32) -> Vec<usize> {
        let time = transform.time(x);
        let reach = MARKER_REACH * transform.seconds_per_point();

        let mut near: Vec<(f32, usize)> = Vec::new();
        for mark in track::marks_near(&self.marks, time, reach) {
            near.push(((mark.time - time).abs(), mark.event));
        }
        near.sort_by(|a, b| a.0.total_cmp(&b.0));

        near.into_iter().map(|(_, event)| event).collect()
    }

    /// Each clip's start and end in seconds
    fn clip_times(&self) -> Vec<(f32, f32)> {
        self.clips
            .iter()
            .map(|c| (secs(c.start), secs(c.end)))
            .collect()
    }

    /// Act on the pointer over a timeline surface by what a press there
    /// was over. `under` holds the shown events within reach of the
    /// pointer, nearest first
    fn surface_input(
        &mut self,
        ui: &Ui,
        response: &Response,
        surface: &Surface,
        under: &[usize],
        extent: (f32, f32),
    ) {
        if response.drag_started()
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            self.begin_drag(surface, origin);
        }
        if response.dragged()
            && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
        {
            self.continue_drag(surface.transform, pointer.x);
        }
        if response.drag_stopped() {
            self.end_drag();
        }

        // The wheel zooms around the pointer, and any scroll area around the
        // surface is not scrolled too
        let scroll = ui.input(|i| i.smooth_scroll_delta.x + i.smooth_scroll_delta.y);
        if scroll != 0.
            && self.drag.is_none()
            && let Some(pointer) = response.hover_pos()
        {
            let anchor = surface.transform.time(pointer.x);
            let factor = (-scroll * WHEEL_ZOOM).exp();
            self.view = track::zoom(self.view, extent, anchor, factor);
            ui.ctx().input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO);
        }

        // A double click on a mark or a clip is two clicks on it, and only
        // the empty track fits the whole demo
        let clicked_at = response
            .interact_pointer_pos()
            .filter(|_| response.clicked());
        if let Some(at) = clicked_at {
            self.click(surface, at, under, extent);

            let empty = track::hit(surface, at, !under.is_empty()) == Target::Background;
            if response.double_clicked() && empty {
                self.actions.push(DemoAction::Fit);
            }
        }

        set_cursor(ui, response, self.drag, surface, under);
    }

    /// Decide what a drag does by what its press was over. A press anywhere
    /// but on a clip's edge or the scrub zone moves the view along
    fn begin_drag(&mut self, surface: &Surface, origin: Pos2) {
        let target = track::hit(surface, origin, false);
        if let Target::ClipEdge { clip, edge } = target {
            self.commit_typing();
            let Some(c) = self.clips.get(clip) else {
                return;
            };

            let original = match edge {
                Edge::Start => c.start,
                Edge::End => c.end,
            };
            self.chosen_clip = Some(clip);
            self.drag = Some(Drag::ClipEdge {
                clip,
                edge,
                original,
            });
            return;
        }

        if target == Target::Scrub && self.viewer.is_some() {
            self.drag = Some(Drag::Scrub);
            return;
        }

        self.drag = Some(Drag::Pan {
            grab: surface.transform.time(origin.x),
            original: self.view,
            transform: surface.transform,
        });
    }

    /// Carry on the drag under way with the pointer at `x`
    fn continue_drag(&mut self, transform: Transform, x: f32) {
        match self.drag {
            Some(Drag::Scrub) => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.scrub(transform.time(x).max(0.));
                }
            }
            Some(Drag::ClipEdge { clip, edge, .. }) => {
                self.move_clip_edge(clip, edge, transform.time(x));
            }
            Some(Drag::Pan {
                grab,
                original,
                transform: pressed,
            }) => {
                // The moment grabbed stays under the pointer
                let moved = grab - pressed.time(x);
                self.view = track::pan(original, self.extent(), moved);
            }
            _ => {}
        }
    }

    /// Move a clip's edge to `time` in whole milliseconds, as its fields
    /// show it, never past its other edge, and show that moment in the viewer
    fn move_clip_edge(&mut self, clip: usize, edge: Edge, time: f32) {
        let Some(c) = self.clips.get_mut(clip) else {
            return;
        };

        let ms = (f64::from(time.max(0.)) * 1000.).round() as i64;
        let wanted = DemoTime::from_micros(ms * 1000);
        let moved = match edge {
            Edge::Start => {
                c.start = wanted.min(c.end);
                c.start
            }
            Edge::End => {
                c.end = wanted.max(c.start);
                c.end
            }
        };

        if let Some(viewer) = &mut self.viewer {
            viewer.preview(secs(moved));
        }
    }

    /// The drag is let go. A scrub plays on from where it went, and a moved
    /// clip edge is kept
    pub(super) fn end_drag(&mut self) {
        match self.drag.take() {
            Some(Drag::Scrub) => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.end_scrub();
                }
            }
            Some(Drag::ClipEdge { .. }) => self.clips_changed = true,
            _ => {}
        }
    }

    /// Give up the drag under way on Escape, putting back what it moved.
    /// Giving up a scrub is the viewer's
    fn cancel_drag(&mut self, ui: &Ui) {
        let restorable = matches!(
            self.drag,
            Some(Drag::ClipEdge { .. } | Drag::Pan { .. } | Drag::OverviewPan { .. })
        );
        if !restorable {
            return;
        }

        let escape = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if !escape {
            return;
        }

        match self.drag {
            Some(Drag::ClipEdge {
                clip,
                edge,
                original,
            }) => {
                if let Some(c) = self.clips.get_mut(clip) {
                    match edge {
                        Edge::Start => c.start = original,
                        Edge::End => c.end = original,
                    }
                }
                if let Some(viewer) = &mut self.viewer {
                    viewer.preview(secs(original));
                }
            }
            Some(Drag::Pan { original, .. } | Drag::OverviewPan { original, .. }) => {
                self.view = original;
            }
            _ => {}
        }

        self.drag = Some(Drag::Cancelled);
    }

    /// A click on a surface. A marker selects its event, a clip's band
    /// chooses the clip, and anywhere else seeks there
    fn click(&mut self, surface: &Surface, at: Pos2, under: &[usize], extent: (f32, f32)) {
        match track::hit(surface, at, !under.is_empty()) {
            Target::Marker => self.choose_marker(under, at),
            Target::ClipEdge { clip, .. } | Target::ClipBody(clip) => {
                self.actions.push(DemoAction::ChooseClip(clip));
            }
            Target::Scrub | Target::Background => {
                let time = surface.transform.time(at.x).clamp(extent.0, extent.1);
                self.actions.push(DemoAction::Seek(time));
            }
        }
    }

    /// Select the nearest of the events under a click, unless they belong
    /// to different sessions, which can share times, so they are offered to
    /// choose from
    fn choose_marker(&mut self, under: &[usize], at: Pos2) {
        let Some(model) = self.model.clone() else {
            return;
        };
        let Some(&nearest) = under.first() else {
            return;
        };

        let segment = model.events[nearest].segment;
        let one_session = under.iter().all(|&e| model.events[e].segment == segment);
        if one_session {
            self.actions.push(DemoAction::Select {
                event: nearest,
                from_list: false,
            });
            return;
        }

        // Offered in time order, as the events are
        let mut events: Vec<usize> = under.iter().copied().take(CHOOSER_MOST).collect();
        events.sort_unstable();
        self.chooser = Some(Chooser {
            at,
            events,
            fresh: true,
        });
    }

    /// Turn a page of the timeline when playing takes the playhead out of
    /// view, unless a drag holds the view or the playhead was already out of
    /// a view the user had moved away
    fn follow_playhead(&mut self, extent: (f32, f32)) {
        let Some(viewer) = &self.viewer else {
            return;
        };
        let now = viewer.time();
        let backward = viewer.is_backward();
        let playing = viewer.is_playing();

        let inside = |view: (f32, f32)| now >= view.0 && now <= view.1;
        if playing && self.drag.is_none() && self.playhead_in_view && !inside(self.view) {
            self.view = track::page_to(self.view, extent, now, backward);
        }
        self.playhead_in_view = inside(self.view);
    }

    /// The shown events' density, worked out again once they change
    fn density(&mut self) -> Option<&track::Density> {
        let current = matches!(&self.density, Some((version, _)) if *version == self.rows_version);
        if !current {
            let density = track::Density::new(&self.marks, self.extent());
            self.density = Some((self.rows_version, density));
        }

        self.density.as_ref().and_then(|(_, d)| d.as_ref())
    }

    /// Bars of how many shown events fall in each stretch, from the bottom
    /// of their strip
    fn paint_density(&mut self, painter: &Painter, strip: Rangef, transform: Transform) {
        let Some(density) = self.density() else {
            return;
        };

        let level = density.level_for(transform.seconds_per_point(), DENSITY_BAR);
        for bar in density.bars(level, transform.view) {
            let height = (bar.share * strip.span()).max(BAR_LEAST);
            let across = Rangef::new(transform.x(bar.start), transform.x(bar.end));
            let rect = Rect::from_x_y_ranges(across, Rangef::new(strip.max - height, strip.max));
            painter.rect_filled(rect, 0., DENSITY_FILL);
        }
    }

    /// The chosen highlight's stretch, the clips' bands and edges, and the
    /// range clipping the selected event would take as dashed edges, all
    /// across `height`
    fn paint_bands(&self, painter: &Painter, height: Rangef, transform: Transform) {
        let (start, end) = transform.view;
        let shown = |from: f32, to: f32| {
            let left = transform.x(from.max(start));
            let right = transform.x(to.min(end));
            (right > left).then(|| Rect::from_x_y_ranges(Rangef::new(left, right), height))
        };
        let line = |time: f32| {
            let x = transform.x(time);
            [Pos2::new(x, height.min), Pos2::new(x, height.max)]
        };

        let highlight = self
            .window
            .and_then(|i| self.model.as_ref().and_then(|m| m.windows.get(i)));
        if let Some(w) = highlight
            && let Some(shade) = shown(w.start, w.end)
        {
            painter.rect_filled(shade, 0., WINDOW_TINT);
        }

        for (i, c) in self.clips.iter().enumerate() {
            let (from, to) = (secs(c.start), secs(c.end));
            let Some(band) = shown(from, to) else {
                continue;
            };

            let chosen = self.chosen_clip == Some(i);
            let (fill, width) = if chosen {
                (CLIP_CHOSEN_FILL, CLIP_CHOSEN_EDGE_WIDTH)
            } else {
                (CLIP_FILL, CLIP_EDGE_WIDTH)
            };
            painter.rect_filled(band, 0., fill);
            for time in [from, to] {
                if time >= start && time <= end {
                    painter.line_segment(line(time), Stroke::new(width, CLIP_EDGE));
                }
            }
        }

        let Some((from, to)) = self.proposal() else {
            return;
        };
        for time in [from, to] {
            if time >= start && time <= end {
                let stroke = Stroke::new(CLIP_EDGE_WIDTH, CLIP_EDGE);
                let dashes = Shape::dashed_line(&line(time), stroke, PROPOSAL_DASH, PROPOSAL_GAP);
                painter.extend(dashes);
            }
        }
    }

    /// The markers for the view, worked out again once the view, its width
    /// or the shown events change
    fn marker_bins(&mut self, transform: Transform) -> &[track::MarkerBin] {
        let current = self.markers.as_ref().is_some_and(|c| {
            c.view == transform.view
                && c.width == transform.width
                && c.rows_version == self.rows_version
        });
        if !current {
            let width = MARKER_BIN * transform.seconds_per_point();
            let bins = track::marker_bins(&self.marks, transform.view, width);
            self.markers = Some(MarkerCache {
                view: transform.view,
                width: transform.width,
                rows_version: self.rows_version,
                bins,
            });
        }

        self.markers.as_ref().map_or(&[], |c| &c.bins)
    }

    /// The shown events as bars hanging from the top of their strip, a
    /// crowded stretch sharing one bar per kind
    fn paint_markers(&mut self, painter: &Painter, strip: Rangef, transform: Transform) {
        let half_bin = MARKER_BIN * transform.seconds_per_point() / 2.;
        let top = strip.min + MARK_TOP;

        for bin in self.marker_bins(transform) {
            let x = transform.x(bin.start + half_bin);
            let width = if bin.count > 1 {
                CROWDED_WIDTH
            } else {
                MARK_WIDTH
            };

            let (height, colour) = match bin.kind {
                MarkKind::Kill if bin.headshot => {
                    (Rangef::new(top, top + HEADSHOT_MARK), side_colour(bin.side))
                }
                MarkKind::Kill => (Rangef::new(top, top + KILL_MARK), side_colour(bin.side)),
                MarkKind::Shot => {
                    let tick = Rangef::new(strip.min + SHOT_TICK.min, strip.min + SHOT_TICK.max);
                    (tick, theme::MUTED)
                }
                MarkKind::Switch => (Rangef::new(top, top + OTHER_MARK), weapon_color(bin.weapon)),
                MarkKind::Action | MarkKind::Death | MarkKind::Chat | MarkKind::Match => {
                    (Rangef::new(top, top + OTHER_MARK), theme::MUTED)
                }
            };

            let across = Rangef::new(x - width / 2., x + width / 2.);
            painter.rect_filled(Rect::from_x_y_ranges(across, height), MARK_RADIUS, colour);
        }
    }

    /// An outline around the selected event's marker while it is shown and
    /// in view
    fn paint_selection(&self, painter: &Painter, strip: Rangef, transform: Transform) {
        let Some(event) = self.selected else {
            return;
        };
        if !self.shown_events.get(event).copied().unwrap_or(false) {
            return;
        }
        let Some(e) = self.model.as_ref().and_then(|m| m.events.get(event)) else {
            return;
        };
        if e.time < transform.view.0 || e.time > transform.view.1 {
            return;
        }

        let x = transform.x(e.time);
        let across = Rangef::new(x - SELECTED_HALF, x + SELECTED_HALF);
        let outline = Rect::from_x_y_ranges(across, strip.shrink(SELECTED_INSET));
        let stroke = Stroke::new(SELECTED_STROKE, theme::FOREGROUND);
        painter.rect_stroke(outline, SELECTED_RADIUS, stroke, StrokeKind::Inside);
    }

    /// The viewer's playhead across `height`, headed by a diamond on the
    /// track
    fn paint_playhead(&self, painter: &Painter, height: Rangef, transform: Transform, head: bool) {
        let Some(now) = self.viewer.as_ref().map(|v| v.time()) else {
            return;
        };
        if now < transform.view.0 || now > transform.view.1 {
            return;
        }

        let x = transform.x(now);
        let line = [Pos2::new(x, height.min), Pos2::new(x, height.max)];
        painter.line_segment(line, Stroke::new(HAIRLINE, theme::ACCENT));

        if head {
            let diamond = vec![
                Pos2::new(x, height.min),
                Pos2::new(x + PLAYHEAD_HEAD, height.min + PLAYHEAD_HEAD),
                Pos2::new(x, height.min + PLAYHEAD_HEAD * 2.),
                Pos2::new(x - PLAYHEAD_HEAD, height.min + PLAYHEAD_HEAD),
            ];
            painter.add(Shape::convex_polygon(diamond, theme::ACCENT, Stroke::NONE));
        }
    }

    /// Each clip's name in its band and, for the chosen clip, handles on
    /// its edges, in the strip under the markers where they stay easy to
    /// take
    fn paint_clip_strip(&self, ui: &Ui, painter: &Painter, strip: Rangef, transform: Transform) {
        let (start, end) = transform.view;

        for (i, c) in self.clips.iter().enumerate() {
            let (from, to) = (secs(c.start), secs(c.end));
            if to < start || from > end {
                continue;
            }

            if self.chosen_clip == Some(i) {
                for time in [from, to] {
                    if time >= start && time <= end {
                        let centre = Pos2::new(transform.x(time), strip.center());
                        let handle = Rect::from_center_size(centre, HANDLE);
                        painter.rect_filled(handle, MARK_RADIUS, CLIP_EDGE);
                    }
                }
            }

            let left = transform.x(from.max(start));
            let right = transform.x(to.min(end));
            let room = right - left - CLIP_LABEL_INSET * 2.;
            if room < CLIP_LABEL_LEAST {
                continue;
            }

            let name = text::widget_text(
                ui.ctx(),
                &c.name,
                text::starts_rtl(&c.name),
                None,
                Look::Plain,
            );
            let galley = name.into_galley(
                ui,
                Some(egui::TextWrapMode::Truncate),
                room,
                egui::TextStyle::Small,
            );
            let at = Pos2::new(
                left + CLIP_LABEL_INSET,
                strip.center() - galley.size().y / 2.,
            );
            painter.galley(at, galley, theme::ACCENT);
        }
    }

    /// What is under the pointer, shown at once rather than after egui's
    /// usual delay. The nearest marker's event, or over the density strip
    /// how busy that stretch is
    fn track_tooltip(
        &mut self,
        response: &Response,
        catalog: &Catalog,
        rtl: bool,
        under: &[usize],
        density: Rangef,
        transform: Transform,
    ) {
        if self.drag.is_some() {
            return;
        }
        let Some(pointer) = response.hover_pos() else {
            return;
        };

        let tip = match under.first() {
            Some(&event) => Some(self.event_summary(catalog, event, false)),
            None if density.contains(pointer.y) => self.density_tip(catalog, transform, pointer.x),
            None => None,
        };
        let Some(tip) = tip else {
            return;
        };

        let tip = text::widget_text(&response.ctx, &tip, rtl, None, Look::Plain);
        egui::Tooltip::for_widget(response).at_pointer().show(|ui| {
            ui.label(tip);
        });
    }

    /// How many shown events the density bar at `x` stands for, by session
    /// where more than one has events there
    fn density_tip(&mut self, catalog: &Catalog, transform: Transform, x: f32) -> Option<String> {
        let time = transform.time(x);
        let density = self.density()?;
        let level = density.level_for(transform.seconds_per_point(), DENSITY_BAR);
        let (total, sessions) = density.at(level, time);
        if total == 0 {
            return None;
        }

        let count = FluentValue::from(i64::from(total));
        let mut lines = vec![catalog.text_with("timeline-density-count", &[("count", count)])];
        if sessions.len() > 1 {
            for (segment, count) in sessions {
                let session = FluentValue::from(i64::from(segment) + 1);
                let count = FluentValue::from(i64::from(count));
                let args = [("session", session), ("count", count)];
                lines.push(catalog.text_with("timeline-session-count", &args));
            }
        }

        Some(lines.join("\n"))
    }

    /// An event in one line, its time, actor and what happened, led by its
    /// session where sessions need telling apart
    pub(super) fn event_summary(&self, catalog: &Catalog, event: usize, session: bool) -> String {
        let Some(e) = self.model.as_ref().and_then(|m| m.events.get(event)) else {
            return String::new();
        };

        let time = DemoTime::from_f32(e.time)
            .map(|t| t.to_string())
            .unwrap_or_default();
        let actor = e
            .actor
            .as_deref()
            .map_or_else(|| catalog.text("lane-unknown"), text::player_name);
        let what = self.what(catalog, event);

        let mut parts = vec![time, actor, what];
        if session {
            let number = FluentValue::from(i64::from(e.segment) + 1);
            parts.insert(
                0,
                catalog.text_with("event-session", &[("session", number)]),
            );
        }
        parts.join(" · ")
    }

    /// The whole recording in a thin strip, its density and a window over
    /// the stretch the track shows. Dragging moves the window, and a click
    /// outside it centres the view there
    fn overview(&mut self, ui: &mut Ui, extent: (f32, f32)) {
        let size = Vec2::new(ui.available_width(), OVERVIEW_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let whole = Transform {
            view: extent,
            left: rect.min.x,
            width: rect.width(),
        };
        let span = self.view.1 - self.view.0;
        let in_view = |time: f32, view: (f32, f32)| time >= view.0 && time <= view.1;

        // Grabbed inside the window it keeps its place under the pointer, and
        // grabbed outside it the window centres on the pointer
        if response.drag_started()
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            let grabbed = whole.time(origin.x);
            let offset = if in_view(grabbed, self.view) {
                grabbed - self.view.0
            } else {
                span / 2.
            };
            self.drag = Some(Drag::OverviewPan {
                offset,
                original: self.view,
            });
        }
        if response.dragged()
            && let Some(Drag::OverviewPan { offset, .. }) = self.drag
            && let Some(pointer) = ui.input(|i| i.pointer.interact_pos())
        {
            let start = whole.time(pointer.x) - offset;
            self.view = track::clamp_view((start, start + span), extent);
        }
        if response.drag_stopped() {
            self.end_drag();
        }

        if response.clicked()
            && let Some(at) = response.interact_pointer_pos()
        {
            let time = whole.time(at.x);
            if !in_view(time, self.view) {
                self.view = track::centre_on(self.view, extent, time);
            }
        }

        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, theme::RADIUS_SMALL, theme::SURFACE);
        if let Some(density) = self.density() {
            let level = density.level_for(whole.seconds_per_point(), DENSITY_BAR);
            for bar in density.bars(level, extent) {
                let height = (bar.share * rect.height()).max(BAR_LEAST);
                let across = Rangef::new(whole.x(bar.start), whole.x(bar.end));
                let up = Rangef::new(rect.max.y - height, rect.max.y);
                painter.rect_filled(Rect::from_x_y_ranges(across, up), 0., OVERVIEW_FILL);
            }
        }

        let left = whole.x(self.view.0);
        let right = whole.x(self.view.1).max(left + WINDOW_LEAST);
        let window = Rect::from_x_y_ranges(Rangef::new(left, right), rect.y_range());
        painter.rect_filled(window, theme::RADIUS_SMALL, WINDOW_FILL);
        let edge = Stroke::new(HAIRLINE, WINDOW_EDGE);
        painter.rect_stroke(window, theme::RADIUS_SMALL, edge, StrokeKind::Inside);

        self.paint_playhead(&painter, rect.y_range(), whole, false);
    }

    /// The events offered after a click found several sessions under it,
    /// until one is chosen, a click lands elsewhere or Escape is pressed
    fn chooser_ui(&mut self, ui: &Ui, catalog: &Catalog, rtl: bool) {
        let Some(chooser) = self.chooser.clone() else {
            return;
        };

        let mut chosen = None;
        let id = ui.scope_id().with(("event chooser", self.generation));
        let shown = egui::Area::new(id)
            .order(egui::Order::Foreground)
            .fixed_pos(chooser.at)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    widgets::section_title(ui, &catalog.text("timeline-choose"), rtl);
                    for &event in &chooser.events {
                        let line = self.event_summary(catalog, event, true);
                        let line = text::widget_text(ui.ctx(), &line, rtl, None, Look::Plain);
                        if ui.add(egui::Button::selectable(false, line)).clicked() {
                            chosen = Some(event);
                        }
                    }
                });
            });

        if let Some(event) = chosen {
            self.actions.push(DemoAction::Select {
                event,
                from_list: false,
            });
            self.chooser = None;
            return;
        }

        // The click that opened it lands outside it too
        if chooser.fresh {
            if let Some(open) = &mut self.chooser {
                open.fresh = false;
            }
            return;
        }

        let elsewhere = shown.response.clicked_elsewhere();
        let escape = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if elsewhere || escape {
            self.chooser = None;
        }
    }

    /// The player lanes under the timeline sharing its view `across` the
    /// same points, in a scroll area no taller than `most`
    fn lanes(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, most: f32, across: Rangef) {
        let Some(model) = self.model.clone() else {
            return;
        };

        self.lanes_legend(ui, catalog, rtl);
        let height = LANE_HEIGHT * model.lanes.len() as f32;
        egui::ScrollArea::vertical()
            .id_salt(("lanes", self.generation))
            .max_height(height.min(most))
            .show(ui, |ui| {
                self.lanes_surface(ui, catalog, rtl, &model, across)
            });
    }

    /// What the lanes' marks mean
    fn lanes_legend(&self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let items = [
            (LaneMark::Shot, "legend-shot"),
            (LaneMark::Charge, "legend-charge"),
            (LaneMark::Kill, "legend-kill"),
            (LaneMark::Death, "legend-death"),
            (LaneMark::Switch, "legend-switch"),
        ];
        let gauss = weapon_color(Some(Weapon::Gauss));

        layout::wrapping_row(ui, rtl, |ui| {
            for (mark, id) in items {
                // A swatch reads before its label in either direction
                let (rect, _) = ui.allocate_exact_size(LANE_SWATCH, Sense::hover());
                paint_lane_mark(ui.painter(), mark, rect.center(), gauss, 0.);

                let label = catalog.text(id);
                ui.label(text::widget_text(ui.ctx(), &label, rtl, None, Look::Plain));
                ui.add_space(LANE_LEGEND_GAP);
            }
        });
    }

    /// The lanes themselves, one per player, with the track's hit test
    fn lanes_surface(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        model: &Model,
        across: Rangef,
    ) {
        let height = LANE_HEIGHT * model.lanes.len() as f32;
        let (row, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let rect = Rect::from_x_y_ranges(across, row.y_range());
        let response = ui.interact(rect, ui.scope_id().with("lanes"), Sense::click_and_drag());

        // What the pointer is over, worked out before acting on it
        let transform = Transform {
            view: self.view,
            left: across.min,
            width: across.span(),
        };
        let under: Vec<usize> = response
            .hover_pos()
            .and_then(|p| self.lane_mark_under(model, rect, transform, p))
            .into_iter()
            .collect();
        let clips = self.clip_times();
        let surface = Surface {
            transform,
            ruler: None,
            markers: rect.y_range(),
            strip: None,
            clips: &clips,
            chosen_clip: self.chosen_clip,
            playhead: self.viewer.as_ref().map(|v| v.time()),
        };
        self.surface_input(ui, &response, &surface, &under, self.extent());

        // Painted with the view the input left
        let transform = Transform {
            view: self.view,
            ..transform
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0., theme::BACKGROUND);
        self.paint_bands(&painter, rect.y_range(), transform);
        for (i, lane) in model.lanes.iter().enumerate() {
            let top = rect.min.y + LANE_HEIGHT * i as f32;
            let lane_rect =
                Rect::from_x_y_ranges(rect.x_range(), Rangef::new(top, top + LANE_HEIGHT));
            self.paint_lane(ui, &painter, catalog, rtl, lane, i, lane_rect, transform);
        }
        self.paint_playhead(&painter, rect.y_range(), transform, false);

        if self.drag.is_none()
            && let Some(&event) = under.first()
        {
            let tip = self.event_summary(catalog, event, false);
            let tip = text::widget_text(ui.ctx(), &tip, rtl, None, Look::Plain);
            egui::Tooltip::for_widget(&response)
                .at_pointer()
                .show(|ui| {
                    ui.label(tip);
                });
        }
    }

    /// The shown event whose mark in the lane under `pointer` is nearest it
    /// within reach
    fn lane_mark_under(
        &self,
        model: &Model,
        rect: Rect,
        transform: Transform,
        pointer: Pos2,
    ) -> Option<usize> {
        let index = ((pointer.y - rect.min.y) / LANE_HEIGHT).floor();
        if index < 0. {
            return None;
        }
        let lane = model.lanes.get(index as usize)?;

        let time = transform.time(pointer.x);
        let reach = MARKER_REACH * transform.seconds_per_point();
        let shown = |m: &Mark| self.shown_events.get(m.event).copied().unwrap_or(false);
        nearest(lane, time, reach, shown).map(|m| m.event)
    }

    /// One lane's stripe, its charge and beam spans, its marks grouped into
    /// bins at wide zoom, and its name over them at the leading edge
    #[allow(clippy::too_many_arguments)]
    fn paint_lane(
        &self,
        ui: &Ui,
        painter: &Painter,
        catalog: &Catalog,
        rtl: bool,
        lane: &Lane,
        index: usize,
        rect: Rect,
        transform: Transform,
    ) {
        let (start, end) = transform.view;
        let middle = rect.center().y;

        if index % 2 == 1 {
            painter.rect_filled(rect, 0., LANE_STRIPE);
        }

        // Spans show with shots or actions, for the weapons shown
        let spans_shown = self.shown.shots || self.shown.actions;
        for span in &lane.spans {
            let in_view = span.end >= start && span.start <= end;
            if !spans_shown || !in_view || !self.shown.weapon(Some(span.weapon)) {
                continue;
            }

            let left = transform.x(span.start.max(start));
            let right = transform.x(span.end.min(end)).max(left + SPAN_LEAST);
            let across = Rangef::new(left, right);
            let band =
                Rect::from_x_y_ranges(across, Rangef::new(middle - SPAN_HALF, middle + SPAN_HALF));
            let alpha = match span.kind {
                SpanKind::Charge => CHARGE_ALPHA,
                SpanKind::Beam => BEAM_ALPHA,
            };
            let colour = weapon_color(Some(span.weapon)).gamma_multiply(alpha);
            painter.rect_filled(band, SPAN_RADIUS, colour);
        }

        let bin_width = MARKER_BIN * transform.seconds_per_point();
        let shown = |m: &Mark| self.shown_events.get(m.event).copied().unwrap_or(false);
        for bin in bins(lane, start, end, bin_width, shown) {
            let centre = Pos2::new(transform.x(bin.start + bin_width / 2.), middle);
            let crowd = (bin.count as f32).min(CROWD_FULL) / CROWD_FULL;
            let (mark, colour) = match bin.kind {
                MarkKind::Shot => (LaneMark::Shot, weapon_color(bin.weapon)),
                MarkKind::Action | MarkKind::Chat | MarkKind::Match => {
                    (LaneMark::Action, theme::MUTED)
                }
                MarkKind::Kill => (LaneMark::Kill, KILL_COLOR),
                MarkKind::Death => (LaneMark::Death, KILL_COLOR),
                MarkKind::Switch => (LaneMark::Switch, weapon_color(bin.weapon)),
            };

            // A switch sits at the bottom of its lane, under the shots
            let centre = if mark == LaneMark::Switch {
                Pos2::new(centre.x, rect.max.y - SWITCH_SIZE)
            } else {
                centre
            };
            paint_lane_mark(painter, mark, centre, colour, crowd);
        }

        self.paint_lane_label(ui, painter, catalog, rtl, lane, rect);
    }

    /// A lane's name over its marks at the leading edge, on a backing so it
    /// stays readable
    fn paint_lane_label(
        &self,
        ui: &Ui,
        painter: &Painter,
        catalog: &Catalog,
        rtl: bool,
        lane: &Lane,
        rect: Rect,
    ) {
        let label = lane_label(catalog, lane);

        // A recorder's label is a sentence of the interface, other labels
        // are names with their own direction
        let base = if lane.recorder || lane.key == LaneKey::Unknown {
            rtl
        } else {
            text::starts_rtl(&label)
        };
        let colour = if lane.recorder {
            theme::FOREGROUND
        } else {
            theme::MUTED
        };
        let galley = text::widget_text(ui.ctx(), &label, base, None, Look::Plain).into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            LABEL_WIDTH - LABEL_PADDING * 2.,
            egui::TextStyle::Small,
        );

        let width = galley.size().x + LABEL_PADDING * 2.;
        let left = if rtl { rect.max.x - width } else { rect.min.x };
        let backing = Rect::from_x_y_ranges(Rangef::new(left, left + width), rect.y_range());
        painter.rect_filled(backing, 0., LABEL_FILL);

        let at = Pos2::new(left + LABEL_PADDING, rect.center().y - galley.size().y / 2.);
        painter.galley(at, galley, colour);
    }
}

/// A lane's name, every name its player used, marked when it is the
/// recording player's
fn lane_label(catalog: &Catalog, lane: &Lane) -> String {
    let name = match lane.key {
        LaneKey::Unknown => catalog.text("lane-unknown"),
        LaneKey::Player { .. } => {
            let names: Vec<String> = lane.names.iter().map(|n| text::player_name(n)).collect();
            names.join(" / ")
        }
    };

    if lane.recorder {
        catalog.text_with("lane-recorder", &[("name", FluentValue::from(name))])
    } else {
        name
    }
}

/// One mark of the player lanes centred on `centre`. `crowd` from nothing to
/// one makes a crowded bin's shot taller
fn paint_lane_mark(painter: &Painter, mark: LaneMark, centre: Pos2, colour: Color32, crowd: f32) {
    let vertical = |half: f32| [centre + Vec2::new(0., -half), centre + Vec2::new(0., half)];

    match mark {
        LaneMark::Shot => {
            let half = SHOT_HALF + SHOT_GROWTH * crowd;
            painter.line_segment(vertical(half), Stroke::new(SHOT_STROKE, colour));
        }
        LaneMark::Charge => {
            let band = Rect::from_center_size(centre, CHARGE_SWATCH);
            painter.rect_filled(band, SPAN_RADIUS, colour.gamma_multiply(CHARGE_ALPHA));
        }
        LaneMark::Action => {
            painter.line_segment(vertical(ACTION_HALF), Stroke::new(ACTION_STROKE, colour));
        }
        LaneMark::Kill => {
            let triangle = vec![
                centre + Vec2::new(0., -KILL_HALF - KILL_LIFT),
                centre + Vec2::new(KILL_HALF, KILL_HALF - KILL_LIFT),
                centre + Vec2::new(-KILL_HALF, KILL_HALF - KILL_LIFT),
            ];
            painter.add(Shape::convex_polygon(triangle, KILL_COLOR, Stroke::NONE));
        }
        LaneMark::Death => {
            painter.circle_stroke(centre, DEATH_RADIUS, Stroke::new(DEATH_STROKE, KILL_COLOR));
        }
        LaneMark::Switch => {
            let square = Rect::from_center_size(centre, Vec2::splat(SWITCH_SIZE));
            painter.rect_filled(square, 0., colour);
        }
    }
}

/// Ticks along the ruler with their times, always reading left to right,
/// and faint lines from them down to `bottom`
fn paint_ruler(painter: &Painter, ruler: Rangef, bottom: f32, transform: Transform) {
    let step = tick_step(TICK_GAP * transform.seconds_per_point());
    let precise = step < 1.;
    let font = egui::FontId::monospace(theme::TEXT_SMALL);

    let mut time = (transform.view.0 / step).ceil() * step;
    while time <= transform.view.1 {
        let x = transform.x(time);
        let line = [Pos2::new(x, ruler.min), Pos2::new(x, bottom)];
        painter.line_segment(line, Stroke::new(HAIRLINE, theme::BORDER));

        let at = Pos2::new(x, ruler.min) + TICK_LABEL_INSET;
        let label = clock(time, precise);
        painter.text(
            at,
            egui::Align2::LEFT_TOP,
            label,
            font.clone(),
            theme::FAINT,
        );

        time += step;
    }
}

/// The colour of a kill by its killer's side
pub(super) fn side_colour(side: Side) -> Color32 {
    match side {
        Side::Unknown => theme::FOREGROUND,
        Side::Red => theme::TEAM_RED,
        Side::Blue => theme::TEAM_BLUE,
        Side::Other(i) => OTHER_TEAMS[usize::from(i) % OTHER_TEAMS.len()],
    }
}

/// The view as two times, to the millisecond once the view is short enough
/// for whole seconds to read the same
fn range_text(view: (f32, f32)) -> String {
    let precise = view.1 - view.0 < PRECISE_VIEW;
    format!("{} – {}", clock(view.0, precise), clock(view.1, precise))
}

/// A time as the timeline labels it, to the second or to the millisecond
pub(super) fn clock(time: f32, precise: bool) -> String {
    let full = DemoTime::from_f32(time.max(0.))
        .map(|t| t.to_string())
        .unwrap_or_default();
    if precise {
        return full;
    }

    full.split('.').next().unwrap_or_default().to_string()
}

/// The pointer's look over a surface, by the drag under way or by what the
/// pointer is over
fn set_cursor(
    ui: &Ui,
    response: &Response,
    drag: Option<Drag>,
    surface: &Surface,
    under: &[usize],
) {
    let icon = if response.dragged() {
        match drag {
            Some(Drag::Scrub | Drag::ClipEdge { .. }) => CursorIcon::ResizeHorizontal,
            Some(Drag::Pan { .. }) => CursorIcon::Grabbing,
            _ => return,
        }
    } else {
        let Some(pointer) = response.hover_pos() else {
            return;
        };
        match track::hit(surface, pointer, !under.is_empty()) {
            Target::ClipEdge { .. } | Target::Scrub => CursorIcon::ResizeHorizontal,
            Target::Marker | Target::ClipBody(_) => CursorIcon::PointingHand,
            Target::Background => return,
        }
    };

    ui.ctx().set_cursor_icon(icon);
}
