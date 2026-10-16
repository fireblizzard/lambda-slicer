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

impl Default for Shown {
    fn default() -> Self {
        Self {
            shots: true,
            actions: true,
            kills: true,
            switches: true,
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
    /// The action window shaded on the timeline, and one to jump to once
    /// the analysis is in
    window: Option<usize>,
    pub open_window: Option<usize>,
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
            window: None,
            open_window: None,
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
            EventKind::Kill { victim, .. } => catalog.text_with(
                "event-kill",
                &[(
                    "victim",
                    FluentValue::from(
                        victim
                            .as_deref()
                            .map_or_else(|| "?".into(), text::player_name),
                    ),
                )],
            ),
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
                painter.rect_filled(shade, 0., Color32::from_rgba_unmultiplied(255, 200, 60, 28));
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
                "column-player" => Column::initial(170.).at_least(60.).clip(true),
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
                    for c in &cols {
                        row.col(|ui| {
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
            if !model.scoreboard.sides.is_empty() {
                let sides: Vec<String> = model
                    .scoreboard
                    .sides
                    .iter()
                    .take(4)
                    .map(|s| format!("{} {}", text::player_name(&s.name), s.frags))
                    .collect();
                let mut score = sides.join(" – ");
                if model.scoreboard.sides.len() > 4 {
                    score.push_str(" – …");
                }
                let line =
                    catalog.text_with("demo-final-score", &[("score", FluentValue::from(score))]);
                ui.label(text::widget_text(&ctx, &line, rtl, None, Look::Strong));
            }
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
        let timeline_height = (AXIS_HEIGHT + LANE_HEIGHT * model.lanes.len() as f32)
            .min(ui.available_height() * 0.55);
        egui::ScrollArea::vertical()
            .id_salt("lanes")
            .max_height(timeline_height)
            .show(ui, |ui| self.timeline(ui, catalog, rtl));
        ui.separator();
        self.table(ui, catalog, rtl);
        close
    }
}
