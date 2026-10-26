//! The settings window, which can be moved anywhere in the app's window,
//! with a search box and its six tabs down one side and the chosen tab
//! beside them
//!
//! A tab's settings stand in sections, each in a frame named on its top
//! edge as a fieldset is, in two columns when the window is wide enough.
//! While the search box has text, every section with a setting it finds
//! shows instead, under its tab's name, with only the settings found, or
//! all of a section or tab whose name is found
//!
//! Every change applies at once, with no Save to press, and is written out
//! once nothing is being dragged. Each tab keeps its own scroll place, and
//! the window keeps where it was left for the next run

use eframe::egui::{self, Align, Layout, Pos2, Vec2};
use fluent_bundle::FluentValue;
use lambda_slicer_playback::decals::MAX_DECALS;

use super::App;
use crate::i18n::{Catalog, Locale};
use crate::layout;
use crate::library::{self, parse_number};
use crate::settings::{
    FOV, GAME_BOB, MAX_PADDING, MAX_SCORE_ROWS, SPREE_LEAST, SPREE_MOST, ScoreboardShown, Settings,
    SpeedAxes, TEXT_SIZE, VITALS_SIZE, ViewerSettings, WEAPON_BOB,
};
use crate::shortcuts::{self, Action, Chosen, Group, Shortcuts};
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Icon, IconButton, Variant};

/// The window's first size, the width of its list of tabs, and the least
/// room a control takes beside its label before going under it
const WINDOW_SIZE: Vec2 = Vec2::new(940., 690.);
const SIDEBAR_WIDTH: f32 = 160.;
const CONTROL_LEAST: f32 = 160.;

/// The least width of a column of sections, for two to stand side by side
const COLUMN_LEAST: f32 = 330.;

/// The room inside a section's frame, and between its corner and its name
const FRAME_MARGIN: f32 = 12.;
const LEGEND_INSET: f32 = 10.;

/// The room between parts of a section
const SETTING_GAP: f32 = theme::GAP_S;

/// The steps of the picture settings and of the field of view as they are
/// dragged, the decimals they show, and the field of view a manual one
/// starts at
const PICTURE_STEP: f64 = 0.01;
const PICTURE_DECIMALS: usize = 2;
const FOV_STEP: f64 = 0.5;
const FIRST_FOV: f32 = 90.;
/// How a drag changes the weapon's bob, and the decimals it shows
const BOB_STEP: f64 = 0.0005;
const BOB_DECIMALS: usize = 3;

/// The least width of a shortcut's key buttons
const KEY_WIDTH: f32 = 96.;

/// The step and decimals of a clip's padding
const PADDING_STEP: f64 = 0.1;

/// How many kills dragging the killing spree's length moves it for a point
const SPREE_STEP: f64 = 0.2;
const PADDING_DECIMALS: usize = 1;

/// The width of the dialog asking whether to restore the defaults, and the
/// most height its list of settings takes before it scrolls
const DIALOG_WIDTH: f32 = 360.;
const DIALOG_LIST_HEIGHT: f32 = 240.;

/// The settings window's tabs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SettingsTab {
    /// The game, the language and how many demos are analysed at once
    #[default]
    General,
    /// The folders scanned for demos, how the library starts, and where
    /// what is found is kept
    Library,
    /// How the demo viewer shows and plays a demo
    Viewer,
    /// What the HUD over the viewer's picture shows
    Hud,
    /// Where clips are exported, and what a clip keeps around its event
    Clips,
    /// The keys and mouse buttons that run the app's commands
    Shortcuts,
}

impl SettingsTab {
    const ALL: [SettingsTab; 6] = [
        SettingsTab::General,
        SettingsTab::Library,
        SettingsTab::Viewer,
        SettingsTab::Hud,
        SettingsTab::Clips,
        SettingsTab::Shortcuts,
    ];

    fn title(self) -> &'static str {
        match self {
            SettingsTab::General => "settings-tab-general",
            SettingsTab::Library => "settings-tab-library",
            SettingsTab::Viewer => "settings-viewer",
            SettingsTab::Hud => "settings-tab-hud",
            SettingsTab::Clips => "settings-tab-clips",
            SettingsTab::Shortcuts => "settings-tab-shortcuts",
        }
    }

    /// Its sections in the two columns they stand in when the window is
    /// wide enough, one under the other when it isn't
    fn columns(self) -> [&'static [Section]; 2] {
        use Section::*;
        match self {
            SettingsTab::General => [&[Game], &[Language, Workers]],
            SettingsTab::Library => [&[Folders, Storage], &[List, Events]],
            SettingsTab::Viewer => [&[Picture, Sound], &[View, Effects]],
            SettingsTab::Hud => [&[Vitals, Scores, Speeds], &[Chat, Colours, Server]],
            SettingsTab::Clips => [&[ExportFolder, FileNames], &[Padding]],
            SettingsTab::Shortcuts => [
                &[
                    Keys(Group::Anywhere),
                    Keys(Group::Demo),
                    Keys(Group::Playback),
                ],
                &[
                    Keys(Group::View),
                    Keys(Group::Players),
                    Keys(Group::FreeCamera),
                ],
            ],
        }
    }
}

