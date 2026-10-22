//! The application window

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{self, RichText, WidgetText};
use fluent_bundle::FluentValue;
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::library::cache::default_path;
use lambda_slicer_core::library::scan::{ScanFloors, ScanOptions};

mod settings_window;

pub use settings_window::SettingsTab;

use crate::clips::{ClipStore, LoadProblem};
use crate::demo::{self, DemoView, InspectorTab, Request, Setup, Surroundings, WorkspaceUi};
use crate::export::FileNaming;
use crate::game_setup::{self, Refusal};
use crate::i18n::{Catalog, Locale};
use crate::jobs::{Problem, ScanJob};
use crate::layout;
use crate::library::{LibraryView, ListAction};
use crate::settings::Settings;
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Icon, IconButton, Variant};

/// The narrow demo list's width beside an open demo at first, and the
/// narrowest and widest it can be dragged to
const LIST_WIDTH: f32 = 280.;
const LIST_WIDTH_RANGE: std::ops::RangeInclusive<f32> = 240.0..=360.0;

/// The narrowest a window keeps the list, the view and the inspector side by
/// side in, and the least room the view keeps between them
const THREE_PANES: f32 = 1180.;
const MIDDLE_MIN: f32 = 500.;
/// Narrower than this the inspector goes under the view
const NARROW_WINDOW: f32 = 960.;
/// How much more room than its threshold the list wants back before it
/// returns, so resizing near the threshold does not make it flicker
const ROOM_MARGIN: f32 = 32.;

/// How far from its first width the inspector must be to count as dragged
const DRAGGED: f32 = 0.5;

/// Save a screenshot after this many frames and close, for checking how a
/// language renders
pub struct Screenshot {
    pub path: PathBuf,
    pub frames: u32,
}

/// Choices given on the command line for one run
pub struct Launch {
    /// Open the settings window, asking whether to restore the defaults
    /// with `confirm_restore` or to clear the library with `confirm_clear`
    pub show_settings: bool,
    pub confirm_restore: bool,
    pub confirm_clear: bool,
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
    /// The shown event selected, the kill refinements on and the player's
    /// name filtered by in that demo
    pub event: Option<usize>,
    pub refine: crate::kill_filters::KillFilters,
    pub player: String,
    /// The kinds of event shown, comma-separated, none for the usual ones
    pub show: Option<String>,
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
    /// The inspector tab shown at start
    pub tab: InspectorTab,
    /// The settings window's section shown at start
    pub section: SettingsTab,
}

