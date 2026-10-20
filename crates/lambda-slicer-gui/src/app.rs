//! The application window

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, RichText, WidgetText};
use fluent_bundle::FluentValue;
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::library::cache::default_path;
use lambda_slicer_core::library::scan::ScanOptions;

use crate::clips::{ClipStore, LoadProblem};
use crate::demo::DemoView;
use crate::i18n::{Catalog, Locale};
use crate::jobs::{Problem, ScanJob};
use crate::layout;
use crate::library::{LibraryView, parse_number};
use crate::settings::{FOV, MAX_PADDING, Settings, ViewerSettings};
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Icon, IconButton, Variant};

/// Save a screenshot after this many frames and close, for checking how a
/// language renders
pub struct Screenshot {
    pub path: PathBuf,
    pub frames: u32,
}

/// Choices given on the command line for one run
pub struct Launch {
    pub show_settings: bool,
    pub screenshot: Option<Screenshot>,
    /// A cache used instead of the configured one, without saving it
    pub cache: Option<PathBuf>,
    /// The clips file, saved to unless the settings are not
    pub clips: Option<PathBuf>,
    /// Demos and folders opened for this session
    pub open: Vec<PathBuf>,
    /// A cached demo to show at start, and an action window to open it on
    pub demo: Option<PathBuf>,
    pub window: Option<usize>,
    /// Turn on both date filters, for checking how they wrap
    pub dates: bool,
    /// Export every clip of the demo shown at start
    pub export: bool,
    /// Pseudo-localise every message, to find text that does not fit
    pub pseudo: bool,
    /// The game's installation folder, for the viewer
    pub game: Option<PathBuf>,
    /// Where the viewer seeks to once the demo is loaded
    pub viewer_at: Option<f64>,
    pub viewer_bench: Option<crate::viewer::Bench>,
    pub viewer_capture: Option<crate::viewer::Capture>,
    /// Seconds the viewer plays with sound before the window closes
    pub viewer_play: Option<f64>,
    /// Gamma settings for this run only, in place of the saved ones
    pub viewer_gamma: Option<lambda_slicer_render::gamma::Gamma>,
    /// Players shown through walls for this run
    pub viewer_see_through: bool,
}

pub struct App {
    settings: Settings,
    settings_path: Option<PathBuf>,
    catalog: Catalog,
    title: Option<Locale>,
    show_settings: bool,
    settings_tab: SettingsTab,
    show_problems: bool,
    screenshot: Option<Screenshot>,
    /// A screenshot was asked for and its image has not come back
    screenshot_requested: bool,
    cache_path: Option<PathBuf>,
    clips: ClipStore,
    clips_path: Option<PathBuf>,
    /// A problem with the clips file, shown until dismissed
    clips_notice: Option<String>,
    library: Option<LibraryView>,
    scan: Option<ScanJob>,
    demo: Option<DemoView>,
    open_window: Option<usize>,
    export_on_open: bool,
    /// A scan to start once the stopping one has stopped
    rescan: bool,
    /// A demo opened as a file, shown without a library row, so opening
    /// and analysing one never needs the library cache (plan §2.3)
    direct: Option<PathBuf>,
    /// Files opened or dropped this session, scanned along with the folders
    opened: Vec<PathBuf>,
    /// Dropped items that were not demos or folders
    rejected: Vec<String>,
    game: Option<PathBuf>,
    viewer_at: Option<f64>,
    viewer_bench: Option<crate::viewer::Bench>,
    viewer_capture: Option<crate::viewer::Capture>,
    viewer_play: Option<f64>,
    viewer_gamma: Option<lambda_slicer_render::gamma::Gamma>,
    viewer_see_through: bool,
    /// The viewer changed a setting, saved once the pointer is let go
    viewer_unsaved: bool,
    /// The window's GPU device, which the viewer draws with
    pub render_state: Option<eframe::egui_wgpu::RenderState>,
    /// The app's icon for the toolbar, made on the first frame
    brand: Option<egui::TextureHandle>,
}

/// The app's icon as a texture, for the toolbar
fn brand_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let png = include_bytes!("../../../assets/branding/icons/icon-48.png");
    let image = image::load_from_memory(png)
        .expect("the bundled application icon must be a valid PNG")
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());

    ctx.load_texture("brand", pixels, egui::TextureOptions::LINEAR)
}

/// Whether `path` starts with the GoldSrc demo magic
fn is_demo(path: &Path) -> bool {
    let mut magic = [0u8; 8];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .is_ok_and(|_| &magic == b"HLDEMO\0\0")
}

