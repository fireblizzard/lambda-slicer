//! The Clips tab, each clip a card with its name, its start and end, the
//! ways to play, show, export and remove it, and how its export went (GUI
//! redesign plan §10.2)
//!
//! The buttons to add and export clips stay at the top and where exports go
//! at the bottom, and only the cards scroll. A time typed into a card is
//! kept when its field loses the focus, which the click on Export does too,
//! so exports read the clips only once every card has drawn

use eframe::egui::{self, Sense, Stroke, Ui};
use fluent_bundle::FluentValue;
use lambda_slicer_core::timeline::time::DemoTime;

use super::{DemoAction, DemoView, Drag, Edge, Request, secs};
use crate::app::SettingsTab;
use crate::clips::parse_time;
use crate::export::ExportState;
use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Icon, IconButton, Variant};

/// A time field holds this many digits' width, enough for HH:MM:SS.mmm
const TIME_DIGITS: f32 = 13.;

/// The room around a card's contents, the gap between cards, and how thick
/// the chosen card's outline is
const CARD_PADDING: i8 = theme::GAP_S as i8;
const CARD_GAP: f32 = theme::GAP_S;
const CHOSEN_OUTLINE: f32 = 1.;

/// The thickness of the outline round a time that does not read as one
const BAD_TIME_OUTLINE: f32 = 1.5;

/// The room above and below where exports go
const FOOTER_MARGIN: i8 = theme::GAP_XS as i8;

/// Clips to export, acted on once every card has drawn
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExportRequest {
    All,
    One(usize),
}

/// What a card's buttons asked for
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardAction {
    Remove,
    Export,
    /// Play the clip once
    Play,
    /// Loop the clip, or stop looping it
    Loop(bool),
}

impl DemoView {
    /// The buttons to add a clip and to export them all, the cards, and
    /// where exports go
    pub(super) fn clips_tab(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let mut request = None;
        self.clips_toolbar(ui, catalog, rtl, &mut request);
        ui.separator();

        // Where exports go stays at the bottom, under the cards
        let footer = egui::Frame::new().inner_margin(egui::Margin::symmetric(0, FOOTER_MARGIN));
        egui::Panel::bottom("clips destination")
            .frame(footer)
            .resizable(false)
            .show(ui, |ui| self.export_destination(ui, catalog, rtl));

        egui::ScrollArea::vertical()
            .id_salt(("clips", self.generation))
            .auto_shrink(false)
            .show(ui, |ui| {
                if self.clips.is_empty() {
                    let hint = catalog.text("clips-empty-hint");
                    let width = Some(ui.available_width());
                    let hint =
                        text::widget_text(ui.ctx(), &hint, rtl, width, Look::Color(theme::MUTED));
                    ui.add(egui::Label::new(hint).wrap());
                    return;
                }

                let mut remove = None;
                for i in 0..self.clips.len() {
                    match self.clip_card(ui, catalog, rtl, i) {
                        Some(CardAction::Remove) => remove = Some(i),
                        Some(CardAction::Export) => request = Some(ExportRequest::One(i)),
                        Some(CardAction::Play) => self.play_clip(i),
                        Some(CardAction::Loop(on)) => self.loop_clip(i, on),
                        None => {}
                    }
                    ui.add_space(CARD_GAP);
                }

                if let Some(i) = remove {
                    self.remove_clip(i);
                }
            });

        // A time typed into a card is kept as the click on Export takes its
        // field's focus, while the cards draw, so exports read the clips
        // only now
        if let Some(request) = request {
            self.export_clips(ui.ctx(), request);
        }
    }

    /// New clip at the leading end, and exporting every clip or stopping the
    /// export under way at the trailing end
    fn clips_toolbar(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        request: &mut Option<ExportRequest>,
    ) {
        let label = |ui: &Ui, id: &str| {
            text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain)
        };