pub struct App {
    settings: Settings,
    settings_path: Option<PathBuf>,
    catalog: Catalog,
    title: Option<Locale>,
    show_settings: bool,
    /// The section the settings window shows, the last one visited this run
    settings_tab: SettingsTab,
    /// The settings window has the keys, so playback leaves them alone
    settings_keyboard: bool,
    /// Asking whether to restore the default settings
    confirm_restore: bool,
    /// Asking whether to clear the library, and clearing it once no scan
    /// writes to it, then scanning again
    confirm_clear: bool,
    clear_library: bool,
    /// A control had the focus at the end of the last frame, so an Escape
    /// now only takes the focus from it
    focused_last_frame: bool,
    show_problems: bool,
    screenshot: Option<Screenshot>,
    /// A screenshot was asked for and its image has not come back
    screenshot_requested: bool,
    cache_path: Option<PathBuf>,
    clips: ClipStore,
    clips_path: Option<PathBuf>,
    /// A problem with the clips file, shown until dismissed
    clips_notice: Option<String>,
    /// Why a folder could not be shown, until dismissed
    path_notice: Option<String>,
    library: Option<LibraryView>,
    scan: Option<ScanJob>,
    demo: Option<DemoView>,
    open_window: Option<usize>,
    open_event: Option<usize>,
    open_refine: Option<crate::kill_filters::KillFilters>,
    open_player: String,
    open_show: Option<String>,
    export_on_open: bool,
    /// A scan to start once the stopping one has stopped
    rescan: bool,
    /// Clips exported into the library folders, scanned on their own once
    /// no scan runs, so they show without a rescan
    exported: Vec<PathBuf>,
    /// The game folder the last scan read maps from, none for a scan
    /// without the game's files, and nothing before the first scan
    scanned_with: Option<Option<PathBuf>>,
    /// The floor under players on the maps of a game folder, kept while
    /// the folder stays the same
    floors: Option<(PathBuf, ScanFloors)>,
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
    /// A setting changed, saved once the pointer is let go
    settings_unsaved: bool,
    /// The window's GPU device, which the viewer draws with
    pub render_state: Option<eframe::egui_wgpu::RenderState>,
    /// The app's icon for the toolbar, made on the first frame
    brand: Option<egui::TextureHandle>,
    /// How the open demo's panes are arranged
    workspace: WorkspaceUi,
    /// Why the last choice of the game's installation was refused
    setup_refusal: Option<Refusal>,
    /// The game folder last checked for the game's files, and whether it
    /// held them, so the disk is read once for each folder
    install_checked: Option<(PathBuf, bool)>,
    /// The same for the game's executable the settings name
    executable_checked: Option<(PathBuf, bool)>,
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
        // Settings saved before the executable was kept find it in their
        // folder, saved with the next change
        let mut settings = settings;
        if settings.viewer.executable.is_none()
            && let Some(root) = &settings.viewer.game
        {
            settings.viewer.executable = game_setup::executable_in(root);
        }

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
        if let Some(l) = &mut library {
            l.filters.min_minutes = settings.start_min_minutes;
            l.filters.min_kills = settings.start_min_kills;
        }

        // The filters as they were left, and the order chosen to start with
        if let Some(l) = &mut library {
            l.filters_open = settings.layout.filters_open;
            l.list_filters_open = settings.layout.list_filters_open;
            l.sort_by_key(&settings.sort_column, settings.sort_descending);
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
        let settings_layout = settings.layout.clone();
        Self {
            settings,
            settings_path,
            clips,
            clips_path,
            clips_notice,
            path_notice: None,
            catalog,
            title: None,
            show_settings: launch.show_settings,
            settings_tab: launch.section,
            settings_keyboard: false,
            confirm_restore: launch.confirm_restore,
            confirm_clear: launch.confirm_clear,
            clear_library: false,
            focused_last_frame: false,
            show_problems: false,
            screenshot: launch.screenshot,
            screenshot_requested: false,
            cache_path,
            library,
            scan: None,
            demo: None,
            open_window: launch.window,
            open_event: launch.event,
            open_refine: Some(launch.refine),
            open_player: launch.player,
            open_show: launch.show,
            export_on_open: launch.export,
            rescan: false,
            exported: Vec::new(),
            scanned_with: None,
            floors: None,
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
            settings_unsaved: false,
            render_state: None,
            brand: None,
            workspace: WorkspaceUi {
                inspector_tab: launch.tab,
                inspector_width: settings_layout.inspector_width,
                list_hidden: settings_layout.list_hidden,
                ..WorkspaceUi::default()
            },
            setup_refusal: None,
            install_checked: None,
            executable_checked: None,
        }
    }

    fn rtl(&self) -> bool {
        self.catalog.locale.rtl()
    }

    /// A message as widget text, ordered for the interface's direction
    fn t(&self, ctx: &egui::Context, id: &str) -> WidgetText {
        text::widget_text(ctx, &self.catalog.text(id), self.rtl(), None, Look::Plain)
    }

    /// The folder the viewer reads the game's files from, the one `--game`
    /// names before the one in Settings, when it holds the game. Whether it
    /// does is read from disk once for each folder, not every frame
    fn install_root(&mut self) -> Option<PathBuf> {
        let wanted = self
            .game
            .clone()
            .or_else(|| self.settings.viewer.game.clone())?;
        let checked = matches!(&self.install_checked, Some((folder, _)) if *folder == wanted);
        if !checked {
            let installed = game_setup::looks_installed(&wanted);
            self.install_checked = Some((wanted.clone(), installed));
        }

        let installed = self.install_checked.as_ref().is_some_and(|(_, ok)| *ok);
        installed.then_some(wanted)
    }

