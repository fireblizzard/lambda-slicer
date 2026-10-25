//! The Events tab, its filters and kill refinements, the shown events and
//! the way through them
//!
//! The filters at the top and the footer stay put, and only the events
//! scroll. The filters fold to one row saying what they let through. The
//! list, the timeline and its density read the same shown rows

use std::sync::Arc;

use eframe::egui::{
    self, Color32, Galley, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetText,
};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::context::KillCause;
use lambda_slicer_core::analysis::motion::MotionEvidence;
use lambda_slicer_core::analysis::weapons::Weapon;
use lambda_slicer_playback::kill_features::{Doubt, OtherRoute, WallGauss, WallRoute};

use super::compact::{clock, side_colour};
use super::{DemoAction, DemoView, Pointing, Request, Shown, WINDOW_TINT, fold_triangle, track};
use crate::i18n::Catalog;
use crate::kill_analysis::WallStatus;
use crate::kill_filters::{self, AirBadge, MidairMode, SpeedBadge, SpeedSubject};
use crate::layout;
use crate::library::WEAPONS;
use crate::text::{self, Look};
use crate::theme;
use crate::timeline::{Event, EventKind, KillFeatures, KillMotion, Model, SampledAt};
use crate::widgets::{self, Icon, IconButton, Variant};

/// The most of the tab the filters take before they scroll, the least they
/// keep in view while an event still shows under them, and the events
/// they leave in view when there is room
const FILTERS_SHARE: f32 = 0.45;
const FILTERS_LEAST: f32 = theme::CONTROL_HEIGHT;
const EVENT_ROWS_KEPT: f32 = 3.;

/// The least height of the tab that shows the selected kill's movement over
/// its buttons
const DETAILS_ROOM: f32 = 360.;

/// An event row's height and the room between its parts
const ROW_HEIGHT: f32 = 28.;
const PART_GAP: f32 = 6.;

/// How far down the list the playhead is kept while playing, and how thick
/// its line is
const FOLLOW_AT: f32 = 2. / 3.;
const PLAYHEAD_WIDTH: f32 = 2.;

/// How far the list must move without being told to before it counts as
/// scrolled by hand
const SCROLLED_BY_HAND: f32 = 0.5;

/// A badge's room around its text, and the least room badges leave for the
/// players' names
const BADGE_PADDING: Vec2 = Vec2::new(4., 1.);
const NAMES_LEAST: f32 = 120.;

/// The kinds menu's width, the weapon menu's and the player's name field's
const KINDS_WIDTH: f32 = 140.;
const WEAPON_WIDTH: f32 = 120.;
const PLAYER_WIDTH: f32 = 140.;

/// The highlight menu's width, and the least it narrows to beside the
/// timeline's other controls
pub(super) const HIGHLIGHT_WIDTH: f32 = 220.;
pub(super) const HIGHLIGHT_LEAST: f32 = 120.;

/// How many ups dragging the least speed moves it for a point
const UPS_PER_POINT: f64 = 10.;

/// Seconds in a minute and in an hour
const MINUTE: f32 = 60.;
const HOUR: f32 = 3600.;

/// How a badge stands out
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    /// In the accent, for a headshot
    Accent,
    Muted,
    /// For a value the badge cannot give
    Faint,
    /// Text alone, for how a kill came about
    Tag,
}

/// A badge at the trailing end of an event row, with what its hover tells
struct Badge {
    text: String,
    tone: Tone,
    tip: Option<String>,
}

/// Places the parts of an event row from its leading end and from its
/// trailing end towards the middle
struct RowLine {
    rect: Rect,
    rtl: bool,
    leading: f32,
    trailing: f32,
}

impl RowLine {
    fn new(rect: Rect, rtl: bool) -> Self {
        Self {
            rect,
            rtl,
            leading: 0.,
            trailing: 0.,
        }
    }

    /// The width left between the parts placed so far
    fn room(&self) -> f32 {
        (self.rect.width() - self.leading - self.trailing).max(0.)
    }

    /// The place of a part `size` large after those at the leading end
    fn take_leading(&mut self, size: Vec2) -> Rect {
        let left = if self.rtl {
            self.rect.max.x - self.leading - size.x
        } else {
            self.rect.min.x + self.leading
        };
        self.leading += size.x + PART_GAP;
        self.centred(left, size)
    }

    /// The place of a part `size` large before those at the trailing end
    fn take_trailing(&mut self, size: Vec2) -> Rect {
        let left = if self.rtl {
            self.rect.min.x + self.trailing
        } else {
            self.rect.max.x - self.trailing - size.x
        };
        self.trailing += size.x + PART_GAP;
        self.centred(left, size)
    }

    fn centred(&self, left: f32, size: Vec2) -> Rect {
        let top = self.rect.center().y - size.y / 2.;
        Rect::from_min_size(Pos2::new(left, top), size)
    }
}

