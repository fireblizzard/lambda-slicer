//! The application window

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, Align, Layout, RichText, WidgetText};
use fluent_bundle::FluentValue;

use crate::i18n::{Catalog, Locale};
use crate::settings::Settings;
use crate::text::{self, Look};

/// Save a screenshot after this many frames and close, for checking how a
/// language renders
pub struct Screenshot {
    pub path: PathBuf,
    pub frames: u32,
}

pub struct App {
    settings: Settings,
    settings_path: Option<PathBuf>,
    catalog: Catalog,
    show_settings: bool,
    screenshot: Option<Screenshot>,
}

impl App {
    pub fn new(
        settings: Settings,
        settings_path: Option<PathBuf>,
        show_settings: bool,
        screenshot: Option<Screenshot>,
    ) -> Self {
        let catalog = Catalog::new(settings.locale());
        Self {
            settings,
            settings_path,
            catalog,
            show_settings,
            screenshot,
        }
    }

    fn rtl(&self) -> bool {
        self.catalog.locale.rtl()
    }

    /// A message as widget text, ordered for the interface's direction
    fn t(&self, ctx: &egui::Context, id: &str) -> WidgetText {
        text::widget_text(ctx, &self.catalog.text(id), self.rtl(), None, Look::Plain)
    }

    /// A message as a heading
    fn strong(&self, ctx: &egui::Context, id: &str) -> WidgetText {
        text::widget_text(ctx, &self.catalog.text(id), self.rtl(), None, Look::Strong)
    }

    fn t_with(
        &self,
        ctx: &egui::Context,
        id: &str,
        args: &[(&str, FluentValue<'static>)],
    ) -> WidgetText {
        let text = self.catalog.text_with(id, args);
        text::widget_text(ctx, &text, self.rtl(), None, Look::Plain)
    }

    fn save(&self) {
        if let Some(p) = &self.settings_path
            && let Err(e) = self.settings.save_to(p)
        {
            eprintln!("cannot save settings to {}: {e}", p.display());
        }
    }

    fn set_locale(&mut self, locale: Option<Locale>) {
        self.settings.locale = locale;
        self.catalog = Catalog::new(self.settings.locale());
        self.save();
    }

    /// A row that runs right to left in a right-to-left interface
    fn row(&self) -> Layout {
        if self.rtl() {
            Layout::right_to_left(Align::Center)
        } else {
            Layout::left_to_right(Align::Center)
        }
    }

    /// A column whose items align right in a right-to-left interface
    fn column(&self) -> Layout {
        if self.rtl() {
            Layout::top_down(Align::Max)
        } else {
            Layout::top_down(Align::Min)
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(self.row(), |ui| {
            ui.add_enabled(false, egui::Button::new(self.t(ui.ctx(), "toolbar-open")));
            ui.add_enabled(
                false,
                egui::Button::new(self.t(ui.ctx(), "toolbar-add-folder")),
            );
            ui.add_enabled(false, egui::Button::new(self.t(ui.ctx(), "toolbar-rescan")));
            ui.add_enabled(false, egui::Button::new(self.t(ui.ctx(), "toolbar-cancel")));
            ui.separator();
            if ui
                .selectable_label(self.show_settings, self.t(ui.ctx(), "toolbar-settings"))
                .clicked()
            {
                self.show_settings = !self.show_settings;
            }
        });
    }

    fn library(&self, ui: &mut egui::Ui) {
        let rtl = self.rtl();
        ui.with_layout(self.column(), |ui| {
            text::label(ui, &self.catalog.text("library-empty"), rtl);
            ui.add_space(12.);
            // Sample counts so each language's plurals and numbers show
            for n in [0, 1, 2, 5, 21, 55059] {
                ui.label(self.t_with(ui.ctx(), "library-demos", &[("count", n.into())]));
            }
            ui.label(self.t_with(ui.ctx(), "library-kills", &[("count", 22.into())]));
            ui.label(self.t_with(ui.ctx(), "library-action", &[("score", 8.0603.into())]));
            ui.label(self.t_with(ui.ctx(), "library-spectating", &[("percent", 62.into())]));
        });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings;
        let rtl = self.rtl();
        let title = self.t(ctx, "settings-title");
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .default_width(420.)
            .anchor(
                if rtl {
                    egui::Align2::LEFT_TOP
                } else {
                    egui::Align2::RIGHT_TOP
                },
                egui::vec2(if rtl { 16. } else { -16. }, 48.),
            )
            .show(ctx, |ui| {
                ui.with_layout(self.column(), |ui| {
                    ui.label(self.strong(ui.ctx(), "settings-language"));
                    let current = self.catalog.locale;
                    let mut chosen = None;
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(current.native_name())
                        .show_ui(ui, |ui| {
                            for l in Locale::ALL {
                                let name = text::widget_text(
                                    ui.ctx(),
                                    l.native_name(),
                                    false,
                                    None,
                                    Look::Plain,
                                );
                                if ui.selectable_label(l == current, name).clicked() {
                                    chosen = Some(l);
                                }
                            }
                        });
                    if let Some(l) = chosen
                        && l != current
                    {
                        self.set_locale(Some(l));
                    }
                    if !self.catalog.locale.reviewed() {
                        text::label(ui, &self.catalog.text("settings-unreviewed"), rtl);
                    }

                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-workers"));
                    let mut workers = self.settings.workers;
                    if ui.add(egui::Slider::new(&mut workers, 1..=4)).changed() {
                        self.settings.workers = workers;
                        self.save();
                    }
                    text::label(ui, &self.catalog.text("settings-workers-hint"), rtl);

                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-folders"));
                    if self.settings.folders.is_empty() {
                        ui.label(self.t(ui.ctx(), "settings-folders-empty"));
                    }
                    let mut remove = None;
                    for (i, f) in self.settings.folders.iter().enumerate() {
                        ui.with_layout(self.row(), |ui| {
                            ui.label(f.display().to_string());
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
                        self.save();
                    }

                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-cache"));
                    let cache = self
                        .settings
                        .cache
                        .clone()
                        .or_else(lambda_slicer_core::library::cache::default_path);
                    ui.label(
                        RichText::new(cache.map(|p| p.display().to_string()).unwrap_or_default())
                            .monospace(),
                    );
                });
            });
        self.show_settings = open;
    }

    fn take_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.screenshot else {
            return;
        };
        if shot.frames > 0 {
            shot.frames -= 1;
            ctx.request_repaint();
            return;
        }
        let path = shot.path.clone();
        self.screenshot = None;
        let closer = ctx.clone();
        ctx.request_screenshot(move |image: Arc<egui::ColorImage>| {
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
            if let Err(e) = image::save_buffer(
                &path,
                &bytes,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ColorType::Rgba8,
            ) {
                eprintln!("cannot save {}: {e}", path.display());
            }
            closer.send_viewport_cmd(egui::ViewportCommand::Close);
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.catalog.text("app-title")));

        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.add_space(4.);
            self.toolbar(ui);
            ui.add_space(4.);
        });
        egui::CentralPanel::default().show(ui, |ui| self.library(ui));
        self.settings_window(&ctx);
        self.take_screenshot(&ctx);
    }
}