    /// Whether the game's executable the settings name is still there,
    /// read from disk once for each path
    fn executable_present(&mut self) -> Option<bool> {
        let executable = self.settings.viewer.executable.clone()?;
        let checked = matches!(&self.executable_checked, Some((path, _)) if *path == executable);
        if !checked {
            let present = executable.is_file();
            self.executable_checked = Some((executable, present));
        }

        self.executable_checked
            .as_ref()
            .map(|(_, present)| *present)
    }

    /// Why the viewer has no game files, for its setup notice
    fn setup(&self) -> Setup {
        let missing = match &self.install_checked {
            Some((folder, false)) => Some(folder.clone()),
            _ => None,
        };
        Setup {
            missing,
            refusal: self
                .setup_refusal
                .as_ref()
                .map(|r| r.message(&self.catalog)),
        }
    }

    /// Ask where the game is installed, through the same dialog the setup
    /// notice and Settings use, and keep the choice when it holds the game.
    /// A cancelled dialog changes nothing
    fn locate_install(&mut self) {
        let start = self
            .game
            .clone()
            .or_else(|| self.settings.viewer.game.clone());
        let filter = self.catalog.text("dialog-executables");
        if let Some(chosen) = game_setup::pick(start.as_deref(), &filter) {
            self.accept_install(&chosen);
        }
    }

    /// Keep `chosen` as the game's installation when it holds the game, and
    /// otherwise say why, keeping the installation chosen before
    fn accept_install(&mut self, chosen: &Path) {
        match game_setup::accept(chosen) {
            Ok(found) => {
                self.settings.viewer.game = Some(found.root);
                self.settings.viewer.executable = found.executable;
                self.setup_refusal = None;
                self.save_settings();

                // Read again even for the same folder, which may only now
                // hold the game, as when its drive is back
                self.install_checked = None;
                self.executable_checked = None;
            }
            Err(refusal) => self.setup_refusal = Some(refusal),
        }
    }

    /// The demo's viewer, reading the game's files when they are found and
    /// otherwise playing the demo with nothing to draw or hear. It opens
    /// again when the folder it reads changes, going on where it stood
    fn open_viewer(&mut self, ctx: &egui::Context) {
        let install = self.install_root();
        let (Some(d), Some(render_state)) = (&mut self.demo, &self.render_state) else {
            return;
        };
        if d.viewer.is_some() && d.viewer_game == install {
            return;
        }

        let carry = d.viewer.as_ref().map(crate::viewer::Viewer::playback);
        let dev = crate::viewer::Dev {
            start: self.viewer_at.take(),
            bench: self.viewer_bench.take(),
            capture: self.viewer_capture.take(),
            play: self.viewer_play.take(),
        };
        d.viewer_game = install.clone();
        d.viewer = Some(crate::viewer::Viewer::open(
            render_state.clone(),
            install,
            d.path.clone(),
            d.duration,
            dev,
            carry,
            ctx,
        ));
    }