impl App {
    pub fn new(settings: Settings, settings_path: Option<PathBuf>, launch: Launch) -> Self {
        let mut catalog = Catalog::new(settings.locale());
        catalog.pseudo = launch.pseudo;
        let cache_path = launch
            .cache
            .clone()
            .or_else(|| settings.cache.clone())
            .or_else(default_path);
        let mut library = cache_path
            .clone()
            .map(|p| LibraryView::new(p, settings.hidden_columns.clone()));
        if let (Some(l), Some(d)) = (&mut library, &launch.demo) {
            l.select_path(d);
        }
        if let Some(l) = &mut library
            && launch.dates
        {
            let day = crate::i18n::Ymd {
                year: 2019,
                month: 8,
                day: 15,
            };
            l.filters.from = Some(day);
            l.filters.to = Some(day);
        }
        let (clips, problem) = launch
            .clips
            .as_deref()
            .map(ClipStore::load)
            .unwrap_or_default();
        let mut clips_path = settings_path.as_ref().and(launch.clips.clone());
        let shown = launch
            .clips
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let clips_notice = problem.map(|problem| match problem {
            LoadProblem::Moved { moved, detail } => catalog.text_with(
                "clips-store-moved",
                &[
                    ("path", FluentValue::from(shown)),
                    ("moved", FluentValue::from(moved.display().to_string())),
                    ("detail", FluentValue::from(detail)),
                ],
            ),
            LoadProblem::Unreadable { detail } => {
                // Never saved over, so nothing in it is lost
                clips_path = None;
                catalog.text_with(
                    "clips-store-unreadable",
                    &[
                        ("path", FluentValue::from(shown)),
                        ("detail", FluentValue::from(detail)),
                    ],
                )
            }
        });
        Self {
            settings,
            settings_path,
            clips,
            clips_path,
            clips_notice,
            catalog,
            title: None,
            show_settings: launch.show_settings,
            settings_tab: SettingsTab::default(),
            show_problems: false,
            screenshot: launch.screenshot,
            screenshot_requested: false,
            cache_path,
            library,
            scan: None,
            demo: None,
            open_window: launch.window,
            export_on_open: launch.export,
            rescan: false,
            direct: launch.open.iter().find(|p| p.is_file()).cloned(),
            opened: launch.open,
            rejected: Vec::new(),
            game: launch.game,
            viewer_at: launch.viewer_at,
            viewer_bench: launch.viewer_bench,
            viewer_capture: launch.viewer_capture,
            viewer_play: launch.viewer_play,
            viewer_gamma: launch.viewer_gamma,
            viewer_see_through: launch.viewer_see_through,
            viewer_unsaved: false,
            render_state: None,
            brand: None,
        }
    }

    fn rtl(&self) -> bool {
        self.catalog.locale.rtl()
    }

    /// A message as widget text, ordered for the interface's direction
    fn t(&self, ctx: &egui::Context, id: &str) -> WidgetText {
        text::widget_text(ctx, &self.catalog.text(id), self.rtl(), None, Look::Plain)
    }

    /// The demo view's viewer, from the game folder `--game` names or else
    /// the one in Settings, opened again when that folder changes
    fn open_viewer(&mut self, ctx: &egui::Context) {
        let game = self
            .game
            .clone()
            .or_else(|| self.settings.viewer.game.clone());
        let (Some(d), Some(render_state)) = (&mut self.demo, &self.render_state) else {
            return;
        };
        if d.viewer_game == game {
            return;
        }
        d.viewer_game = game.clone();
        d.viewer = game.map(|game| {
            crate::viewer::Viewer::open(
                render_state.clone(),
                game,
                d.path.clone(),
                d.duration,
                crate::viewer::Dev {
                    start: self.viewer_at.take(),
                    bench: self.viewer_bench.take(),
                    capture: self.viewer_capture.take(),
                    play: self.viewer_play.take(),
                },
                ctx,
            )
        });
    }

    /// Give the open demo the settings it works with this frame, this run's
    /// choices from the command line over the saved ones
    fn apply_demo_settings(&mut self) {
        let Some(d) = &mut self.demo else {
            return;
        };
        d.padding = (self.settings.preroll, self.settings.postroll);
        d.export_folder = self.settings.export_folder.clone();

        let Some(v) = &mut d.viewer else {
            return;
        };
        v.settings = self.settings.viewer.clone();

        if let Some(g) = self.viewer_gamma {
            v.settings.gamma = g.gamma;
            v.settings.brightness = g.brightness;
            v.settings.texgamma = g.texgamma;
            v.settings.lightgamma = g.lightgamma;
            v.settings.overbright = g.overbright;
        }
        v.settings.see_through |= self.viewer_see_through;
    }