/// Each kind of event, whether `shown` shows it and its name
fn kinds(shown: &Shown) -> [(bool, &'static str); 6] {
    [
        (shown.kills, "show-kills"),
        (shown.chat, "show-chat"),
        (shown.matches, "show-match"),
        (shown.shots, "show-shots"),
        (shown.switches, "show-switches"),
        (shown.actions, "show-actions"),
    ]
}

/// The kinds `shown` shows by name, or that it shows none
fn kinds_summary(shown: &Shown, catalog: &Catalog) -> String {
    let mut names = Vec::new();
    for (on, id) in kinds(shown) {
        if on {
            names.push(catalog.text(id));
        }
    }

    if names.is_empty() {
        return catalog.text("events-kinds-none");
    }
    names.join(", ")
}

impl DemoView {
    /// The event filters, open or folded to their summary, the shown events
    /// and the footer under them, with the footer placed first so the events
    /// never push it out of view
    pub(super) fn events_tab(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        filters_open: &mut bool,
    ) {
        let Some(model) = self.model.clone() else {
            return;
        };

        // The selected kill's movement shows over the buttons in a tab tall
        // enough for it
        let details = ui.available_height() >= DETAILS_ROOM;
        let footer =
            egui::Frame::new().inner_margin(egui::Margin::symmetric(0, theme::GAP_XS as i8));
        egui::Panel::bottom("events footer")
            .frame(footer)
            .resizable(false)
            .show(ui, |ui| {
                self.events_footer(ui, catalog, rtl, &model, details)
            });

        // Why events stop early is never folded away
        if let Some(e) = &model.decode_error {
            let width = Some(ui.available_width());
            let line = catalog.message(e);
            let line = text::widget_text(ui.ctx(), &line, rtl, width, Look::Color(theme::WARNING));
            ui.add(egui::Label::new(line).wrap());
        }
        self.filters_row(ui, catalog, rtl, filters_open, &model);

        // The filters scroll rather than take the room a few events need, and
        // never more than their share. In a short tab they keep a row of
        // themselves in view as long as one event still shows
        if *filters_open {
            let room = ui.available_height();
            let beside_rows = room - ROW_HEIGHT * EVENT_ROWS_KEPT;
            let beside_one = room - ROW_HEIGHT;
            let most = beside_rows
                .max(FILTERS_LEAST.min(beside_one))
                .clamp(0., (room * FILTERS_SHARE).max(0.));
            egui::ScrollArea::vertical()
                .id_salt(("event filters", self.generation))
                .min_scrolled_height(0.)
                .max_height(most)
                .show(ui, |ui| self.event_filters(ui, catalog, rtl));
        }
        ui.separator();

        // What the filters changed this frame shows in this frame's list
        self.refresh_rows();
        self.unknown_kills_note(ui, catalog, rtl);
        self.events_list(ui, catalog, rtl, &model);
    }

    /// The button that opens or folds the filters, counting those on, the
    /// kinds of event shown, and the button that keeps the list on the
    /// playhead at the trailing end
    fn filters_row(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        open: &mut bool,
        model: &Model,
    ) {
        // The button first, so the row ends short of it. Turned back on, the
        // list goes straight back to the playhead
        layout::trailing(ui, rtl, |ui| {
            let label = catalog.text("events-follow");
            let button = IconButton::new(Icon::Crosshair, &label).selected(self.follow_playhead);
            if ui.add_enabled(self.viewer.is_some(), button).clicked() {
                self.follow_playhead = !self.follow_playhead;
                self.pan_to_playhead = self.follow_playhead;
            }

            layout::row(ui, rtl, |ui| {
                let pointing = if *open {
                    Pointing::Down
                } else {
                    Pointing::Along
                };
                let mut clicked = fold_triangle(ui, pointing, rtl).clicked();

                // The title says how many filters past the kinds are on, so
                // a folded list that shows less says why
                let on = self.filters_on();
                let title = if on == 0 {
                    catalog.text("events-filters")
                } else {
                    let count = FluentValue::from(on as i64);
                    catalog.text_with("events-filters-on", &[("count", count)])
                };
                let title = text::widget_text(ui.ctx(), &title, rtl, None, Look::Plain);
                clicked |= ui.add(egui::Button::new(title)).clicked();
                if clicked {
                    *open = !*open;
                }

                self.kinds_menu(ui, catalog, rtl, model);
            });
        });
    }

    /// How many filters besides the kinds are on, the weapon, the text in
    /// the players' names and each kill refinement while kills show
    fn filters_on(&self) -> usize {
        let mut on = 0;
        if self.shown.weapon.is_some() {
            on += 1;
        }
        if !self.player.trim().is_empty() {
            on += 1;
        }

        let filters = &self.shown.kill_filters;
        if self.shown.kills {
            let refinements = [
                filters.midair != MidairMode::Off,
                filters.speed,
                filters.wallgauss,
            ];
            on += refinements.iter().filter(|&&r| r).count();
        }
        on
    }

    /// The weapon menu and the player's name, and the kill refinements while
    /// kills are shown
    fn event_filters(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        layout::wrapping_row(ui, rtl, |ui| {
            self.weapon_menu(ui, catalog, rtl);
            self.player_field(ui, catalog, rtl);
        });

        // Hidden with the kills, the refinements keep their values
        if self.shown.kills {
            self.refinements(ui, catalog, rtl);
        }
    }

    /// The kinds of event shown, each with its checkbox in one menu, and
    /// what the recording cannot show behind its help
    fn kinds_menu(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, model: &Model) {
        let summary = kinds_summary(&self.shown, catalog);
        let summary = text::widget_text(ui.ctx(), &summary, rtl, None, Look::Plain);
        egui::ComboBox::from_id_salt(("event kinds", self.generation))
            .selected_text(summary)
            .width(KINDS_WIDTH)
            .truncate()
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show_ui(ui, |ui| {
                let shown = &mut self.shown;
                let switches = [
                    (&mut shown.kills, "show-kills"),
                    (&mut shown.chat, "show-chat"),
                    (&mut shown.matches, "show-match"),
                    (&mut shown.shots, "show-shots"),
                    (&mut shown.switches, "show-switches"),
                    (&mut shown.actions, "show-actions"),
                ];
                for (on, id) in switches {
                    ui.checkbox(on, catalog.text(id));
                }
            });

        if !model.coverage_notes.is_empty() {
            let notes: Vec<String> = model
                .coverage_notes
                .iter()
                .map(|n| catalog.message(n))
                .collect();
            widgets::help(ui, &notes.join("\n\n"), rtl);
        }
    }

    /// The one weapon shown, or any
    fn weapon_menu(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let any = text::widget_text(
            ui.ctx(),
            &catalog.text("filter-weapon-any"),
            rtl,
            None,
            Look::Plain,
        );
        let current = match self.shown.weapon {
            Some(w) => WidgetText::from(w.name()),
            None => any.clone(),
        };

        egui::ComboBox::from_id_salt(("event weapon", self.generation))
            .selected_text(current)
            .width(WEAPON_WIDTH)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.shown.weapon, None, any);
                for w in WEAPONS {
                    ui.selectable_value(&mut self.shown.weapon, Some(w), w.name());
                }
            });
    }

    /// Text a player's name in each shown event must have
    fn player_field(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let hint = catalog.text("filter-player");
        let hint = text::widget_text(ui.ctx(), &hint, rtl, None, Look::Plain);
        let field = egui::TextEdit::singleline(&mut self.player)
            .id_salt(("event player", self.generation))
            .hint_text(hint)
            .desired_width(PLAYER_WIDTH);
        ui.add(field);
    }

    /// The most intense moments to jump to, the best first, `width` wide,
    /// and the way to clip the chosen one whole
    pub(super) fn highlight_menu(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        model: &Model,
        width: f32,
    ) {
        if model.windows.is_empty() {
            return;
        }

        let ctx = ui.ctx().clone();
        let label = |i: usize| {
            let line = highlight_label(catalog, model, i);
            text::widget_text(&ctx, &line, rtl, None, Look::Plain)
        };
        let current = match self.window.filter(|&i| i < model.windows.len()) {
            Some(i) => label(i),
            None => {
                let jump = catalog.text("highlight-jump");
                text::widget_text(&ctx, &jump, rtl, None, Look::Plain)
            }
        };

        let mut chosen = None;
        let menu = egui::ComboBox::from_id_salt(("highlights", self.generation))
            .selected_text(current)
            .width(width)
            .wrap_mode(egui::TextWrapMode::Truncate)
            .show_ui(ui, |ui| {
                for i in 0..model.windows.len() {
                    if ui
                        .selectable_label(self.window == Some(i), label(i))
                        .clicked()
                    {
                        chosen = Some(i);
                    }
                }
            });
        menu.response
            .on_hover_text(text::help(ui.ctx(), &catalog.text("highlight-help"), rtl));
        if let Some(i) = chosen {
            self.actions.push(DemoAction::ChooseHighlight(i));
        }

        // The whole highlight padded as an event's clip is, where a clip can
        // start
        if self.window.is_some() {
            let clip = catalog.text("clips-add-window");
            let clip = text::widget_text(ui.ctx(), &clip, rtl, None, Look::Plain);
            let button = widgets::Button::new(Some(Icon::Scissors), clip, Variant::Ghost, rtl);
            let response = ui
                .add_enabled(self.highlight_range().is_some(), button)
                .on_disabled_hover_text(catalog.text("clips-other-session"));
            if response.clicked() {
                self.actions.push(DemoAction::ClipHighlight);
            }
        }
    }

    /// The kill refinements on one row, each one's options under it while
    /// it is on, and Clear while any is on
    fn refinements(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let weapon = self.shown.weapon;
        let filters = &mut self.shown.kill_filters;
        let frame = egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(theme::RADIUS_CONTROL)
            .inner_margin(theme::GAP_XS as i8);

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            layout::wrapping_row(ui, rtl, |ui| {
                widgets::section_title(ui, &catalog.text("kills-with"), rtl);

                let midair = filters.midair != MidairMode::Off;
                if selectable(ui, midair, &catalog.text("refine-midair"), rtl).clicked() {
                    filters.midair = if midair {
                        MidairMode::Off
                    } else {
                        MidairMode::Either
                    };
                }
                if selectable(ui, filters.speed, &catalog.text("refine-speed"), rtl).clicked() {
                    filters.speed = !filters.speed;
                }
                let wallgauss = catalog.text("refine-wallgauss");
                if selectable(ui, filters.wallgauss, &wallgauss, rtl).clicked() {
                    filters.wallgauss = !filters.wallgauss;
                }

                if filters.any() {
                    let clear = catalog.text("refine-clear");
                    let clear = text::widget_text(ui.ctx(), &clear, rtl, None, Look::Plain);
                    let button = widgets::Button::new(None, clear, Variant::Ghost, rtl);
                    if ui.add(button).clicked() {
                        filters.clear();
                    }
                }
            });

            if filters.midair != MidairMode::Off {
                layout::wrapping_row(ui, rtl, |ui| {
                    widgets::section_title(ui, &catalog.text("refine-airborne"), rtl);
                    let modes = [
                        (MidairMode::Either, "refine-either", "help-midair-either"),
                        (MidairMode::Both, "midair-both", "help-midair-both"),
                        (MidairMode::ExactlyOne, "midair-one", "help-midair-one"),
                    ];
                    for (mode, id, help) in modes {
                        let on = filters.midair == mode;
                        let response = selectable(ui, on, &catalog.text(id), rtl)
                            .on_hover_text(text::help(ui.ctx(), &catalog.text(help), rtl));
                        if response.clicked() {
                            filters.midair = mode;
                        }
                    }
                });
            }

            if filters.speed {
                layout::wrapping_row(ui, rtl, |ui| {
                    widgets::section_title(ui, &catalog.text("speed-at-least"), rtl);

                    // Only a whole number of ups is taken, and other text
                    // leaves the last speed as it was
                    let unit = catalog.text("speed-unit");
                    let speed = egui::DragValue::new(&mut filters.min_ups)
                        .speed(UPS_PER_POINT)
                        .suffix(format!(" {unit}"))
                        .custom_parser(parse_ups);
                    ui.add(speed).on_hover_text(text::help(
                        ui.ctx(),
                        &catalog.text("help-speed"),
                        rtl,
                    ));

                    let subjects = [
                        (SpeedSubject::Either, "refine-either"),
                        (SpeedSubject::Killer, "speed-killer"),
                        (SpeedSubject::Victim, "speed-victim"),
                    ];
                    for (subject, id) in subjects {
                        let on = filters.speed_subject == subject;
                        if selectable(ui, on, &catalog.text(id), rtl).clicked() {
                            filters.speed_subject = subject;
                        }
                    }
                });
            }

            if filters.wallgauss {
                let warning = Look::Color(theme::WARNING);

                // How far telling kills through a wall has got, and the way
                // to give it the game's files when it needs them
                let status = self.walls.status();
                let line = match status {
                    WallStatus::Checking => {
                        let progress = self.walls.progress();
                        let args = [
                            ("examined", FluentValue::from(progress.examined as i64)),
                            ("total", FluentValue::from(progress.total as i64)),
                        ];
                        let line = catalog.text_with("wallgauss-checking", &args);
                        Some((line, Look::Color(theme::MUTED)))
                    }
                    WallStatus::NeedsFiles => {
                        Some((catalog.text("wallgauss-needs-files"), warning))
                    }
                    WallStatus::Failed(reason) => {
                        let reason = FluentValue::from(catalog.message(reason));
                        let line = catalog.text_with("wallgauss-failed", &[("reason", reason)]);
                        Some((line, warning))
                    }
                    WallStatus::Done => None,
                };
                layout::wrapping_row(ui, rtl, |ui| {
                    if *status == WallStatus::Checking {
                        ui.spinner();
                    }
                    if let Some((line, look)) = line {
                        let line = text::widget_text(ui.ctx(), &line, rtl, None, look);
                        ui.add(egui::Label::new(line).wrap());
                    }
                    widgets::help(ui, &catalog.text("help-wallgauss"), rtl);
                });

                let locate = *status == WallStatus::NeedsFiles && self.viewer.is_some();
                if locate {
                    let id = if cfg!(windows) {
                        "setup-locate-exe"
                    } else {
                        "setup-locate-folder"
                    };
                    let label =
                        text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain);
                    let button =
                        widgets::Button::new(Some(Icon::Folder), label, Variant::Default, rtl);
                    if ui.add(button).clicked() {
                        self.requests.push(Request::Locate);
                    }
                }

                if let Some(w) = weapon.filter(|w| *w != Weapon::Gauss) {
                    let weapon = FluentValue::from(w.name());
                    let line = catalog.text_with("wallgauss-weapon", &[("weapon", weapon)]);
                    let width = Some(ui.available_width());
                    let line = text::widget_text(ui.ctx(), &line, rtl, width, warning);
                    ui.add(egui::Label::new(line).wrap());
                }
            }
        });
    }

    /// How many kills a refinement could not tell about, over the list, and
    /// nothing while it told about them all. The tab counts the events shown
    fn unknown_kills_note(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        if self.unknown_kills == 0 {
            return;
        }

        layout::wrapping_row(ui, rtl, |ui| {
            let unknown = FluentValue::from(self.unknown_kills as i64);
            let unknown = catalog.text_with("events-unknown", &[("count", unknown)]);
            let look = Look::Color(theme::WARNING);
            ui.label(text::styled_text(
                ui.ctx(),
                &unknown,
                rtl,
                look,
                &theme::secondary(),
            ));
            widgets::help(ui, &catalog.text("help-unknown"), rtl);
        });
    }

    /// The shown events, only the rows in view drawn. A click selects an
    /// event and a double click plays it
    fn events_list(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, model: &Model) {
        let window = self
            .window
            .and_then(|i| model.windows.get(i))
            .map(|w| (w.start, w.end));
        let mut list = egui::ScrollArea::vertical()
            .id_salt(("events", self.generation))
            .auto_shrink(false)
            .min_scrolled_height(0.);

        // An event chosen elsewhere comes to the middle of the list, and a
        // chosen highlight's first event to the top
        let height = ui.available_height();
        let selected_row = self
            .selected
            .and_then(|s| self.rows.iter().position(|&r| r == s));
        // The playhead falls after the events it has passed, which playing
        // keeps in view, two thirds of the way down, while the list follows
        // it. The follow button brings it back at once, even while paused
        let playhead = self.viewer.as_ref().map(|v| v.time());
        let passed =
            playhead.map(|now| self.rows.partition_point(|&r| model.events[r].time <= now));
        let playing = self.viewer.as_ref().is_some_and(|v| v.is_playing());

        // Turning the wheel over the list, or dragging from it, scrolls it
        // by hand and stops it following before it is placed, as fast play
        // places it nearly every frame
        let area = ui.available_rect_before_wrap();
        let wheel = ui.rect_contains_pointer(area)
            && ui.input(|i| {
                let turned = |e: &egui::Event| matches!(e, egui::Event::MouseWheel { .. });
                i.events.iter().any(turned) || i.smooth_scroll_delta.y != 0.
            });
        let dragged = ui.input(|i| {
            let origin = i.pointer.press_origin();
            let from_list = i.pointer.primary_down() && origin.is_some_and(|p| area.contains(p));
            from_list && i.pointer.delta() != Vec2::ZERO
        });
        if wheel || dragged {
            self.follow_playhead = false;
        }

        let moved_on = playing && passed != self.followed;
        let follow = self.follow_playhead && (moved_on || self.pan_to_playhead);
        self.followed = if playing { passed } else { None };
        self.pan_to_playhead = false;

        let mut placed = true;
        if self.scroll_table {
            if let Some(row) = selected_row {
                let offset = row as f32 * ROW_HEIGHT - (height - ROW_HEIGHT) / 2.;
                list = list.vertical_scroll_offset(offset.max(0.));
            }
            self.scroll_table = false;
        } else if self.scroll_to_window > 0
            && let Some((start, _)) = window
        {
            let first = self.rows.partition_point(|&r| model.events[r].time < start);
            list = list.vertical_scroll_offset(first as f32 * ROW_HEIGHT);
            self.scroll_to_window -= 1;
        } else if follow && let Some(passed) = passed {
            let offset = passed as f32 * ROW_HEIGHT - height * FOLLOW_AT;
            list = list.vertical_scroll_offset(offset.max(0.));
        } else {
            placed = false;
        }

        // Rows touch, so the rows the scroll area works out are the rows it
        // draws, at the same places
        let mut clicked = None;
        let mut played = None;
        ui.spacing_mut().item_spacing.y = 0.;

        // Times leave out hours, or minutes too, when no event reaches them
        let longest = model
            .events
            .iter()
            .map(|e| e.time)
            .fold(model.end, f32::max);
        let shared = RowsShare { window, longest };
        let shown = list.show_rows(ui, ROW_HEIGHT, self.rows.len(), |ui, rows| {
            for row in rows {
                let event = self.rows[row];
                let response = self.event_row(ui, catalog, rtl, model, event, shared);
                if response.double_clicked() {
                    played = Some(event);
                } else if response.clicked() {
                    clicked = Some(event);
                }
            }
        });

        // The playhead as a line between the events passed and those to come
        if let Some(passed) = passed {
            let list = shown.inner_rect;
            let y = list.top() + passed as f32 * ROW_HEIGHT - shown.state.clamped_offset().y;
            if (list.top()..=list.bottom()).contains(&y) {
                let line = Stroke::new(PLAYHEAD_WIDTH, theme::ACCENT);
                ui.painter()
                    .with_clip_rect(list)
                    .hline(list.x_range(), y, line);
            }
        }

        // A list that moved on its own while the pointer was on it, or
        // dragging from it, was scrolled by hand, and stops following
        let offset = shown.state.clamped_offset().y;
        let moved = (offset - self.list_offset).abs() > SCROLLED_BY_HAND;
        let held_from_list = ui.input(|i| {
            let origin = i.pointer.press_origin();
            i.pointer.primary_down() && origin.is_some_and(|p| area.contains(p))
        });
        let by_hand = ui.rect_contains_pointer(area) || held_from_list;
        if moved && !placed && by_hand {
            self.follow_playhead = false;
        }
        self.list_offset = offset;

        // Ctrl adds or takes away a clicked event, and Shift chooses every
        // shown event from the last one clicked, neither moving the playhead
        if let Some(event) = clicked {
            let held = ui.input(|i| i.modifiers);
            if held.command {
                self.toggle_mark(event);
            } else if held.shift {
                self.mark_through(event);
            } else {
                self.actions.push(DemoAction::Select {
                    event,
                    from_list: true,
                });
            }
        }

        // Ctrl+A chooses every shown event and Ctrl+C copies those chosen,
        // unless text is being typed
        // A flies a free camera left, with Ctrl held to fly down too
        let free = self.viewer.as_ref().is_some_and(|v| v.free_camera());
        if !text::typing(ui.ctx()) && !free {
            let all = ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::A));
            if all {
                self.marked = self.rows.iter().copied().collect();
                self.highlight_last = false;
            }

            let copy = ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Copy)));
            if copy && !self.marked.is_empty() {
                ui.ctx()
                    .copy_text(self.copied_marks(catalog, model, longest));
            }
        }

        // The clicked event is chosen before it plays, so a double click
        // never plays the event chosen before it
        if let Some(event) = played {
            self.actions.push(DemoAction::Select {
                event,
                from_list: true,
            });
            self.actions.push(DemoAction::PlayEvent(event));
        }
    }

    /// `event` added to the chosen events, or taken away when it is
    /// already among them. The first one added with nothing chosen yet
    /// joins the selected event
    fn toggle_mark(&mut self, event: usize) {
        if self.marked.is_empty()
            && let Some(selected) = self.selected
        {
            self.marked.insert(selected);
        }

        if !self.marked.remove(&event) {
            self.marked.insert(event);
        }
        self.mark_from = Some(event);

        // Clip it now clips the chosen events rather than a highlight
        self.highlight_last = false;
    }

    /// Every shown event from the last one clicked through `event`, in the
    /// list's order, in place of those chosen before
    fn mark_through(&mut self, event: usize) {
        let from = self.mark_from.or(self.selected).unwrap_or(event);
        let row = |e: usize| self.rows.iter().position(|&r| r == e);
        let (Some(a), Some(b)) = (row(from), row(event)) else {
            self.toggle_mark(event);
            return;
        };

        let (first, last) = (a.min(b), a.max(b));
        self.marked = self.rows[first..=last].iter().copied().collect();
        self.highlight_last = false;
    }

    /// The chosen events that show, one line each in the list's order
    fn copied_marks(&self, catalog: &Catalog, model: &Model, longest: f32) -> String {
        let mut lines = Vec::new();
        for &index in &self.rows {
            if !self.marked.contains(&index) {
                continue;
            }
            let what = self.what(catalog, index);
            lines.push(copied(catalog, &model.events[index], &what, longest));
        }
        lines.join("\n")
    }

    /// One event as its time, its players and what happened, and its badges
    /// at the trailing end. Badges that would leave the names too little
    /// room fold into a count whose hover lists them
    fn event_row(
        &self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        model: &Model,
        index: usize,
        shared: RowsShare,
    ) -> Response {
        let RowsShare { window, longest } = shared;
        let event = &model.events[index];
        let size = Vec2::new(ui.available_width(), ROW_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let selected = self.selected == Some(index);
        let marked = self.marked.contains(&index);
        let in_window = window.is_some_and(|(a, b)| event.time >= a && event.time <= b);
        let painter = ui.painter();
        if marked && !selected {
            painter.rect_filled(rect, 0., theme::ACCENT_SOFT);
        } else if selected {
            painter.rect_filled(rect, 0., theme::ACCENT_SOFT);
            let edge_x = if rtl {
                rect.max.x - theme::SELECTED_EDGE
            } else {
                rect.min.x
            };
            let edge = Rect::from_min_size(
                Pos2::new(edge_x, rect.min.y),
                Vec2::new(theme::SELECTED_EDGE, rect.height()),
            );
            painter.rect_filled(edge, 0., theme::ACCENT);
        } else if response.hovered() {
            painter.rect_filled(rect, 0., theme::RAISED);
        } else if in_window {
            painter.rect_filled(rect, 0., WINDOW_TINT);
        }

        let mut line = RowLine::new(rect.shrink2(Vec2::new(theme::GAP_S, 0.)), rtl);

        // The time always reads left to right
        let time = event_clock(event.time, longest);
        let time = egui::RichText::new(time).monospace().color(theme::FAINT);
        let time = layout_text(ui, time.into(), None);
        let place = line.take_leading(time.size());
        ui.painter().galley(place.min, time, theme::FAINT);

        // Badges take their room before the names, as many as leave the
        // names enough
        let mut badges = self.badges(catalog, event);
        let widths: Vec<f32> = badges.iter().map(|b| badge_width(ui, b)).collect();
        let mut kept = badges.len();
        let mut used: f32 = widths.iter().map(|w| w + PART_GAP).sum();
        while kept > 0 && line.room() - used < NAMES_LEAST {
            kept -= 1;
            used -= widths[kept] + PART_GAP;
        }
        let folded = badges.split_off(kept);
        if !folded.is_empty() {
            badges.push(folded_badge(&folded));
        }
        for (k, badge) in badges.iter().enumerate().rev() {
            paint_badge(ui, &mut line, &response, badge, k);
        }

        // A right click copies the event as one line of text
        response.context_menu(|ui| {
            let label = catalog.text("event-copy");
            let label = text::widget_text(ui.ctx(), &label, rtl, None, Look::Plain);
            let copy = widgets::Button::new(Some(Icon::Copy), label, Variant::Default, rtl);
            if ui.add(copy).clicked() {
                // An event among several chosen copies them all
                let text = if marked && self.marked.len() > 1 {
                    self.copied_marks(catalog, model, longest)
                } else {
                    copied(catalog, event, &self.what(catalog, index), longest)
                };
                ui.ctx().copy_text(text);
                ui.close();
            }
        });

        let truncated = self.paint_parts(ui, &mut line, catalog, rtl, model, index);
        if truncated {
            let full = self.event_summary(catalog, index, false);
            return response.on_hover_text(full);
        }
        response
    }

    /// The actor, the weapon and, for a kill, the victim, each name cut
    /// short on its own when they do not fit. Returns whether a name was
    /// cut short
    fn paint_parts(
        &self,
        ui: &Ui,
        line: &mut RowLine,
        catalog: &Catalog,
        rtl: bool,
        model: &Model,
        index: usize,
    ) -> bool {
        let event = &model.events[index];
        let teams = model.team_play(event.segment);
        let name_colour = |team: Option<&str>| {
            let team = team.filter(|_| teams);
            side_colour(track::side_of(team))
        };
        // A line of chat or a change in the match with no player behind it
        // is the server's, and names nobody
        let server = matches!(event.kind, EventKind::Chat { .. } | EventKind::Match(_));
        let actor = match event.actor.as_deref() {
            Some(name) => text::player_name(name),
            None if server => String::new(),
            None => catalog.text("lane-unknown"),
        };

        let (actor_colour, weapon, second, second_colour) = match &event.kind {
            EventKind::Kill {
                victim,
                weapon_name,
                features,
                ..
            } => {
                let victim = victim
                    .as_deref()
                    .map_or_else(|| "?".to_string(), text::player_name);
                (
                    name_colour(features.killer_team.as_deref()),
                    Some(kill_weapon(weapon_name, event.weapon)),
                    victim,
                    name_colour(features.victim_team.as_deref()),
                )
            }
            // A line of chat reads as what was said
            EventKind::Chat { .. } => (
                theme::FOREGROUND,
                None,
                self.what(catalog, index),
                theme::FOREGROUND,
            ),
            _ => (
                theme::FOREGROUND,
                event.weapon.map(|w| w.name().to_string()),
                self.what(catalog, index),
                theme::MUTED,
            ),
        };
        let second_is_name = matches!(event.kind, EventKind::Kill { .. });

        // The weapon as a chip between the two
        let chip = weapon.map(|w| {
            let chip = Badge {
                text: w,
                tone: Tone::Muted,
                tip: None,
            };
            (badge_width(ui, &chip), chip)
        });
        let chip_room = chip.as_ref().map_or(0., |(width, _)| width + PART_GAP);

        // Each name in its own direction, the description in the interface's
        let first = text::widget_text(
            ui.ctx(),
            &actor,
            text::starts_rtl(&actor),
            None,
            Look::Color(actor_colour),
        );
        let second_base = if second_is_name {
            text::starts_rtl(&second)
        } else {
            rtl
        };
        let second = text::widget_text(
            ui.ctx(),
            &second,
            second_base,
            None,
            Look::Color(second_colour),
        );

        // The shorter keeps its width when the other can give up enough, and
        // otherwise both get half
        let room = (line.room() - chip_room - PART_GAP).max(0.);
        let first_width = layout_text(ui, first.clone(), None).size().x;
        let second_width = layout_text(ui, second.clone(), None).size().x;
        let half = room / 2.;
        let (first_most, second_most) = if first_width + second_width <= room {
            (first_width, second_width)
        } else if first_width <= half {
            (first_width, room - first_width)
        } else if second_width <= half {
            (room - second_width, second_width)
        } else {
            (half, half)
        };

        let first = layout_text(ui, first, Some(first_most));
        let place = line.take_leading(first.size());
        let mut truncated = first.elided;
        ui.painter().galley(place.min, first, actor_colour);

        if let Some((_, chip)) = &chip {
            let size = badge_size(ui, chip);
            let place = line.take_leading(size);
            draw_badge(ui, place, chip);
        }

        let second = layout_text(ui, second, Some(second_most));
        let place = line.take_leading(second.size());
        truncated |= second.elided;
        ui.painter().galley(place.min, second, second_colour);

        truncated
    }

    /// A kill's badges in their order, headshot, airborne, speed and how
    /// the kill came about, none for other events
    fn badges(&self, catalog: &Catalog, event: &Event) -> Vec<Badge> {
        let EventKind::Kill {
            headshot,
            cause,
            features,
            ..
        } = &event.kind
        else {
            return Vec::new();
        };
        let mut badges = Vec::new();

        if *headshot == Some(true) {
            badges.push(Badge {
                text: catalog.text("badge-headshot"),
                tone: Tone::Accent,
                tip: None,
            });
        }

        // Only a kill reconstructed as through a wall shows it
        let wall = self.walls.get(&features.key);
        if let Some(WallGauss::Yes(_)) = wall {
            badges.push(Badge {
                text: catalog.text("badge-wall"),
                tone: Tone::Accent,
                tip: Some(wall_tip(catalog, wall)),
            });
        }

        if let Some(air) = kill_filters::air_badge(features) {
            let id = match air {
                AirBadge::Both => "badge-air-both",
                AirBadge::One => "badge-air",
            };
            badges.push(Badge {
                text: catalog.text(id),
                tone: Tone::Muted,
                tip: Some(air_tip(catalog, features, air)),
            });
        }

        let filters = &self.shown.kill_filters;
        if let Some(speed) = kill_filters::speed_badge(features, filters) {
            let (text, tone) = match speed {
                SpeedBadge::Known(ups) => (speed_text(catalog, ups), Tone::Muted),
                SpeedBadge::Unknown => (catalog.text("speed-ups-unknown"), Tone::Faint),
            };
            let mut tip = vec![catalog.text("speed-horizontal")];
            tip.extend(motion_lines(catalog, features));
            badges.push(Badge {
                text,
                tone,
                tip: Some(tip.join("\n")),
            });
        }

        let how = match cause {
            KillCause::Suicide => Some("badge-suicide"),
            KillCause::Teammate => Some("badge-teamkill"),
            KillCause::World | KillCause::Environment => Some("badge-world"),
            _ => None,
        };
        if let Some(id) = how {
            badges.push(Badge {
                text: catalog.text(id),
                tone: Tone::Tag,
                tip: None,
            });
        }

        badges
    }

    /// The selected kill's movement when `details` has room for it and why
    /// the selected event cannot be played, over the buttons through the
    /// events, to play the selected one and to clip it
    fn events_footer(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        model: &Model,
        details: bool,
    ) {
        let selected = self.selected.filter(|&i| i < model.events.len());
        let shown = selected.filter(|&i| self.shown_events.get(i).copied().unwrap_or(false));
        let unreachable = selected.is_some_and(|e| !self.reachable(e));

        // The full movement on a click, not only behind a badge's hover
        let width = Some(ui.available_width());
        if details
            && let Some(event) = shown
            && let EventKind::Kill { features, .. } = &model.events[event].kind
        {
            let mut lines = motion_lines(catalog, features);
            if features.gauss {
                lines.push(wall_tip(catalog, self.walls.get(&features.key)));
            }
            for line in lines {
                let look = Look::Color(theme::MUTED);
                let line = text::widget_text(ui.ctx(), &line, rtl, width, look);
                ui.add(egui::Label::new(line).wrap());
            }
        }
        if unreachable {
            let line = catalog.text("event-unreachable");
            let line = text::widget_text(ui.ctx(), &line, rtl, width, Look::Color(theme::WARNING));
            ui.add(egui::Label::new(line).wrap());
        }

        // Why Play and Clip it are off, in the order a user would fix it
        let blocked = match (selected, shown) {
            (None, _) => Some(catalog.text("clips-select-event")),
            (Some(_), None) => Some(catalog.text("event-hidden")),
            _ if unreachable => Some(catalog.text("event-unreachable")),
            _ => None,
        };
        // Clip it takes the highlight chosen after the event, whatever is
        // selected, or else the several events chosen
        let clip_blocked = match &blocked {
            _ if self.clips_highlight() => self
                .clip_it_range()
                .is_none()
                .then(|| catalog.text("clips-other-session")),
            _ if self.chosen_span().is_some() => self
                .proposal()
                .is_none()
                .then(|| catalog.text("clips-other-session")),
            Some(why) => Some(why.clone()),
            None if self.proposal().is_none() => Some(catalog.text("clips-other-session")),
            None => None,
        };

        layout::row(ui, rtl, |ui| {
            self.event_nav(ui, catalog, model);

            layout::trailing(ui, rtl, |ui| {
                let clip = catalog.text("clip-it");
                let clip = text::widget_text(ui.ctx(), &clip, rtl, None, Look::Plain);
                let button =
                    widgets::Button::new(Some(Icon::Scissors), clip, Variant::Default, rtl);
                let response = ui.add_enabled(clip_blocked.is_none(), button);
                let response = match &clip_blocked {
                    Some(why) => response.on_disabled_hover_text(why),
                    None => response,
                };
                if response.clicked() {
                    self.actions.push(DemoAction::ClipEvent);
                }

                if self.viewer.is_none() {
                    return;
                }

                let play = catalog.text("event-play");
                let play = text::widget_text(ui.ctx(), &play, rtl, None, Look::Plain);
                let button = widgets::Button::new(Some(Icon::Play), play, Variant::Default, rtl);
                let response = ui.add_enabled(blocked.is_none(), button);
                let response = match &blocked {
                    Some(why) => response.on_disabled_hover_text(why),
                    None => response,
                };
                if response.clicked()
                    && let Some(event) = shown
                {
                    self.actions.push(DemoAction::PlayEvent(event));
                }
            });
        });
    }

    /// Buttons to the shown event before or after the selected one, or
    /// before or after the playhead when the selected one is not shown
    fn event_nav(&mut self, ui: &mut Ui, catalog: &Catalog, model: &Model) {
        let (previous, next) = self.neighbours(model);

        let mut chosen = None;
        let label = catalog.text("event-previous");
        if ui
            .add_enabled(previous.is_some(), IconButton::new(Icon::Previous, &label))
            .clicked()
        {
            chosen = previous;
        }
        let label = catalog.text("event-next");
        if ui
            .add_enabled(next.is_some(), IconButton::new(Icon::Next, &label))
            .clicked()
        {
            chosen = next;
        }

        if let Some(event) = chosen.and_then(|i| self.rows.get(i).copied()) {
            self.actions.push(DemoAction::Select {
                event,
                from_list: false,
            });
        }
    }

    /// The places among the shown events of the ones before and after the
    /// selected event, or before and after the current moment when the
    /// selected event is not shown
    pub(super) fn neighbours(&self, model: &Model) -> (Option<usize>, Option<usize>) {
        let at = self
            .selected
            .and_then(|s| self.rows.iter().position(|&r| r == s));
        let now = self.viewer.as_ref().map_or(0., |v| v.time());

        let previous = match at {
            Some(i) => i.checked_sub(1),
            None => self.rows.iter().rposition(|&r| model.events[r].time < now),
        };
        let next = match at {
            Some(i) => Some(i + 1).filter(|&n| n < self.rows.len()),
            None => self.rows.iter().position(|&r| model.events[r].time > now),
        };
        (previous, next)
    }
}

