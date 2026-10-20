//! The settings window, which can be moved anywhere in the app's window,
//! with its four sections down one side and the chosen one beside them (GUI
//! redesign plan §11)
//!
//! Every change applies at once, with no Save to press, and is written out
//! once nothing is being dragged. Each section keeps its own scroll place,
//! and the window keeps where it was left for the next run

use eframe::egui::{self, Align, Layout, Pos2, Vec2};
use fluent_bundle::FluentValue;

use super::App;
use crate::game_setup;
use crate::i18n::{Catalog, Locale};
use crate::layout;
use crate::library::parse_number;
use crate::settings::{FOV, MAX_PADDING, ViewerSettings};
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Icon, Variant};

/// The window's first size, the width of its list of sections, and the
/// least room a control takes beside its label before going under it
const WINDOW_SIZE: Vec2 = Vec2::new(600., 460.);
const SIDEBAR_WIDTH: f32 = 144.;
const CONTROL_LEAST: f32 = 160.;

/// The room above a heading that is not the first, and between settings
/// under one heading
const HEADING_GAP: f32 = theme::GAP_M;
const SETTING_GAP: f32 = theme::GAP_XS;

/// The steps of the picture settings and of the field of view as they are
/// dragged, the decimals they show, and the field of view a manual one
/// starts at
const PICTURE_STEP: f64 = 0.01;
const PICTURE_DECIMALS: usize = 2;
const FOV_STEP: f64 = 0.5;
const FIRST_FOV: f32 = 90.;

/// The step and decimals of a clip's padding
const PADDING_STEP: f64 = 0.1;
const PADDING_DECIMALS: usize = 1;

/// The settings window's sections
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SettingsTab {
    /// The game, the language and how many demos are analysed at once
    #[default]
    General,
    /// The folders scanned for demos, and where what is found is kept
    Library,
    /// What changes only how the demo viewer shows and plays a demo
    Viewer,
    /// Where clips are exported, and what a clip keeps around its event
    Clips,
}