        layout::row(ui, rtl, |ui| {
            let new = widgets::Button::new(
                Some(Icon::Plus),
                label(ui, "clips-new"),
                Variant::Default,
                rtl,
            );
            let new = ui.add(new).on_hover_text(catalog.text("clips-new-help"));
            if new.clicked() {
                self.actions.push(DemoAction::NewClip);
            }

            layout::trailing(ui, rtl, |ui| match &self.export {
                Some(job) => {
                    let stop = widgets::Button::new(
                        None,
                        label(ui, "export-cancel"),
                        Variant::Default,
                        rtl,
                    );
                    if ui.add_enabled(!job.cancelling(), stop).clicked() {
                        job.cancel();
                    }
                }
                None => {
                    let all = label(ui, "export-all");
                    let all =
                        widgets::Button::new(Some(Icon::Download), all, Variant::Primary, rtl);
                    if ui.add_enabled(!self.clips.is_empty(), all).clicked() {
                        *request = Some(ExportRequest::All);
                    }
                }
            });
        });
    }

    /// Where exports go, a click away from the setting that chooses it
    fn export_destination(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let folder = FluentValue::from(self.export_dir().display().to_string());
        let id = if self.export_folder.is_some() {
            "export-folder"
        } else {
            "export-folder-beside"
        };
        let line = catalog.text_with(id, &[("folder", folder)]);

        layout::row(ui, rtl, |ui| {
            let change = catalog.text("export-destination");
            let button = ui.add(IconButton::new(Icon::Folder, &change));
            let line = text::styled_text(
                ui.ctx(),
                &line,
                rtl,
                Look::Color(theme::MUTED),
                &theme::secondary(),
            );
            let label = ui.add(egui::Label::new(line).truncate().sense(Sense::click()));
            if button.clicked() || label.clicked() {
                self.requests.push(Request::Settings(SettingsTab::Clips));
            }
        });
    }

    /// One clip's card. Returns what its buttons asked for
    fn clip_card(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        i: usize,
    ) -> Option<CardAction> {
        let chosen = self.chosen_clip == Some(i);
        let outline = if chosen {
            Stroke::new(CHOSEN_OUTLINE, theme::ACCENT)
        } else {
            Stroke::NONE
        };
        let frame = egui::Frame::new()
            .fill(theme::SURFACE)
            .stroke(outline)
            .corner_radius(theme::RADIUS_CARD)
            .inner_margin(CARD_PADDING);

        let mut action = None;
        let card = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.card_name(ui, catalog, rtl, i);
            self.card_times(ui, catalog, rtl, i);
            action = self.card_buttons(ui, catalog, rtl, i);
            self.card_outcome(ui, catalog, rtl, i);
        });

        // A click anywhere on the card chooses it
        if card.response.interact(Sense::click()).clicked() {
            self.chosen_clip = Some(i);
        }

        action
    }

    /// The clip's name, and how long it asks to be at the trailing end
    fn card_name(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, i: usize) {
        let length = self.clips[i].length().to_string();
        let length = egui::RichText::new(length).monospace().color(theme::MUTED);
        let length_width = {
            let galley = egui::WidgetText::from(length.clone()).into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                egui::TextStyle::Monospace,
            );
            galley.size().x + ui.spacing().item_spacing.x
        };

        layout::row(ui, rtl, |ui| {
            let room = (ui.available_width() - length_width).max(0.);
            let name = egui::TextEdit::singleline(&mut self.clips[i].name)
                .desired_width(room)
                .hint_text(catalog.text("clip-name"));
            let name = ui.add(name);
            if name.changed() {
                self.clips_changed = true;
            }
            if name.gained_focus() {
                self.chosen_clip = Some(i);
            }

            let duration = FluentValue::from(self.clips[i].length().to_string());
            let help = catalog.text_with("clip-length", &[("duration", duration)]);
            ui.label(length).on_hover_text(help);
        });
    }

    /// The start and the end, each with the way to set it to the playhead,
    /// side by side where they fit
    fn card_times(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, i: usize) {
        layout::wrapping_row(ui, rtl, |ui| {
            for (edge, id) in [(Edge::Start, "clip-in"), (Edge::End, "clip-out")] {
                let salt = format!("clip time {i} {id}");
                layout::unbroken(ui, &salt, |ui| {
                    widgets::section_title(ui, &catalog.text(id), rtl);
                    self.time_field(ui, catalog, i, edge);
                    self.set_to_playhead(ui, catalog, i, edge);
                });
            }
        });
    }

    /// The button setting an edge of the clip to where the playhead is,
    /// kept as typing it would be
    fn set_to_playhead(&mut self, ui: &mut Ui, catalog: &Catalog, i: usize, edge: Edge) {
        let id = match edge {
            Edge::Start => "clip-set-in",
            Edge::End => "clip-set-out",
        };
        let label = catalog.text(id);
        let playhead = self.viewer.as_ref().map(|v| v.time());
        let button = ui
            .add_enabled(playhead.is_some(), IconButton::new(Icon::Crosshair, &label))
            .on_disabled_hover_text(catalog.text("clip-no-playhead"));
        if button.clicked() {
            self.edge_to_playhead(i, edge);
        }
    }

    /// Set an edge of clip `i` to the current moment, to the whole
    /// millisecond as the fields show it. Nothing changes without a viewer
    /// to give the moment
    pub(super) fn edge_to_playhead(&mut self, i: usize, edge: Edge) {
        let Some(time) = self.viewer.as_ref().map(|v| v.time()) else {
            return;
        };
        let Some(clip) = self.clips.get_mut(i) else {
            return;
        };

        // Whole milliseconds, as the fields show them
        let ms = (f64::from(time.max(0.)) * 1000.).round() as i64;
        let time = DemoTime::from_micros(ms * 1000);
        match edge {
            Edge::Start => clip.start = time,
            Edge::End => clip.end = time,
        }
        self.chosen_clip = Some(i);
        self.typing = None;
        self.clips_changed = true;
    }

    /// Play, loop and show the clip, the rounding of a cut that is not
    /// exact behind a help, and export and remove it at the trailing end
    fn card_buttons(
        &mut self,
        ui: &mut Ui,
        catalog: &Catalog,
        rtl: bool,
        i: usize,
    ) -> Option<CardAction> {
        let clip = self.clips[i].clone();
        let snapped = self.snap(&clip);
        let can_export = snapped.is_ok() && self.export.is_none();
        let mut action = None;

        layout::row(ui, rtl, |ui| {
            if self.viewer.is_some() {
                let play = catalog.text("clip-play");
                if ui.add(IconButton::new(Icon::Play, &play)).clicked() {
                    action = Some(CardAction::Play);
                }

                let looping = self.looping == Some(i);
                let label = catalog.text("clip-loop");
                let toggle = IconButton::new(Icon::Repeat, &label).selected(looping);
                if ui.add(toggle).clicked() {
                    action = Some(CardAction::Loop(!looping));
                }
            }

            let show = catalog.text("clip-show-on-timeline");
            if ui.add(IconButton::new(Icon::Fit, &show)).clicked() {
                self.show_clip(i);
            }

            // Where the cut really falls, when the frames move it
            if let Ok(s) = &snapped
                && (s.start != clip.start || s.end != clip.end)
            {
                let length = DemoTime::from_micros((s.end.micros() - s.start.micros()).max(0));
                let args = [
                    ("start", FluentValue::from(s.start.to_string())),
                    ("end", FluentValue::from(s.end.to_string())),
                    ("duration", FluentValue::from(length.to_string())),
                ];
                widgets::help(ui, &catalog.text_with("clip-effective", &args), rtl);
            }

            layout::trailing(ui, rtl, |ui| {
                let export = catalog.text("export-one");
                let export = text::widget_text(ui.ctx(), &export, rtl, None, Look::Plain);
                let export =
                    widgets::Button::new(Some(Icon::Download), export, Variant::Default, rtl);
                if ui.add_enabled(can_export, export).clicked() {
                    action = Some(CardAction::Export);
                }

                let remove = catalog.text("clip-remove");
                if ui.add(IconButton::new(Icon::Trash, &remove)).clicked() {
                    action = Some(CardAction::Remove);
                }
            });
        });

        action
    }

    /// Why the clip cannot be exported, and how its latest export went
    fn card_outcome(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, i: usize) {
        let clip = self.clips[i].clone();
        let width = Some(ui.available_width());
        let warning = Look::Color(theme::WARNING);

        // A refusal is never only behind a hover
        if let Err(reason) = self.snap(&clip) {
            let line = text::widget_text(ui.ctx(), &catalog.message(&reason), rtl, width, warning);
            ui.add(egui::Label::new(line).wrap());
        }

        let Some(id) = self.clip_ids.get(i) else {
            return;
        };
        let Some((range, state)) = self.exports.get(id).cloned() else {
            return;
        };

        // A file of the range the clip had before it was edited says so
        if range != (clip.start, clip.end) {
            let args = [
                ("start", FluentValue::from(range.0.to_string())),
                ("end", FluentValue::from(range.1.to_string())),
            ];
            let line = catalog.text_with("export-earlier-range", &args);
            let line = text::widget_text(ui.ctx(), &line, rtl, width, Look::Weak);
            ui.add(egui::Label::new(line).wrap());
        }
        export_state(ui, catalog, rtl, &state);
    }

    /// A clip's start or end as text, read back when typing ends. Text that
    /// is not a time is outlined and left as it was, and the clip keeps its
    /// time
    fn time_field(&mut self, ui: &mut Ui, catalog: &Catalog, i: usize, edge: Edge) {
        let current = match edge {
            Edge::Start => self.clips[i].start,
            Edge::End => self.clips[i].end,
        };
        let mut text = match &self.typing {
            Some((c, e, t)) if *c == i && *e == edge => t.clone(),
            _ => current.to_string(),
        };

        // Wide enough for HH:MM:SS.mmm, and wider for longer text
        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let width = ui.fonts_mut(|f| f.glyph_width(&font, '0')) * TIME_DIGITS;
        let field = egui::TextEdit::singleline(&mut text)
            .desired_width(width)
            .clip_text(false)
            .font(egui::TextStyle::Monospace);
        let r = ui.add(field);
        if r.changed() {
            self.typing = Some((i, edge, text.clone()));
        }
        if r.gained_focus() {
            self.chosen_clip = Some(i);
        }

        let parsed = parse_time(&text);
        if parsed.is_none() {
            let stroke = Stroke::new(BAD_TIME_OUTLINE, ui.visuals().error_fg_color);
            ui.painter().rect_stroke(
                r.rect,
                theme::RADIUS_SMALL,
                stroke,
                egui::StrokeKind::Outside,
            );
        }

        let r = r.on_hover_text(catalog.text("clip-time-help"));
        if !r.lost_focus() {
            return;
        }
        if let Some(t) = parsed
            && t != current
        {
            match edge {
                Edge::Start => self.clips[i].start = t,
                Edge::End => self.clips[i].end = t,
            }
            self.clips_changed = true;
            if let Some(viewer) = &mut self.viewer {
                viewer.preview(secs(t));
            }
        }
        self.typing = None;
    }

    /// Play a clip once, through any clip's loop
    fn play_clip(&mut self, i: usize) {
        self.looping = None;
        if let Some(viewer) = &mut self.viewer {
            let c = &self.clips[i];
            viewer.set_loop(None);
            viewer.play_range(secs(c.start), secs(c.end));
        }
    }

    /// Loop a clip from its start, or stop looping it
    fn loop_clip(&mut self, i: usize, on: bool) {
        self.looping = on.then_some(i);
        let Some(viewer) = &mut self.viewer else {
            return;
        };

        let c = &self.clips[i];
        if on {
            viewer.play_loop(secs(c.start), secs(c.end));
        } else {
            viewer.set_loop(None);
        }
    }

    /// Export one clip, or every clip that can be, marking those that cannot
    /// as left out with why
    pub(super) fn export_clips(&mut self, ctx: &egui::Context, request: ExportRequest) {
        let asked: Vec<usize> = match request {
            ExportRequest::All => (0..self.clips.len()).collect(),
            ExportRequest::One(i) => vec![i],
        };

        let mut eligible = Vec::new();
        for i in asked {
            let Some(clip) = self.clips.get(i).cloned() else {
                continue;
            };
            match self.snap(&clip) {
                Ok(_) => eligible.push(i),
                Err(reason) => self.mark_export(i, ExportState::Skipped(reason)),
            }
        }

        self.start_export(ctx, &eligible);
    }

    /// Remove a clip, its loop, its export result and any drag of its edge
    /// with it, keeping the loop and the choice on the clips after it
    pub(super) fn remove_clip(&mut self, i: usize) {
        if i >= self.clips.len() {
            return;
        }

        self.clips.remove(i);
        if i < self.clip_ids.len() {
            let id = self.clip_ids.remove(i);
            self.exports.remove(&id);
        }
        self.chosen_clip = match self.chosen_clip {
            Some(c) if c == i => None,
            Some(c) if c > i => Some(c - 1),
            c => c,
        };
        self.typing = None;
        self.clips_changed = true;

        // The removed clip's loop goes with it, and a later clip's loop moves
        // up a place with its clip
        self.looping = match self.looping {
            Some(c) if c == i => {
                if let Some(viewer) = &mut self.viewer {
                    viewer.set_loop(None);
                }
                None
            }
            Some(c) if c > i => Some(c - 1),
            c => c,
        };

        // The clips after it move up a place, so a drag of a clip's edge
        // could now move another clip's, and it is given up
        if matches!(self.drag, Some(Drag::ClipEdge { .. })) {
            self.drag = Some(Drag::Cancelled);
        }
    }
}

