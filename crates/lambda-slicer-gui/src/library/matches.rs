//! The library's Matches tab, each match of the demos the filters find on
//! a row of its own
//!
//! A demo of two matches has two rows, and a stretch of a demo with no
//! match on that lasts five minutes or more is free play. The map, length,
//! kills, players and teams filters test each match, and the others its
//! demo. Opening a row opens its demo at the match's countdown, or where
//! the recording began for a match already on. The narrow list beside the
//! open demo then lists the matches too, two lines a match

use std::collections::{HashMap, HashSet};

use eframe::egui::{
    self, Pos2, Rect, Response, RichText, Role, Sense, TextStyle, TextWrapMode, Ui, Vec2,
    WidgetInfo, WidgetText,
};
use egui_extras::{Column, TableBuilder};
use lambda_slicer_core::analysis::parts::PartStatus;
use lambda_slicer_core::library::cache::{FileQuery, FileRow, MatchQuery, MatchRow, MatchSort};

use super::worker::PageOf;
use super::{
    LIST_LINES, LIST_ROW, LibraryView, PAGE, SORT_MENU_WIDTH, TABLE_ROW, clock, day_cell,
    direction_button, display_name, named_cell, number_cell, score_cell, score_of, sort_heading,
    text_cell,
};
use crate::i18n::{Catalog, Ymd};
use crate::text::{self, Look};
use crate::theme;

/// Whether the full table lists demos, their matches or their chat
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Demos,
    Matches,
    Chat,
}

impl Tab {
    /// Every tab, in the order they are shown
    pub const ALL: [Tab; 3] = [Tab::Demos, Tab::Matches, Tab::Chat];

    /// The name kept in settings
    pub fn key(self) -> &'static str {
        match self {
            Tab::Demos => "demos",
            Tab::Matches => "matches",
            Tab::Chat => "chat",
        }
    }

    /// The tab kept in settings as `key`, the demos for a key this
    /// version does not know
    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|t| t.key() == key)
            .unwrap_or_default()
    }

    /// Its title's message
    pub fn title(self) -> &'static str {
        match self {
            Tab::Demos => "tab-demos",
            Tab::Matches => "tab-matches",
            Tab::Chat => "tab-chat",
        }
    }
}

/// The matches' order, their own filter and their loaded pages
#[derive(Debug)]
pub struct MatchList {
    pub sort: MatchSort,
    pub descending: bool,
    /// The statuses left out, none for every match
    pub hidden: Vec<PartStatus>,
    /// Matches the filters find
    pub total: u64,
    pub pages: HashMap<usize, Vec<MatchRow>>,
    /// The pages asked of the library's thread
    pub asked: HashSet<usize>,
    /// The match last opened, by its demo and its place in it
    pub selected: Option<(i64, i64)>,
    /// The match whose menu was open as the rows were last drawn, kept lit
    /// as if hovered, and the one found open this frame, by its demo and
    /// its place in it
    menu_row: Option<(i64, i64)>,
    menu_open: Option<(i64, i64)>,
}

impl Default for MatchList {
    /// The newest demos' matches first
    fn default() -> Self {
        Self {
            sort: MatchSort::Date,
            descending: true,
            hidden: Vec::new(),
            total: 0,
            pages: HashMap::new(),
            asked: HashSet::new(),
            selected: None,
            menu_row: None,
            menu_open: None,
        }
    }
}