/// A framed group of settings on a tab, named on its frame
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Game,
    Language,
    Workers,
    Folders,
    List,
    Events,
    Storage,
    Picture,
    View,
    Effects,
    Sound,
    Vitals,
    Scores,
    Chat,
    Speeds,
    Server,
    Colours,
    ExportFolder,
    FileNames,
    Padding,
    Keys(Group),
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Section::Game => "settings-section-game",
            Section::Language => "settings-language",
            Section::Workers => "settings-workers",
            Section::Folders => "settings-folders",
            Section::List => "settings-section-list",
            Section::Events => "tab-events",
            Section::Storage => "settings-cache",
            Section::Picture => "settings-section-picture",
            Section::View => "settings-section-view",
            Section::Effects => "settings-section-effects",
            Section::Sound => "settings-section-sound",
            Section::Vitals => "settings-section-vitals",
            Section::Scores => "settings-section-scores",
            Section::Chat => "settings-section-chat",
            Section::Speeds => "settings-section-speeds",
            Section::Server => "settings-section-server",
            Section::Colours => "settings-section-colours",
            Section::ExportFolder => "settings-export-folder",
            Section::FileNames => "settings-file-names",
            Section::Padding => "settings-clip-padding",
            Section::Keys(group) => group.text_id(),
        }
    }

    /// What the question mark beside its name explains, a draft
    /// translation saying so beside the language
    fn hint(self, catalog: &Catalog) -> Option<&'static str> {
        match self {
            Section::Language => (!catalog.locale.reviewed()).then_some("settings-unreviewed"),
            Section::Workers => Some("settings-workers-hint"),
            Section::Folders => Some("settings-folders-hint"),
            Section::Picture => Some("settings-gamma-hint"),
            Section::ExportFolder => Some("settings-export-hint"),
            Section::FileNames => Some("settings-file-names-hint"),
            Section::Padding => Some("settings-clip-padding-hint"),
            _ => None,
        }
    }

    /// The names and explanations of the settings it holds, which the
    /// search looks through. The shortcuts' actions are named apart
    fn words(self) -> &'static [&'static str] {
        match self {
            Section::Game => &[
                "settings-viewer-game",
                "settings-viewer-game-hint",
                "settings-more-games",
                "settings-more-games-hint",
                "settings-game-content",
                "settings-game-content-hint",
                "settings-hd-models",
                "settings-hd-models-hint",
                "settings-addon-content",
                "settings-addon-content-hint",
                "settings-localized-content",
                "settings-localized-content-hint",
            ],
            Section::Language => &["settings-language-system"],
            Section::Workers => &["settings-workers-hint"],
            Section::Folders => &["settings-folders-hint", "toolbar-add-folder"],
            Section::List => &[
                "settings-start-tab",
                "settings-start-tab-hint",
                "settings-sort",
                "settings-sort-hint",
                "settings-start-filters",
                "settings-start-filters-hint",
                "filter-min-minutes",
                "filter-min-kills",
                "settings-hide-unplayed-maps",
            ],
            Section::Events => &[
                "settings-sprees",
                "settings-sprees-hint",
                "settings-spree-least",
            ],
            Section::Storage => &["settings-clear-library"],
            Section::Picture => &[
                "settings-gamma",
                "settings-brightness",
                "settings-texgamma",
                "settings-lightgamma",
                "settings-overbright",
                "settings-gamma-defaults",
            ],
            Section::View => &[
                "settings-fov-recorded",
                "settings-fov",
                "settings-zoom-fov",
                "settings-viewmodel",
                "settings-weapon-bob-recorded",
                "settings-weapon-bob",
                "settings-crosshair",
                "viewer-see-through",
                "settings-see-through-hint",
                "settings-see-through",
                "settings-see-through-items",
                "settings-player-names",
                "settings-player-names-hint",
                "settings-show-position",
                "settings-show-position-hint",
                "settings-events-leave-free",
                "settings-events-leave-free-hint",
            ],
            Section::Effects => &[
                "settings-particles",
                "settings-particles-hint",
                "settings-section-decals",
                "settings-decals-hint",
                "settings-decals",
            ],
            Section::Sound => &["viewer-volume"],
            Section::Vitals => &[
                "settings-vitals",
                "settings-vitals-size",
                "settings-free-vitals",
            ],
            Section::Scores => &[
                "settings-scores",
                "settings-score-rows",
                "settings-scoreboard",
                "settings-scoreboard-playing",
            ],
            Section::Chat => &[
                "settings-chat",
                "settings-chat-places",
                "settings-chat-places-hint",
                "settings-chat-long-jump",
                "settings-chat-long-jump-hint",
            ],
            Section::Speeds => &[
                "settings-speedometer",
                "settings-jump-speed",
                "settings-jump-speed-hint",
                "settings-speed-axes",
                "settings-recorder-fps",
                "settings-recorder-fps-hint",
            ],
            Section::Server => &["settings-server-settings", "settings-hud-messages"],
            Section::Colours => &[
                "settings-hud-color",
                "settings-hud-color-hint",
                "settings-text-color",
                "settings-text-color-hint",
                "settings-text-size",
            ],
            Section::ExportFolder => &["settings-export-choose", "settings-export-beside"],
            Section::FileNames => &["settings-clip-name-files", "settings-demo-name-first"],
            Section::Padding => &["settings-preroll", "settings-postroll"],
            Section::Keys(_) => &[],
        }
    }
}

/// What the search box asks for, in lower case, matched against the
/// settings' names and explanations in the language shown
#[derive(Clone, Debug, Default)]
struct Search(String);

impl Search {
    fn new(text: &str) -> Self {
        Search(text.trim().to_lowercase())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether `text` holds what is searched for, as any does with nothing
    /// searched
    fn finds(&self, text: &str) -> bool {
        self.is_empty() || text.to_lowercase().contains(&self.0)
    }

    /// Whether a setting named and explained by the catalog's `ids` shows
    fn shows(&self, catalog: &Catalog, ids: &[&str]) -> bool {
        self.is_empty() || ids.iter().any(|id| self.finds(&catalog.text(id)))
    }
}

/// The shortcut being chosen, one of an action's two `slot`s, and the
/// modifiers held last frame, to tell one newly held for the free camera
#[derive(Clone, Copy, Debug)]
pub(super) struct Choosing {
    action: Action,
    slot: usize,
    before: egui::Modifiers,
    /// Where its key was last drawn, which the wheel binds over
    rect: egui::Rect,
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