/// How a clip's export went, and for a saved file how to play it
fn export_state(ui: &mut Ui, catalog: &Catalog, rtl: bool, state: &ExportState) {
    let ctx = ui.ctx().clone();
    let width = Some(ui.available_width());
    let line = |ui: &mut Ui, s: &str, look: Look| {
        let line = text::widget_text(&ctx, s, rtl, width, look);
        ui.add(egui::Label::new(line).wrap());
    };
    let warning = Look::Color(theme::WARNING);

    match state {
        ExportState::Waiting => line(ui, &catalog.text("export-waiting"), Look::Weak),
        ExportState::Running => {
            layout::row(ui, rtl, |ui| {
                ui.spinner();
                line(ui, &catalog.text("export-running"), Look::Plain);
            });
        }
        ExportState::Cancelled => line(ui, &catalog.text("export-cancelled"), Look::Weak),
        ExportState::Failed(m) => line(ui, &catalog.message(m), warning),
        ExportState::Skipped(m) => {
            let reason = FluentValue::from(catalog.message(m));
            line(
                ui,
                &catalog.text_with("export-skipped", &[("reason", reason)]),
                warning,
            );
        }
        ExportState::Done(e) => {
            let args = [
                ("path", FluentValue::from(e.path.display().to_string())),
                ("count", FluentValue::from(e.compared as f64)),
            ];
            line(
                ui,
                &catalog.text_with("export-saved", &args),
                Look::Color(theme::SUCCESS),
            );

            // The console command, or where the file must go first
            let name = e
                .path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let (hint, command) = match &e.playdemo {
                Some(c) => (catalog.text("export-playdemo"), c.clone()),
                None => {
                    let file = FluentValue::from(format!("{name}.dem"));
                    let hint = catalog.text_with("export-copy-first", &[("name", file)]);
                    (hint, format!("playdemo {name}"))
                }
            };
            line(ui, &hint, Look::Weak);

            // The command is cut short rather than widening the tab, and
            // copied whole
            layout::row(ui, rtl, |ui| {
                let copy_room = theme::CONTROL_HEIGHT + ui.spacing().item_spacing.x;
                let room = (ui.available_width() - copy_room).max(0.);
                layout::cell(ui, rtl, room, |ui| {
                    ui.set_max_width(room);
                    let text = egui::RichText::new(&command).monospace().strong();
                    ui.add(egui::Label::new(text).truncate())
                        .on_hover_text(&command);
                });

                let copy = catalog.text("export-copy");
                if ui.add(IconButton::new(Icon::Copy, &copy)).clicked() {
                    ui.ctx().copy_text(command.clone());
                }
            });
        }
    }
}