impl MatchList {
    /// What the count depends on besides the demos' filters
    pub fn order(&self) -> (MatchSort, bool, Vec<PartStatus>) {
        (self.sort, self.descending, self.hidden.clone())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchCol {
    Demo,
    Date,
    Map,
    Status,
    Length,
    Score,
    Players,
    Teams,
    Kills,
    Multikill,
    Spree,
}

impl MatchCol {
    const ALL: [MatchCol; 11] = [
        MatchCol::Demo,
        MatchCol::Date,
        MatchCol::Map,
        MatchCol::Status,
        MatchCol::Length,
        MatchCol::Score,
        MatchCol::Players,
        MatchCol::Teams,
        MatchCol::Kills,
        MatchCol::Multikill,
        MatchCol::Spree,
    ];

    fn title(self) -> &'static str {
        match self {
            MatchCol::Demo => "column-name",
            MatchCol::Date => "column-date",
            MatchCol::Map => "column-map",
            MatchCol::Status => "column-status",
            MatchCol::Length => "column-duration",
            MatchCol::Score => "column-score",
            MatchCol::Players => "column-players",
            MatchCol::Teams => "column-teams",
            MatchCol::Kills => "column-kills",
            MatchCol::Multikill => "column-multikill",
            MatchCol::Spree => "column-spree",
        }
    }

    /// What the column's figure means, for its heading's tooltip
    fn help(self) -> Option<&'static str> {
        match self {
            MatchCol::Date => Some("help-date-column"),
            MatchCol::Status => Some("help-status-column"),
            MatchCol::Players => Some("help-match-players"),
            MatchCol::Teams => Some("help-match-teams"),
            MatchCol::Multikill => Some("help-match-multikill"),
            MatchCol::Spree => Some("help-match-spree"),
            _ => None,
        }
    }

    fn sort(self) -> MatchSort {
        match self {
            MatchCol::Demo => MatchSort::Path,
            MatchCol::Date => MatchSort::Date,
            MatchCol::Map => MatchSort::Map,
            MatchCol::Status => MatchSort::Status,
            MatchCol::Length => MatchSort::Length,
            MatchCol::Score => MatchSort::Score,
            MatchCol::Players => MatchSort::Players,
            MatchCol::Teams => MatchSort::Teams,
            MatchCol::Kills => MatchSort::Kills,
            MatchCol::Multikill => MatchSort::Multikill,
            MatchCol::Spree => MatchSort::Spree,
        }
    }

    /// Every column is a size of its own the user can change, as the
    /// demos' are
    fn column(self) -> Column {
        let (width, least) = match self {
            MatchCol::Demo => (280., 180.),
            MatchCol::Date => (112., 80.),
            MatchCol::Map => (128., 60.),
            MatchCol::Status => (120., 80.),
            MatchCol::Length => (80., 64.),
            MatchCol::Score => (128., 64.),
            MatchCol::Players | MatchCol::Teams | MatchCol::Kills => (64., 48.),
            MatchCol::Multikill | MatchCol::Spree => (72., 48.),
        };
        Column::initial(width).at_least(least).clip(true)
    }
}

/// The statuses shown once `status`'s box is ticked or cleared, as
/// `hidden` leaves them out. The last one shown stays, as no match would
/// be left
fn status_toggled(hidden: &[PartStatus], status: PartStatus) -> Vec<PartStatus> {
    let mut out = hidden.to_vec();
    if let Some(at) = out.iter().position(|s| *s == status) {
        out.remove(at);
    } else if out.len() + 1 < PartStatus::ALL.len() {
        out.push(status);
    }
    out
}

/// The messages naming `status` and saying what it means
pub(super) fn status_text(status: PartStatus) -> (&'static str, &'static str) {
    match status {
        PartStatus::Complete => ("match-complete", "help-match-complete"),
        PartStatus::JoinedLate => ("match-joined-late", "help-match-joined-late"),
        PartStatus::Interrupted => ("match-interrupted", "help-match-interrupted"),
        PartStatus::CutOff => ("match-cut-off", "help-match-cut-off"),
        PartStatus::FreePlay => ("match-free-play", "help-match-free-play"),
    }
}

/// Names sort from the start of the alphabet, figures and dates from the
/// highest
fn descending_first(sort: MatchSort) -> bool {
    !matches!(sort, MatchSort::Path | MatchSort::Map | MatchSort::Status)
}