    /// The search box and the list of tabs down the leading side, and the
    /// chosen tab beside them in a scroll area of its own, or what the
    /// search finds on every tab
    fn settings_contents(&mut self, ui: &mut egui::Ui, rtl: bool) {
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
                let hint = self.catalog.text("settings-search");
                let search = egui::TextEdit::singleline(&mut self.settings_search)
                    .hint_text(hint)
                    .background_color(theme::RAISED)
                    .align(if rtl {
                        egui::Align2::RIGHT_CENTER
                    } else {
                        egui::Align2::LEFT_CENTER
                    });
                ui.add(search);
                ui.add_space(theme::GAP_S);

                // A tab clicked while searching goes back to it
                let searching = !self.settings_search.trim().is_empty();
                for tab in SettingsTab::ALL {
                    let chosen = !searching && self.settings_tab == tab;
                    let label = self.t(ui.ctx(), tab.title());
                    if ui.add(egui::Button::selectable(chosen, label)).clicked() {
                        self.settings_tab = tab;
                        self.settings_search.clear();
                    }
                }

                // Restoring the defaults at the foot of the tabs, asked
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

            // Each tab scrolls on its own and keeps its place, and so do
            // the search's results
            let search = Search::new(&self.settings_search);
            let tab = self.settings_tab;
            let shown = search.is_empty().then_some(tab);
            egui::ScrollArea::vertical()
                .id_salt(("settings tab", shown))
                .auto_shrink(false)
                .show(ui, |ui| {
                    layout::column(ui, rtl, |ui| {
                        if search.is_empty() {
                            self.tab_contents(ui, rtl, tab);
                        } else {
                            self.search_contents(ui, rtl, &search);
                        }
                    });
                });
        });
    }

    /// The tab's sections in two columns when there is room for both,
    /// the first column's on the leading side
    fn tab_contents(&mut self, ui: &mut egui::Ui, rtl: bool, tab: SettingsTab) {
        if tab == SettingsTab::Shortcuts {
            self.shortcuts_top(ui, rtl);
        }

        let [first, second] = tab.columns();
        let least = self.column_least(ui, rtl, tab);
        let gap = ui.spacing().item_spacing.x;
        let everything = Search::default();
        if ui.available_width() < 2. * least + gap {
            for &section in first.iter().chain(second) {
                self.section(ui, rtl, section, &everything, None);
            }
            return;
        }

        ui.columns(2, |columns| {
            let (leading, trailing) = columns.split_at_mut(1);
            let (left, right) = if rtl {
                (&mut trailing[0], &mut leading[0])
            } else {
                (&mut leading[0], &mut trailing[0])
            };
            for (ui, sections) in [(left, first), (right, second)] {
                layout::column(ui, rtl, |ui| {
                    for &section in sections {
                        self.section(ui, rtl, section, &everything, None);
                    }
                });
            }
        });
    }

    /// The least width a column of the tab's sections takes, room for a
    /// shortcut's name and its two keys on the Shortcuts tab
    fn column_least(&self, ui: &egui::Ui, rtl: bool, tab: SettingsTab) -> f32 {
        if tab != SettingsTab::Shortcuts {
            return COLUMN_LEAST;
        }
        let names: Vec<String> = Action::ALL.iter().map(|a| a.text(&self.catalog)).collect();
        let spacing = ui.spacing().item_spacing.x;
        texts_width(ui, &names, rtl) + 2. * (KEY_WIDTH + spacing) + 2. * FRAME_MARGIN
    }

    /// Every section on every tab with a setting the search finds, named
    /// after its tab and itself, in one column
    fn search_contents(&mut self, ui: &mut egui::Ui, rtl: bool, search: &Search) {
        let mut found = false;
        for tab in SettingsTab::ALL {
            let [first, second] = tab.columns();
            let tab_title = self.catalog.text(tab.title());
            for &section in first.iter().chain(second) {
                // Every setting of a tab or section named as searched shows
                let named =
                    search.finds(&tab_title) || search.finds(&self.catalog.text(section.title()));
                let within = if named {
                    Search::default()
                } else {
                    search.clone()
                };
                if !named && !self.section_finds(section, search) {
                    continue;
                }
                found = true;
                self.section(ui, rtl, section, &within, Some(&tab_title));
            }
        }

        if !found {
            let none = self.catalog.text("settings-search-none");
            ui.label(text::widget_text(ui.ctx(), &none, rtl, None, Look::Weak));
        }
    }

    /// Whether the search finds any of the section's settings
    fn section_finds(&self, section: Section, search: &Search) -> bool {
        let catalog = &self.catalog;
        if search.shows(catalog, section.words()) {
            return true;
        }
        if let Some(hint) = section.hint(catalog)
            && search.shows(catalog, &[hint])
        {
            return true;
        }
        let Section::Keys(group) = section else {
            return false;
        };
        Action::ALL
            .iter()
            .filter(|a| a.group() == group)
            .any(|a| self.action_found(*a, search))
    }

    /// Whether the search finds a shortcut's action, by its name or keys
    fn action_found(&self, action: Action, search: &Search) -> bool {
        let catalog = &self.catalog;
        let keys = self.settings.shortcuts.keys(action, catalog);
        search.finds(&action.text(catalog)) || search.finds(&keys)
    }

    /// A section in its frame, named on it after its tab when `tab` is
    /// given, with the settings `search` finds in it
    fn section(
        &mut self,
        ui: &mut egui::Ui,
        rtl: bool,
        section: Section,
        search: &Search,
        tab: Option<&str>,
    ) {
        let name = self.catalog.text(section.title());
        let legend = match tab {
            Some(tab) => format!("{tab} › {name}"),
            None => name,
        };
        let hint = section.hint(&self.catalog).map(|id| self.catalog.text(id));
        fieldset(ui, &legend, hint.as_deref(), rtl, |ui| {
            self.section_contents(ui, rtl, section, search);
        });
    }

    fn section_contents(
        &mut self,
        ui: &mut egui::Ui,
        rtl: bool,
        section: Section,
        search: &Search,
    ) {
        match section {
            Section::Game => self.game_section(ui, rtl, search),
            Section::Language => self.language_menu(ui, rtl),
            Section::Workers => self.workers_setting(ui, rtl),
            Section::Folders => self.folders_section(ui, rtl),
            Section::List => self.list_section(ui, rtl, search),
            Section::Events => self.spree_setting(ui, rtl),
            Section::Storage => self.storage_section(ui, rtl),
            Section::ExportFolder => self.export_folder(ui, rtl),
            Section::FileNames => self.file_names(ui, rtl, search),
            Section::Padding => self.clip_padding(ui, rtl, search),
            Section::Keys(group) => self.shortcut_group(ui, rtl, group, search),
            // The viewer's and the HUD's sections change a copy of the
            // viewer's settings
            _ => {
                let mut viewer = self.settings.viewer.clone();
                let catalog = &self.catalog;
                let draw = Viewing {
                    catalog,
                    rtl,
                    search,
                };
                match section {
                    Section::Picture => draw.picture(ui, &mut viewer),
                    Section::View => draw.view(ui, &mut viewer),
                    Section::Effects => draw.effects(ui, &mut viewer),
                    Section::Sound => draw.sound(ui, &mut viewer),
                    Section::Vitals => draw.vitals(ui, &mut viewer),
                    Section::Scores => draw.scores(ui, &mut viewer),
                    Section::Chat => draw.chat(ui, &mut viewer),
                    Section::Speeds => draw.speeds(ui, &mut viewer),
                    Section::Server => draw.server(ui, &mut viewer),
                    Section::Colours => draw.colours(ui, &mut viewer),
                    _ => {}
                }
                if viewer != self.settings.viewer {
                    self.settings.viewer = viewer;
                    self.settings_unsaved = true;
                }
            }
        }
    }

    /// The game the viewer reads, other folders searched after it, and
    /// the game's content switches
    fn game_section(&mut self, ui: &mut egui::Ui, rtl: bool, search: &Search) {
        let mut gaps = Gaps::default();
        let ids = ["settings-viewer-game", "settings-viewer-game-hint"];
        if search.shows(&self.catalog, &ids) {
            gaps.next(ui);
            self.installation_settings(ui, rtl);
        }
        let ids = ["settings-more-games", "settings-more-games-hint"];
        if search.shows(&self.catalog, &ids) {
            gaps.next(ui);
            self.more_games(ui, rtl);
        }
        if search.shows(&self.catalog, &GAME_CONTENT) {
            gaps.next(ui);
            self.game_content(ui, rtl);
        }
    }

    /// How many demos are analysed at once
    fn workers_setting(&mut self, ui: &mut egui::Ui, rtl: bool) {
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

    /// The Half-Life folder the viewer reads, checked the same way as the
    /// setup notice's choice. A choice that holds no game is refused with
    /// the reason, keeping the one before
    fn installation_settings(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        let setup_needed = self.install_root().is_none();
        let mut chosen = None;

        subheading(
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

        // Highlighted only while the viewer has no game files
        let variant = if setup_needed {
            Variant::Primary
        } else {
            Variant::Default
        };
        layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "settings-export-choose");
            let choose = widgets::Button::new(None, label, variant, rtl);
            if ui.add(choose).clicked() {
                let start = self.settings.viewer.game.clone().filter(|g| g.is_dir());
                let mut dialog = rfd::FileDialog::new();
                if let Some(start) = start {
                    dialog = dialog.set_directory(start);
                }
                chosen = dialog.pick_folder();
            }
        });

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

    /// The game's HD models, custom addon content and localized content,
    /// each shown as the game has it until changed here, which changes it
    /// for the viewer alone. Back as the game has it, a setting follows the
    /// game again
    fn game_content(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        subheading(
            ui,
            &self.catalog,
            "settings-game-content",
            Some("settings-game-content-hint"),
            rtl,
        );

        // Read from the registry and Steam each frame the window is open,
        // which is cheap, so a change in the game shows here
        let (hd, addon) = crate::content::switches();
        let language = self
            .install_root()
            .and_then(|root| crate::content::language(&root));
        let viewer = &mut self.settings.viewer;
        let rows = [
            (&mut viewer.hd_models, hd, "settings-hd-models"),
            (&mut viewer.addon_content, addon, "settings-addon-content"),
            (
                &mut viewer.localized_content,
                true,
                "settings-localized-content",
            ),
        ];
        let mut changed = false;
        for (setting, game, id) in rows {
            let mut on = setting.unwrap_or(game);
            let label = text::widget_text(&ctx, &self.catalog.text(id), rtl, None, Look::Plain);
            let hint = self.catalog.text(&format!("{id}-hint"));
            let response = layout::row(ui, rtl, |ui| {
                let response = ui.checkbox(&mut on, label);
                widgets::help(ui, &hint, rtl);
                response
            });
            if response.changed() {
                *setting = (on != game).then_some(on);
                changed = true;
            }
        }
        if changed {
            self.settings_unsaved = true;
        }

        // Which language that is, as Steam names it
        let line = match &language {
            Some(language) => {
                let language = FluentValue::from(language.clone());
                self.catalog
                    .text_with("settings-steam-language", &[("language", language)])
            }
            None => self.catalog.text("settings-steam-english"),
        };
        let width = Some(ui.available_width());
        ui.add(egui::Label::new(text::widget_text(&ctx, &line, rtl, width, Look::Weak)).wrap());
    }

    /// Other Half-Life folders searched in turn for what the first lacks,
    /// each moved up or down the order or removed
    fn more_games(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        subheading(
            ui,
            &self.catalog,
            "settings-more-games",
            Some("settings-more-games-hint"),
            rtl,
        );
        let folders = &self.settings.viewer.more_games;
        if folders.is_empty() {
            path(ui, &self.catalog.text("settings-more-games-empty"), rtl);
        }

        // One change a frame, made once the rows are drawn
        let (up, down) = (
            self.catalog.text("settings-move-up"),
            self.catalog.text("settings-move-down"),
        );
        let last = folders.len().saturating_sub(1);
        let mut swap = None;
        let mut remove = None;
        for (i, folder) in folders.iter().enumerate() {
            // The buttons come first, so a long path wraps after them
            layout::row(ui, rtl, |ui| {
                if ui
                    .add_enabled(i > 0, IconButton::new(Icon::Previous, &up))
                    .clicked()
                {
                    swap = Some((i - 1, i));
                }
                if ui
                    .add_enabled(i < last, IconButton::new(Icon::Next, &down))
                    .clicked()
                {
                    swap = Some((i, i + 1));
                }
                if ui.small_button(self.t(&ctx, "settings-remove")).clicked() {
                    remove = Some(i);
                }
                let shown = folder.display().to_string();
                ui.label(text::widget_text(&ctx, &shown, rtl, None, Look::Weak));
            });
        }

        let add = layout::row(ui, rtl, |ui| {
            let label = self.t(&ctx, "toolbar-add-folder");
            let add = widgets::Button::new(Some(Icon::AddFolder), label, Variant::Default, rtl);
            ui.add(add).clicked()
        });
        let folders = &mut self.settings.viewer.more_games;
        if let Some((a, b)) = swap {
            folders.swap(a, b);
            self.settings_unsaved = true;
        }
        if let Some(i) = remove {
            folders.remove(i);
            self.settings_unsaved = true;
        }
        if add && let Some(chosen) = rfd::FileDialog::new().pick_folders() {
            for folder in chosen {
                if !folders.contains(&folder) {
                    folders.push(folder);
                }
            }
            self.settings_unsaved = true;
        }
    }

    /// The folders scanned for demos, removed without touching their files
    fn folders_section(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        if self.settings.folders.is_empty() {
            path(ui, &self.catalog.text("settings-folders-empty"), rtl);
        }

        let mut remove = None;
        for (i, folder) in self.settings.folders.iter().enumerate() {
            // The button comes first, so a long path wraps after it
            layout::row(ui, rtl, |ui| {
                if ui
                    .small_button(self.t(ui.ctx(), "settings-remove"))
                    .clicked()
                {
                    remove = Some(i);
                }
                let shown = folder.display().to_string();
                ui.label(text::widget_text(ui.ctx(), &shown, rtl, None, Look::Weak));
            });
        }
        // Its demos leave the library with it
        if let Some(i) = remove {
            let folder = self.settings.folders.remove(i);
            self.forget_folder(folder);
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
    }

    /// How the library starts, its tab, order and filters, and whether
    /// demos of several maps leave out the maps hardly played
    fn list_section(&mut self, ui: &mut egui::Ui, rtl: bool, search: &Search) {
        let mut gaps = Gaps::default();
        if search.shows(
            &self.catalog,
            &["settings-start-tab", "settings-start-tab-hint"],
        ) {
            gaps.next(ui);
            self.starting_tab(ui, rtl);
        }
        if search.shows(&self.catalog, &["settings-sort", "settings-sort-hint"]) {
            gaps.next(ui);
            self.starting_order(ui, rtl);
        }
        let ids = [
            "settings-start-filters",
            "settings-start-filters-hint",
            "filter-min-minutes",
            "filter-min-kills",
        ];
        if search.shows(&self.catalog, &ids) {
            gaps.next(ui);
            self.starting_filters(ui, rtl);
        }
        if search.shows(&self.catalog, &["settings-hide-unplayed-maps"]) {
            gaps.next(ui);
            layout::row(ui, rtl, |ui| {
                let label = self.catalog.text("settings-hide-unplayed-maps");
                if ui
                    .checkbox(&mut self.settings.hide_unplayed_maps, label)
                    .changed()
                {
                    if let Some(l) = &mut self.library {
                        l.set_every_map(!self.settings.hide_unplayed_maps);
                    }
                    self.settings_unsaved = true;
                }
            });
        }
    }

    /// Where what is found about the demos is kept, which can be cleared
    fn storage_section(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
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

    /// The library tab the app opens on, kept for the next start, as the
    /// tab shown now is the one last clicked
    fn starting_tab(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        subheading(
            ui,
            &self.catalog,
            "settings-start-tab",
            Some("settings-start-tab-hint"),
            rtl,
        );

        let catalog = &self.catalog;
        let mut tab = library::Tab::from_key(&self.settings.start_tab);
        let before = tab;
        layout::row(ui, rtl, |ui| {
            let title = |t: library::Tab| {
                text::widget_text(&ctx, &catalog.text(t.title()), rtl, None, Look::Plain)
            };
            egui::ComboBox::from_id_salt("starting tab")
                .selected_text(title(tab))
                .show_ui(ui, |ui| {
                    for t in library::Tab::ALL {
                        ui.selectable_value(&mut tab, t, title(t));
                    }
                });
        });
        if tab != before {
            self.settings.start_tab = tab.key().to_string();
            self.settings_unsaved = true;
        }
    }

    /// The column the library is sorted by when the app starts, and which
    /// way round. A change shows at once
    fn starting_order(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
        subheading(
            ui,
            &self.catalog,
            "settings-sort",
            Some("settings-sort-hint"),
            rtl,
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
        subheading(
            ui,
            &self.catalog,
            "settings-start-filters",
            Some("settings-start-filters-hint"),
            rtl,
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

    /// From how many kills a killing spree's last kill shows a badge in the
    /// events, applied to the events shown at once
    fn spree_setting(&mut self, ui: &mut egui::Ui, rtl: bool) {
        subheading(
            ui,
            &self.catalog,
            "settings-sprees",
            Some("settings-sprees-hint"),
            rtl,
        );

        let catalog = &self.catalog;
        let ids = ["settings-spree-least"];
        let width = label_width(ui, catalog, &ids, rtl);
        let mut least = self.settings.spree_badge;
        field(ui, catalog, "settings-spree-least", width, rtl, |ui| {
            ui.add(
                egui::DragValue::new(&mut least)
                    .range(SPREE_LEAST..=SPREE_MOST)
                    .speed(SPREE_STEP)
                    .custom_formatter(|v, _| catalog.number(v, 0))
                    .custom_parser(parse_number),
            );
        });
        if least != self.settings.spree_badge {
            self.settings.spree_badge = least;
            self.settings_unsaved = true;
        }
    }

    /// The key or button pressed for the shortcut being chosen, taken before
    /// anything else sees it. Escape gives up, as does leaving the section.
    /// Returns whether a shortcut was being chosen, as the mouse's buttons
    /// can't be taken from the input
    pub(super) fn choose_shortcut(&mut self, ctx: &egui::Context) -> bool {
        let Some(choosing) = self.choosing else {
            return false;
        };
        if !self.show_settings || self.settings_tab != SettingsTab::Shortcuts {
            self.choosing = None;
            return false;
        }

        let flies = choosing.action.flies();
        let pointer = ctx.input(|i| i.pointer.hover_pos());
        let hovered = pointer.is_some_and(|p| choosing.rect.contains(p));
        match shortcuts::choose(ctx, flies, choosing.before, hovered) {
            Some(Chosen::Cancelled) => self.choosing = None,
            Some(Chosen::Binding(binding)) => {
                let shortcuts = &mut self.settings.shortcuts;
                let taken = shortcuts.set(choosing.action, choosing.slot, Some(binding));
                self.shortcut_moved = taken.map(|from| (binding, from));
                self.settings_unsaved = true;
                self.choosing = None;
            }
            None => {
                let before = ctx.input(|i| i.modifiers);
                self.choosing = Some(Choosing { before, ..choosing });
            }
        }
        true
    }

    /// What the Shortcuts tab says above its sections, how keys are
    /// changed, the defaults button, and what the last key chosen was
    /// taken from
    fn shortcuts_top(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let catalog = &self.catalog;
        let hint = catalog.text("settings-shortcuts-hint");
        let width = Some(ui.available_width());
        let hint = text::widget_text(ui.ctx(), &hint, rtl, width, Look::Weak);
        ui.add(egui::Label::new(hint).wrap());
        layout::row(ui, rtl, |ui| {
            if ui
                .button(catalog.text("settings-shortcuts-defaults"))
                .clicked()
            {
                if self.settings.shortcuts != Shortcuts::default() {
                    self.settings.shortcuts = Shortcuts::default();
                    self.settings_unsaved = true;
                }
                self.choosing = None;
                self.shortcut_moved = None;
            }
        });

        if let Some((binding, from)) = self.shortcut_moved {
            let key = FluentValue::from(binding.shown(catalog));
            let action = FluentValue::from(from.text(catalog));
            let line = catalog.text_with(
                "settings-shortcut-moved",
                &[("key", key), ("action", action)],
            );
            let line = text::widget_text(ui.ctx(), &line, rtl, width, Look::Color(theme::WARNING));
            ui.add(egui::Label::new(line).wrap());
        }
        ui.add_space(theme::GAP_S);
    }

    /// The group's actions the search finds, each with its two keys, a
    /// click on one waiting for the key to give it and a right click
    /// clearing it
    fn shortcut_group(&mut self, ui: &mut egui::Ui, rtl: bool, group: Group, search: &Search) {
        let catalog = &self.catalog;
        let mut shortcuts = self.settings.shortcuts.clone();
        let mut choosing = self.choosing;
        let mut moved = self.shortcut_moved;

        // Every group's names line up alike
        let names: Vec<String> = Action::ALL.iter().map(|a| a.text(catalog)).collect();
        let label_width = texts_width(ui, &names, rtl);
        let press = catalog.text("settings-shortcut-press");
        let change = catalog.text("settings-shortcut-change");
        for (action, name) in Action::ALL.iter().zip(&names) {
            if action.group() != group || !(search.is_empty() || self.action_found(*action, search))
            {
                continue;
            }
            // An action with no key is muted, as it simply goes unused
            let slots = shortcuts.of(*action);
            let look = if slots.iter().all(Option::is_none) {
                Look::Weak
            } else {
                Look::Plain
            };
            layout::row(ui, rtl, |ui| {
                layout::cell(ui, rtl, label_width, |ui| {
                    ui.label(text::widget_text(ui.ctx(), name, rtl, None, look));
                });
                for (slot, binding) in slots.into_iter().enumerate() {
                    let waiting = choosing.is_some_and(|c| c.action == *action && c.slot == slot);
                    let shown = if waiting {
                        press.clone()
                    } else {
                        binding.map_or_else(|| "—".to_owned(), |b| b.shown(catalog))
                    };
                    let shown = text::widget_text(ui.ctx(), &shown, rtl, None, Look::Plain);
                    let button = egui::Button::new(shown)
                        .selected(waiting)
                        .min_size(Vec2::new(KEY_WIDTH, 0.));
                    let response = ui.add(button).on_hover_text(&change);
                    if waiting && let Some(c) = &mut choosing {
                        c.rect = response.rect;
                    }
                    if response.clicked() {
                        let before = ui.input(|i| i.modifiers);
                        choosing = Some(Choosing {
                            action: *action,
                            slot,
                            before,
                            rect: response.rect,
                        });
                        moved = None;
                    }
                    if response.secondary_clicked() {
                        shortcuts.set(*action, slot, None);
                        choosing = None;
                        moved = None;
                    }
                }
            });
        }

        self.choosing = choosing;
        self.shortcut_moved = moved;
        if shortcuts != self.settings.shortcuts {
            self.settings.shortcuts = shortcuts;
            self.settings_unsaved = true;
        }
    }

    /// Where clips are exported, beside their demo or in a folder chosen
    fn export_folder(&mut self, ui: &mut egui::Ui, rtl: bool) {
        let ctx = ui.ctx().clone();
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
    }

    /// Whether clips are named after themselves, and the demo's name
    /// first, which only means something with the clip's name
    fn file_names(&mut self, ui: &mut egui::Ui, rtl: bool, search: &Search) {
        let ctx = ui.ctx().clone();
        let mut naming = (self.settings.clip_name_files, self.settings.demo_name_first);
        if search.shows(&self.catalog, &["settings-clip-name-files"]) {
            layout::row(ui, rtl, |ui| {
                let label = self.t(&ctx, "settings-clip-name-files");
                ui.checkbox(&mut naming.0, label);
            });
        }
        if search.shows(&self.catalog, &["settings-demo-name-first"]) {
            layout::row(ui, rtl, |ui| {
                let label = self.t(&ctx, "settings-demo-name-first");
                ui.add_enabled(naming.0, egui::Checkbox::new(&mut naming.1, label));
            });
        }
        if naming != (self.settings.clip_name_files, self.settings.demo_name_first) {
            (self.settings.clip_name_files, self.settings.demo_name_first) = naming;
            self.settings_unsaved = true;
        }
    }

    /// How much a clip made around an event keeps on each side of it
    fn clip_padding(&mut self, ui: &mut egui::Ui, rtl: bool, search: &Search) {
        let catalog = &self.catalog;
        let ids = ["settings-preroll", "settings-postroll"];
        let width = label_width(ui, catalog, &ids, rtl);
        let mut padding = [self.settings.preroll, self.settings.postroll];
        let mut changed = false;
        for (id, value) in ids.into_iter().zip(&mut padding) {
            if !search.shows(catalog, &[id]) {
                continue;
            }
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
            l.set_every_map(!restored.hide_unplayed_maps);
        }
        if let Some(d) = &mut self.demo {
            d.lanes_open = restored.player_lanes;
        }

        // The inspector forgets the width and the divider it was dragged to
        self.workspace.list_hidden = restored.layout.list_hidden;
        self.workspace.event_details_hidden = restored.layout.event_details_hidden;
        self.workspace.inspector_width = None;
        self.workspace.scores_share = None;
        self.workspace.restored += 1;

        let locale = restored.locale;
        self.settings = restored;
        self.set_locale(locale);
    }
}

/// Draws the sections of the viewer's and the HUD's settings, those the
/// search finds in them, changing a copy of the viewer's settings
struct Viewing<'a> {
    catalog: &'a Catalog,
    rtl: bool,
    search: &'a Search,
}

impl Viewing<'_> {
    fn shows(&self, ids: &[&str]) -> bool {
        self.search.shows(self.catalog, ids)
    }

    fn t(&self, ui: &egui::Ui, id: &str) -> egui::WidgetText {
        text::widget_text(
            ui.ctx(),
            &self.catalog.text(id),
            self.rtl,
            None,
            Look::Plain,
        )
    }

    /// A setting turned on or off, greyed while it can't be changed, with
    /// a question mark explaining `hint` when there is one
    fn check(&self, ui: &mut egui::Ui, on: &mut bool, id: &str, hint: Option<&str>, enabled: bool) {
        let mut ids = vec![id];
        ids.extend(hint);
        if !self.shows(&ids) {
            return;
        }
        layout::row(ui, self.rtl, |ui| {
            ui.add_enabled(enabled, egui::Checkbox::new(on, self.t(ui, id)));
            if let Some(hint) = hint {
                widgets::help(ui, &self.catalog.text(hint), self.rtl);
            }
        });
    }

    /// A size or loudness from its range, slid and shown in percent
    fn percent(
        &self,
        ui: &mut egui::Ui,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        id: &str,
        enabled: bool,
    ) {
        if !self.shows(&[id]) {
            return;
        }
        self.control(ui, id, enabled, |ui| {
            ui.add(egui::Slider::new(value, range).show_value(false));

            let percent = FluentValue::from(f64::from((*value * 100.).round()));
            let percent = self
                .catalog
                .text_with("library-percent", &[("percent", percent)]);
            ui.label(egui::RichText::new(percent).monospace().color(theme::MUTED));
        });
    }

    /// A setting's name and its control beside it, or under it where the
    /// two don't fit side by side, greyed while it can't be changed
    fn control(&self, ui: &mut egui::Ui, id: &str, enabled: bool, add: impl FnOnce(&mut egui::Ui)) {
        let width = label_width(ui, self.catalog, &[id], self.rtl);
        ui.add_enabled_ui(enabled, |ui| {
            field(ui, self.catalog, id, width, self.rtl, add);
        });
    }

    /// The game's gamma settings, at the game's steps, and a way back to
    /// the game's own
    fn picture(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        let (catalog, rtl) = (self.catalog, self.rtl);
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
            if !self.shows(&[id]) {
                continue;
            }
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
        self.check(
            ui,
            &mut viewer.overbright,
            "settings-overbright",
            None,
            true,
        );

        // Only the picture goes back to the game's own
        if self.shows(&["settings-gamma-defaults"]) {
            layout::row(ui, rtl, |ui| {
                if ui.button(self.t(ui, "settings-gamma-defaults")).clicked() {
                    let d = ViewerSettings::default();
                    (viewer.gamma, viewer.brightness) = (d.gamma, d.brightness);
                    (viewer.texgamma, viewer.lightgamma) = (d.texgamma, d.lightgamma);
                    viewer.overbright = d.overbright;
                }
            });
        }
    }

    /// The fields of view, the weapon in view and its bob, the crosshair,
    /// seeing through walls, names, the position, and the free camera
    fn view(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        let (catalog, rtl) = (self.catalog, self.rtl);
        if self.shows(&["settings-fov-recorded", "settings-fov"]) {
            layout::row(ui, rtl, |ui| {
                let mut recorded = viewer.fov.is_none();
                let label = self.t(ui, "settings-fov-recorded");
                if ui.checkbox(&mut recorded, label).changed() {
                    viewer.fov = (!recorded).then_some(FIRST_FOV);
                }
            });
        }
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
        if let Some(fov) = &mut viewer.fov
            && self.shows(&["settings-fov-recorded", "settings-fov"])
        {
            fov_field(ui, "settings-fov", fov);
        }
        if self.shows(&["settings-zoom-fov"]) {
            fov_field(ui, "settings-zoom-fov", &mut viewer.zoom_fov);
        }

        // The bob is the weapon's, so it goes with it
        self.check(ui, &mut viewer.viewmodel, "settings-viewmodel", None, true);
        if self.shows(&["settings-weapon-bob-recorded", "settings-weapon-bob"]) {
            ui.add_enabled_ui(viewer.viewmodel, |ui| {
                layout::row(ui, rtl, |ui| {
                    let mut recorded = viewer.weapon_bob.is_none();
                    let label = self.t(ui, "settings-weapon-bob-recorded");
                    if ui.checkbox(&mut recorded, label).changed() {
                        viewer.weapon_bob = (!recorded).then_some(GAME_BOB);
                    }
                });
                if let Some(bob) = &mut viewer.weapon_bob {
                    let width = label_width(ui, catalog, &["settings-weapon-bob"], rtl);
                    field(ui, catalog, "settings-weapon-bob", width, rtl, |ui| {
                        ui.add(
                            egui::DragValue::new(bob)
                                .range(WEAPON_BOB.0..=WEAPON_BOB.1)
                                .speed(BOB_STEP)
                                .custom_formatter(|v, _| catalog.number(v, BOB_DECIMALS))
                                .custom_parser(parse_number),
                        );
                    });
                }
            });
        }
        self.check(ui, &mut viewer.crosshair, "settings-crosshair", None, true);

        // Projectiles and items show through walls only with the players,
        // and keep their choice while the players do not
        let ids = [
            "viewer-see-through",
            "settings-see-through-hint",
            "settings-see-through",
            "settings-see-through-items",
        ];
        if self.shows(&ids) {
            ui.add_space(SETTING_GAP);
            subheading(
                ui,
                catalog,
                "viewer-see-through",
                Some("settings-see-through-hint"),
                rtl,
            );
            let everything = Viewing {
                search: &Search::default(),
                ..*self
            };
            everything.check(
                ui,
                &mut viewer.see_through,
                "settings-see-through",
                None,
                true,
            );
            let see_through = viewer.see_through;
            let items = &mut viewer.see_through_items;
            everything.check(ui, items, "settings-see-through-items", None, see_through);
            ui.add_space(SETTING_GAP);
        }
        let names = Some("settings-player-names-hint");
        self.check(
            ui,
            &mut viewer.player_names,
            "settings-player-names",
            names,
            true,
        );
        let position = Some("settings-show-position-hint");
        self.check(
            ui,
            &mut viewer.show_position,
            "settings-show-position",
            position,
            true,
        );
        let leave = Some("settings-events-leave-free-hint");
        let free = &mut viewer.events_leave_free;
        self.check(ui, free, "settings-events-leave-free", leave, true);
    }

    /// Particles, and how many bullet holes and other marks are kept, the
    /// newest
    fn effects(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        let (catalog, rtl) = (self.catalog, self.rtl);
        let hint = Some("settings-particles-hint");
        self.check(ui, &mut viewer.particles, "settings-particles", hint, true);
        let ids = [
            "settings-section-decals",
            "settings-decals-hint",
            "settings-decals",
        ];
        if !self.shows(&ids) {
            return;
        }
        ui.add_space(SETTING_GAP);
        subheading(
            ui,
            catalog,
            "settings-section-decals",
            Some("settings-decals-hint"),
            rtl,
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
    }

    fn sound(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        self.percent(ui, &mut viewer.volume, 0. ..=1., "viewer-volume", true);
    }

    /// The recorder's health, armour, ammo and pickups, their size, and
    /// whether they show with the free camera too
    fn vitals(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        self.check(ui, &mut viewer.vitals, "settings-vitals", None, true);
        let range = VITALS_SIZE.0..=VITALS_SIZE.1;
        let size = &mut viewer.vitals_size;
        self.percent(ui, size, range, "settings-vitals-size", viewer.vitals);
        let vitals = viewer.vitals;
        let free = &mut viewer.free_vitals;
        self.check(ui, free, "settings-free-vitals", None, vitals);
    }

    /// The small scoreboard and its rows, and when the full one shows
    fn scores(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        let (catalog, rtl) = (self.catalog, self.rtl);
        self.check(ui, &mut viewer.scores, "settings-scores", None, true);
        if self.shows(&["settings-score-rows"]) {
            let width = label_width(ui, catalog, &["settings-score-rows"], rtl);
            field(ui, catalog, "settings-score-rows", width, rtl, |ui| {
                let rows = egui::DragValue::new(&mut viewer.score_rows)
                    .range(1..=MAX_SCORE_ROWS)
                    .custom_formatter(|v, _| catalog.number(v, 0))
                    .custom_parser(parse_number);
                ui.add_enabled(viewer.scores, rows);
            });
        }
        if self.shows(&["settings-scoreboard"]) {
            self.control(ui, "settings-scoreboard", true, |ui| {
                egui::ComboBox::from_id_salt("scoreboard")
                    .selected_text(self.t(ui, viewer.scoreboard.text_id()))
                    .show_ui(ui, |ui| {
                        for shown in ScoreboardShown::ALL {
                            let label = self.t(ui, shown.text_id());
                            ui.selectable_value(&mut viewer.scoreboard, shown, label);
                        }
                    });
            });
        }
        let on = viewer.scoreboard != ScoreboardShown::Off;
        let playing = &mut viewer.scoreboard_playing;
        self.check(ui, playing, "settings-scoreboard-playing", None, on);
    }

    /// The chat, and whether it shows the lines players' binds send
    fn chat(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        self.check(ui, &mut viewer.chat, "settings-chat", None, true);
        let chat = viewer.chat;
        let places = &mut viewer.chat_places;
        let hint = Some("settings-chat-places-hint");
        self.check(ui, places, "settings-chat-places", hint, chat);
        let long_jump = &mut viewer.chat_long_jump;
        let hint = Some("settings-chat-long-jump-hint");
        self.check(ui, long_jump, "settings-chat-long-jump", hint, chat);
    }

    /// The speeds, each on its own and counted the same way, and the
    /// recorder's frames a second
    fn speeds(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        self.check(
            ui,
            &mut viewer.speedometer,
            "settings-speedometer",
            None,
            true,
        );
        let hint = Some("settings-jump-speed-hint");
        self.check(
            ui,
            &mut viewer.jump_speed,
            "settings-jump-speed",
            hint,
            true,
        );
        if self.shows(&["settings-speed-axes"]) {
            let counted = viewer.speedometer || viewer.jump_speed;
            self.control(ui, "settings-speed-axes", counted, |ui| {
                let current = self.t(ui, viewer.speed_axes.text_id());
                egui::ComboBox::from_id_salt("speed axes")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for axes in SpeedAxes::ALL {
                            let label = self.t(ui, axes.text_id());
                            ui.selectable_value(&mut viewer.speed_axes, axes, label);
                        }
                    });
            });
        }
        let hint = Some("settings-recorder-fps-hint");
        self.check(
            ui,
            &mut viewer.recorder_fps,
            "settings-recorder-fps",
            hint,
            true,
        );
    }

    /// What the server sends to show, its settings and its own HUD text
    fn server(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        let settings = &mut viewer.server_settings;
        self.check(ui, settings, "settings-server-settings", None, true);
        self.check(
            ui,
            &mut viewer.hud_messages,
            "settings-hud-messages",
            None,
            true,
        );
    }

    /// The HUD's and the text's colours, each with a way back to the
    /// game's, and the text's size
    fn colours(&self, ui: &mut egui::Ui, viewer: &mut ViewerSettings) {
        let rtl = self.rtl;
        let defaults = ViewerSettings::default();
        let colours = [
            (
                &mut viewer.hud_color,
                defaults.hud_color,
                "settings-hud-color",
                "settings-hud-color-hint",
            ),
            (
                &mut viewer.text_color,
                defaults.text_color,
                "settings-text-color",
                "settings-text-color-hint",
            ),
        ];
        for (colour, game, id, hint) in colours {
            if !self.shows(&[id, hint]) {
                continue;
            }
            layout::row(ui, rtl, |ui| {
                ui.label(self.t(ui, id));
                ui.color_edit_button_srgb(colour);
                if ui
                    .button(self.t(ui, "settings-text-color-default"))
                    .clicked()
                {
                    *colour = game;
                }
                widgets::help(ui, &self.catalog.text(hint), rtl);
            });
        }
        let range = TEXT_SIZE.0..=TEXT_SIZE.1;
        self.percent(ui, &mut viewer.text_size, range, "settings-text-size", true);
    }
}

/// The Half-Life content switches' names and explanations, which the
/// search looks through
const GAME_CONTENT: [&str; 8] = [
    "settings-game-content",
    "settings-game-content-hint",
    "settings-hd-models",
    "settings-hd-models-hint",
    "settings-addon-content",
    "settings-addon-content-hint",
    "settings-localized-content",
    "settings-localized-content-hint",
];

/// Room between the parts of a section, above all but the first
#[derive(Default)]
struct Gaps {
    started: bool,
}

impl Gaps {
    fn next(&mut self, ui: &mut egui::Ui) {
        if self.started {
            ui.add_space(SETTING_GAP);
        }
        self.started = true;
    }
}

/// A group of settings in a frame with `title` on its top edge, as a
/// fieldset has its legend, and a question mark explaining `hint` beside
/// it when there is one
fn fieldset(
    ui: &mut egui::Ui,
    title: &str,
    hint: Option<&str>,
    rtl: bool,
    add: impl FnOnce(&mut egui::Ui),
) {
    let legend = text::widget_text(ui.ctx(), title, rtl, None, Look::Strong);
    let galley = legend.into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        ui.available_width() - 2. * LEGEND_INSET - theme::CONTROL_HEIGHT,
        egui::TextStyle::Body,
    );
    let help = if hint.is_some() {
        theme::CONTROL_HEIGHT
    } else {
        0.
    };
    let half = galley.size().y.max(help) / 2.;

    // The frame starts halfway down the legend
    ui.add_space(half);
    let margin = FRAME_MARGIN as i8;
    let frame = egui::Frame::new()
        .stroke(egui::Stroke::new(1., theme::HOVER))
        .corner_radius(theme::RADIUS_CARD)
        .inner_margin(egui::Margin {
            left: margin,
            right: margin,
            top: (half + theme::GAP_XS) as i8,
            bottom: margin,
        });
    // Long names and paths wrap within the frame rather than widen it
    let shown = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        layout::column(ui, rtl, add);
    });

    // The legend cuts the frame's top edge, its question mark after it
    let rect = shown.response.rect;
    let width = galley.size().x + help;
    let start = if rtl {
        rect.right() - LEGEND_INSET - width
    } else {
        rect.left() + LEGEND_INSET
    };
    let legend_rect = egui::Rect::from_min_size(
        Pos2::new(start, rect.top() - half),
        Vec2::new(width, 2. * half),
    );
    let behind = legend_rect.expand2(Vec2::new(theme::GAP_XS, 0.));
    ui.painter()
        .rect_filled(behind, 0., ui.visuals().window_fill);
    let text_x = if rtl {
        legend_rect.right() - galley.size().x
    } else {
        legend_rect.left()
    };
    let text_at = Pos2::new(text_x, legend_rect.center().y - galley.size().y / 2.);
    ui.painter().galley(text_at, galley, theme::FOREGROUND);
    if let Some(hint) = hint {
        let x = if rtl {
            legend_rect.left()
        } else {
            legend_rect.right() - help
        };
        let at = egui::Rect::from_min_size(Pos2::new(x, legend_rect.top()), Vec2::splat(help));

        // Placed apart, so what follows the frame stays under it
        let mut place = ui.new_child(egui::UiBuilder::new().max_rect(at));
        widgets::help(&mut place, hint, rtl);
    }
    ui.add_space(theme::GAP_S);
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
        (now.spree_badge != then.spree_badge, "settings-sprees"),
        (view.hd_models != first.hd_models, "settings-hd-models"),
        (
            view.addon_content != first.addon_content,
            "settings-addon-content",
        ),
        (
            view.localized_content != first.localized_content,
            "settings-localized-content",
        ),
        (now.locale != then.locale, "settings-language"),
        (now.workers != then.workers, "settings-workers"),
        (now.start_tab != then.start_tab, "settings-start-tab"),
        (order(now) != order(then), "settings-sort"),
        (starting(now) != starting(then), "settings-start-filters"),
        (
            now.hide_unplayed_maps != then.hide_unplayed_maps,
            "settings-hide-unplayed-maps",
        ),
        (view.gamma != first.gamma, "settings-gamma"),
        (view.brightness != first.brightness, "settings-brightness"),
        (view.texgamma != first.texgamma, "settings-texgamma"),
        (view.lightgamma != first.lightgamma, "settings-lightgamma"),
        (view.overbright != first.overbright, "settings-overbright"),
        (view.fov != first.fov, "settings-fov"),
        (view.zoom_fov != first.zoom_fov, "settings-zoom-fov"),
        (view.viewmodel != first.viewmodel, "settings-viewmodel"),
        (view.weapon_bob != first.weapon_bob, "settings-weapon-bob"),
        (view.crosshair != first.crosshair, "settings-crosshair"),
        (
            view.see_through != first.see_through,
            "settings-see-through",
        ),
        (
            view.see_through_items != first.see_through_items,
            "settings-see-through-items",
        ),
        (
            view.player_names != first.player_names,
            "settings-player-names",
        ),
        (
            view.show_position != first.show_position,
            "settings-show-position",
        ),
        (
            view.events_leave_free != first.events_leave_free,
            "settings-events-leave-free",
        ),
        (view.particles != first.particles, "settings-particles"),
        (view.decals != first.decals, "settings-section-decals"),
        (view.volume != first.volume, "viewer-volume"),
        (view.vitals != first.vitals, "settings-vitals"),
        (
            view.vitals_size != first.vitals_size,
            "settings-vitals-size",
        ),
        (
            view.free_vitals != first.free_vitals,
            "settings-free-vitals",
        ),
        (view.scores != first.scores, "settings-scores"),
        (view.score_rows != first.score_rows, "settings-score-rows"),
        (view.scoreboard != first.scoreboard, "settings-scoreboard"),
        (
            view.scoreboard_playing != first.scoreboard_playing,
            "settings-scoreboard-playing",
        ),
        (
            view.speedometer != first.speedometer,
            "settings-speedometer",
        ),
        (view.jump_speed != first.jump_speed, "settings-jump-speed"),
        (view.speed_axes != first.speed_axes, "settings-speed-axes"),
        (
            view.recorder_fps != first.recorder_fps,
            "settings-recorder-fps",
        ),
        (view.chat != first.chat, "settings-chat"),
        (
            view.chat_places != first.chat_places,
            "settings-chat-places",
        ),
        (
            view.chat_long_jump != first.chat_long_jump,
            "settings-chat-long-jump",
        ),
        (view.hud_color != first.hud_color, "settings-hud-color"),
        (view.text_color != first.text_color, "settings-text-color"),
        (view.text_size != first.text_size, "settings-text-size"),
        (
            view.server_settings != first.server_settings,
            "settings-server-settings",
        ),
        (
            view.hud_messages != first.hud_messages,
            "settings-hud-messages",
        ),
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
        (now.shortcuts != then.shortcuts, "settings-tab-shortcuts"),
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

/// The name of a few settings within a section, with a line after it to
/// the section's edge, and a question mark explaining `hint` when there is
/// one
fn subheading(ui: &mut egui::Ui, catalog: &Catalog, id: &str, hint: Option<&str>, rtl: bool) {
    let hint = hint.map(|h| catalog.text(h));
    widgets::group_title(ui, &catalog.text(id), hint.as_deref(), rtl);
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
    let texts: Vec<String> = ids.iter().map(|id| catalog.text(id)).collect();
    texts_width(ui, &texts, rtl)
}

/// The width of the widest of `texts` as labels
fn texts_width(ui: &egui::Ui, texts: &[String], rtl: bool) -> f32 {
    let mut widest: f32 = 0.;
    for text in texts {
        let label = text::widget_text(ui.ctx(), text, rtl, None, Look::Plain);
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
