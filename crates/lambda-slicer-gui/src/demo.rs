//! One demo's view, a timeline of lanes per player and its event table
//! (implementation plan §8)
//!
//! Selecting a demo analyses it again on a thread of its own, since weapon
//! occurrences are not kept in the cache. Only the compact timeline model
//! crosses back to the window. Choosing another demo cancels the analysis

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::analyze_demo_with;
use lambda_slicer_core::analysis::weapons::{Action, Weapon};
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::state::DecodeStatus;
use lambda_slicer_core::timeline::time::DemoTime;

use crate::i18n::Catalog;
use crate::layout;
use crate::library::WEAPONS;
use crate::text::{self, Look};
use crate::timeline::{EventKind, LaneKey, MarkKind, Model, SpanKind, bins, nearest, tick_step};
use lambda_slicer_core::analysis::accuracy::accuracy;
use lambda_slicer_core::analysis::context::{PlayerScore, StandingSide, standing_at};

const LABEL_WIDTH: f32 = 170.;
const LANE_HEIGHT: f32 = 22.;
const AXIS_HEIGHT: f32 = 20.;
/// Width of a bin at wide zoom, and how close a click must be to a mark
const BIN_PIXELS: f32 = 3.;
const HIT_PIXELS: f32 = 5.;
/// Narrowest view in seconds
const MIN_SPAN: f32 = 0.5;

pub fn weapon_color(w: Option<Weapon>) -> Color32 {
    match w {
        Some(Weapon::Gauss) => Color32::from_rgb(90, 170, 255),
        Some(Weapon::Crossbow) => Color32::from_rgb(120, 220, 120),
        Some(Weapon::Rpg) => Color32::from_rgb(255, 150, 60),
        Some(Weapon::Mp5) => Color32::from_rgb(230, 210, 90),
        Some(Weapon::Shotgun) => Color32::from_rgb(220, 120, 200),
        Some(Weapon::Python) => Color32::from_rgb(170, 140, 255),
        Some(Weapon::Glock) => Color32::from_rgb(180, 180, 180),
        Some(Weapon::Crowbar) => Color32::from_rgb(200, 160, 120),
        Some(Weapon::Egon) => Color32::from_rgb(120, 230, 230),
        Some(Weapon::Hivehand) => Color32::from_rgb(170, 220, 60),
        Some(_) => Color32::from_rgb(200, 200, 140),
        None => Color32::from_rgb(160, 160, 160),
    }
}

const KILL_COLOR: Color32 = Color32::from_rgb(240, 70, 70);

/// The tint of a chosen action window, on the timeline and its table rows
const WINDOW_TINT: Color32 = Color32::from_rgba_premultiplied(40, 32, 10, 40);

enum Loaded {
    Model(Arc<Model>),
    Failed(String),
}

/// A demo's analysis running in the background
struct Loader {
    cancel: Arc<AtomicBool>,
    rx: Receiver<Loaded>,
    thread: Option<JoinHandle<()>>,
}

impl Loader {
    fn start(path: PathBuf, duration: f32, ctx: egui::Context) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let thread_cancel = cancel.clone();
        let thread = std::thread::Builder::new()
            .name("demo analysis".into())
            .spawn(move || {
                let report = analyze_demo_with(&path, Limits::default(), Some(thread_cancel));
                let loaded = match report.decode {
                    DecodeStatus::Cancelled => return,
                    DecodeStatus::Failed | DecodeStatus::Unsupported => Loaded::Failed(
                        report
                            .decode_error
                            .map(|m| m.code.to_string())
                            .unwrap_or_default(),
                    ),
                    _ => Loaded::Model(Arc::new(Model::build(&report, duration))),
                };
                let _ = tx.send(loaded);
                ctx.request_repaint();
            })
            .expect("spawn demo analysis");
        Self {
            cancel,
            rx,
            thread: Some(thread),
        }
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub shots: bool,
    pub actions: bool,
    pub kills: bool,
    pub switches: bool,
    pub weapon: Option<Weapon>,
}

/// Only kills at first, the rest one checkbox away
impl Default for Shown {
    fn default() -> Self {
        Self {
            shots: false,
            actions: false,
            kills: true,
            switches: false,
            weapon: None,
        }
    }
}

impl Shown {
    fn kind(&self, k: MarkKind) -> bool {
        match k {
            MarkKind::Shot => self.shots,
            MarkKind::Action => self.actions,
            MarkKind::Kill | MarkKind::Death => self.kills,
            MarkKind::Switch => self.switches,
        }
    }

    fn weapon(&self, w: Option<Weapon>) -> bool {
        self.weapon.is_none() || self.weapon == w
    }
}