    /// Give the open demo the settings it works with this frame, this run's
    /// choices from the command line over the saved ones
    fn apply_demo_settings(&mut self) {
        let Some(d) = &mut self.demo else {
            return;
        };
        d.padding = (self.settings.preroll, self.settings.postroll);
        d.export_folder = self.settings.export_folder.clone();
        d.file_naming = FileNaming {
            clip_name: self.settings.clip_name_files,
            demo_first: self.settings.demo_name_first,
        };

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

    /// Whether the window has room for the demo list beside the demo, and
    /// whether it is too narrow for the inspector beside the view. Losing
    /// room hides the list until asked for without changing whether it was
    /// hidden, and it returns once there is a little more room than it lost
    /// it at. Returns true for a narrow window
    fn arrange(&mut self, ctx: &egui::Context, list_id: egui::Id, width: f32) -> bool {
        let narrow = width < NARROW_WINDOW;
        let list = egui::containers::panel::PanelState::load(ctx, list_id)
            .map_or(LIST_WIDTH, |state| state.size().x);

        // The inspector as wide as it was dragged, its first width before
        // it was drawn
        let inspector = match self.workspace.inspector_width {
            _ if narrow => 0.,
            Some(width) => width,
            None => demo::INSPECTOR_WIDTH,
        };

        let margin = if self.workspace.list_squeezed {
            ROOM_MARGIN
        } else {
            0.
        };
        let middle = width - list - inspector;
        let room = width >= THREE_PANES + margin && middle >= MIDDLE_MIN + margin;
        if room {
            self.workspace.drawer_open = false;
        }
        self.workspace.list_squeezed = !room;

        narrow
    }

    /// The narrow demo list, or why there is none
    fn list_contents(&mut self, ui: &mut egui::Ui, rtl: bool) -> Option<ListAction> {
        match &mut self.library {
            Some(l) => l.list_ui(ui, &self.catalog, rtl),
            None => {
                text::label(ui, &self.catalog.text("library-no-cache"), rtl);
                None
            }
        }
    }

    /// The demo list over the view from the leading edge, while the window
    /// has no room for it beside the view. Choosing a demo, a click outside
    /// it or Escape closes it
    fn drawer(&mut self, ctx: &egui::Context, area: egui::Rect, rtl: bool) -> Option<ListAction> {
        let x = if rtl {
            area.max.x - LIST_WIDTH
        } else {
            area.min.x
        };
        let rect = egui::Rect::from_min_size(
            egui::pos2(x, area.min.y),
            egui::vec2(LIST_WIDTH, area.height()),
        );

        let chosen_before = self
            .library
            .as_ref()
            .and_then(|l| l.selected.as_ref().map(|f| f.id));

        let mut action = None;
        let frame = egui::Frame::new()
            .fill(theme::PANE)
            .stroke(egui::Stroke::new(1., theme::BORDER));
        let drawn = egui::Area::new(egui::Id::unique("list drawer"))
            .fixed_pos(rect.min)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                frame.show(ui, |ui| {
                    ui.set_min_size(rect.size());
                    ui.set_max_size(rect.size());
                    action = self.list_contents(ui, rtl);
                });
            });

        let chosen = self
            .library
            .as_ref()
            .and_then(|l| l.selected.as_ref().map(|f| f.id));
        let pointer = ctx.input(|i| i.pointer.interact_pos());
        let clicked_outside = ctx.input(|i| i.pointer.any_click())
            && pointer.is_some_and(|p| !drawn.response.rect.contains(p));
        let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if chosen != chosen_before || clicked_outside || escape {
            self.workspace.drawer_open = false;
        }