impl LibraryView {
    /// The statuses to show, as a list that opens with a box to tick for
    /// each and stays open while they are ticked
    pub(super) fn status_filter(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
        let hidden = self.matches.hidden.clone();

        // Every status, or those shown by name
        let shown = if hidden.is_empty() {
            catalog.text("filter-status-all")
        } else {
            let mut names = Vec::new();
            for status in PartStatus::ALL {
                if !hidden.contains(&status) {
                    names.push(catalog.text(status_text(status).0));
                }
            }
            names.join(", ")
        };

        let mut toggled = None;
        egui::ComboBox::from_id_salt("match statuses")
            .selected_text(text::widget_text(&ctx, &shown, rtl, None, Look::Plain))
            .width(ui.available_width())
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show_ui(ui, |ui| {
                for status in PartStatus::ALL {
                    let mut on = !hidden.contains(&status);
                    if ui.checkbox(&mut on, t(status_text(status).0)).changed() {
                        toggled = Some(status);
                    }
                }
            });
        if let Some(status) = toggled {
            self.matches.hidden = status_toggled(&hidden, status);
        }
    }

    /// Show `tab` in the full table
    pub fn show_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    /// The cache query for the filters and the matches' order, a page long
    pub(super) fn match_query(&self) -> MatchQuery {
        MatchQuery {
            files: FileQuery {
                descending: self.matches.descending,
                ..self.query()
            },
            hidden: self.matches.hidden.clone(),
            sort: self.matches.sort,
        }
    }

    fn match_row(&mut self, index: usize) -> Option<MatchRow> {
        let page = index / PAGE;
        if self.matches.asked.insert(page) {
            let mut query = self.match_query();
            query.files.offset = page * PAGE;
            self.ask_page(page, PageOf::Matches(query));
        }
        self.matches.pages.get(&page)?.get(index % PAGE).cloned()
    }

    /// Sort the matches by `sort`, turning the order round when it is
    /// already the order
    fn sort_matches_by(&mut self, sort: MatchSort) {
        if sort == self.matches.sort {
            self.matches.descending = !self.matches.descending;
        } else {
            self.matches.sort = sort;
            self.matches.descending = descending_first(sort);
        }
    }

    /// Open the demo `played` was recorded in, where watching it begins
    fn open_match(&mut self, played: &MatchRow) {
        self.select_path(&played.source());
        if self.selected.is_some() {
            self.open_at = Some(played.opens_at);
            self.matches.selected = Some((played.file_id, played.ordinal));
        }
    }

    /// The match last opened, by its demo and its place in it
    pub fn opened_match(&self) -> Option<(i64, i64)> {
        self.matches.selected
    }

    /// Show `row` as the open demo again, with `opened` as the match last
    /// opened in it
    pub fn reselect(&mut self, row: FileRow, opened: Option<(i64, i64)>) {
        self.selected = Some(row);
        self.matches.selected = opened;
    }

    /// The demo's menu on the row of `played`, which shows its demo in its
    /// folder, makes it a favourite or no longer one, or hides it or shows
    /// it again, the row lit while it is open
    fn match_menu(&mut self, response: &Response, played: &MatchRow, catalog: &Catalog, rtl: bool) {
        let marks = self.marks(&played.path);
        let choice = super::row_menu(response, &played.path, marks, catalog, rtl);
        if choice.open {
            self.matches.menu_open = Some((played.file_id, played.ordinal));
        }
        self.take_choice(choice, &played.path);
    }

    /// Whether `played` is the match last opened, in the demo still open
    fn is_open(&self, played: &MatchRow) -> bool {
        let demo_open = self
            .selected
            .as_ref()
            .is_some_and(|s| s.id == played.file_id);
        demo_open && self.matches.selected == Some((played.file_id, played.ordinal))
    }

    /// The narrow list's order of the matches, by a column's title, and
    /// the button turning it round
    pub(super) fn match_sort_menu(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);