    /// Close the open demo, keeping a clip time still being typed and any
    /// clip edits not saved yet
    fn drop_demo(&mut self) {
        let Some(mut d) = self.demo.take() else {
            return;
        };
        d.commit_typing();
        if d.clips_changed {
            self.save_clips(&d.path, d.clips.clone());
        }
    }

    fn save_settings(&self) {
        if let Some(p) = &self.settings_path
            && let Err(e) = self.settings.save_to(p)
        {
            eprintln!("cannot save settings to {}: {e}", p.display());
        }
    }

    /// Keep a demo's clips and write them out
    fn save_clips(&mut self, demo: &Path, clips: Vec<crate::clips::Clip>) {
        self.clips.set(demo, clips);
        if let Some(p) = &self.clips_path
            && let Err(e) = self.clips.save_to(p)
        {
            self.clips_notice = Some(self.catalog.text_with(
                "clips-save-failed",
                &[
                    ("path", FluentValue::from(p.display().to_string())),
                    ("detail", FluentValue::from(e.to_string())),
                ],
            ));
        }
    }

    fn set_locale(&mut self, locale: Option<Locale>) {
        self.settings.locale = locale;
        let pseudo = self.catalog.pseudo;
        self.catalog = Catalog::new(self.settings.locale());
        self.catalog.pseudo = pseudo;
        self.save_settings();
    }

    fn scanning(&self) -> bool {
        self.scan.as_ref().is_some_and(ScanJob::running)
    }

    /// Scan the library folders and opened files. A running scan is stopped
    /// first, keeping what it finished. Stopping is not waited for, as the
    /// window would hang while a worker finishes its current read, so the
    /// new scan starts once the old one reports it stopped
    pub fn start_scan(&mut self, ctx: &egui::Context) {
        if let Some(job) = &self.scan
            && job.running()
        {
            job.cancel();
            self.rescan = true;
            return;
        }
        self.rescan = false;
        let Some(cache) = self.cache_path.clone() else {
            return;
        };
        let mut roots = self.settings.folders.clone();
        roots.extend(self.opened.iter().cloned());
        if roots.is_empty() {
            return;
        }
        self.scan = None;
        let options = ScanOptions {
            roots,
            cache,
            workers: usize::from(self.settings.workers),
            force: false,
            full_hash: true,
            limits: Limits::default(),
        };
        self.scan = Some(ScanJob::start(options, ctx.clone()));
    }

    fn add_folders(&mut self, ctx: &egui::Context, folders: Vec<PathBuf>) {
        let mut added = false;
        for f in folders {
            if !self.settings.folders.contains(&f) {
                self.settings.folders.push(f);
                added = true;
            }
        }
        if added {
            self.save_settings();
            self.start_scan(ctx);
        }
    }

    fn open_files(&mut self, ctx: &egui::Context, files: Vec<PathBuf>) {
        // The first one shows at once, analysed by the demo view itself,
        // and joins the library as the scan stores it
        if let Some(first) = files.first() {
            self.direct = Some(first.clone());
            if let Some(l) = &mut self.library {
                l.selected = None;
            }
        }
        let new: Vec<PathBuf> = files
            .into_iter()
            .filter(|f| !self.opened.contains(f))
            .collect();
        if new.is_empty() {
            return;
        }
        self.opened.extend(new.iter().cloned());
        match &self.scan {
            Some(job) if job.running() => {
                for f in new {
                    job.prioritize(f);
                }
            }
            _ => self.start_scan(ctx),
        }
    }