/// A toggle button showing whether it is on
fn selectable(ui: &mut Ui, on: bool, label: &str, rtl: bool) -> Response {
    let label = text::widget_text(ui.ctx(), label, rtl, None, Look::Plain);
    ui.add(egui::Button::selectable(on, label))
}

/// A whole number of ups, without the unit. Anything else is refused, and
/// the speed keeps its last value
fn parse_ups(text: &str) -> Option<f64> {
    let digits = text.trim().trim_end_matches(char::is_alphabetic).trim();
    let ups: u32 = digits.parse().ok()?;
    Some(f64::from(ups))
}

/// A highlight in the menu, the best one named the golden minute
fn highlight_label(catalog: &Catalog, model: &Model, i: usize) -> String {
    let w = &model.windows[i];
    let sides: Vec<String> = w
        .sides
        .iter()
        .map(|(name, kills)| format!("{} {kills}", text::player_name(name)))
        .collect();

    let line = catalog.text_with(
        "timeline-window",
        &[
            ("rank", ((i + 1) as i64).into()),
            ("score", w.score.into()),
            ("start", FluentValue::from(clock(w.start, false))),
            ("end", FluentValue::from(clock(w.end, false))),
            ("sides", FluentValue::from(sides.join(" – "))),
        ],
    );
    if i == 0 {
        format!("{} · {line}", catalog.text("timeline-best-action"))
    } else {
        line
    }
}