        action
    }

    /// The open demo beside the narrow demo list, or with the list over it
    /// while the window has no room for it, then what the demo asked of the
    /// app and the settings and clips it changed
    fn workspace_ui(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        let area = ui.available_rect_before_wrap();

        // The list's panel keeps its width under the id egui gives it here
        let list_id = ui.scope_id().with_salt(egui::IdSalt::new("library"));
        let narrow = self.arrange(&ctx, list_id, area.width());

        let mut list_action = None;
        if !self.workspace.list_hidden && !self.workspace.list_squeezed {
            let pane = egui::Frame::side_top_panel(&ctx.global_style()).fill(theme::PANE);
            layout::leading_panel("library", rtl)
                .frame(pane)
                .resizable(true)
                .default_size(LIST_WIDTH)
                .size_range(LIST_WIDTH_RANGE)
                .show(ui, |ui| list_action = self.list_contents(ui, rtl));
        }

        let setup = self.setup();
        let file = self.library.as_ref().and_then(|l| l.selected.as_ref());
        let surroundings = Surroundings {
            catalog: &self.catalog,
            rtl,
            file,
            narrow,
            setup: &setup,
        };
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(d) = &mut self.demo {
                d.ui(ui, &surroundings, &mut self.workspace);
            }
        });

        if self.workspace.list_squeezed && self.workspace.drawer_open {
            list_action = list_action.or(self.drawer(&ctx, area, rtl));
        }

        // Back to the table closes the demo as Close does, and hiding the
        // list keeps the demo open
        let mut close = list_action == Some(ListAction::Back);
        if list_action == Some(ListAction::Hide) {
            if self.workspace.list_squeezed {
                self.workspace.drawer_open = false;
            } else {
                self.workspace.list_hidden = true;
            }
        }

        let requests = match &mut self.demo {
            Some(d) => d.finish_frame(&ctx, &self.catalog),
            None => Vec::new(),
        };
        for request in requests {
            match request {
                Request::Close => close = true,
                Request::Settings(tab) => {
                    self.show_settings = true;
                    self.settings_tab = tab;
                }
                Request::Locate => self.locate_install(),
                Request::ShowList if self.workspace.list_squeezed => {
                    self.workspace.drawer_open = true;
                }
                Request::ShowList => self.workspace.list_hidden = false,
                Request::Notice(notice) => self.path_notice = Some(notice),
                Request::Exported(path) => {
                    let in_library = self.settings.folders.iter().any(|f| path.starts_with(f));
                    if in_library {
                        self.exported.push(path);
                    }
                }
            }
        }

        // The viewer's volume slider and see-through button are settings
        if let Some(v) = self.demo.as_ref().and_then(|d| d.viewer.as_ref()) {
            let saved = &mut self.settings.viewer;

            // As the run's flag left it, which is not saved
            let shown = saved.see_through || self.viewer_see_through;
            if (v.settings.volume, v.settings.see_through) != (saved.volume, shown) {
                saved.volume = v.settings.volume;
                saved.see_through = v.settings.see_through;
                self.settings_unsaved = true;
            }
        }

        if let Some(d) = &mut self.demo
            && d.clips_changed
        {
            d.clips_changed = false;
            let (path, clips) = (d.path.clone(), d.clips.clone());
            self.save_clips(&path, clips);
        }

        // The player lanes carry over to the next demo and the next run
        if let Some(d) = &self.demo
            && d.lanes_open != self.settings.player_lanes
        {
            self.settings.player_lanes = d.lanes_open;
            self.settings_unsaved = true;
        }

        if close {
            self.close_demo();
        }
    }

    /// Close the open demo, back to the library
    fn close_demo(&mut self) {
        self.drop_demo();
        self.direct = None;
        if let Some(l) = &mut self.library {
            l.selected = None;
        }
    }

    /// Ask which demos to open
    fn open_dialog(&mut self, ctx: &egui::Context) {
        let files = rfd::FileDialog::new()
            .add_filter(self.catalog.text("dialog-demos"), &["dem"])
            .pick_files();
        if let Some(files) = files {
            self.open_files(ctx, files);
        }
    }

    /// Ctrl+O opens demos and Ctrl+comma opens or closes the settings, in
    /// any part of the app
    fn app_shortcuts(&mut self, ctx: &egui::Context) {
        let command = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, key));
        if command(egui::Key::O) {
            self.open_dialog(ctx);
        }
        if command(egui::Key::Comma) {
            self.show_settings = !self.show_settings;
        }
    }

    /// Escape nothing closer took, a drag, a menu, the list over the view or
    /// a control leaving its focus, closes the settings, or else the demo
    fn escape(&mut self, ctx: &egui::Context) {
        let focused = std::mem::replace(
            &mut self.focused_last_frame,
            ctx.memory(|m| m.focused().is_some()),
        );
        if focused || egui::Popup::is_any_open(ctx) || text::typing(ctx) {
            return;
        }
        if !ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            return;
        }

        if self.show_settings {
            self.show_settings = false;
        } else if self.demo.is_some() {
            self.close_demo();
        }
    }

    /// Note how the window is laid out, for the next run to open the same
    /// way. The settings window keeps its own place as it moves
    fn keep_layout(&mut self) {
        let mut layout = self.settings.layout.clone();
        if let Some(l) = &self.library {
            layout.filters_open = l.filters_open;
            layout.list_filters_open = l.list_filters_open;
        }
        layout.list_hidden = self.workspace.list_hidden;

        // The inspector's width only once dragged away from its first one
        if let Some(width) = self.workspace.inspector_width {
            let dragged = (width - self.workspace.inspector_first).abs() > DRAGGED;
            layout.inspector_width = dragged.then_some(width);
        }

        if layout != self.settings.layout {
            self.settings.layout = layout;
            self.settings_unsaved = true;
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
        self.exported.clear();
        self.scanned_with = Some(self.install_root());
        let options = ScanOptions {
            roots,
            cache,
            workers: usize::from(self.settings.workers),
            force: false,
            full_hash: true,
            limits: Limits::default(),
            floors: self.floor_reader(),
            forget_missing: true,
        };
        self.scan = Some(ScanJob::start(options, ctx.clone()));
    }

    /// Scan the clips exported into the library on their own, once no other
    /// scan runs. A scan of the whole library still to come finds them anyway
    fn scan_exported(&mut self, ctx: &egui::Context) {
        if self.exported.is_empty() || self.scanning() || self.rescan {
            return;
        }
        let Some(cache) = self.cache_path.clone() else {
            return;
        };

        let options = ScanOptions {
            roots: std::mem::take(&mut self.exported),
            cache,
            workers: usize::from(self.settings.workers),
            force: false,
            full_hash: true,
            limits: Limits::default(),
            floors: self.floor_reader(),
            forget_missing: false,
        };
        self.scan = Some(ScanJob::start(options, ctx.clone()));
    }

    /// The floor under players on the game's maps, none without the game's
    /// files
    fn floor_reader(&mut self) -> Option<ScanFloors> {
        let install = self.install_root()?;
        let kept = self
            .floors
            .as_ref()
            .is_some_and(|(folder, _)| *folder == install);
        if !kept {
            self.floors = Some((install.clone(), crate::floors::reader(&install)));
        }
        self.floors.as_ref().map(|(_, floors)| floors.clone())
    }

    /// Scan again when the game's files change, so midair kills are told
    /// with the new maps or without any
    fn rescan_for_game(&mut self, ctx: &egui::Context) {
        let Some(scanned) = self.scanned_with.clone() else {
            return;
        };
        if scanned != self.install_root() {
            self.start_scan(ctx);
        }
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
            {
                self.open_dialog(&ctx);
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

        // Without a scan, how many demos the library holds
        let Some(job) = &self.scan else {
            let files = self.library.as_ref().map_or(0, LibraryView::library_files);
            let count = FluentValue::from(files as i64);
            let line = self.catalog.text_with("library-total", &[("count", count)]);
            layout::row(ui, rtl, |ui| {
                ui.label(text::widget_text(&ctx, &line, rtl, None, Look::Weak));
            });
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

    /// A notice that wraps, in the warning colour, with its dismiss button
    /// under it. Returns true when dismissed
    fn notice(&self, ui: &mut egui::Ui, notice: &str) -> bool {
        let rtl = self.rtl();
        layout::column(ui, rtl, |ui| {
            let warn = Look::Color(ui.visuals().warn_fg_color);
            let width = Some(ui.available_width());
            let w = text::widget_text(ui.ctx(), notice, rtl, width, warn);
            ui.add(egui::Label::new(w).wrap());

            ui.small_button(self.t(ui.ctx(), "notice-dismiss"))
                .clicked()
        })
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
        self.app_shortcuts(&ctx);
        if let Some(job) = &mut self.scan
            && job.drain()
            && let Some(l) = &mut self.library
        {
            l.mark_stale();
        }

        // The library is cleared once a scan stopped writing to it, and
        // filled again from the start
        if self.clear_library && !self.scanning() {
            self.clear_library = false;
            if let Some(l) = &mut self.library {
                l.clear();
            }
            self.start_scan(&ctx);
        }
        if self.rescan && !self.scanning() {
            self.start_scan(&ctx);
        }
        self.rescan_for_game(&ctx);
        self.scan_exported(&ctx);

        // The toolbar's controls sit in the middle of its height
        let margin_y = (theme::TOOLBAR_HEIGHT - theme::CONTROL_HEIGHT) / 2.;
        let toolbar = egui::Frame::new()
            .fill(theme::SURFACE)
            .inner_margin(egui::Margin::symmetric(theme::GAP_M as i8, margin_y as i8));
        egui::Panel::top("toolbar")
            .exact_size(theme::TOOLBAR_HEIGHT)
            .frame(toolbar)
            .show(ui, |ui| self.toolbar(ui));

        // Always there, so the library's state can always be read
        let status = egui::Frame::new()
            .fill(theme::SURFACE)
            .inner_margin(egui::Margin::symmetric(theme::GAP_M as i8, 2));
        egui::Panel::bottom("status")
            .min_size(theme::STATUS_HEIGHT)
            .frame(status)
            .show(ui, |ui| {
                ui.add_space(2.);

                // Notices wrap, each with its dismiss button under it
                if let Some(notice) = self.clips_notice.clone()
                    && self.notice(ui, &notice)
                {
                    self.clips_notice = None;
                }
                if let Some(notice) = self.path_notice.clone()
                    && self.notice(ui, &notice)
                {
                    self.path_notice = None;
                }

                self.status_bar(ui);

                if !self.rejected.is_empty() {
                    let names = self.rejected.join(", ");
                    let msg = self
                        .catalog
                        .text_with("drop-not-demo", &[("names", FluentValue::from(names))]);
                    text::label(ui, &msg, self.rtl());
                }

                ui.add_space(2.);
            });

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

                let floors = self.floor_reader().map(|f| f.floors);
                let mut d = DemoView::open(path.clone(), *duration, floors, &ctx);
                d.open_window = self.open_window.take();
                d.open_event = self.open_event.take();
                if let Some(filters) = self.open_refine.take() {
                    d.refine(filters);
                }
                d.filter_player(std::mem::take(&mut self.open_player));
                if let Some(kinds) = self.open_show.take() {
                    d.show_kinds(&kinds);
                }
                d.export_on_load = std::mem::take(&mut self.export_on_open);
                d.lanes_open = self.settings.player_lanes;
                d.clips = self.clips.get(path);
                self.demo = Some(d);
            }
            (None, _) => self.drop_demo(),
        }
        self.open_viewer(&ctx);

        // The demo takes in its background work once, before any of it draws
        self.apply_demo_settings();
        if let Some(d) = &mut self.demo {
            d.begin_frame(&ctx, !self.settings_keyboard);
        }

        if let Some(l) = &mut self.library
            && l.columns_changed
        {
            l.columns_changed = false;
            self.settings.hidden_columns = l.hidden.clone();
            self.save_settings();
        }
        if self.demo.is_some() {
            self.workspace_ui(ui, rtl);
        } else {
            egui::CentralPanel::default().show(ui, |ui| match &mut self.library {
                Some(l) => l.table_ui(ui, &self.catalog, rtl),
                None => {
                    text::label(ui, &self.catalog.text("library-no-cache"), rtl);
                }
            });
        }

        // A folder a right click on a demo could not show
        if let Some(notice) = self.library.as_mut().and_then(|l| l.path_notice.take()) {
            self.path_notice = Some(notice);
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
        self.escape(&ctx);
        self.keep_layout();

        // Saved once nothing is being dragged, so a slider is not written out
        // on every step
        if self.settings_unsaved && !ui.input(|i| i.pointer.any_down()) {
            self.settings_unsaved = false;
            self.save_settings();
        }

        self.take_screenshot(&ctx);
    }
}