pub struct DemoView {
    pub path: PathBuf,
    loader: Option<Loader>,
    model: Option<Arc<Model>>,
    failed: Option<String>,
    view: (f32, f32),
    shown: Shown,
    /// Rows of the event table, indices into the model's events
    rows: Vec<usize>,
    rows_for: Option<Shown>,
    selected: Option<usize>,
    scroll_table: bool,
    /// Frames left to scroll the table to the chosen window's first event.
    /// A table seen for the first time has no size yet, so it gets a few
    scroll_to_window: u8,
    /// The action window shaded on the timeline, and one to jump to once
    /// the analysis is in
    window: Option<usize>,
    pub open_window: Option<usize>,
    /// The timeline is hidden until asked for, as it takes some reading
    pub show_timeline: bool,
}

impl DemoView {
    pub fn open(path: PathBuf, duration: f32, ctx: &egui::Context) -> Self {
        Self {
            loader: Some(Loader::start(path.clone(), duration, ctx.clone())),
            path,
            model: None,
            failed: None,
            view: (0., duration.max(MIN_SPAN)),
            shown: Shown::default(),
            rows: Vec::new(),
            rows_for: None,
            selected: None,
            scroll_table: false,
            scroll_to_window: 0,
            window: None,
            open_window: None,
            show_timeline: false,
        }
    }

    pub fn loading(&self) -> bool {
        self.loader.is_some()
    }

    fn poll(&mut self) {
        let Some(loader) = &self.loader else {
            return;
        };
        match loader.rx.try_recv() {
            Ok(Loaded::Model(m)) => {
                self.view = (m.start, m.end.max(m.start + MIN_SPAN));
                if let Some(w) = self
                    .open_window
                    .and_then(|i| m.windows.get(i).map(|w| (i, w)))
                {
                    let pad = ((w.1.end - w.1.start) * 0.1).max(3.);
                    self.view = (w.1.start - pad, w.1.end + pad);
                    self.window = Some(w.0);
                    self.scroll_to_window = 3;
                }
                self.model = Some(m);
                self.loader = None;
            }
            Ok(Loaded::Failed(code)) => {
                self.failed = Some(code);
                self.loader = None;
            }
            Err(_) => {}
        }
    }