/// A speed in ups, rounded only for showing
fn speed_text(catalog: &Catalog, ups: f32) -> String {
    let ups = FluentValue::from(f64::from(ups.round()));
    catalog.text_with("speed-ups", &[("speed", ups)])
}

/// Both players' movement around a kill, a line each, saying what is
/// unknown and how and when each value was found
fn motion_lines(catalog: &Catalog, features: &KillFeatures) -> Vec<String> {
    let players = [
        ("motion-who-killer", features.killer_motion),
        ("motion-who-victim", features.victim_motion),
    ];

    let mut lines = Vec::new();
    for (who, motion) in players {
        if let Some(motion) = motion {
            lines.push(motion_line(catalog, who, &motion));
        }
    }

    if lines.is_empty() {
        lines.push(catalog.text("motion-none"));
    }
    lines
}

/// One player's movement in a line, `who` naming them as killer or victim
fn motion_line(catalog: &Catalog, who: &str, motion: &KillMotion) -> String {
    let air = match motion.airborne {
        Some(true) => "motion-air-yes",
        Some(false) => "motion-air-no",
        None => "motion-air-unknown",
    };
    let speed = match motion.horizontal_speed {
        Some(ups) => speed_text(catalog, ups),
        None => catalog.text("motion-speed-unknown"),
    };
    let how = match motion.evidence {
        MotionEvidence::Exact => "motion-exact",
        MotionEvidence::Trajectory => "motion-trajectory",
        MotionEvidence::Unclear => "motion-unclear",
        MotionEvidence::NotVisible => "motion-not-visible",
    };
    let when = match motion.sampled_at {
        SampledAt::Shot(_) => "motion-at-shot",
        SampledAt::Kill => "motion-at-kill",
    };

    let args = [
        ("who", FluentValue::from(catalog.text(who))),
        ("air", FluentValue::from(catalog.text(air))),
        ("speed", FluentValue::from(speed)),
        ("how", FluentValue::from(catalog.text(how))),
        ("when", FluentValue::from(catalog.text(when))),
    ];
    catalog.text_with("motion-line", &args)
}