    fn handle_drops(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if dropped.is_empty() {
            return;
        }
        let mut folders = Vec::new();
        let mut files = Vec::new();
        self.rejected.clear();
        for p in dropped {
            if p.is_dir() {
                folders.push(p);
            } else if is_demo(&p) {
                files.push(p);
            } else {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| p.display().to_string());
                self.rejected.push(name);
            }
        }
        self.add_folders(ctx, folders);
        self.open_files(ctx, files);
    }

    /// A toolbar button with text after its icon
    fn tool(&self, ctx: &egui::Context, icon: Icon, id: &str) -> widgets::Button {
        widgets::Button::new(Some(icon), self.t(ctx, id), Variant::Ghost, self.rtl())
    }

    /// The app's icon and name, leading the toolbar
    fn brand(&mut self, ui: &mut egui::Ui) {
        let texture = self.brand.get_or_insert_with(|| brand_texture(ui.ctx()));
        let size = egui::Vec2::splat(20.);
        ui.add(egui::Image::new((texture.id(), size)));

        let name = self.catalog.text("app-title");
        ui.label(text::widget_text(
            ui.ctx(),
            &name,
            false,
            None,
            Look::Strong,
        ));
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let rtl = self.rtl();
        layout::row(ui, rtl, |ui| {
            self.brand(ui);
            ui.add_space(theme::GAP_M);

            if ui
                .add(self.tool(&ctx, Icon::Open, "toolbar-open"))
                .clicked()
                && let Some(files) = rfd::FileDialog::new()
                    .add_filter(self.catalog.text("dialog-demos"), &["dem"])
                    .pick_files()
            {
                self.open_files(&ctx, files);
            }

            let add_folder = self.tool(&ctx, Icon::AddFolder, "toolbar-add-folder");
            if ui.add(add_folder).clicked()
                && let Some(folders) = rfd::FileDialog::new().pick_folders()
            {
                self.add_folders(&ctx, folders);
            }

            // A disabled button says why it cannot be used
            let busy = self.scanning();
            let nothing = self.settings.folders.is_empty() && self.opened.is_empty();
            let why = if busy {
                "toolbar-rescan-busy"
            } else {
                "toolbar-rescan-nothing"
            };
            let rescan = self.tool(&ctx, Icon::Rescan, "toolbar-rescan");
            if ui
                .add_enabled(!busy && !nothing, rescan)
                .on_disabled_hover_text(self.t(&ctx, why))
                .clicked()
            {
                self.start_scan(&ctx);
            }

            let cancelling = self.scan.as_ref().is_some_and(ScanJob::cancelling);
            let why = if cancelling {
                "scan-cancelling"
            } else {
                "toolbar-cancel-idle"
            };
            let cancel = self.tool(&ctx, Icon::Close, "toolbar-cancel");
            if ui
                .add_enabled(busy && !cancelling, cancel)
                .on_disabled_hover_text(self.t(&ctx, why))
                .clicked()
                && let Some(job) = &self.scan
            {
                job.cancel();
            }

            layout::trailing(ui, rtl, |ui| {
                let label = self.catalog.text("toolbar-settings");
                let settings = IconButton::new(Icon::Settings, &label).selected(self.show_settings);
                if ui.add(settings).clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });
    }

    fn mb(&self, bytes: u64) -> FluentValue<'static> {
        FluentValue::from(self.catalog.number(bytes as f64 / 1e6, 0))
    }

    /// The scan's state in one line, and the problems found
    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let rtl = self.rtl();
        let Some(job) = &self.scan else {
            return;
        };
        let p = &job.progress;
        let n = |v: u64| FluentValue::from(v as i64);
        let line = match &job.finished {
            None if job.cancelling() => self.catalog.text("scan-cancelling"),
            None => self.catalog.text_with(
                "scan-progress",
                &[
                    ("found", n(p.discovered)),
                    ("done", n(p.completed)),
                    ("cached", n(p.cache_hits)),
                    ("analysed", n(p.analyzed)),
                    ("megabytes", self.mb(p.bytes_read)),
                ],
            ),
            Some(Ok(p)) if p.cancelled => self
                .catalog
                .text_with("scan-cancelled", &[("left", n(p.pending()))]),
            Some(Ok(p)) => self.catalog.text_with(
                "scan-finished",
                &[
                    ("done", n(p.completed)),
                    ("analysed", n(p.analyzed)),
                    ("seconds", FluentValue::from(p.elapsed_seconds.round())),
                ],
            ),
            Some(Err(e)) => self.catalog.message(e),
        };
        let problems = job.problems.len();
        let mut toggle = false;
        layout::row(ui, self.rtl(), |ui| {
            if job.running() {
                ui.spinner();
            }
            ui.label(text::widget_text(&ctx, &line, rtl, None, Look::Plain));
            if problems > 0 {
                let label = self
                    .catalog
                    .text_with("scan-problems", &[("count", n(problems as u64))]);
                let w = text::widget_text(&ctx, &label, rtl, None, Look::Plain);
                if ui.link(w).clicked() {
                    toggle = true;
                }
            }
        });
        if toggle {
            self.show_problems = !self.show_problems;
        }
    }

    fn problems_window(&mut self, ctx: &egui::Context) {
        let Some(job) = &self.scan else {
            return;
        };
        let mut open = self.show_problems;
        let rtl = self.rtl();
        egui::Window::new(self.t(ctx, "problems-title"))
            .open(&mut open)
            .default_width(560.)
            .default_height(300.)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (path, problem) in job.problems.iter().rev() {
                        let why = match problem {
                            Problem::Message(m) => self.catalog.message(m),
                            Problem::Status(s) => self.catalog.text(&format!("status-{s}")),
                            Problem::Changed => self.catalog.text("problem-changed"),
                        };
                        ui.label(RichText::new(path.display().to_string()).monospace());
                        text::label(ui, &why, rtl);
                        ui.separator();
                    }
                });
            });
        self.show_problems = open;
    }

    fn details(&self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let rtl = self.rtl();
        let Some(f) = self.library.as_ref().and_then(|l| l.selected.as_ref()) else {
            return;
        };
        layout::column(ui, self.rtl(), |ui| {
            ui.label(RichText::new(&f.path).monospace());
            let mut facts = Vec::new();
            if let Some(r) = &f.recorder {
                facts.push(self.catalog.text_with(
                    "details-recorder",
                    &[("name", FluentValue::from(text::player_name(r)))],
                ));
            }
            if let Some(k) = &f.recording_kind {
                facts.push(self.catalog.text(&format!("kind-{}", k.replace('_', "-"))));
            }
            if let Some(s) = f.shots {
                facts.push(
                    self.catalog
                        .text_with("details-shots", &[("count", FluentValue::from(s))]),
                );
            }
            if let Some(k) = f.kills {
                facts.push(
                    self.catalog
                        .text_with("details-kills", &[("count", FluentValue::from(k))]),
                );
            }
            if let Some(a) = f.best_window {
                facts.push(
                    self.catalog
                        .text_with("details-action", &[("score", FluentValue::from(a))]),
                );
            }
            if !facts.is_empty() {
                ui.label(text::widget_text(
                    &ctx,
                    &facts.join(" · "),
                    rtl,
                    None,
                    Look::Plain,
                ));
            }
            if let Some(e) = &f.error {
                text::label(ui, &self.catalog.stored(e), rtl);
            }
        });
    }

    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings;
        let rtl = self.rtl();
        let title = self.t(ctx, "settings-title");
        let tabs = [
            (SettingsTab::General, self.t(ctx, "settings-tab-general")),
            (SettingsTab::Viewer, self.t(ctx, "settings-viewer")),
        ];
        // No taller than the app's window below the toolbar, scrolling
        // whatever does not fit
        let max_height = (ctx.content_rect().height() - 140.).max(160.);
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
                layout::row(ui, rtl, |ui| {
                    for (tab, label) in tabs {
                        ui.selectable_value(&mut self.settings_tab, tab, label);
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .min_scrolled_height(max_height)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        layout::column(ui, rtl, |ui| match self.settings_tab {
                            SettingsTab::General => self.general_settings(ui, rtl),
                            SettingsTab::Viewer => self.viewer_settings(ui, rtl),
                        });
                    });
            });
        self.show_settings = open;
    }

    /// The settings window's general tab, every folder first
    fn general_settings(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();

        heading(
            ui,
            &self.catalog,
            "settings-section-folders",
            None,
            rtl,
            true,
        );
        let mut game = self.settings.viewer.game.clone();
        labelled(
            ui,
            &self.catalog,
            "settings-viewer-game",
            Some("settings-viewer-game-hint"),
            rtl,
        );
        let shown = match &game {
            Some(f) => f.display().to_string(),
            None => self.catalog.text("settings-viewer-game-none"),
        };
        path(ui, &shown, rtl);
        layout::row(ui, rtl, |ui| {
            if ui.button(self.t(&ctx, "settings-export-choose")).clicked()
                && let Some(f) = rfd::FileDialog::new().pick_folder()
            {
                game = Some(f);
            }
        });
        if game != self.settings.viewer.game {
            self.settings.viewer.game = game;
            self.save_settings();
        }

        ui.add_space(6.);
        labelled(
            ui,
            &self.catalog,
            "settings-folders",
            Some("settings-folders-hint"),
            rtl,
        );
        if self.settings.folders.is_empty() {
            path(ui, &self.catalog.text("settings-folders-empty"), rtl);
        }
        let mut remove = None;
        for (i, f) in self.settings.folders.iter().enumerate() {
            layout::row(ui, rtl, |ui| {
                ui.label(text::widget_text(
                    ui.ctx(),
                    &f.display().to_string(),
                    rtl,
                    None,
                    Look::Weak,
                ));
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
            self.save_settings();
        }
        let add = layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "toolbar-add-folder");
            let add = widgets::Button::new(Some(Icon::AddFolder), label, Variant::Default, rtl);
            ui.add(add).clicked()
        });
        if add && let Some(folders) = rfd::FileDialog::new().pick_folders() {
            self.add_folders(&ctx, folders);
        }

        ui.add_space(6.);
        labelled(
            ui,
            &self.catalog,
            "settings-export-folder",
            Some("settings-export-hint"),
            rtl,
        );
        let shown = match &self.settings.export_folder {
            Some(f) => f.display().to_string(),
            None => self.catalog.text("settings-export-beside"),
        };
        path(ui, &shown, rtl);
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
        if let Some(f) = chosen {
            self.settings.export_folder = f;
            self.save_settings();
        }

        ui.add_space(6.);
        labelled(ui, &self.catalog, "settings-cache", None, rtl);
        let cache = self.cache_path.as_ref().map(|p| p.display().to_string());
        path(ui, &cache.unwrap_or_default(), rtl);

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
        let current = self.catalog.locale;
        let mut chosen = None;
        layout::row(ui, rtl, |ui| {
            egui::ComboBox::from_id_salt("language")
                .selected_text(current.native_name())
                .show_ui(ui, |ui| {
                    for l in Locale::ALL {
                        let name =
                            text::widget_text(ui.ctx(), l.native_name(), false, None, Look::Plain);
                        if ui.selectable_label(l == current, name).clicked() {
                            chosen = Some(l);
                        }
                    }
                });
        });
        if let Some(l) = chosen
            && l != current
        {
            self.set_locale(Some(l));
        }

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
            self.save_settings();
        }

        // How much a clip made around an event keeps on each side
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
                changed |= ui
                    .add(
                        egui::DragValue::new(value)
                            .range(0.0..=MAX_PADDING)
                            .speed(0.1)
                            .custom_formatter(|v, _| catalog.number(v, 1))
                            .custom_parser(parse_number),
                    )
                    .changed();
            });
        }
        if changed {
            [self.settings.preroll, self.settings.postroll] = padding;
            self.save_settings();
        }
    }

    /// The settings window's tab for the demo viewer, every setting that
    /// changes only how it shows and plays a demo
    fn viewer_settings(&mut self, ui: &mut egui::Ui, rtl: bool) {
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
                        .speed(0.01)
                        .custom_formatter(|v, _| catalog.number(v, 2))
                        .custom_parser(parse_number),
                );
            });
        }
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.overbright, t("settings-overbright"));
        });
        layout::row(ui, rtl, |ui| {
            if ui.button(t("settings-gamma-defaults")).clicked() {
                let d = ViewerSettings::default();
                (viewer.gamma, viewer.brightness) = (d.gamma, d.brightness);
                (viewer.texgamma, viewer.lightgamma) = (d.texgamma, d.lightgamma);
                viewer.overbright = d.overbright;
            }
        });

        ui.add_space(6.);
        layout::row(ui, rtl, |ui| {
            let mut recorded = viewer.fov.is_none();
            if ui
                .checkbox(&mut recorded, t("settings-fov-recorded"))
                .changed()
            {
                viewer.fov = (!recorded).then_some(90.);
            }
            if let Some(fov) = &mut viewer.fov {
                ui.label(t("settings-fov"));
                ui.add(
                    egui::DragValue::new(fov)
                        .range(FOV.0..=FOV.1)
                        .speed(0.5)
                        .custom_formatter(|v, _| catalog.number(v, 0))
                        .custom_parser(parse_number),
                );
            }
        });
        layout::row(ui, rtl, |ui| {
            ui.checkbox(&mut viewer.viewmodel, t("settings-viewmodel"));
        });

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
            ui.checkbox(
                &mut viewer.see_through_items,
                t("settings-see-through-items"),
            );
        });

        heading(ui, catalog, "settings-section-sound", None, rtl, false);
        layout::row(ui, rtl, |ui| {
            ui.label(t("viewer-volume"));
            ui.add(egui::Slider::new(&mut viewer.volume, 0. ..=1.).show_value(false));
        });

        if viewer != self.settings.viewer {
            self.settings.viewer = viewer;
            self.save_settings();
        }
    }

    fn take_screenshot(&mut self, ctx: &egui::Context) {
        // The image comes back once later frames have read it from the GPU,
        // and nothing else may ask for them
        if self.screenshot_requested {
            ctx.request_repaint();
            return;
        }
        let Some(shot) = &mut self.screenshot else {
            return;
        };
        // A demo still being analysed is waited for
        let loading = self.demo.as_ref().is_some_and(DemoView::busy);
        if shot.frames > 0 || loading {
            if !loading {
                shot.frames -= 1;
            }
            ctx.request_repaint();
            return;
        }
        let path = shot.path.clone();
        self.screenshot = None;
        self.screenshot_requested = true;
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
        ctx.request_repaint();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.title != Some(self.catalog.locale) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.catalog.text("app-title")));
            self.title = Some(self.catalog.locale);
        }

        self.handle_drops(&ctx);
        if let Some(job) = &mut self.scan
            && job.drain()
            && let Some(l) = &mut self.library
        {
            l.mark_stale();
        }
        if self.rescan && !self.scanning() {
            self.start_scan(&ctx);
        }

        // The toolbar's controls sit in the middle of its height
        let margin_y = (theme::TOOLBAR_HEIGHT - theme::CONTROL_HEIGHT) / 2.;
        let toolbar = egui::Frame::new()
            .fill(theme::SURFACE)
            .inner_margin(egui::Margin::symmetric(theme::GAP_M as i8, margin_y as i8));
        egui::Panel::top("toolbar")
            .exact_size(theme::TOOLBAR_HEIGHT)
            .frame(toolbar)
            .show(ui, |ui| self.toolbar(ui));

        if self.scan.is_some() || !self.rejected.is_empty() || self.clips_notice.is_some() {
            let status = egui::Frame::new()
                .fill(theme::SURFACE)
                .inner_margin(egui::Margin::symmetric(theme::GAP_M as i8, 2));
            egui::Panel::bottom("status")
                .min_size(theme::STATUS_HEIGHT)
                .frame(status)
                .show(ui, |ui| {
                    ui.add_space(2.);

                    // The message wraps, with the button under it
                    if let Some(notice) = self.clips_notice.clone() {
                        let rtl = self.rtl();
                        layout::column(ui, rtl, |ui| {
                            let warn = Look::Color(ui.visuals().warn_fg_color);
                            let width = Some(ui.available_width());
                            let w = text::widget_text(ui.ctx(), &notice, rtl, width, warn);
                            ui.add(egui::Label::new(w).wrap());
                            if ui
                                .small_button(self.t(ui.ctx(), "notice-dismiss"))
                                .clicked()
                            {
                                self.clips_notice = None;
                            }
                        });
                    }

                    if self.scan.is_some() {
                        self.status_bar(ui);
                    }

                    if !self.rejected.is_empty() {
                        let names = self.rejected.join(", ");
                        let msg = self
                            .catalog
                            .text_with("drop-not-demo", &[("names", FluentValue::from(names))]);
                        text::label(ui, &msg, self.rtl());
                    }

                    ui.add_space(2.);
                });
        }

        if self.library.as_ref().is_some_and(|l| l.selected.is_some()) {
            egui::Panel::bottom("details").show(ui, |ui| {
                ui.add_space(4.);
                self.details(ui);
                ui.add_space(4.);
            });
        }
        let rtl = self.rtl();

        // A selected demo opens beside the library, replacing an earlier one,
        // and otherwise a demo opened as a file
        let selected = self
            .library
            .as_ref()
            .and_then(|l| l.selected.as_ref())
            .map(|f| (f.source(), f.duration.unwrap_or(0.)));
        if selected.is_some() {
            self.direct = None;
        }
        let selected = selected.or_else(|| self.direct.clone().map(|p| (p, 0.)));
        match (&selected, &self.demo) {
            (Some((path, _)), Some(d)) if *path == d.path => {}
            (Some((path, duration)), _) => {
                self.drop_demo();

                let mut d = DemoView::open(path.clone(), *duration, &ctx);
                d.open_window = self.open_window.take();
                d.export_on_load = std::mem::take(&mut self.export_on_open);
                d.show_timeline = self.settings.show_timeline;
                d.clips = self.clips.get(path);
                self.demo = Some(d);
            }
            (None, _) => self.drop_demo(),
        }
        self.open_viewer(&ctx);

        // The demo takes in its background work once, before any of it draws
        self.apply_demo_settings();
        if let Some(d) = &mut self.demo {
            d.begin_frame(&ctx);
        }

        let compact = self.demo.is_some();
        let library = |app: &mut Self, ui: &mut egui::Ui| match &mut app.library {
            Some(l) => l.ui(ui, &app.catalog, rtl, compact),
            None => {
                text::label(ui, &app.catalog.text("library-no-cache"), rtl);
            }
        };
        if let Some(l) = &mut self.library
            && l.columns_changed
        {
            l.columns_changed = false;
            self.settings.hidden_columns = l.hidden.clone();
            self.save_settings();
        }
        if self.demo.is_some() {
            let width = ctx.content_rect().width();
            layout::leading_panel("library", rtl)
                .resizable(true)
                .default_size(width * 0.42)
                .size_range(320.0..=width * 0.8)
                .show(ui, |ui| library(self, ui));
            egui::CentralPanel::default().show(ui, |ui| {
                if let Some(d) = &mut self.demo {
                    d.ui(ui, &self.catalog, rtl);
                }
            });

            let mut close = false;
            if let Some(d) = &mut self.demo {
                close = d.finish_frame(&ctx, &self.catalog);

                // The viewer's volume slider and see-through button are
                // settings
                if let Some(v) = &d.viewer {
                    let saved = &mut self.settings.viewer;
                    // As the run's flag left it, which is not saved
                    let shown = saved.see_through || self.viewer_see_through;
                    if (v.settings.volume, v.settings.see_through) != (saved.volume, shown) {
                        saved.volume = v.settings.volume;
                        saved.see_through = v.settings.see_through;
                        self.viewer_unsaved = true;
                    }
                }
            }
            if self.viewer_unsaved && !ui.input(|i| i.pointer.any_down()) {
                self.viewer_unsaved = false;
                self.save_settings();
            }
            if let Some(d) = &mut self.demo
                && d.clips_changed
            {
                d.clips_changed = false;
                let (path, clips) = (d.path.clone(), d.clips.clone());
                self.save_clips(&path, clips);
            }
            // The timeline choice carries over to the next demo and the next run
            if let Some(d) = &self.demo
                && d.show_timeline != self.settings.show_timeline
            {
                self.settings.show_timeline = d.show_timeline;
                self.save_settings();
            }
            if close {
                self.drop_demo();
                self.direct = None;
                if let Some(l) = &mut self.library {
                    l.selected = None;
                }
            }
        } else {
            egui::CentralPanel::default().show(ui, |ui| library(self, ui));
        }

        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::unique("drop"),
            ));
            let rect = ctx.content_rect();
            painter.rect_filled(rect, 0., egui::Color32::from_black_alpha(160));
            let hint = text::widget_text(
                &ctx,
                &self.catalog.text("drop-hint"),
                rtl,
                None,
                Look::Strong,
            );
            let galley = hint.into_galley(ui, None, f32::INFINITY, egui::TextStyle::Heading);
            painter.galley(
                rect.center() - galley.size() / 2.,
                galley,
                egui::Color32::WHITE,
            );
        }

        self.settings_window(&ctx);
        self.problems_window(&ctx);
        self.take_screenshot(&ctx);
    }
}

