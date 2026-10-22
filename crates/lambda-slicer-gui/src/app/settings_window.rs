//! The settings window, which can be moved anywhere in the app's window,
//! with its four sections down one side and the chosen one beside them (GUI
//! redesign plan §11)
//!
//! Every change applies at once, with no Save to press, and is written out
//! once nothing is being dragged. Each section keeps its own scroll place,
//! and the window keeps where it was left for the next run

use eframe::egui::{self, Align, Layout, Pos2, Vec2};
use fluent_bundle::FluentValue;
use lambda_slicer_playback::decals::MAX_DECALS;

use super::App;
use crate::game_setup;
use crate::i18n::{Catalog, Locale};
use crate::layout;
use crate::library::{self, parse_number};
use crate::settings::{FOV, MAX_PADDING, Settings, ViewerSettings};
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Icon, IconButton, Variant};

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

/// The width of the dialog asking whether to restore the defaults, and the
/// most height its list of settings takes before it scrolls
const DIALOG_WIDTH: f32 = 360.;
const DIALOG_LIST_HEIGHT: f32 = 240.;

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
        if self.confirm_restore {
            self.restore_dialog(ctx, rtl);
        }
        if self.confirm_clear {
            self.clear_dialog(ctx, rtl);
        }

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

                // Restoring the defaults at the foot of the sections, asked
                // about first
                let foot = Layout::bottom_up(if rtl { Align::Max } else { Align::Min });
                ui.with_layout(foot, |ui| {
                    let label = self.t(ui.ctx(), "settings-restore");
                    let button = widgets::Button::new(None, label, Variant::Ghost, rtl);
                    let unchanged = self.settings.restored() == self.settings;
                    let response = ui
                        .add_enabled(!unchanged, button)
                        .on_disabled_hover_text(self.catalog.text("settings-restore-none"));
                    if response.clicked() {
                        self.confirm_restore = true;
                    }
                });
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
    /// and where what is found about them is kept, which can be cleared
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
        // Its demos leave the library with it
        if let Some(i) = remove {
            let folder = self.settings.folders.remove(i);
            if let Some(l) = &mut self.library {
                l.forget_folder(&folder, &self.settings.folders);
            }
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

        self.starting_order(ui, rtl);
        self.starting_filters(ui, rtl);

        heading(ui, &self.catalog, "settings-cache", None, rtl, false);
        let cache = self.cache_path.as_ref().map(|p| p.display().to_string());
        path(ui, &cache.unwrap_or_default(), rtl);

        // Asked about first, and nothing to clear while it is clearing
        let clear = layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "settings-clear-library");
            let button = widgets::Button::new(None, label, Variant::Default, rtl);
            let can = self.library.is_some() && !self.clear_library;
            ui.add_enabled(can, button).clicked()
        });
        if clear {
            self.confirm_clear = true;
        }
    }

    /// The column the library is sorted by when the app starts, and which
    /// way round. A change shows at once
    fn starting_order(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        heading(
            ui,
            &self.catalog,
            "settings-sort",
            Some("settings-sort-hint"),
            rtl,
            false,
        );

        let catalog = &self.catalog;
        let mut column = self.settings.sort_column.clone();
        let mut descending = self.settings.sort_descending;
        layout::row(ui, rtl, |ui| {
            let title =
                |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
            let current = library::sort_columns()
                .find(|(key, _)| *key == column)
                .map_or_else(|| title("column-file-date"), |(_, id)| title(id));
            egui::ComboBox::from_id_salt("starting order")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for (key, id) in library::sort_columns() {
                        ui.selectable_value(&mut column, key.to_string(), title(id));
                    }
                });

            let (icon, id) = if descending {
                (Icon::SortDown, "sort-descending")
            } else {
                (Icon::SortUp, "sort-ascending")
            };
            if ui.add(IconButton::new(icon, &catalog.text(id))).clicked() {
                descending = !descending;
            }
        });

        let unchanged =
            (&column, descending) == (&self.settings.sort_column, self.settings.sort_descending);
        if unchanged {
            return;
        }
        if let Some(l) = &mut self.library {
            l.sort_by_key(&column, descending);
        }
        self.settings.sort_column = column;
        self.settings.sort_descending = descending;
        self.settings_unsaved = true;
    }

    /// The filters the library starts with, the least length of a demo and
    /// the fewest kills. The filters themselves change only from the next
    /// start
    fn starting_filters(&mut self, ui: &mut egui::Ui, rtl: bool) {
        heading(
            ui,
            &self.catalog,
            "settings-start-filters",
            Some("settings-start-filters-hint"),
            rtl,
            false,
        );

        let catalog = &self.catalog;
        let mut minutes = self.settings.start_min_minutes;
        let mut kills = self.settings.start_min_kills;
        let ids = ["filter-min-minutes", "filter-min-kills"];
        let width = label_width(ui, catalog, &ids, rtl);
        field(ui, catalog, "filter-min-minutes", width, rtl, |ui| {
            ui.add(
                egui::DragValue::new(&mut minutes)
                    .range(0.0..=library::MAX_MINUTES)
                    .speed(library::MINUTES_STEP)
                    .custom_formatter(|v, _| catalog.number(v, 1))
                    .custom_parser(parse_number),
            );
        });
        field(ui, catalog, "filter-min-kills", width, rtl, |ui| {
            ui.add(
                egui::DragValue::new(&mut kills)
                    .range(0..=library::MAX_KILLS)
                    .speed(library::KILLS_STEP)
                    .custom_formatter(|v, _| catalog.number(v, 0))
                    .custom_parser(parse_number),
            );
        });

        let wanted = (minutes, kills);
        if wanted
            != (
                self.settings.start_min_minutes,
                self.settings.start_min_kills,
            )
        {
            (
                self.settings.start_min_minutes,
                self.settings.start_min_kills,
            ) = wanted;
            self.settings_unsaved = true;
        }
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
        // Both fields of view line up, and take the same steps
        let width = label_width(ui, catalog, &["settings-fov", "settings-zoom-fov"], rtl);
        let fov_field = |ui: &mut egui::Ui, id: &str, fov: &mut f32| {
            field(ui, catalog, id, width, rtl, |ui| {
                ui.add(
                    egui::DragValue::new(fov)
                        .range(FOV.0..=FOV.1)
                        .speed(FOV_STEP)
                        .custom_formatter(|v, _| catalog.number(v, 0))
                        .custom_parser(parse_number),
                );
            });
        };
        if let Some(fov) = &mut viewer.fov {
            fov_field(ui, "settings-fov", fov);
        }
        fov_field(ui, "settings-zoom-fov", &mut viewer.zoom_fov);
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.viewmodel, t("settings-viewmodel"));
        });
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.crosshair, t("settings-crosshair"));
        });
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.chat, t("settings-chat"));
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

        // Bullet holes and other marks, the newest kept
        heading(
            ui,
            catalog,
            "settings-section-decals",
            Some("settings-decals-hint"),
            rtl,
            false,
        );
        let width = label_width(ui, catalog, &["settings-decals"], rtl);
        field(ui, catalog, "settings-decals", width, rtl, |ui| {
            ui.add(
                egui::DragValue::new(&mut viewer.decals)
                    .range(0..=MAX_DECALS)
                    .custom_formatter(|v, _| catalog.number(v, 0))
                    .custom_parser(parse_number),
            );
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

        // The demo's name first only means something with the clip's name
        heading(
            ui,
            &self.catalog,
            "settings-file-names",
            Some("settings-file-names-hint"),
            rtl,
            false,
        );
        let mut naming = (self.settings.clip_name_files, self.settings.demo_name_first);
        layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "settings-clip-name-files");
            ui.checkbox(&mut naming.0, label);
        });
        layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "settings-demo-name-first");
            ui.add_enabled(naming.0, egui::Checkbox::new(&mut naming.1, label));
        });
        if naming != (self.settings.clip_name_files, self.settings.demo_name_first) {
            (self.settings.clip_name_files, self.settings.demo_name_first) = naming;
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

    /// Asks whether to restore the defaults, naming every setting that
    /// would change, over the whole app until answered
    fn restore_dialog(&mut self, ctx: &egui::Context, rtl: bool) {
        let restored = self.settings.restored();
        let mut changes = Vec::new();
        for id in changed_settings(&self.settings, &restored) {
            changes.push(self.catalog.text(id));
        }

        let text = self.catalog.text("settings-restore-list");
        let answer = ask(
            ctx,
            &self.catalog,
            rtl,
            "settings-restore-title",
            &text,
            &changes,
        );
        match answer {
            Some(true) => {
                self.restore_defaults(restored);
                self.confirm_restore = false;
            }
            Some(false) => self.confirm_restore = false,
            None => {}
        }
    }

    /// Asks whether to clear the library, saying how many demos it holds.
    /// A scan running is stopped first, and the clearing waits for it
    fn clear_dialog(&mut self, ctx: &egui::Context, rtl: bool) {
        let files = self
            .library
            .as_ref()
            .map_or(0, library::LibraryView::library_files);
        let count = FluentValue::from(files as i64);
        let text = self
            .catalog
            .text_with("settings-clear-library-text", &[("count", count)]);

        let answer = ask(
            ctx,
            &self.catalog,
            rtl,
            "settings-clear-library-title",
            &text,
            &[],
        );
        match answer {
            Some(true) => {
                if let Some(job) = &self.scan {
                    job.cancel();
                }
                self.clear_library = true;
                self.confirm_clear = false;
            }
            Some(false) => self.confirm_clear = false,
            None => {}
        }
    }

    /// Take `restored` as the settings, and show it at once in the library,
    /// the open demo and how the window is laid out
    fn restore_defaults(&mut self, restored: Settings) {
        if let Some(l) = &mut self.library {
            l.hidden = restored.hidden_columns.clone();
            l.filters_open = restored.layout.filters_open;
            l.list_filters_open = restored.layout.list_filters_open;
            l.sort_by_key(&restored.sort_column, restored.sort_descending);
        }
        if let Some(d) = &mut self.demo {
            d.lanes_open = restored.player_lanes;
        }

        // The inspector forgets the width it was dragged to
        self.workspace.list_hidden = restored.layout.list_hidden;
        self.workspace.inspector_width = None;
        self.workspace.restored += 1;

        let locale = restored.locale;
        self.settings = restored;
        self.set_locale(locale);
    }
}