/// Who an airborne badge says was in the air, and whether the other's
/// state is unknown, then both players' movement
fn air_tip(catalog: &Catalog, features: &KillFeatures, badge: AirBadge) -> String {
    let killer = features.killer_motion.and_then(|m| m.airborne);
    let victim = features.victim_motion.and_then(|m| m.airborne);

    let mut lines = Vec::new();
    match badge {
        AirBadge::Both => lines.push(catalog.text("air-both")),
        AirBadge::One => {
            let (id, other) = if killer == Some(true) {
                ("air-killer", victim)
            } else {
                ("air-victim", killer)
            };
            lines.push(catalog.text(id));
            if other.is_none() {
                lines.push(catalog.text("air-other-unknown"));
            }
        }
    }

    lines.extend(motion_lines(catalog, features));
    lines.join("\n")
}

/// What is told of a Gauss kill going through a wall, with what a
/// reconstruction cannot know when it was through one
fn wall_tip(catalog: &Catalog, wall: Option<&WallGauss>) -> String {
    let doubt = |doubt: &Doubt| match doubt {
        Doubt::NoShot => "wall-doubt-no-shot",
        Doubt::NoMap => "wall-doubt-no-map",
        Doubt::VictimUnseen => "wall-doubt-unseen",
        Doubt::BrushEntity => "wall-doubt-brush",
        Doubt::Ambiguous => "wall-doubt-ambiguous",
        Doubt::Unexplained => "wall-doubt-unexplained",
        Doubt::Unconfirmed => "wall-doubt-unconfirmed",
        Doubt::NotReached => "wall-doubt-not-reached",
    };

    let Some(WallGauss::Yes(details)) = wall else {
        let id = match wall {
            Some(WallGauss::No(OtherRoute::Direct)) => "wall-no-direct",
            Some(WallGauss::No(OtherRoute::Reflection)) => "wall-no-reflection",
            Some(WallGauss::Unknown(why)) => doubt(why),
            _ => "wall-pending",
        };
        return catalog.text(id);
    };

    let id = match details.route {
        WallRoute::ExitBlast => "wall-yes-exit",
        WallRoute::Beyond => "wall-yes-beyond",
    };
    let thickness = FluentValue::from(f64::from(details.thickness.round()));
    let line = catalog.text_with(id, &[("thickness", thickness)]);
    format!("{line}\n{}", catalog.text("wall-limits"))
}

