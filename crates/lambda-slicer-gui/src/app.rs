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
use crate::settings::{MAX_PADDING, Settings};
use crate::text::{self, Look};

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
    /// The game's installation folder, for the backend spike's view
    pub game: Option<PathBuf>,
    /// Where the viewer seeks to once the demo is loaded
    pub viewer_at: Option<f64>,
    pub viewer_bench: Option<crate::viewer::Bench>,
}

pub struct App {
    settings: Settings,
    settings_path: Option<PathBuf>,
    catalog: Catalog,
    title: Option<Locale>,
    show_settings: bool,
    show_problems: bool,
    screenshot: Option<Screenshot>,
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
    /// The window's GPU device, which the viewer draws with
    pub render_state: Option<eframe::egui_wgpu::RenderState>,
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
            show_problems: false,
            screenshot: launch.screenshot,
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
            render_state: None,
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

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        layout::row(ui, self.rtl(), |ui| {
            if ui.button(self.t(&ctx, "toolbar-open")).clicked()
                && let Some(files) = rfd::FileDialog::new()
                    .add_filter(self.catalog.text("dialog-demos"), &["dem"])
                    .pick_files()
            {
                self.open_files(&ctx, files);
            }
            if ui.button(self.t(&ctx, "toolbar-add-folder")).clicked()
                && let Some(folders) = rfd::FileDialog::new().pick_folders()
            {
                self.add_folders(&ctx, folders);
            }
            let can_scan =
                !self.scanning() && (!self.settings.folders.is_empty() || !self.opened.is_empty());
            if ui
                .add_enabled(can_scan, egui::Button::new(self.t(&ctx, "toolbar-rescan")))
                .clicked()
            {
                self.start_scan(&ctx);
            }
            let can_cancel = self
                .scan
                .as_ref()
                .is_some_and(|j| j.running() && !j.cancelling());
            if ui
                .add_enabled(
                    can_cancel,
                    egui::Button::new(self.t(&ctx, "toolbar-cancel")),
                )
                .clicked()
                && let Some(job) = &self.scan
            {
                job.cancel();
            }
            ui.separator();
            if ui
                .selectable_label(self.show_settings, self.t(&ctx, "toolbar-settings"))
                .clicked()
            {
                self.show_settings = !self.show_settings;
            }
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
                layout::column(ui, self.rtl(), |ui| {
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
                        self.save_settings();
                    }
                    text::label(ui, &self.catalog.text("settings-workers-hint"), rtl);

                    // How much a clip made around an event keeps on each side
                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-clip-padding"));
                    let catalog = &self.catalog;
                    let mut padding = [self.settings.preroll, self.settings.postroll];
                    let mut changed = false;
                    for (id, value) in ["settings-preroll", "settings-postroll"]
                        .into_iter()
                        .zip(&mut padding)
                    {
                        layout::row(ui, rtl, |ui| {
                            let label = text::widget_text(
                                ui.ctx(),
                                &catalog.text(id),
                                rtl,
                                None,
                                Look::Plain,
                            );
                            ui.label(label);
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

                    // Where exported clips go
                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-export-folder"));
                    let folder = match &self.settings.export_folder {
                        Some(f) => f.display().to_string(),
                        None => self.catalog.text("settings-export-beside"),
                    };
                    ui.label(text::widget_text(ui.ctx(), &folder, rtl, None, Look::Plain));
                    let mut chosen = None;
                    layout::row(ui, rtl, |ui| {
                        if ui
                            .button(self.t(ui.ctx(), "settings-export-choose"))
                            .clicked()
                            && let Some(f) = rfd::FileDialog::new().pick_folder()
                        {
                            chosen = Some(Some(f));
                        }
                        if self.settings.export_folder.is_some()
                            && ui
                                .button(self.t(ui.ctx(), "settings-export-beside"))
                                .clicked()
                        {
                            chosen = Some(None);
                        }
                    });
                    if let Some(f) = chosen {
                        self.settings.export_folder = f;
                        self.save_settings();
                    }
                    text::label(ui, &self.catalog.text("settings-export-hint"), rtl);

                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-folders"));
                    if self.settings.folders.is_empty() {
                        ui.label(self.t(ui.ctx(), "settings-folders-empty"));
                    }
                    let mut remove = None;
                    for (i, f) in self.settings.folders.iter().enumerate() {
                        layout::row(ui, self.rtl(), |ui| {
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
                        self.save_settings();
                    }

                    ui.add_space(8.);
                    ui.label(self.strong(ui.ctx(), "settings-cache"));
                    let cache = self.cache_path.as_ref().map(|p| p.display().to_string());
                    ui.label(RichText::new(cache.unwrap_or_default()).monospace());
                });
            });
        self.show_settings = open;
    }

    fn take_screenshot(&mut self, ctx: &egui::Context) {
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

        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.add_space(4.);
            self.toolbar(ui);
            ui.add_space(4.);
        });
        if self.scan.is_some() || !self.rejected.is_empty() || self.clips_notice.is_some() {
            egui::Panel::bottom("status").show(ui, |ui| {
                ui.add_space(2.);
                if let Some(notice) = self.clips_notice.clone() {
                    let rtl = self.rtl();
                    // The message wraps, with the button under it
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
                let mut d = DemoView::open(path.clone(), *duration, &ctx);
                d.open_window = self.open_window.take();
                d.export_on_load = std::mem::take(&mut self.export_on_open);
                d.show_timeline = self.settings.show_timeline;
                d.clips = self.clips.get(path);
                if let (Some(game), Some(render_state)) = (&self.game, &self.render_state) {
                    d.viewer = Some(crate::viewer::Viewer::open(
                        render_state.clone(),
                        game.clone(),
                        path.clone(),
                        *duration,
                        self.viewer_at.take(),
                        self.viewer_bench.take(),
                        &ctx,
                    ));
                }
                self.demo = Some(d);
            }
            (None, _) => self.demo = None,
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
            let side = if rtl {
                egui::Panel::right("library")
            } else {
                egui::Panel::left("library")
            };
            let width = ctx.content_rect().width();
            side.resizable(true)
                .default_size(width * 0.42)
                .size_range(320.0..=width * 0.8)
                .show(ui, |ui| library(self, ui));
            let mut close = false;
            egui::CentralPanel::default().show(ui, |ui| {
                if let Some(d) = &mut self.demo {
                    d.padding = (self.settings.preroll, self.settings.postroll);
                    d.export_folder = self.settings.export_folder.clone();
                    close = d.ui(ui, &self.catalog, rtl);
                }
            });
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
                self.demo = None;
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