/// A question over the whole app until it is answered, titled by the
/// catalog's `title`, with `body` and then `items` in a list under it.
/// Going ahead is the button named as the title is. Returns whether to go
/// ahead once answered, no when cancelled, closed with Escape or by a click
/// beside it, and none while it waits
fn ask(
    ctx: &egui::Context,
    catalog: &Catalog,
    rtl: bool,
    title: &str,
    body: &str,
    items: &[String],
) -> Option<bool> {
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::unique(title)).show(ctx, |ui| {
        ui.set_width(DIALOG_WIDTH);
        let ctx = ui.ctx().clone();
        let t = |id: &str, look| text::widget_text(&ctx, &catalog.text(id), rtl, None, look);

        layout::column(ui, rtl, |ui| ui.label(t(title, Look::Strong)));
        ui.add_space(theme::GAP_S);
        let width = Some(ui.available_width());
        let line = text::widget_text(&ctx, body, rtl, width, Look::Plain);
        ui.add(egui::Label::new(line).wrap());

        if !items.is_empty() {
            ui.add_space(theme::GAP_XS);
            egui::ScrollArea::vertical()
                .max_height(DIALOG_LIST_HEIGHT)
                .show(ui, |ui| {
                    for item in items {
                        let line = format!("• {item}");
                        let line = text::widget_text(&ctx, &line, rtl, None, Look::Weak);
                        layout::column(ui, rtl, |ui| ui.label(line));
                    }
                });
        }
        ui.add_space(theme::GAP_M);

        // Going ahead at the trailing end, and the way back before it
        layout::trailing(ui, rtl, |ui| {
            let ahead = widgets::Button::new(None, t(title, Look::Plain), Variant::Primary, rtl);
            if ui.add(ahead).clicked() {
                answer = Some(true);
            }
            let back = t("toolbar-cancel", Look::Plain);
            let back = widgets::Button::new(None, back, Variant::Default, rtl);
            if ui.add(back).clicked() {
                answer = Some(false);
            }
        });
    });

    if modal.should_close() {
        answer = answer.or(Some(false));
    }
    answer
}