/// Badges folded away for room, as a count whose hover lists them
fn folded_badge(folded: &[Badge]) -> Badge {
    let mut lines = Vec::new();
    for badge in folded {
        lines.push(badge.text.clone());
        if let Some(tip) = &badge.tip {
            lines.push(tip.clone());
        }
    }

    Badge {
        text: format!("+{}", folded.len()),
        tone: Tone::Muted,
        tip: Some(lines.join("\n")),
    }
}

/// Text laid out on one line, cut short at `most` points when given
fn layout_text(ui: &Ui, text: WidgetText, most: Option<f32>) -> Arc<Galley> {
    let (wrap, width) = match most {
        Some(most) => (egui::TextWrapMode::Truncate, most),
        None => (egui::TextWrapMode::Extend, f32::INFINITY),
    };
    text.into_galley(ui, Some(wrap), width, egui::TextStyle::Body)
}

/// A badge's text in its small style
fn badge_galley(ui: &Ui, badge: &Badge) -> Arc<Galley> {
    let text = WidgetText::from(egui::RichText::new(&badge.text));
    text.into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Small,
    )
}

/// The room a badge takes, its text and the chip around it
fn badge_size(ui: &Ui, badge: &Badge) -> Vec2 {
    let text = badge_galley(ui, badge).size();
    match badge.tone {
        Tone::Tag => text,
        _ => text + BADGE_PADDING * 2.,
    }
}