        let mut sort = self.matches.sort;
        let current = MatchCol::ALL
            .into_iter()
            .find(|c| c.sort() == sort)
            .map_or_else(String::new, |c| catalog.text(c.title()));
        egui::ComboBox::from_id_salt("match list sort")
            .width(SORT_MENU_WIDTH)
            .selected_text(text::widget_text(&ctx, &current, rtl, None, Look::Plain))
            .show_ui(ui, |ui| {
                for col in MatchCol::ALL {
                    ui.selectable_value(&mut sort, col.sort(), t(col.title()));
                }
            })
            .response
            .on_hover_text(catalog.text("sort-by"));
        if sort != self.matches.sort {
            self.matches.sort = sort;
            self.matches.descending = descending_first(sort);
        }

        direction_button(ui, &mut self.matches.descending, catalog);
    }

    /// The matches in the narrow list, two lines a match
    pub(super) fn match_list(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let mut clicked = None;

        // Rows touch, so the rows the scroll area works out are the rows it
        // draws, at the same places
        ui.spacing_mut().item_spacing.y = 0.;
        egui::ScrollArea::vertical()
            .id_salt("match list")
            .auto_shrink(false)
            .show_rows(ui, LIST_ROW, self.matches.total as usize, |ui, rows| {
                for index in rows {
                    let Some(played) = self.match_row(index) else {
                        continue;
                    };
                    let response = self.match_list_row(ui, &played, catalog, rtl);
                    self.match_menu(&response, &played, catalog, rtl);
                    if response.clicked() {
                        clicked = Some(played);
                    }
                }
            });
        self.matches.menu_row = self.matches.menu_open.take();

        if let Some(played) = clicked {
            self.open_match(&played);
        }
    }

    /// One match in two lines, its map and how much was recorded, then its
    /// demo and length with the score. The path, date, players, teams and
    /// kills show on hover
    fn match_list_row(
        &self,
        ui: &mut Ui,
        played: &MatchRow,
        catalog: &Catalog,
        rtl: bool,
    ) -> Response {
        let size = Vec2::new(ui.available_width(), LIST_ROW);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let selected = self.is_open(played);
        let key = (played.file_id, played.ordinal);
        let map = played.map.clone();
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, &map));

        if !ui.is_rect_visible(rect) {
            return response;
        }

        // A chosen row in the soft accent with an accent edge on its leading
        // side, a hovered one raised, as a demo's
        let painter = ui.painter();
        if selected {
            painter.rect_filled(rect, 0., theme::ACCENT_SOFT);
            let edge_x = if rtl {
                rect.max.x - theme::SELECTED_EDGE
            } else {
                rect.min.x
            };
            let size = Vec2::new(theme::SELECTED_EDGE, LIST_ROW);
            let edge = Rect::from_min_size(Pos2::new(edge_x, rect.min.y), size);
            painter.rect_filled(edge, 0., theme::ACCENT);
        } else if response.hovered() || self.matches.menu_row == Some(key) {
            painter.rect_filled(rect, 0., theme::RAISED);
        }

        // The first line is the map with how much was recorded, the second
        // the demo and the match's length with the score
        let (status, _) = status_text(played.status);
        let status = text::styled_text(
            ui.ctx(),
            &catalog.text(status),
            rtl,
            Look::Color(theme::MUTED),
            &theme::secondary(),
        );
        let score = score_of(played.scoreboard.as_ref(), played.score.as_deref())
            .map(|(shown, _)| WidgetText::from(RichText::new(shown).monospace()));

        let look = if selected {
            Look::Color(theme::ACCENT)
        } else {
            Look::Plain
        };
        let map_rtl = text::starts_rtl(&played.map);
        let map_text = text::widget_text(ui.ctx(), &played.map, map_rtl, None, look);
        let demo = display_name(&played.path);
        let facts = match clock(Some(played.length)) {
            Some(length) => format!("{demo} · {length}"),
            None => demo,
        };
        let facts_text = text::styled_text(
            ui.ctx(),
            &facts,
            rtl,
            Look::Color(theme::MUTED),
            &theme::secondary(),
        );

        let lines = [
            (
                map_text,
                TextStyle::Body,
                Some((status, theme::secondary())),
            ),
            (
                facts_text,
                theme::secondary(),
                score.map(|s| (s, TextStyle::Monospace)),
            ),
        ];
        let inner = rect.shrink2(Vec2::new(theme::GAP_M, 0.));
        for (k, (leading, style, trailing)) in lines.into_iter().enumerate() {
            let line = rect.min.y + LIST_LINES[k];
            let mut room = inner.width();

            // The trailing text first, the leading text cut short to the
            // room it leaves
            if let Some((trailing, style)) = trailing {
                let galley =
                    trailing.into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, style);
                let x = if rtl {
                    inner.min.x
                } else {
                    inner.max.x - galley.size().x
                };
                room -= galley.size().x + theme::GAP_S;
                let at = Pos2::new(x, line - galley.size().y / 2.);
                ui.painter().galley(at, galley, theme::MUTED);
            }

            let galley = leading.into_galley(ui, Some(TextWrapMode::Truncate), room.max(0.), style);
            let x = if rtl {
                inner.max.x - galley.size().x
            } else {
                inner.min.x
            };
            let at = Pos2::new(x, line - galley.size().y / 2.);
            ui.painter().galley(at, galley, theme::FOREGROUND);
        }

        response.on_hover_ui(|ui| match_details(ui, played, catalog, rtl))
    }

    /// The matches' table, which scrolls sideways as the demos' does
    pub(super) fn match_table(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let open = self.keyboard(&ctx, self.matches.total);
        let mut cols = MatchCol::ALL.to_vec();
        if rtl {
            cols.reverse();
        }
        let mut clicked = None;
        let mut sort_by = None;

        let cell_layout = if rtl {
            egui::Layout::right_to_left(egui::Align::Center)
        } else {
            egui::Layout::left_to_right(egui::Align::Center)
        };

        // A right-to-left table starts at its leading columns, on the right
        egui::ScrollArea::horizontal()
            .id_salt("match columns")
            .stick_to_right(rtl)
            .show(ui, |ui| {
                let mut table = TableBuilder::new(ui)
                    .id_salt("matches")
                    .cell_layout(cell_layout)
                    .resizable(true)
                    .sense(Sense::click())
                    .max_scroll_height(f32::INFINITY)
                    .min_scrolled_height(0.)
                    .auto_shrink(false);
                for c in &cols {
                    table = table.column(c.column());
                }
                if std::mem::take(&mut self.scroll_to_cursor)
                    && let Some(cursor) = self.cursor
                {
                    table = table.scroll_to_row(cursor, None);
                }

                table
                    .header(TABLE_ROW, |mut header| {
                        for c in &cols {
                            header.col(|ui| {
                                // Clear of the filter pane's edge, less than
                                // the names below it
                                if *c == MatchCol::Demo {
                                    ui.add_space(theme::GAP_S);
                                }
                                let sorted = (c.sort() == self.matches.sort)
                                    .then_some(self.matches.descending);
                                let help = c.help().map(|key| catalog.text(key));
                                let title = catalog.text(c.title());
                                if sort_heading(ui, &title, help.as_deref(), sorted, rtl).clicked()
                                {
                                    sort_by = Some(c.sort());
                                }
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(TABLE_ROW, self.matches.total as usize, |mut row| {
                            let index = row.index();
                            let Some(played) = self.match_row(index) else {
                                return;
                            };
                            let key = (played.file_id, played.ordinal);
                            let menu = self.matches.menu_row == Some(key);
                            row.set_hovered(self.cursor == Some(index) || menu);
                            let favourite = self.favourite(&played.path);
                            for c in &cols {
                                row.col(|ui| match_cell(ui, *c, &played, favourite, catalog, rtl));
                            }
                            let response = row.response();
                            if response.clicked() {
                                clicked = Some(index);
                            }
                            self.match_menu(&response, &played, catalog, rtl);
                        });
                    });
            });
        self.matches.menu_row = self.matches.menu_open.take();

        if let Some(sort) = sort_by {
            self.sort_matches_by(sort);
        }

        if let Some(index) = clicked.or(open) {
            self.cursor = Some(index);
            if let Some(played) = self.match_row(index) {
                self.open_match(&played);
            }
        }
    }
}

/// One cell of a match's row, its demo's name starred when the demo is a
/// `favourite`
fn match_cell(
    ui: &mut Ui,
    col: MatchCol,
    played: &MatchRow,
    favourite: bool,
    catalog: &Catalog,
    rtl: bool,
) {
    let whole = |n: i64| Some(catalog.number(n as f64, 0));
    match col {
        MatchCol::Demo => {
            // Where a demo's row has its status symbol, so the name keeps
            // clear of the filter pane's edge
            ui.add_space(theme::ICON_SIZE);
            let name = display_name(&played.path);
            named_cell(ui, &name, &played.path, favourite, catalog);
        }
        MatchCol::Date => {
            let (day, source) = (played.recorded_at.as_deref(), played.date_source.as_deref());
            day_cell(ui, day, source, catalog, rtl);
        }
        MatchCol::Map => {
            text_cell(ui, &played.map, text::starts_rtl(&played.map), Look::Plain);
        }
        MatchCol::Status => {
            let (title, help) = status_text(played.status);
            let look = if played.status == PartStatus::Complete {
                Look::Plain
            } else {
                Look::Weak
            };
            let help = text::help(ui.ctx(), &catalog.text(help), rtl);
            text_cell(ui, &catalog.text(title), rtl, look).on_hover_text(help);
        }
        MatchCol::Length => {
            number_cell(ui, clock(Some(played.length)), catalog, rtl);
        }
        MatchCol::Score => {
            let (board, summary) = (played.scoreboard.as_ref(), played.score.as_deref());
            score_cell(ui, board, summary, catalog, rtl);
        }
        MatchCol::Players => {
            number_cell(ui, whole(played.players), catalog, rtl);
        }
        MatchCol::Teams => {
            number_cell(ui, whole(played.teams), catalog, rtl);
        }
        MatchCol::Kills => {
            number_cell(ui, whole(played.kills), catalog, rtl);
        }

        // One kill at once or in a row is no multikill or spree, a dash
        MatchCol::Multikill => {
            let best = played.best_multikill.filter(|n| *n >= 2);
            number_cell(ui, best.and_then(whole), catalog, rtl);
        }
        MatchCol::Spree => {
            let best = played.best_spree.filter(|n| *n >= 2);
            number_cell(ui, best.and_then(whole), catalog, rtl);
        }
    }
}

/// What a match's narrow row leaves out, for its tooltip
fn match_details(ui: &mut Ui, played: &MatchRow, catalog: &Catalog, rtl: bool) {
    let mut lines = vec![played.path.clone()];
    if let Some(day) = played.recorded_at.as_deref().and_then(Ymd::parse) {
        lines.push(catalog.date(day));
    }

    let figures = [
        ("column-players", played.players),
        ("column-teams", played.teams),
        ("column-kills", played.kills),
    ];
    let mut counts = Vec::new();
    for (title, value) in figures {
        let value = catalog.number(value as f64, 0);
        counts.push(format!("{} {value}", catalog.text(title)));
    }
    lines.push(counts.join(" · "));

    let (_, help) = status_text(played.status);
    lines.push(catalog.text(help));

    for line in lines {
        text::label(ui, &line, rtl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_box_shows_or_hides_it_and_the_last_one_stays() {
        use PartStatus::{Complete, CutOff, FreePlay, Interrupted, JoinedLate};
        assert_eq!(status_toggled(&[], FreePlay), [FreePlay]);
        assert_eq!(status_toggled(&[FreePlay, CutOff], FreePlay), [CutOff]);

        // Complete alone is shown, and clearing it leaves it shown
        let others = [JoinedLate, Interrupted, CutOff, FreePlay];
        assert_eq!(status_toggled(&others, Complete), others);
    }
}