/// The catalog names of the settings that differ between `current` and
/// `restored`, in the order the settings window shows them, then those kept
/// outside it
fn changed_settings(current: &Settings, restored: &Settings) -> Vec<&'static str> {
    let (now, then) = (current, restored);
    let (view, first) = (&now.viewer, &then.viewer);
    let order = |s: &Settings| (s.sort_column.clone(), s.sort_descending);
    let starting = |s: &Settings| (s.start_min_minutes, s.start_min_kills);
    let checks = [
        (now.locale != then.locale, "settings-language"),
        (now.workers != then.workers, "settings-workers"),
        (order(now) != order(then), "settings-sort"),
        (starting(now) != starting(then), "settings-start-filters"),
        (view.gamma != first.gamma, "settings-gamma"),
        (view.brightness != first.brightness, "settings-brightness"),
        (view.texgamma != first.texgamma, "settings-texgamma"),
        (view.lightgamma != first.lightgamma, "settings-lightgamma"),
        (view.overbright != first.overbright, "settings-overbright"),
        (view.fov != first.fov, "settings-fov"),
        (view.zoom_fov != first.zoom_fov, "settings-zoom-fov"),
        (view.viewmodel != first.viewmodel, "settings-viewmodel"),
        (view.crosshair != first.crosshair, "settings-crosshair"),
        (view.chat != first.chat, "settings-chat"),
        (
            view.see_through != first.see_through,
            "settings-see-through",
        ),
        (
            view.see_through_items != first.see_through_items,
            "settings-see-through-items",
        ),
        (view.decals != first.decals, "settings-section-decals"),
        (view.volume != first.volume, "viewer-volume"),
        (
            now.export_folder != then.export_folder,
            "settings-export-folder",
        ),
        (
            now.clip_name_files != then.clip_name_files,
            "settings-clip-name-files",
        ),
        (
            now.demo_name_first != then.demo_name_first,
            "settings-demo-name-first",
        ),
        (now.preroll != then.preroll, "settings-preroll"),
        (now.postroll != then.postroll, "settings-postroll"),
        (
            now.hidden_columns != then.hidden_columns,
            "settings-restore-columns",
        ),
        (now.player_lanes != then.player_lanes, "timeline-lanes"),
        (now.layout != then.layout, "settings-restore-layout"),
    ];

    let mut names = Vec::new();
    for (changed, id) in checks {
        if changed {
            names.push(id);
        }
    }
    names
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_settings_that_change_are_named_in_the_windows_order() {
        let defaults = Settings::default();
        assert!(changed_settings(&defaults, &defaults.restored()).is_empty());

        let mut s = Settings {
            workers: 3,
            player_lanes: true,
            ..Settings::default()
        };
        s.viewer.gamma = 3.;
        s.viewer.decals = 5;
        s.layout.list_hidden = true;

        // The game's folder is kept, so it is never named
        s.viewer.game = Some(std::path::PathBuf::from("F:/Half-Life"));
        let names = changed_settings(&s, &s.restored());
        assert_eq!(
            names,
            [
                "settings-workers",
                "settings-gamma",
                "settings-section-decals",
                "timeline-lanes",
                "settings-restore-layout",
            ]
        );
    }
}