/// The width a badge takes along its row
fn badge_width(ui: &Ui, badge: &Badge) -> f32 {
    badge_size(ui, badge).x
}

/// A badge at `place`, its colours by its tone
fn draw_badge(ui: &Ui, place: Rect, badge: &Badge) {
    let (fill, colour) = match badge.tone {
        Tone::Accent => (theme::ACCENT_SOFT, theme::ACCENT),
        Tone::Muted => (theme::RAISED, theme::MUTED),
        Tone::Faint => (theme::RAISED, theme::FAINT),
        Tone::Tag => (Color32::TRANSPARENT, theme::FAINT),
    };
    let painter = ui.painter();
    painter.rect_filled(place, theme::RADIUS_SMALL, fill);

    let galley = badge_galley(ui, badge);
    let at = place.center() - galley.size() / 2.;
    painter.galley(at, galley, colour);
}

/// A badge placed from the row's trailing end, with its hover text over it
fn paint_badge(ui: &Ui, line: &mut RowLine, row: &Response, badge: &Badge, k: usize) {
    let place = line.take_trailing(badge_size(ui, badge));
    draw_badge(ui, place, badge);

    if let Some(tip) = &badge.tip {
        let hover = ui.interact(place, row.id.with(("badge", k)), Sense::hover());
        hover.on_hover_text(tip);
    }
}

/// `event` as one line of text to copy, its time as the list shows it and
/// then what happened, as `what` words it. A kill reads as a sentence, such
/// as `01:46.817 | andriu killed -<[ATK]>-naz with rpg_rocket`, and chat as
/// its sender and line. Names lose their colour codes, and the marks Fluent
/// puts around arguments are left out
fn copied(catalog: &Catalog, event: &Event, what: &str, longest: f32) -> String {
    let time = event_clock(event.time, longest);
    let actor = event.actor.as_deref().map(text::player_name);

    let line = match &event.kind {
        EventKind::Kill {
            victim,
            weapon_name,
            cause,
            headshot,
            ..
        } => {
            let weapon = kill_weapon(weapon_name, event.weapon);
            let killed = kill_sentence(catalog, event, *cause, victim.as_deref(), &weapon);
            if *headshot == Some(true) {
                format!("{killed} · {}", catalog.text("event-headshot"))
            } else {
                killed
            }
        }
        EventKind::Chat { .. } => match actor {
            Some(sender) => format!("{sender}: {what}"),
            None => what.to_owned(),
        },
        EventKind::Match(_) => match actor {
            Some(player) => format!("{player} · {what}"),
            None => what.to_owned(),
        },
        EventKind::Action { .. } | EventKind::Switch => {
            let actor = actor.unwrap_or_else(|| catalog.text("lane-unknown"));
            format!("{actor} · {what}")
        }
    };

    format!("{time} | {line}").replace(['\u{2068}', '\u{2069}'], "")
}

/// A kill as a sentence by its cause. One with no killer named, such as a
/// fall or a monster, says the victim died
/// The weapon a kill shows, the name the server sent, except a `grenade`
/// the analysis told the weapon of, as the mp5's and the hand grenade's
/// both kill by that name
fn kill_weapon(weapon_name: &str, weapon: Option<Weapon>) -> String {
    match (weapon_name, weapon) {
        ("grenade", Some(Weapon::Mp5)) => "mp5 grenade".to_string(),
        ("grenade", Some(Weapon::Handgrenade)) => "handgrenade".to_string(),
        _ => weapon_name.to_string(),
    }
}

fn kill_sentence(
    catalog: &Catalog,
    event: &Event,
    cause: KillCause,
    victim: Option<&str>,
    weapon: &str,
) -> String {
    let name = |n: Option<&str>| n.map_or_else(|| catalog.text("lane-unknown"), text::player_name);
    let id = match cause {
        KillCause::Suicide => "event-copy-suicide",
        KillCause::Teammate => "event-copy-teamkill",
        KillCause::World | KillCause::Environment | KillCause::Monster => "event-copy-death",
        _ if event.actor.is_none() => "event-copy-death",
        _ => "event-copy-kill",
    };

    let args = [
        ("killer", FluentValue::from(name(event.actor.as_deref()))),
        ("victim", FluentValue::from(name(victim))),
        ("weapon", FluentValue::from(weapon.to_owned())),
    ];
    catalog.text_with(id, &args)
}

/// What every row of the event list shares, the highlight chosen and the
/// longest time of the demo
#[derive(Clone, Copy)]
struct RowsShare {
    window: Option<(f32, f32)>,
    longest: f32,
}