    fn refresh_rows(&mut self) {
        let Some(m) = &self.model else {
            return;
        };
        if self.rows_for == Some(self.shown) {
            return;
        }
        let shown = self.shown;
        self.rows = m
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let kind = match e.kind {
                    EventKind::Action { action, .. } if action.is_shot() => MarkKind::Shot,
                    EventKind::Action { .. } => MarkKind::Action,
                    EventKind::Kill { .. } => MarkKind::Kill,
                    EventKind::Switch => MarkKind::Switch,
                };
                shown.kind(kind) && shown.weapon(e.weapon)
            })
            .map(|(i, _)| i)
            .collect();
        self.rows_for = Some(shown);
    }

    /// Select an event and bring it into view
    fn select(&mut self, event: usize, from_table: bool) {
        self.selected = Some(event);
        self.scroll_table = !from_table;
        if let Some(m) = &self.model {
            let t = m.events[event].time;
            let (a, b) = self.view;
            if t < a || t > b {
                let half = (b - a) / 2.;
                self.view = (t - half, t + half);
            }
        }
    }

    fn controls(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
        layout::wrapping_row(ui, rtl, |ui| {
            ui.checkbox(&mut self.shown.shots, t("show-shots"));
            ui.checkbox(&mut self.shown.actions, t("show-actions"));
            ui.checkbox(&mut self.shown.kills, t("show-kills"));
            ui.checkbox(&mut self.shown.switches, t("show-switches"));
            let current = self
                .shown
                .weapon
                .map_or_else(|| t("filter-weapon-any"), |w| w.name().into());
            egui::ComboBox::from_id_salt("timeline weapon")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.shown.weapon, None, t("filter-weapon-any"));
                    for w in WEAPONS {
                        ui.selectable_value(&mut self.shown.weapon, Some(w), w.name());
                    }
                });
            if ui.button(t("timeline-fit")).clicked()
                && let Some(m) = &self.model
            {
                self.view = (m.start, m.end.max(m.start + MIN_SPAN));
                self.window = None;
            }

            // Jump to the best stretches of play, the first being the best
            let Some(m) = self.model.clone() else {
                return;
            };
            if m.windows.is_empty() {
                return;
            }
            let mut chosen = None;
            if ui.button(t("timeline-best-action")).clicked() {
                chosen = Some(0);
            }
            let describe = |i: usize| {
                let w = &m.windows[i];
                let clock = |t: f32| {
                    DemoTime::from_f32(t)
                        .map(|d| d.to_string())
                        .unwrap_or_default()
                        .split('.')
                        .next()
                        .unwrap_or("")
                        .to_string()
                };
                let sides: Vec<String> = w
                    .sides
                    .iter()
                    .map(|(n, k)| format!("{} {k}", text::player_name(n)))
                    .collect();
                catalog.text_with(
                    "timeline-window",
                    &[
                        ("rank", ((i + 1) as i64).into()),
                        ("score", w.score.into()),
                        ("start", FluentValue::from(clock(w.start))),
                        ("end", FluentValue::from(clock(w.end))),
                        ("sides", FluentValue::from(sides.join(" – "))),
                    ],
                )
            };
            let current = self
                .window
                .map_or_else(|| catalog.text("timeline-windows"), describe);
            egui::ComboBox::from_id_salt("action windows")
                .selected_text(text::widget_text(&ctx, &current, rtl, None, Look::Plain))
                .width(260.)
                .show_ui(ui, |ui| {
                    for i in 0..m.windows.len() {
                        let label = text::widget_text(&ctx, &describe(i), rtl, None, Look::Plain);
                        if ui.selectable_label(self.window == Some(i), label).clicked() {
                            chosen = Some(i);
                        }
                    }
                });
            if let Some(i) = chosen {
                let w = &m.windows[i];
                // The minute with a little room on both sides
                let pad = ((w.end - w.start) * 0.1).max(3.);
                self.view = (w.start - pad, w.end + pad);
                self.window = Some(i);
                self.scroll_to_window = 3;
            }
        });
    }

    fn lane_label(&self, catalog: &Catalog, lane: usize) -> String {
        let Some(m) = &self.model else {
            return String::new();
        };
        let l = &m.lanes[lane];
        let name = match l.key {
            LaneKey::Unknown => catalog.text("lane-unknown"),
            LaneKey::Player { .. } => l
                .names
                .iter()
                .map(|n| text::player_name(n))
                .collect::<Vec<_>>()
                .join(" / "),
        };
        if l.recorder {
            catalog.text_with("lane-recorder", &[("name", FluentValue::from(name))])
        } else {
            name
        }
    }

    /// What an event was, in words
    pub fn describe(catalog: &Catalog, kind: &EventKind) -> String {
        match kind {
            EventKind::Action { action, mode } => {
                let a = catalog.text(match action {
                    Action::Swing => "action-swing",
                    Action::Fire => "action-fire",
                    Action::Launch => "action-launch",
                    Action::ChargeActivity => "action-charge-activity",
                    Action::ChargeStop => "action-charge-stop",
                    Action::BeamStart => "action-beam-start",
                    Action::BeamSustain => "action-beam-sustain",
                    Action::BeamStop => "action-beam-stop",
                    Action::PlacementAttempt => "action-placement-attempt",
                    Action::ReleaseAttempt => "action-release-attempt",
                });
                match mode {
                    Some(m) => {
                        let mode = catalog.text_or(&format!("mode-{m}"), m);
                        format!("{a} · {mode}")
                    }
                    None => a,
                }
            }
            EventKind::Kill {
                victim, headshot, ..
            } => {
                let killed = catalog.text_with(
                    "event-kill",
                    &[(
                        "victim",
                        FluentValue::from(
                            victim
                                .as_deref()
                                .map_or_else(|| "?".into(), text::player_name),
                        ),
                    )],
                );
                if *headshot == Some(true) {
                    format!("{killed} · {}", catalog.text("event-headshot"))
                } else {
                    killed
                }
            }
            EventKind::Switch => catalog.text("event-switch"),
        }
    }

    fn timeline(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let Some(model) = self.model.clone() else {
            return;
        };
        let lanes = model.lanes.len();
        let height = AXIS_HEIGHT + LANE_HEIGHT * lanes as f32;
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let (label_rect, plot) = if rtl {
            (
                Rect::from_min_max(Pos2::new(rect.max.x - LABEL_WIDTH, rect.min.y), rect.max),
                Rect::from_min_max(rect.min, Pos2::new(rect.max.x - LABEL_WIDTH, rect.max.y)),
            )
        } else {
            (
                Rect::from_min_max(rect.min, Pos2::new(rect.min.x + LABEL_WIDTH, rect.max.y)),
                Rect::from_min_max(Pos2::new(rect.min.x + LABEL_WIDTH, rect.min.y), rect.max),
            )
        };
        let response = ui.interact(plot, ui.scope_id().with("plot"), Sense::click_and_drag());

        // Zoom around the pointer, pan by dragging
        let (mut a, mut b) = self.view;
        let span = (b - a).max(MIN_SPAN);
        let per_px = span / plot.width().max(1.);
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y + i.smooth_scroll_delta.x);
            if scroll != 0. {
                let pointer = ui
                    .input(|i| i.pointer.hover_pos())
                    .map_or(a + span / 2., |p| a + (p.x - plot.min.x) * per_px);
                let factor = (-scroll * 0.003).exp();
                let new_span =
                    (span * factor).clamp(MIN_SPAN, (model.end - model.start).max(MIN_SPAN) * 1.2);
                a = pointer - (pointer - a) * new_span / span;
                b = a + new_span;
            }
        }
        if response.dragged() {
            let dx = response.drag_delta().x * per_px;
            a -= dx;
            b -= dx;
        }
        if response.double_clicked() {
            a = model.start;
            b = model.end.max(model.start + MIN_SPAN);
        }
        self.view = (a, b);
        let span = (b - a).max(MIN_SPAN);
        let per_px = span / plot.width().max(1.);
        let x = |t: f32| plot.min.x + (t - a) / per_px;

        let painter = ui.painter_at(rect);
        let visuals = ui.visuals().clone();
        painter.rect_filled(plot, 0., visuals.extreme_bg_color);
        if let Some(w) = self.window.and_then(|i| model.windows.get(i)) {
            let shade = Rect::from_min_max(
                Pos2::new(x(w.start).max(plot.min.x), plot.min.y),
                Pos2::new(x(w.end).min(plot.max.x), plot.max.y),
            );
            if shade.width() > 0. {
                painter.rect_filled(shade, 0., WINDOW_TINT);
            }
        }

        // Time axis with labels at least 90 pixels apart, always left to right
        let step = tick_step(90. * per_px);
        let mut t = (a / step).ceil() * step;
        while t <= b {
            let px = x(t);
            painter.line_segment(
                [
                    Pos2::new(px, plot.min.y + AXIS_HEIGHT - 4.),
                    Pos2::new(px, plot.max.y),
                ],
                Stroke::new(1., visuals.faint_bg_color),
            );
            let label = DemoTime::from_f32(t.max(0.))
                .map(|d| d.to_string())
                .unwrap_or_default();
            let label = if step >= 1. {
                label.split('.').next().unwrap_or("").to_string()
            } else {
                label
            };
            painter.text(
                Pos2::new(px + 3., plot.min.y + 2.),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::monospace(11.),
                visuals.weak_text_color(),
            );
            t += step;
        }

        let bin_width = BIN_PIXELS * per_px;
        let mut hover: Option<(usize, f32)> = None;
        let pointer = ui.input(|i| i.pointer.hover_pos());
        for (i, lane) in model.lanes.iter().enumerate() {
            let top = plot.min.y + AXIS_HEIGHT + LANE_HEIGHT * i as f32;
            let mid = top + LANE_HEIGHT / 2.;
            if i % 2 == 1 {
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(rect.min.x, top),
                        Pos2::new(rect.max.x, top + LANE_HEIGHT),
                    ),
                    0.,
                    visuals.faint_bg_color.gamma_multiply(0.5),
                );
            }

            // Lane label, ordered for its own direction
            // A recorder's label is a sentence of the interface, other
            // labels are names with their own direction
            let label = self.lane_label(catalog, i);
            let base = if lane.recorder || lane.key == LaneKey::Unknown {
                rtl
            } else {
                text::starts_rtl(&label)
            };
            let w = text::widget_text(ui.ctx(), &label, base, None, Look::Plain);
            let galley = w.into_galley(
                ui,
                Some(egui::TextWrapMode::Truncate),
                LABEL_WIDTH - 8.,
                egui::TextStyle::Body,
            );
            let lx = if rtl {
                label_rect.max.x - 4. - galley.size().x
            } else {
                label_rect.min.x + 4.
            };
            let color = if lane.recorder {
                visuals.strong_text_color()
            } else {
                visuals.text_color()
            };
            painter.galley(Pos2::new(lx, mid - galley.size().y / 2.), galley, color);

            if self.shown.kind(MarkKind::Shot) || self.shown.kind(MarkKind::Action) {
                for s in &lane.spans {
                    if s.end < a || s.start > b || !self.shown.weapon(Some(s.weapon)) {
                        continue;
                    }
                    let r = Rect::from_min_max(
                        Pos2::new(x(s.start).max(plot.min.x), mid - 5.),
                        Pos2::new(x(s.end).min(plot.max.x).max(x(s.start) + 1.), mid + 5.),
                    );
                    let alpha = match s.kind {
                        SpanKind::Charge => 0.35,
                        SpanKind::Beam => 0.5,
                    };
                    painter.rect_filled(r, 2., weapon_color(Some(s.weapon)).gamma_multiply(alpha));
                }
            }

            for bin in bins(lane, a, b, bin_width) {
                if !self.shown.kind(bin.kind)
                    || !self.shown.weapon(bin.weapon) && bin.weapon.is_some()
                {
                    continue;
                }
                let px = x(bin.start + bin_width / 2.);
                let strength = (bin.count as f32).min(5.) / 5.;
                match bin.kind {
                    MarkKind::Shot => {
                        let h = 5. + 5. * strength;
                        painter.line_segment(
                            [Pos2::new(px, mid - h), Pos2::new(px, mid + h)],
                            Stroke::new(2., weapon_color(bin.weapon)),
                        );
                    }
                    MarkKind::Action => {
                        painter.line_segment(
                            [Pos2::new(px, mid - 3.), Pos2::new(px, mid + 3.)],
                            Stroke::new(1., visuals.weak_text_color()),
                        );
                    }
                    MarkKind::Kill => {
                        let p = Pos2::new(px, mid);
                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                p + Vec2::new(0., -7.),
                                p + Vec2::new(6., 5.),
                                p + Vec2::new(-6., 5.),
                            ],
                            KILL_COLOR,
                            Stroke::NONE,
                        ));
                    }
                    MarkKind::Death => {
                        painter.circle_stroke(Pos2::new(px, mid), 5., Stroke::new(1.5, KILL_COLOR));
                    }
                    MarkKind::Switch => {
                        painter.rect_filled(
                            Rect::from_center_size(
                                Pos2::new(px, top + LANE_HEIGHT - 4.),
                                Vec2::splat(4.),
                            ),
                            0.,
                            weapon_color(bin.weapon),
                        );
                    }
                }
            }

            if let Some(p) = pointer
                && plot.contains(p)
                && p.y >= top
                && p.y < top + LANE_HEIGHT
            {
                let time = a + (p.x - plot.min.x) * per_px;
                if let Some(m) = nearest(lane, time, HIT_PIXELS * per_px)
                    .filter(|m| self.shown.kind(m.kind) && self.shown.weapon(m.weapon))
                {
                    hover = Some((m.event, m.time));
                }
            }
        }

        if let Some(sel) = self.selected
            && let Some(e) = model.events.get(sel)
            && e.time >= a
            && e.time <= b
        {
            let px = x(e.time);
            painter.line_segment(
                [Pos2::new(px, plot.min.y), Pos2::new(px, plot.max.y)],
                Stroke::new(1.5, visuals.selection.stroke.color),
            );
        }

        if let Some((event, _)) = hover {
            let e = &model.events[event];
            let tip = format!(
                "{} · {} · {}",
                DemoTime::from_f32(e.time)
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                e.actor
                    .as_deref()
                    .map_or_else(|| catalog.text("lane-unknown"), text::player_name),
                Self::describe(catalog, &e.kind)
            );
            let tip = text::widget_text(ui.ctx(), &tip, rtl, None, Look::Plain);
            // Shown at once, not after egui's usual tooltip delay
            egui::Tooltip::for_widget(&response)
                .at_pointer()
                .show(|ui| {
                    ui.label(tip);
                });
            if response.clicked() {
                self.select(event, false);
            }
        }
    }

    fn table(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let Some(model) = self.model.clone() else {
            return;
        };
        self.refresh_rows();
        let ctx = ui.ctx().clone();
        let count = catalog.text_with("demo-events", &[("count", (self.rows.len() as i64).into())]);
        layout::column(ui, rtl, |ui| text::label(ui, &count, rtl));

        let mut cols = vec![
            "column-time",
            "column-player",
            "column-weapon",
            "column-event",
        ];
        if rtl {
            cols.reverse();
        }
        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 4.;
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(Sense::click())
            .auto_shrink(false)
            .min_scrolled_height(0.);
        for c in &cols {
            table = table.column(match *c {
                "column-event" => Column::remainder().at_least(120.),
                "column-player" => Column::initial(170.).at_least(60.),
                _ => Column::initial(100.).at_least(60.),
            });
        }
        if self.scroll_table
            && let Some(sel) = self.selected
            && let Some(row) = self.rows.iter().position(|r| *r == sel)
        {
            table = table.scroll_to_row(row, Some(egui::Align::Center));
            self.scroll_table = false;
        }
        // The chosen window's events are tinted, and the table starts at them
        let window = self
            .window
            .and_then(|i| model.windows.get(i))
            .map(|w| (w.start, w.end));
        if self.scroll_to_window > 0
            && let Some((start, _)) = window
        {
            let first = self.rows.partition_point(|r| model.events[*r].time < start);
            if first < self.rows.len() {
                table = table.scroll_to_row(first, Some(egui::Align::TOP));
            }
            self.scroll_to_window -= 1;
        }
        let mut clicked = None;
        table
            .header(row_height + 4., |mut h| {
                for c in &cols {
                    h.col(|ui| {
                        ui.label(text::widget_text(
                            &ctx,
                            &catalog.text(c),
                            rtl,
                            None,
                            Look::Strong,
                        ));
                    });
                }
            })
            .body(|body| {
                body.rows(row_height, self.rows.len(), |mut row| {
                    let index = self.rows[row.index()];
                    let e = &model.events[index];
                    row.set_selected(self.selected == Some(index));
                    let in_window = window.is_some_and(|(a, b)| e.time >= a && e.time <= b);
                    for c in &cols {
                        row.col(|ui| {
                            if in_window {
                                // Half the spacing on each side, so cells meet
                                let r = ui.max_rect().expand2(ui.spacing().item_spacing / 2.);
                                ui.painter().rect_filled(r, 0., WINDOW_TINT);
                            }
                            let (s, own) = match *c {
                                "column-time" => (
                                    DemoTime::from_f32(e.time)
                                        .map(|d| d.to_string())
                                        .unwrap_or_default(),
                                    false,
                                ),
                                "column-player" => {
                                    let name = e.actor.as_deref().map_or_else(
                                        || catalog.text("lane-unknown"),
                                        text::player_name,
                                    );
                                    let own = text::starts_rtl(&name);
                                    (name, own)
                                }
                                "column-weapon" => (
                                    match &e.kind {
                                        EventKind::Kill { weapon_name, .. } => weapon_name.clone(),
                                        _ => e
                                            .weapon
                                            .map(|w| w.name().to_string())
                                            .unwrap_or_default(),
                                    },
                                    false,
                                ),
                                _ => (Self::describe(catalog, &e.kind), rtl),
                            };
                            let w = text::widget_text(&ctx, &s, own, None, Look::Plain);
                            let w = match (&e.kind, *c) {
                                (EventKind::Kill { .. }, "column-event") => {
                                    egui::Label::new(w.color(KILL_COLOR))
                                }
                                _ => egui::Label::new(w),
                            };
                            ui.add(w.selectable(false).truncate());
                        });
                    }
                    if row.response().clicked() {
                        clicked = Some(index);
                    }
                });
            });
        if let Some(i) = clicked {
            self.select(i, true);
        }
    }

    /// What the marks mean and how to move around, above the lanes
    fn legend(&self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let gauss = weapon_color(Some(Weapon::Gauss));
        let swatch = |ui: &mut Ui, kind: usize| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(16., 14.), Sense::hover());
            let c = rect.center();
            let p = ui.painter();
            match kind {
                0 => {
                    p.line_segment(
                        [c + Vec2::new(0., -6.), c + Vec2::new(0., 6.)],
                        Stroke::new(2., gauss),
                    );
                }
                1 => {
                    p.rect_filled(
                        Rect::from_center_size(c, Vec2::new(14., 8.)),
                        2.,
                        gauss.gamma_multiply(0.35),
                    );
                }
                2 => {
                    p.add(egui::Shape::convex_polygon(
                        vec![
                            c + Vec2::new(0., -6.),
                            c + Vec2::new(5., 5.),
                            c + Vec2::new(-5., 5.),
                        ],
                        KILL_COLOR,
                        Stroke::NONE,
                    ));
                }
                3 => {
                    p.circle_stroke(c, 5., Stroke::new(1.5, KILL_COLOR));
                }
                _ => {
                    p.rect_filled(Rect::from_center_size(c, Vec2::splat(5.)), 0., gauss);
                }
            }
        };
        let items = [
            "legend-shot",
            "legend-charge",
            "legend-kill",
            "legend-death",
            "legend-switch",
        ];
        layout::wrapping_row(ui, rtl, |ui| {
            for (kind, id) in items.iter().enumerate() {
                // A swatch reads before its label in either direction
                swatch(ui, kind);
                ui.label(text::widget_text(
                    &ctx,
                    &catalog.text(id),
                    rtl,
                    None,
                    Look::Plain,
                ));
                ui.add_space(8.);
            }
        });
        layout::column(ui, rtl, |ui| {
            let help = catalog.text("timeline-help");
            ui.label(text::widget_text(&ctx, &help, rtl, None, Look::Plain).weak());
        });
    }

    /// The scores at the selected event, or the final scores. Teams and the
    /// two players of a duel get a column each, and other players share one
    /// ranked list. Headshot kills count those the recording could see
    fn scores(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let Some(model) = self.model.clone() else {
            return;
        };
        let selected = self.selected.and_then(|i| model.events.get(i));
        let time = selected.map(|e| e.time);
        // The selected event's moment, or the last kill for the final scores,
        // since players often leave once a game is over
        let last_kill = model
            .events
            .iter()
            .rev()
            .find(|e| matches!(e.kind, EventKind::Kill { .. }));
        let (segment, at) = match (selected, last_kill) {
            (Some(e), _) | (None, Some(e)) => (e.segment, e.time),
            (None, None) => (
                model.score_updates.last().map_or(0, |u| u.segment),
                f32::INFINITY,
            ),
        };
        let standing = standing_at(&model.score_updates, &model.team_score_updates, segment, at);
        // Other players' kills and shots often happen outside what the
        // recording client receives, so their headshots and accuracy are
        // incomplete. Only the recording player's are shown, and in a duel
        // the opponent's headshots too, since every kill there is of the
        // recording player, whose death the recording always shows
        let recorder = model.recorder;
        // Two players, as teams of one or without teams
        let players: Vec<(u8, u32)> = standing
            .sides
            .iter()
            .flat_map(|s| s.players.iter().map(|p| (p.slot, p.incarnation)))
            .collect();
        let duel = players.len() == 2 && recorder.is_some_and(|r| players.contains(&r));
        let shows_heads = |key: (u8, u32)| duel || Some(key) == recorder;
        let shows_accuracy = |key: (u8, u32)| Some(key) == recorder;
        if standing.sides.is_empty() {
            return;
        }
        let ctx = ui.ctx().clone();
        let heading = match time {
            Some(t) => catalog.text_with(
                "scores-at",
                &[(
                    "time",
                    FluentValue::from(
                        DemoTime::from_f32(t)
                            .map(|d| d.to_string())
                            .unwrap_or_default(),
                    ),
                )],
            ),
            None => catalog.text_with(
                "scores-final",
                &[(
                    "time",
                    FluentValue::from(
                        last_kill
                            .and_then(|e| DemoTime::from_f32(e.time))
                            .map(|d| d.to_string())
                            .unwrap_or_default(),
                    ),
                )],
            ),
        };
        layout::row(ui, rtl, |ui| {
            ui.label(text::widget_text(&ctx, &heading, rtl, None, Look::Strong));
            if time.is_some()
                && ui
                    .small_button(text::widget_text(
                        &ctx,
                        &catalog.text("scores-show-final"),
                        rtl,
                        None,
                        Look::Plain,
                    ))
                    .clicked()
            {
                self.selected = None;
            }
        });

        // Columns run right to left in a right-to-left interface
        let mut sides: Vec<&StandingSide> = if standing.teams || standing.sides.len() == 2 {
            standing.sides.iter().collect()
        } else {
            Vec::new()
        };
        if rtl {
            sides.reverse();
        }
        let frags = catalog.text("scores-frags");
        let deaths = catalog.text("scores-deaths");
        let heads_title = catalog.text("scores-headshots");
        let accuracy_title = catalog.text("scores-accuracy");
        let heads_help = catalog.text("help-headshots");
        let accuracy_help = catalog.text("help-accuracy");
        // Shots, hits, chance matches and exactness per connection up to the
        // moment shown
        let mut aims: std::collections::HashMap<(u8, u32), (u32, u32, u32, bool)> =
            Default::default();
        for s in model
            .shots
            .iter()
            .filter(|s| s.segment == segment && s.time <= at)
        {
            let e = aims.entry((s.slot, s.incarnation)).or_default();
            e.0 += 1;
            e.1 += u32::from(s.hit);
            e.2 += u32::from(s.chance);
            e.3 |= s.exact;
        }
        // Headshot kills per connection up to the moment shown
        let mut heads: std::collections::HashMap<(u8, u32), u32> = Default::default();
        for e in &model.events {
            if e.segment != segment || e.time > at {
                continue;
            }
            if let EventKind::Kill {
                killer: Some(k),
                headshot: Some(true),
                ..
            } = e.kind
            {
                *heads.entry(k).or_default() += 1;
            }
        }
        let side_ui = |ui: &mut Ui, side: &StandingSide, title: bool| {
            if title {
                let name = text::player_name(&side.name);
                let color = match side.name.to_lowercase().as_str() {
                    "red" => Some(Color32::from_rgb(235, 90, 90)),
                    "blue" => Some(Color32::from_rgb(100, 150, 255)),
                    _ => None,
                };
                let line = format!("{name}  {}", side.frags);
                let w = text::widget_text(&ctx, &line, text::starts_rtl(&name), None, Look::Strong);
                let w = match color {
                    Some(c) => w.color(c),
                    None => w,
                };
                ui.label(w);
            }
            // Name, frags and deaths as on the game's scoreboard, the other
            // way round in a right-to-left interface
            let cell = |ui: &mut Ui, k: usize, p: Option<&PlayerScore>| match (k, p) {
                (0, None) => {
                    ui.label("");
                }
                (1, None) => {
                    ui.weak(frags.as_str());
                }
                (2, None) => {
                    ui.weak(deaths.as_str());
                }
                (3, None) => {
                    ui.weak(heads_title.as_str())
                        .on_hover_text(text::help(&ctx, &heads_help, rtl));
                }
                (_, None) => {
                    ui.weak(accuracy_title.as_str()).on_hover_text(text::help(
                        &ctx,
                        &accuracy_help,
                        rtl,
                    ));
                }
                (0, Some(p)) => {
                    let name = text::player_name(&p.name);
                    ui.set_min_width(120.);
                    ui.label(text::widget_text(
                        &ctx,
                        &name,
                        text::starts_rtl(&name),
                        None,
                        Look::Plain,
                    ));
                }
                (1, Some(p)) => {
                    ui.label(p.frags.to_string());
                }
                (2, Some(p)) => {
                    ui.label(p.deaths.to_string());
                }
                (3, Some(p)) => {
                    let key = (p.slot, p.incarnation);
                    if shows_heads(key) {
                        let n = heads.get(&key).copied().unwrap_or(0);
                        ui.label(n.to_string());
                    } else {
                        ui.label("");
                    }
                }
                (_, Some(p)) => {
                    // Blank for others, and with too few shots to tell
                    let key = (p.slot, p.incarnation);
                    let text = aims
                        .get(&key)
                        .filter(|_| shows_accuracy(key))
                        .and_then(|&(s, h, c, _)| accuracy(s, h, c))
                        .map(|v| {
                            catalog.text_with(
                                "library-percent",
                                &[("percent", FluentValue::from((v * 100.).round()))],
                            )
                        })
                        .unwrap_or_default();
                    ui.label(text::widget_text(&ctx, &text, rtl, None, Look::Plain));
                }
            };
            let order: [usize; 5] = if rtl {
                [4, 3, 2, 1, 0]
            } else {
                [0, 1, 2, 3, 4]
            };
            egui::Grid::new(ui.next_auto_id())
                .num_columns(5)
                .spacing([16., 2.])
                .show(ui, |ui| {
                    for k in order {
                        cell(ui, k, None);
                    }
                    ui.end_row();
                    for p in &side.players {
                        for k in order {
                            cell(ui, k, Some(p));
                        }
                        ui.end_row();
                    }
                });
        };

        // The longest column, or every player in a ranked list
        let max_rows = if sides.is_empty() {
            standing.sides.len()
        } else {
            standing
                .sides
                .iter()
                .map(|s| s.players.len())
                .max()
                .unwrap_or(0)
        };
        let height = (ui.text_style_height(&egui::TextStyle::Body) + 4.) * (max_rows as f32 + 2.5);
        egui::ScrollArea::vertical()
            .id_salt("scores")
            .max_height(height.min(200.))
            .show(ui, |ui| {
                if sides.is_empty() {
                    // Three or more players without teams, as one ranked list
                    let all = StandingSide {
                        name: String::new(),
                        frags: 0,
                        deaths: 0,
                        players: standing
                            .sides
                            .iter()
                            .flat_map(|s| s.players.clone())
                            .collect(),
                    };
                    side_ui(ui, &all, false);
                } else {
                    ui.columns(sides.len(), |cols| {
                        for (col, side) in cols.iter_mut().zip(&sides) {
                            side_ui(col, side, true);
                        }
                    });
                }
            });
    }

    /// The whole view. Returns true when it should close
    pub fn ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> bool {
        self.poll();
        let ctx = ui.ctx().clone();
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut close = false;
        layout::row(ui, rtl, |ui| {
            if ui
                .button(text::widget_text(
                    &ctx,
                    &catalog.text("demo-close"),
                    rtl,
                    None,
                    Look::Plain,
                ))
                .clicked()
            {
                close = true;
            }
            ui.label(text::widget_text(
                &ctx,
                &name,
                text::starts_rtl(&name),
                None,
                Look::Strong,
            ));
        });

        if self.loader.is_some() {
            layout::row(ui, rtl, |ui| {
                ui.spinner();
                let msg =
                    catalog.text_with("demo-loading", &[("name", FluentValue::from(name.clone()))]);
                ui.label(text::widget_text(&ctx, &msg, rtl, None, Look::Plain));
            });
            return close;
        }
        if let Some(code) = &self.failed {
            text::label(
                ui,
                &catalog.text_or(code, &catalog.text("status-failed")),
                rtl,
            );
            return close;
        }
        let Some(model) = self.model.clone() else {
            return close;
        };
        layout::column(ui, rtl, |ui| {
            for n in &model.coverage_notes {
                text::label(ui, &catalog.message(n), rtl);
            }
            if let Some(e) = &model.decode_error {
                text::label(ui, &catalog.message(e), rtl);
            }
        });
        ui.separator();
        self.controls(ui, catalog, rtl);
        ui.add_space(4.);
        self.scores(ui, catalog, rtl);
        ui.separator();
        let ctx = ui.ctx().clone();
        layout::row(ui, rtl, |ui| {
            let label =
                text::widget_text(&ctx, &catalog.text("show-timeline"), rtl, None, Look::Plain);
            ui.checkbox(&mut self.show_timeline, label);
        });
        if self.show_timeline {
            self.legend(ui, catalog, rtl);
            let timeline_height = (AXIS_HEIGHT + LANE_HEIGHT * model.lanes.len() as f32)
                .min(ui.available_height() * 0.55);
            egui::ScrollArea::vertical()
                .id_salt("lanes")
                .max_height(timeline_height)
                .show(ui, |ui| self.timeline(ui, catalog, rtl));
        }
        ui.separator();
        self.table(ui, catalog, rtl);
        close
    }
}