/// The settings window's tabs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SettingsTab {
    #[default]
    General,
    /// What changes only how the demo viewer shows and plays a demo
    Viewer,
}

/// A section's heading, with space above it unless it is the first, and a
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
        ui.add_space(10.);
    }
    layout::row(ui, rtl, |ui| {
        ui.label(text::widget_text(
            ui.ctx(),
            &catalog.text(id),
            rtl,
            None,
            Look::Strong,
        ));
        if let Some(hint) = hint {
            widgets::help(ui, &catalog.text(hint), rtl);
        }
    });
}

/// A setting's label, with a question mark explaining `hint` when there is
/// one
fn labelled(ui: &mut egui::Ui, catalog: &Catalog, id: &str, hint: Option<&str>, rtl: bool) {
    layout::row(ui, rtl, |ui| {
        ui.label(text::widget_text(
            ui.ctx(),
            &catalog.text(id),
            rtl,
            None,
            Look::Plain,
        ));
        if let Some(hint) = hint {
            widgets::help(ui, &catalog.text(hint), rtl);
        }
    });
}

/// A folder or file the settings name, or what stands in for one
fn path(ui: &mut egui::Ui, shown: &str, rtl: bool) {
    ui.label(text::widget_text(ui.ctx(), shown, rtl, None, Look::Weak));
}

/// The width of the widest of these labels, for their fields to line up
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

/// A row with a label `width` wide and its field after it, so the fields of
/// rows one under another line up
fn field(
    ui: &mut egui::Ui,
    catalog: &Catalog,
    id: &str,
    width: f32,
    rtl: bool,
    add: impl FnOnce(&mut egui::Ui),
) {
    layout::row(ui, rtl, |ui| {
        layout::cell(ui, rtl, width, |ui| {
            ui.label(text::widget_text(
                ui.ctx(),
                &catalog.text(id),
                rtl,
                None,
                Look::Plain,
            ));
        });
        add(ui);
    });
}