/// `time` to the millisecond as the event list shows it. Hours are left
/// out when the demo's `longest` time is under an hour, and minutes too
/// when it is under a minute
fn event_clock(time: f32, longest: f32) -> String {
    // `clock` writes `HH:MM:SS.mmm`, so leaving out its first fields is enough
    let full = clock(time, true);
    let left_out = if longest >= HOUR {
        0
    } else if longest >= MINUTE {
        "HH:".len()
    } else {
        "HH:MM:".len()
    };
    full[left_out..].to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use lambda_slicer_core::analysis::weapons::Action;

    use super::*;

    /// A kill at 1:46.817 by `killer` with `weapon`, of this cause
    fn kill(killer: Option<&str>, weapon: &str, cause: KillCause) -> Event {
        let features = KillFeatures {
            key: crate::timeline::KillKey {
                segment: 0,
                ordinal: 0,
            },
            killer_motion: None,
            victim_motion: None,
            shot: None,
            victim: None,
            killer_team: None,
            victim_team: None,
            gauss: false,
            gauss_kill: None,
        };

        Event {
            segment: 0,
            time: 106.817,
            lane: 0,
            actor: killer.map(str::to_owned),
            weapon: None,
            kind: EventKind::Kill {
                victim: Some("-<[ATK]>-^1naz".into()),
                weapon_name: weapon.into(),
                cause,
                killer: None,
                victim_player: None,
                headshot: None,
                features: Box::new(features),
            },
        }
    }

    #[test]
    fn a_copied_kill_reads_as_a_sentence_without_colour_codes() {
        let catalog = Catalog::new(crate::i18n::Locale::En);
        let copy = |e: &Event| copied(&catalog, e, "", 1200.);

        let shot = kill(Some("andriu"), "rpg_rocket", KillCause::PlayerWeapon);
        assert_eq!(
            copy(&shot),
            "01:46.817 | andriu killed -<[ATK]>-naz with rpg_rocket"
        );

        let fall = kill(None, "world", KillCause::World);
        assert_eq!(copy(&fall), "01:46.817 | -<[ATK]>-naz died from world");

        let own = kill(Some("-<[ATK]>-naz"), "rpg_rocket", KillCause::Suicide);
        assert_eq!(
            copy(&own),
            "01:46.817 | -<[ATK]>-naz killed themselves with rpg_rocket"
        );
    }

    #[test]
    fn copied_chat_reads_as_its_sender_and_line() {
        let catalog = Catalog::new(crate::i18n::Locale::En);
        let chat = Event {
            segment: 0,
            time: 7.25,
            lane: 0,
            actor: Some("^2andriu".into()),
            weapon: None,
            kind: EventKind::Chat {
                text: "gg".into(),
                place: None,
            },
        };
        assert_eq!(copied(&catalog, &chat, "gg", 50.), "07.250 | andriu: gg");
    }

    #[test]
    fn event_times_leave_out_what_the_demo_never_reaches() {
        // 1:46.817 in demos under a minute, under an hour and longer
        assert_eq!(event_clock(7.25, 50.), "07.250");
        assert_eq!(event_clock(106.817, 1200.), "01:46.817");
        assert_eq!(event_clock(106.817, 4000.), "00:01:46.817");

        // A demo exactly a minute or an hour long reaches it
        assert_eq!(event_clock(60., 60.), "01:00.000");
        assert_eq!(event_clock(3600., 3600.), "01:00:00.000");
    }

    /// A demo view whose list shows `count` shots a second apart
    fn view_of_shots(ctx: &egui::Context, count: usize) -> DemoView {
        let shot = |time: f32| Event {
            segment: 0,
            time,
            lane: 0,
            actor: None,
            weapon: Some(Weapon::Gauss),
            kind: EventKind::Action {
                action: Action::Fire,
                mode: None,
            },
        };
        let events = (0..count).map(|i| shot(i as f32)).collect();

        let mut view = DemoView::open(PathBuf::from("not-there.dem"), 100., None, ctx);
        view.loader = None;
        view.model = Some(Arc::new(Model {
            events,
            end: 100.,
            ..Default::default()
        }));
        view.rows = (0..count).collect();
        view
    }

    #[test]
    fn ctrl_adds_and_takes_away_and_shift_chooses_a_run() {
        let ctx = egui::Context::default();
        let mut view = view_of_shots(&ctx, 5);
        view.selected = Some(1);

        // The first Ctrl click keeps the selected event chosen
        view.toggle_mark(3);
        assert_eq!(view.marked, BTreeSet::from([1, 3]));
        view.toggle_mark(1);
        assert_eq!(view.marked, BTreeSet::from([3]));

        // Shift runs from the last event clicked, in either direction
        view.mark_through(0);
        assert_eq!(view.marked, BTreeSet::from([0, 1]));
        view.mark_through(4);
        assert_eq!(view.marked, BTreeSet::from([1, 2, 3, 4]));
    }

    #[test]
    fn ctrl_a_chooses_every_shown_event_and_ctrl_c_copies_them_in_order() {
        let ctx = egui::Context::default();
        let mut view = view_of_shots(&ctx, 3);
        let ctrl_a = egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        };
        draw_list(&ctx, &mut view, vec![ctrl_a]);
        assert_eq!(view.marked, BTreeSet::from([0, 1, 2]));

        let catalog = Catalog::new(crate::i18n::Locale::En);
        let model = view.model.clone().expect("the test gives a model");
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.))),
            events: vec![egui::Event::Copy],
            ..Default::default()
        };
        let output = ctx.run_ui(input, |ui| view.events_list(ui, &catalog, false, &model));
        let copied = output
            .platform_output
            .commands
            .iter()
            .find_map(|c| match c {
                egui::OutputCommand::CopyText(text) => Some(text.clone()),
                _ => None,
            });
        let lines = [
            "00:00.000 | Unknown player · Fire",
            "00:01.000 | Unknown player · Fire",
            "00:02.000 | Unknown player · Fire",
        ];
        assert_eq!(copied.as_deref(), Some(lines.join("\n").as_str()));
    }

    /// One frame of `view`'s list on a small screen, after `events`
    fn draw_list(ctx: &egui::Context, view: &mut DemoView, events: Vec<egui::Event>) {
        let catalog = Catalog::new(crate::i18n::Locale::En);
        let model = view.model.clone().expect("the test gives a model");
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.))),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| view.events_list(ui, &catalog, false, &model));
        output.textures_delta.clear();
    }

    #[test]
    fn scrolling_the_list_by_hand_stops_it_following_the_playhead() {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut view = view_of_shots(&ctx, 100);

        // Left alone, or with the pointer resting on it, it keeps following
        draw_list(&ctx, &mut view, Vec::new());
        let over_list = egui::Event::PointerMoved(Pos2::new(200., 150.));
        draw_list(&ctx, &mut view, vec![over_list.clone()]);
        assert!(view.follow_playhead);

        // A turn of the wheel over it
        let wheel = egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0., -200.),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Wheel,
            modifiers: egui::Modifiers::NONE,
        };
        draw_list(&ctx, &mut view, vec![over_list, wheel]);
        for _ in 0..30 {
            draw_list(&ctx, &mut view, Vec::new());
        }
        assert!(view.list_offset > 0.);
        assert!(!view.follow_playhead);
    }

    #[test]
    fn a_wheel_turn_while_the_list_is_placed_still_stops_it_following() {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut view = view_of_shots(&ctx, 100);
        draw_list(&ctx, &mut view, Vec::new());

        // The list is placed on the chosen event in the very frame the wheel
        // turns, as fast play places it on the playhead
        view.selected = Some(50);
        view.scroll_table = true;
        let over_list = egui::Event::PointerMoved(Pos2::new(200., 150.));
        let wheel = egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0., -200.),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Wheel,
            modifiers: egui::Modifiers::NONE,
        };
        draw_list(&ctx, &mut view, vec![over_list, wheel]);
        assert!(!view.follow_playhead);
    }
}