impl App {
    /// The settings window while it is open, placed where it was left or in
    /// the middle, and kept inside the app's window
    pub(super) fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            self.settings_keyboard = false;
            return;
        }

        let content = ctx.content_rect();
        let mut open = true;
        let title = self.t(ctx, "settings-title");
        let mut window = egui::Window::new(title)
            .id(egui::Id::unique("settings window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size(WINDOW_SIZE)
            .constrain_to(content);
        window = match self.settings.layout.settings_at {
            Some([x, y]) => window.default_pos(Pos2::new(x, y)),
            // In the middle, as far as the app's window has room for it
            None => {
                let corner = content.center() - WINDOW_SIZE / 2.;
                window.default_pos(corner.max(content.min))
            }
        };

        let rtl = self.rtl();
        let shown = window.show(ctx, |ui| self.settings_contents(ui, rtl));
        self.show_settings = open;

        let Some(shown) = shown else {
            self.settings_keyboard = false;
            return;
        };

        // Where it was left, for the next run
        let at = shown.response.rect.min;
        if self.settings.layout.settings_at != Some([at.x, at.y]) {
            self.settings.layout.settings_at = Some([at.x, at.y]);
            self.settings_unsaved = true;
        }

        // The keys are the window's while the pointer is over it or one of
        // its controls has the focus
        let layer = shown.response.layer_id;
        let focused_here = ctx
            .memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            .is_some_and(|r| r.layer_id == layer);
        self.settings_keyboard = shown.response.contains_pointer() || focused_here;
    }

    /// The list of sections down the leading side, and the chosen one
    /// beside it in a scroll area of its own
    fn settings_contents(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let sections = [
            (SettingsTab::General, "settings-tab-general"),
            (SettingsTab::Library, "settings-tab-library"),
            (SettingsTab::Viewer, "settings-viewer"),
            (SettingsTab::Clips, "settings-tab-clips"),
        ];
        let height = ui.available_height();
        let direction = if rtl {
            Layout::right_to_left(Align::Min)
        } else {
            Layout::left_to_right(Align::Min)
        };

        ui.with_layout(direction, |ui| {
            let sidebar = Vec2::new(SIDEBAR_WIDTH, height);
            let column = Layout::top_down_justified(if rtl { Align::Max } else { Align::Min });
            ui.allocate_ui_with_layout(sidebar, column, |ui| {
                for (section, id) in sections {
                    let chosen = self.settings_tab == section;
                    let label = self.t(ui.ctx(), id);
                    if ui.add(egui::Button::selectable(chosen, label)).clicked() {
                        self.settings_tab = section;
                    }
                }
            });
            ui.separator();

            // Each section scrolls on its own and keeps its place
            let section = self.settings_tab;
            egui::ScrollArea::vertical()
                .id_salt(("settings section", section))
                .auto_shrink(false)
                .show(ui, |ui| {
                    layout::column(ui, rtl, |ui| match section {
                        SettingsTab::General => self.general_section(ui, rtl),
                        SettingsTab::Library => self.library_section(ui, rtl),
                        SettingsTab::Viewer => self.viewer_section(ui, rtl),
                        SettingsTab::Clips => self.clips_section(ui, rtl),
                    });
                });
        });
    }

    /// The game the viewer reads, the language, and how many demos are
    /// analysed at once
    fn general_section(&mut self, ui: &mut egui::Ui, rtl: bool) {
        heading(ui, &self.catalog, "settings-section-game", None, rtl, true);
        self.installation_settings(ui, rtl);

        // A draft translation says so beside the heading
        let unreviewed = (!self.catalog.locale.reviewed()).then_some("settings-unreviewed");
        heading(
            ui,
            &self.catalog,
            "settings-language",
            unreviewed,
            rtl,
            false,
        );
        self.language_menu(ui, rtl);

        heading(
            ui,
            &self.catalog,
            "settings-workers",
            Some("settings-workers-hint"),
            rtl,
            false,
        );
        let mut workers = self.settings.workers;
        let moved = layout::row(ui, rtl, |ui| {
            ui.add(egui::Slider::new(&mut workers, 1..=4)).changed()
        });
        if moved {
            self.settings.workers = workers;
            self.settings_unsaved = true;
        }
    }

    /// The system's language first, which follows the system as it
    /// changes, then every language by its own name
    fn language_menu(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let system = FluentValue::from(Locale::detect().native_name());
        let system = self
            .catalog
            .text_with("settings-language-system", &[("name", system)]);
        let current = match self.settings.locale {
            Some(locale) => locale.native_name().to_string(),
            None => system.clone(),
        };

        let mut chosen = None;
        layout::row(ui, rtl, |ui| {
            let selected = text::widget_text(ui.ctx(), &current, rtl, None, Look::Plain);
            egui::ComboBox::from_id_salt("language")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    let label = text::widget_text(ui.ctx(), &system, rtl, None, Look::Plain);
                    let following = self.settings.locale.is_none();
                    if ui.selectable_label(following, label).clicked() {
                        chosen = Some(None);
                    }

                    for locale in Locale::ALL {
                        let name = text::widget_text(
                            ui.ctx(),
                            locale.native_name(),
                            false,
                            None,
                            Look::Plain,
                        );
                        let picked = self.settings.locale == Some(locale);
                        if ui.selectable_label(picked, name).clicked() {
                            chosen = Some(Some(locale));
                        }
                    }
                });
        });

        if let Some(locale) = chosen
            && locale != self.settings.locale
        {
            self.set_locale(locale);
        }
    }

    /// The Half-Life folder the viewer reads, and on Windows the game's
    /// executable that finds it, both checked the same way as the setup
    /// notice's choice. A choice that holds no game is refused with the
    /// reason, keeping the one before
    fn installation_settings(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        let setup_needed = self.install_root().is_none();
        let mut chosen = None;

        labelled(
            ui,
            &self.catalog,
            "settings-viewer-game",
            Some("settings-viewer-game-hint"),
            rtl,
        );
        let shown = match &self.settings.viewer.game {
            Some(f) => f.display().to_string(),
            None => self.catalog.text("settings-viewer-game-none"),
        };
        path(ui, &shown, rtl);
        layout::row(ui, rtl, |ui| {
            if ui.button(self.t(&ctx, "settings-export-choose")).clicked() {
                let start = self.settings.viewer.game.clone().filter(|g| g.is_dir());
                let mut dialog = rfd::FileDialog::new();
                if let Some(start) = start {
                    dialog = dialog.set_directory(start);
                }
                chosen = dialog.pick_folder();
            }
        });

        // The executable only finds the folder, so a folder that holds the
        // game needs none, and Linux chooses the folder itself
        if cfg!(windows) {
            ui.add_space(SETTING_GAP);
            labelled(
                ui,
                &self.catalog,
                "settings-executable",
                Some("settings-executable-hint"),
                rtl,
            );
            let present = self.executable_present();
            let (line, look) = match (&self.settings.viewer.executable, present) {
                (Some(e), Some(true)) => (e.display().to_string(), Look::Weak),
                (Some(e), _) => {
                    let gone = FluentValue::from(e.display().to_string());
                    let line = self
                        .catalog
                        .text_with("settings-executable-missing", &[("path", gone)]);
                    (line, Look::Color(theme::WARNING))
                }
                (None, _) if setup_needed => (
                    self.catalog.text("settings-executable-not-set"),
                    Look::Color(theme::WARNING),
                ),
                (None, _) => (
                    self.catalog.text("settings-executable-optional"),
                    Look::Weak,
                ),
            };
            let width = Some(ui.available_width());
            ui.add(egui::Label::new(text::widget_text(&ctx, &line, rtl, width, look)).wrap());

            // Highlighted only while the viewer has no game files
            let variant = if setup_needed {
                Variant::Primary
            } else {
                Variant::Default
            };
            layout::row(ui, rtl, |ui| {
                let label = self.t(&ctx, "settings-browse");
                let browse = widgets::Button::new(None, label, variant, rtl);
                if ui.add(browse).clicked() {
                    let start = self.settings.viewer.game.clone();
                    let filter = self.catalog.text("dialog-executables");
                    chosen = game_setup::pick(start.as_deref(), &filter);
                }
            });
        }

        if let Some(chosen) = chosen {
            self.accept_install(&chosen);
        }

        // Why the last choice was refused, where it was made
        if let Some(refusal) = &self.setup_refusal {
            let line = refusal.message(&self.catalog);
            let width = Some(ui.available_width());
            let look = Look::Color(theme::ERROR);
            ui.add(egui::Label::new(text::widget_text(&ctx, &line, rtl, width, look)).wrap());
        }

        // A folder given for this run comes first, and a choice here waits
        // for the next run
        if let Some(game) = &self.game {
            let game = FluentValue::from(game.display().to_string());
            let line = self
                .catalog
                .text_with("settings-game-override", &[("path", game)]);
            let width = Some(ui.available_width());
            ui.add(egui::Label::new(text::widget_text(&ctx, &line, rtl, width, Look::Weak)).wrap());
        }
    }

    /// The folders scanned for demos, removed without touching their files,
    /// and where what is found about them is kept
    fn library_section(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        heading(
            ui,
            &self.catalog,
            "settings-folders",
            Some("settings-folders-hint"),
            rtl,
            true,
        );
        if self.settings.folders.is_empty() {
            path(ui, &self.catalog.text("settings-folders-empty"), rtl);
        }

        let mut remove = None;
        for (i, folder) in self.settings.folders.iter().enumerate() {
            layout::row(ui, rtl, |ui| {
                let shown = folder.display().to_string();
                ui.label(text::widget_text(ui.ctx(), &shown, rtl, None, Look::Weak));
                if ui
                    .small_button(self.t(ui.ctx(), "settings-remove"))
                    .clicked()
                {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            self.settings.folders.remove(i);
            self.settings_unsaved = true;
        }

        let add = layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "toolbar-add-folder");
            let add = widgets::Button::new(Some(Icon::AddFolder), label, Variant::Default, rtl);
            ui.add(add).clicked()
        });
        if add && let Some(folders) = rfd::FileDialog::new().pick_folders() {
            self.add_folders(&ctx, folders);
        }

        heading(ui, &self.catalog, "settings-cache", None, rtl, false);
        let cache = self.cache_path.as_ref().map(|p| p.display().to_string());
        path(ui, &cache.unwrap_or_default(), rtl);
    }

    /// Every setting that changes only how the viewer shows and plays a
    /// demo, applied as it changes without opening the demo again
    fn viewer_section(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        let catalog = &self.catalog;
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
        let mut viewer = self.settings.viewer.clone();

        // The game's gamma settings, at the game's steps
        heading(
            ui,
            catalog,
            "settings-section-picture",
            Some("settings-gamma-hint"),
            rtl,
            true,
        );
        let ids = [
            "settings-gamma",
            "settings-brightness",
            "settings-texgamma",
            "settings-lightgamma",
        ];
        let width = label_width(ui, catalog, &ids, rtl);
        let values = [
            (&mut viewer.gamma, 1.8..=3.),
            (&mut viewer.brightness, 0. ..=3.),
            (&mut viewer.texgamma, 1.8..=3.),
            (&mut viewer.lightgamma, 1.8..=3.),
        ];
        for (id, (value, range)) in ids.into_iter().zip(values) {
            field(ui, catalog, id, width, rtl, |ui| {
                ui.add(
                    egui::DragValue::new(value)
                        .range(range)
                        .speed(PICTURE_STEP)
                        .custom_formatter(|v, _| catalog.number(v, PICTURE_DECIMALS))
                        .custom_parser(parse_number),
                );
            });
        }
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.overbright, t("settings-overbright"));
        });

        // Only the picture goes back to the game's own
        layout::row(ui, rtl, |ui| {
            if ui.button(t("settings-gamma-defaults")).clicked() {
                let d = ViewerSettings::default();
                (viewer.gamma, viewer.brightness) = (d.gamma, d.brightness);
                (viewer.texgamma, viewer.lightgamma) = (d.texgamma, d.lightgamma);
                viewer.overbright = d.overbright;
            }
        });

        heading(ui, catalog, "settings-section-view", None, rtl, false);
        layout::row(ui, rtl, |ui| {
            let mut recorded = viewer.fov.is_none();
            if ui
                .checkbox(&mut recorded, t("settings-fov-recorded"))
                .changed()
            {
                viewer.fov = (!recorded).then_some(FIRST_FOV);
            }
        });
        if let Some(fov) = &mut viewer.fov {
            let width = label_width(ui, catalog, &["settings-fov"], rtl);
            field(ui, catalog, "settings-fov", width, rtl, |ui| {
                ui.add(
                    egui::DragValue::new(fov)
                        .range(FOV.0..=FOV.1)
                        .speed(FOV_STEP)
                        .custom_formatter(|v, _| catalog.number(v, 0))
                        .custom_parser(parse_number),
                );
            });
        }
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.viewmodel, t("settings-viewmodel"));
        });

        // Projectiles and items show through walls only with the players,
        // and keep their choice while the players do not
        heading(
            ui,
            catalog,
            "viewer-see-through",
            Some("settings-see-through-hint"),
            rtl,
            false,
        );
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.see_through, t("settings-see-through"));
        });
        layout::row(ui, rtl, |ui| {
            let items = egui::Checkbox::new(
                &mut viewer.see_through_items,
                t("settings-see-through-items"),
            );
            ui.add_enabled(viewer.see_through, items);
        });

        heading(ui, catalog, "settings-section-sound", None, rtl, false);
        layout::row(ui, rtl, |ui| {
            ui.label(t("viewer-volume"));
            ui.add(egui::Slider::new(&mut viewer.volume, 0. ..=1.).show_value(false));

            let percent = FluentValue::from(f64::from((viewer.volume * 100.).round()));
            let percent = catalog.text_with("library-percent", &[("percent", percent)]);
            ui.label(egui::RichText::new(percent).monospace().color(theme::MUTED));
        });

        if viewer != self.settings.viewer {
            self.settings.viewer = viewer;
            self.settings_unsaved = true;
        }
    }

    /// Where clips are exported, and how much a clip made around an event
    /// keeps on each side of it
    fn clips_section(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        heading(
            ui,
            &self.catalog,
            "settings-export-folder",
            Some("settings-export-hint"),
            rtl,
            true,
        );
        let shown = match &self.settings.export_folder {
            Some(f) => f.display().to_string(),
            None => self.catalog.text("settings-export-beside"),
        };
        path(ui, &shown, rtl);

        // A folder chooser closed without one keeps the folder there was
        let mut chosen = None;
        layout::row(ui, rtl, |ui| {
            if ui.button(self.t(&ctx, "settings-export-choose")).clicked()
                && let Some(f) = rfd::FileDialog::new().pick_folder()
            {
                chosen = Some(Some(f));
            }
            if self.settings.export_folder.is_some()
                && ui.button(self.t(&ctx, "settings-export-beside")).clicked()
            {
                chosen = Some(None);
            }
        });
        if let Some(folder) = chosen {
            self.settings.export_folder = folder;
            self.settings_unsaved = true;
        }

        heading(
            ui,
            &self.catalog,
            "settings-clip-padding",
            Some("settings-clip-padding-hint"),
            rtl,
            false,
        );
        let catalog = &self.catalog;
        let ids = ["settings-preroll", "settings-postroll"];
        let width = label_width(ui, catalog, &ids, rtl);
        let mut padding = [self.settings.preroll, self.settings.postroll];
        let mut changed = false;
        for (id, value) in ids.into_iter().zip(&mut padding) {
            field(ui, catalog, id, width, rtl, |ui| {
                let drag = egui::DragValue::new(value)
                    .range(0.0..=MAX_PADDING)
                    .speed(PADDING_STEP)
                    .custom_formatter(|v, _| catalog.number(v, PADDING_DECIMALS))
                    .custom_parser(parse_number);
                changed |= ui.add(drag).changed();
            });
        }
        if changed {
            [self.settings.preroll, self.settings.postroll] = padding;
            self.settings_unsaved = true;
        }
    }
}

/// A section's heading, with room above it unless it is the first, and a
/// question mark explaining `hint` when there is one
fn heading(
    ui: &mut egui::Ui,
    catalog: &Catalog,
    id: &str,
    hint: Option<&str>,
    rtl: bool,
    first: bool,
) {
    if !first {
        ui.add_space(HEADING_GAP);
    }

    layout::row(ui, rtl, |ui| {
        let title = text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Strong);
        ui.label(title);
        if let Some(hint) = hint {
            widgets::help(ui, &catalog.text(hint), rtl);
        }
    });
}

/// A setting's label, with a question mark explaining `hint` when there is
/// one
fn labelled(ui: &mut egui::Ui, catalog: &Catalog, id: &str, hint: Option<&str>, rtl: bool) {
    layout::row(ui, rtl, |ui| {
        let label = text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain);
        ui.label(label);
        if let Some(hint) = hint {
            widgets::help(ui, &catalog.text(hint), rtl);
        }
    });
}

/// A folder or file the settings name, or what stands in for one
fn path(ui: &mut egui::Ui, shown: &str, rtl: bool) {
    let width = Some(ui.available_width());
    let shown = text::widget_text(ui.ctx(), shown, rtl, width, Look::Weak);
    ui.add(egui::Label::new(shown).wrap());
}

/// The width of the widest of these labels in this language, for their
/// controls to line up
fn label_width(ui: &egui::Ui, catalog: &Catalog, ids: &[&str], rtl: bool) -> f32 {
    let mut widest: f32 = 0.;
    for id in ids {
        let label = text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain);
        let galley = label.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        );
        widest = widest.max(galley.size().x);
    }
    widest
}

/// A setting's label `width` wide on the leading side and its control after
/// it, so the controls of settings one under another line up. Where the two
/// do not fit side by side the control goes under its label
fn field(
    ui: &mut egui::Ui,
    catalog: &Catalog,
    id: &str,
    width: f32,
    rtl: bool,
    add: impl FnOnce(&mut egui::Ui),
) {
    let label = |ui: &mut egui::Ui| {
        let label = text::widget_text(ui.ctx(), &catalog.text(id), rtl, None, Look::Plain);
        ui.label(label);
    };

    if ui.available_width() < width + CONTROL_LEAST {
        layout::column(ui, rtl, label);
        layout::row(ui, rtl, add);
        return;
    }

    layout::row(ui, rtl, |ui| {
        layout::cell(ui, rtl, width, label);
        add(ui);
    });
}
