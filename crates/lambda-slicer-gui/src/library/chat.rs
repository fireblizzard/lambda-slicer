//! The library's Chat tab, every line of chat in the demos the filters
//! find on a row of its own
//!
//! The demos' filters test each line's demo, and the map filter the map the
//! line was sent on. The chat's own filters test what was said, who said it
//! and whom to. A row's tooltip shows the lines sent around it, and opening
//! a row opens its demo a moment before the line was sent, so it is seen
//! arriving. The narrow list beside the open demo lists the lines too, two
//! lines a row

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use eframe::egui::{
    self, Color32, Pos2, Rect, Response, RichText, Role, Sense, TextStyle, TextWrapMode, Ui, Vec2,
    WidgetInfo, WidgetText, text::LayoutJob,
};
use egui_extras::{Column, TableBuilder};
use lambda_slicer_core::library::cache::{
    ChatQuery, ChatRow, ChatSort, Either, FileQuery, LeftOut, fold,
};
use lambda_slicer_core::message::Coded;

use super::worker::PageOf;
use super::{
    GROUP_GAP, LIST_LINES, LIST_ROW, LibraryView, PAGE, SORT_MENU_WIDTH, TABLE_ROW, choice, clock,
    day_cell, direction_button, display_name, named_cell, number_cell, sort_heading, text_cell,
};
use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};
use crate::theme;

/// How long before a line was sent its demo opens, in seconds
const LEAD: f32 = 1.;

/// How many lines before and after a row's its tooltip shows
const AROUND: i64 = 4;

/// What a code for the sender's place shows, as the library reads no map
/// to name it from
const PLACE: &str = "…";

/// The chat's own filters
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatFilters {
    /// Text in what was said, and in the sender's name
    pub text: String,
    pub sender: String,
    /// To everyone or to the team alone, from players or from spectators
    pub to: Either,
    pub from: Either,
    /// The recording player's own lines alone
    pub mine: bool,
    /// The server's own lines left out
    pub hide_server: bool,
}

impl ChatFilters {
    /// How many are on
    pub fn active(&self) -> usize {
        let on = [
            !self.text.trim().is_empty(),
            !self.sender.trim().is_empty(),
            self.to != Either::Both,
            self.from != Either::Both,
            self.mine,
            self.hide_server,
        ];
        on.into_iter().filter(|on| *on).count()
    }
}

/// The lines' filters, order and loaded pages
#[derive(Debug)]
pub struct ChatList {
    pub filters: ChatFilters,
    /// The lines from binds the settings leave out of the demo player's
    /// chat, left out here too
    pub left_out: LeftOut,
    pub sort: ChatSort,
    pub descending: bool,
    /// Lines the filters find
    pub total: u64,
    pub pages: HashMap<usize, Vec<ChatRow>>,
    /// The pages asked of the library's thread
    pub asked: HashSet<usize>,
    /// The line last opened, by its demo and its place in it
    selected: Option<(i64, i64)>,
    /// The line whose menu was open as the rows were last drawn, kept lit
    /// as if hovered, and the one found open this frame
    menu_row: Option<(i64, i64)>,
    menu_open: Option<(i64, i64)>,
    /// The lines around each line hovered, by the line
    around: HashMap<(i64, i64), Vec<ChatRow>>,
}

impl Default for ChatList {
    /// The newest demos' lines first
    fn default() -> Self {
        Self {
            filters: ChatFilters::default(),
            left_out: LeftOut::default(),
            sort: ChatSort::Date,
            descending: true,
            total: 0,
            pages: HashMap::new(),
            asked: HashSet::new(),
            selected: None,
            menu_row: None,
            menu_open: None,
            around: HashMap::new(),
        }
    }
}

impl ChatList {
    /// What the count depends on besides the demos' filters
    pub fn order(&self) -> (ChatFilters, LeftOut, ChatSort, bool) {
        let filters = self.filters.clone();
        (filters, self.left_out, self.sort, self.descending)
    }

    /// Ask for the pages again and drop the lines around, for another
    /// count. The pages loaded show until theirs come
    pub fn clear(&mut self) {
        self.asked.clear();
        self.around.clear();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChatCol {
    Date,
    Map,
    Time,
    Sender,
    Message,
    Demo,
}

impl ChatCol {
    const ALL: [ChatCol; 6] = [
        ChatCol::Date,
        ChatCol::Map,
        ChatCol::Time,
        ChatCol::Sender,
        ChatCol::Message,
        ChatCol::Demo,
    ];

    fn title(self) -> &'static str {
        match self {
            ChatCol::Date => "column-date",
            ChatCol::Map => "column-map",
            ChatCol::Time => "column-time",
            ChatCol::Sender => "column-sender",
            ChatCol::Message => "column-message",
            ChatCol::Demo => "column-name",
        }
    }

    fn help(self) -> Option<&'static str> {
        match self {
            ChatCol::Date => Some("help-date-column"),
            ChatCol::Time => Some("help-chat-time"),
            _ => None,
        }
    }

    /// The order the column sorts by, none for the time, as lines of one
    /// demo are always in the order sent
    fn sort(self) -> Option<ChatSort> {
        match self {
            ChatCol::Date => Some(ChatSort::Date),
            ChatCol::Map => Some(ChatSort::Map),
            ChatCol::Time => None,
            ChatCol::Sender => Some(ChatSort::Sender),
            ChatCol::Message => Some(ChatSort::Message),
            ChatCol::Demo => Some(ChatSort::Path),
        }
    }

    fn column(self) -> Column {
        let (width, least) = match self {
            ChatCol::Date => (112., 80.),
            ChatCol::Map => (112., 60.),
            ChatCol::Time => (80., 64.),
            ChatCol::Sender => (160., 80.),
            ChatCol::Message => (420., 160.),
            ChatCol::Demo => (240., 120.),
        };
        Column::initial(width).at_least(least).clip(true)
    }
}

/// Dates from the newest, names and words from the start of the alphabet
fn descending_first(sort: ChatSort) -> bool {
    sort == ChatSort::Date
}

/// Whom the lines go to, or who sends them, once `toggle` is clicked while
/// it is `now`. Both on is every line, and the last one on stays on, as no
/// line would be left
fn either_toggled(now: Either, toggle: Either) -> Either {
    let other = if toggle == Either::First {
        Either::Second
    } else {
        Either::First
    };
    match now {
        Either::Both => other,
        n if n == other => Either::Both,
        n => n,
    }
}

/// A line's message as the library shows it, without colour codes and
/// with its codes for the sender's place unnamed
fn shown(message: &str) -> String {
    crate::chat::with_place(&text::without_colours(message), PLACE)
}

/// Where `needle` is in `text`, ignoring case as the search does, as byte
/// ranges of `text`, none where they would overlap
fn found(text: &str, needle: &str) -> Vec<Range<usize>> {
    let needle = fold(needle.trim());
    if needle.is_empty() {
        return Vec::new();
    }

    // Each byte of the folded text, with the letter of `text` it came from
    let mut folded = Vec::new();
    let mut origin = Vec::new();
    for (at, c) in text.char_indices() {
        let letter = at..at + c.len_utf8();
        for f in fold(&c.to_string()).bytes() {
            folded.push(f);
            origin.push(letter.clone());
        }
    }

    let mut out = Vec::new();
    let mut from = 0;
    while let Some(start) = folded[from..]
        .windows(needle.len())
        .position(|w| w == needle.as_bytes())
    {
        let start = from + start;
        let end = start + needle.len();
        out.push(origin[start].start..origin[end - 1].end);
        from = end;
    }
    out
}

/// `text` in `colour`, the parts `needle` finds lit in the accent. Text
/// that reads right to left goes unlit, as its pieces are laid out in
/// visual order
fn highlighted(ui: &Ui, text: &str, needle: &str, colour: Color32, rtl: bool) -> WidgetText {
    let parts = found(text, needle);
    if parts.is_empty() || rtl || text::has_rtl(text) {
        return text::widget_text(ui.ctx(), text, rtl, None, Look::Color(colour));
    }

    let font = TextStyle::Body.resolve(ui.style());
    let plain = egui::TextFormat::simple(font.clone(), colour);
    let lit = egui::TextFormat {
        background: theme::ACCENT_SOFT,
        ..egui::TextFormat::simple(font, theme::ACCENT)
    };
    let mut job = LayoutJob::default();
    let mut at = 0;
    for part in parts {
        job.append(&text[at..part.start], 0., plain.clone());
        job.append(&text[part.clone()], 0., lit.clone());
        at = part.end;
    }
    job.append(&text[at..], 0., plain);
    WidgetText::LayoutJob(job.into())
}

/// The message naming whom a line went to or who sent it, none for a
/// player's line to everyone
fn said_tag(line: &ChatRow) -> Option<&'static str> {
    match (line.team_only, line.spectator) {
        (false, false) => None,
        (true, false) => Some("chat-tag-team"),
        (false, true) => Some("chat-tag-spectator"),
        (true, true) => Some("chat-tag-spectators"),
    }
}

/// A line's sender without colour codes in their team's colour, or the
/// server faint
fn sender_text(ui: &Ui, line: &ChatRow, catalog: &Catalog) -> WidgetText {
    match &line.sender {
        Some(name) => {
            let name = text::player_name(name);
            let colour = theme::team_colour(line.team.as_deref().unwrap_or(""));
            text::widget_text(
                ui.ctx(),
                &name,
                text::starts_rtl(&name),
                None,
                Look::Color(colour),
            )
        }
        None => {
            let server = catalog.text("chat-server");
            text::widget_text(ui.ctx(), &server, false, None, Look::Color(theme::FAINT))
        }
    }
}

impl LibraryView {
    /// The chat's own filters, for the Chat tab alone, under the title the
    /// filter pane gives its first group
    pub(super) fn chat_filters(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
        let filters = &mut self.chat.filters;

        let words = egui::TextEdit::singleline(&mut filters.text)
            .hint_text(t("filter-chat-text"))
            .desired_width(f32::INFINITY);
        ui.add(words);
        let sender = egui::TextEdit::singleline(&mut filters.sender)
            .hint_text(t("filter-chat-sender"))
            .desired_width(f32::INFINITY);
        ui.add(sender);

        // Whom to and who from, two toggles each, both on for every line
        let pairs = [
            (
                "filter-chat-to",
                &mut filters.to,
                ["chat-to-everyone", "chat-to-team"],
            ),
            (
                "filter-chat-from",
                &mut filters.from,
                ["chat-from-players", "chat-from-spectators"],
            ),
        ];
        for (title, now, names) in pairs {
            ui.label(t(title));
            layout::row(ui, rtl, |ui| {
                let spacing = ui.spacing().item_spacing.x;
                let width = (ui.available_width() - spacing) / 2.;
                let toggles = [(Either::First, names[0]), (Either::Second, names[1])];
                for (toggle, id) in toggles {
                    let on = *now == Either::Both || *now == toggle;
                    if ui.add(choice(on, t(id), width)).clicked() {
                        *now = either_toggled(*now, toggle);
                    }
                }
            });
        }

        ui.checkbox(&mut filters.mine, t("filter-chat-mine"));
        ui.checkbox(&mut filters.hide_server, t("filter-chat-hide-server"));
        ui.add_space(GROUP_GAP);
    }

    /// The cache query for the filters and the lines' order, a page long
    pub(super) fn chat_query(&self) -> ChatQuery {
        let filters = &self.chat.filters;
        let text = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        ChatQuery {
            files: FileQuery {
                descending: self.chat.descending,
                ..self.query()
            },
            text: text(&filters.text),
            sender: text(&filters.sender),
            to: filters.to,
            from: filters.from,
            mine: filters.mine,
            hide_server: filters.hide_server,
            left_out: self.chat.left_out,
            sort: self.chat.sort,
        }
    }

    fn chat_row(&mut self, index: usize) -> Option<ChatRow> {
        let page = index / PAGE;
        if self.chat.asked.insert(page) {
            let mut query = self.chat_query();
            query.files.offset = page * PAGE;
            self.ask_page(page, PageOf::Chat(query));
        }
        self.chat.pages.get(&page)?.get(index % PAGE).cloned()
    }

    /// The lines sent around `line` in its demo, read once
    fn around(&mut self, line: &ChatRow) -> Vec<ChatRow> {
        let key = (line.file_id, line.ordinal);
        if !self.chat.around.contains_key(&key) {
            let left_out = self.chat.left_out;
            let rows = self
                .cache()
                .map(|c| c.chat_around(line.file_id, line.ordinal, AROUND, left_out));
            let rows = match rows {
                Some(Ok(rows)) => rows,
                Some(Err(e)) => {
                    self.error = Some(e.message());
                    Vec::new()
                }
                None => Vec::new(),
            };
            self.chat.around.insert(key, rows);
        }
        self.chat.around.get(&key).cloned().unwrap_or_default()
    }

    /// Sort the lines by `sort`, turning the order round when it is already
    /// the order
    fn sort_chat_by(&mut self, sort: ChatSort) {
        if sort == self.chat.sort {
            self.chat.descending = !self.chat.descending;
        } else {
            self.chat.sort = sort;
            self.chat.descending = descending_first(sort);
        }
    }

    /// Open the demo `line` was sent in, a moment before it was sent
    fn open_line(&mut self, line: &ChatRow) {
        self.select_path(&line.source());
        if self.selected.is_some() {
            self.open_at = Some((line.time - LEAD).max(0.));
            self.chat.selected = Some((line.file_id, line.ordinal));
        }
    }

    /// Whether `line` is the line last opened, in the demo still open
    fn line_open(&self, line: &ChatRow) -> bool {
        let demo_open = self.selected.as_ref().is_some_and(|s| s.id == line.file_id);
        demo_open && self.chat.selected == Some((line.file_id, line.ordinal))
    }

    /// The demo's menu on the row of `line`, the row lit while it is open
    fn line_menu(&mut self, response: &Response, line: &ChatRow, catalog: &Catalog, rtl: bool) {
        let marks = self.marks(&line.path);
        let choice = super::row_menu(response, &line.path, marks, catalog, rtl);
        if choice.open {
            self.chat.menu_open = Some((line.file_id, line.ordinal));
        }
        self.take_choice(choice, &line.path);
    }

    /// The lines sent around `line`, its own lit, under its demo and map,
    /// for a row's tooltip
    fn around_ui(&mut self, ui: &mut Ui, line: &ChatRow, catalog: &Catalog, rtl: bool) {
        let place = match &line.map {
            Some(map) => format!("{} · {map}", display_name(&line.path)),
            None => display_name(&line.path),
        };
        ui.label(text::widget_text(ui.ctx(), &place, rtl, None, Look::Weak));
        ui.separator();

        for near in self.around(line) {
            let own = near.ordinal == line.ordinal;
            layout::row(ui, rtl, |ui| {
                let time = clock(Some(near.time)).unwrap_or_default();
                ui.label(RichText::new(time).monospace().color(theme::FAINT));
                ui.label(sender_text(ui, &near, catalog));
                let colour = if own {
                    theme::ACCENT
                } else {
                    theme::FOREGROUND
                };
                let message = shown(&near.message);
                let rtl_message = text::starts_rtl(&message);
                let look = Look::Color(colour);
                ui.label(text::widget_text(
                    ui.ctx(),
                    &message,
                    rtl_message,
                    None,
                    look,
                ));
            });
        }
    }

    /// The narrow list's order of the lines, and the button turning it
    /// round
    pub(super) fn chat_sort_menu(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);

        let mut sort = self.chat.sort;
        let current = ChatCol::ALL
            .into_iter()
            .find(|c| c.sort() == Some(sort))
            .map_or_else(String::new, |c| catalog.text(c.title()));
        egui::ComboBox::from_id_salt("chat list sort")
            .width(SORT_MENU_WIDTH)
            .selected_text(text::widget_text(&ctx, &current, rtl, None, Look::Plain))
            .show_ui(ui, |ui| {
                for col in ChatCol::ALL {
                    if let Some(by) = col.sort() {
                        ui.selectable_value(&mut sort, by, t(col.title()));
                    }
                }
            })
            .response
            .on_hover_text(catalog.text("sort-by"));
        if sort != self.chat.sort {
            self.chat.sort = sort;
            self.chat.descending = descending_first(sort);
        }

        direction_button(ui, &mut self.chat.descending, catalog);
    }

    /// The lines in the narrow list, two lines a row
    pub(super) fn chat_list(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let mut clicked = None;

        // Rows touch, so the rows the scroll area works out are the rows it
        // draws, at the same places
        ui.spacing_mut().item_spacing.y = 0.;
        egui::ScrollArea::vertical()
            .id_salt("chat list")
            .auto_shrink(false)
            .show_rows(ui, LIST_ROW, self.chat.total as usize, |ui, rows| {
                for index in rows {
                    let Some(line) = self.chat_row(index) else {
                        continue;
                    };
                    let response = self.chat_list_row(ui, &line, catalog, rtl);
                    self.line_menu(&response, &line, catalog, rtl);
                    if response.clicked() {
                        clicked = Some(line.clone());
                    }
                    if response.hovered() {
                        response.on_hover_ui(|ui| self.around_ui(ui, &line, catalog, rtl));
                    }
                }
            });
        self.chat.menu_row = self.chat.menu_open.take();

        if let Some(line) = clicked {
            self.open_line(&line);
        }
    }

    /// One line of chat in two lines, its sender and message, then its
    /// map, time and demo
    fn chat_list_row(&self, ui: &mut Ui, line: &ChatRow, catalog: &Catalog, rtl: bool) -> Response {
        let size = Vec2::new(ui.available_width(), LIST_ROW);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let selected = self.line_open(line);
        let key = (line.file_id, line.ordinal);
        let message = shown(&line.message);
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, &message));

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
        } else if response.hovered() || self.chat.menu_row == Some(key) {
            painter.rect_filled(rect, 0., theme::RAISED);
        }

        // The first line is the sender and the message, the second the map,
        // the time and the demo
        let sender = sender_text(ui, line, catalog);
        let words = highlighted(
            ui,
            &message,
            &self.chat.filters.text,
            theme::FOREGROUND,
            rtl,
        );
        let mut facts = Vec::new();
        facts.extend(line.map.clone());
        facts.extend(clock(Some(line.time)));
        facts.push(display_name(&line.path));
        let facts = text::styled_text(
            ui.ctx(),
            &facts.join(" · "),
            rtl,
            Look::Color(theme::MUTED),
            &theme::secondary(),
        );

        let inner = rect.shrink2(Vec2::new(theme::GAP_M, 0.));
        let place = |galley: &std::sync::Arc<egui::Galley>, x: f32, k: usize| {
            let line = rect.min.y + LIST_LINES[k];
            Pos2::new(x, line - galley.size().y / 2.)
        };

        // The sender first, the message cut short to the room it leaves
        let sender = sender.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Body,
        );
        let room = inner.width() - sender.size().x - theme::GAP_S;
        let words = words.into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            room.max(0.),
            TextStyle::Body,
        );
        let (sender_x, words_x) = if rtl {
            let sender_x = inner.max.x - sender.size().x;
            (sender_x, sender_x - theme::GAP_S - words.size().x)
        } else {
            (inner.min.x, inner.min.x + sender.size().x + theme::GAP_S)
        };
        ui.painter()
            .galley(place(&sender, sender_x, 0), sender, theme::FOREGROUND);
        ui.painter()
            .galley(place(&words, words_x, 0), words, theme::FOREGROUND);

        let facts = facts.into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            inner.width(),
            theme::secondary(),
        );
        let x = if rtl {
            inner.max.x - facts.size().x
        } else {
            inner.min.x
        };
        ui.painter()
            .galley(place(&facts, x, 1), facts, theme::MUTED);

        response
    }

    /// The lines' table, which scrolls sideways as the demos' does
    pub(super) fn chat_table(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let open = self.keyboard(&ctx, self.chat.total);
        let mut cols = ChatCol::ALL.to_vec();
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
            .id_salt("chat columns")
            .stick_to_right(rtl)
            .show(ui, |ui| {
                let mut table = TableBuilder::new(ui)
                    .id_salt("chat")
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
                                // Clear of the filter pane's edge, as the
                                // other tables' first column
                                if *c == ChatCol::Date {
                                    ui.add_space(theme::GAP_S);
                                }
                                let sorted = c
                                    .sort()
                                    .filter(|s| *s == self.chat.sort)
                                    .map(|_| self.chat.descending);
                                let help = c.help().map(|key| catalog.text(key));
                                let title = catalog.text(c.title());
                                let heading =
                                    sort_heading(ui, &title, help.as_deref(), sorted, rtl);
                                if heading.clicked() {
                                    sort_by = c.sort();
                                }
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(TABLE_ROW, self.chat.total as usize, |mut row| {
                            let index = row.index();
                            let Some(line) = self.chat_row(index) else {
                                return;
                            };
                            let key = (line.file_id, line.ordinal);
                            let menu = self.chat.menu_row == Some(key);
                            row.set_hovered(self.cursor == Some(index) || menu);
                            let favourite = self.favourite(&line.path);
                            let needle = self.chat.filters.text.clone();
                            for c in &cols {
                                row.col(|ui| {
                                    chat_cell(ui, *c, &line, &needle, favourite, catalog, rtl);
                                });
                            }
                            let response = row.response();
                            if response.clicked() {
                                clicked = Some(index);
                            }
                            self.line_menu(&response, &line, catalog, rtl);
                            if response.hovered() {
                                response.on_hover_ui(|ui| self.around_ui(ui, &line, catalog, rtl));
                            }
                        });
                    });
            });
        self.chat.menu_row = self.chat.menu_open.take();

        if let Some(sort) = sort_by {
            self.sort_chat_by(sort);
        }

        if let Some(index) = clicked.or(open) {
            self.cursor = Some(index);
            if let Some(line) = self.chat_row(index) {
                self.open_line(&line);
            }
        }
    }
}

/// One cell of a line's row, its demo's name starred when the demo is a
/// `favourite`, the parts of the message `needle` finds lit
fn chat_cell(
    ui: &mut Ui,
    col: ChatCol,
    line: &ChatRow,
    needle: &str,
    favourite: bool,
    catalog: &Catalog,
    rtl: bool,
) {
    match col {
        ChatCol::Date => {
            // Where a demo's row has its status symbol, so the date keeps
            // clear of the filter pane's edge
            ui.add_space(theme::GAP_S);
            let (day, source) = (line.recorded_at.as_deref(), line.date_source.as_deref());
            day_cell(ui, day, source, catalog, rtl);
        }
        ChatCol::Map => {
            let map = line.map.as_deref().unwrap_or("");
            text_cell(ui, map, text::starts_rtl(map), Look::Plain);
        }
        ChatCol::Time => {
            number_cell(ui, clock(Some(line.time)), catalog, rtl);
        }
        ChatCol::Sender => {
            let sender = sender_text(ui, line, catalog);
            ui.add(egui::Label::new(sender).selectable(false).truncate());
        }
        ChatCol::Message => {
            if let Some(tag) = said_tag(line) {
                let tag = RichText::new(catalog.text(tag))
                    .text_style(theme::secondary())
                    .color(theme::MUTED);
                ui.add(egui::Label::new(tag).selectable(false));
            }
            let message = shown(&line.message);
            let rtl_message = text::starts_rtl(&message);
            let words = highlighted(ui, &message, needle, theme::FOREGROUND, rtl_message);
            ui.add(egui::Label::new(words).selectable(false).truncate());
        }
        ChatCol::Demo => {
            let name = display_name(&line.path);
            named_cell(ui, &name, &line.path, favourite, catalog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn found_text_is_where_the_search_finds_it_ignoring_case() {
        assert_eq!(found("GG wp, gg", "gg"), [0..2, 7..9]);
        assert_eq!(found("Iİı", "i"), [0..1, 1..3, 3..5]);
        assert!(found("hello", " ").is_empty());
        assert!(found("hello", "xyz").is_empty());
    }

    #[test]
    fn a_toggle_shows_or_hides_its_lines_and_the_last_one_stays() {
        use Either::{Both, First, Second};
        assert_eq!(either_toggled(Both, First), Second);
        assert_eq!(either_toggled(Second, First), Both);
        assert_eq!(either_toggled(Second, Second), Second);
    }

    #[test]
    fn a_message_shows_without_colour_codes_and_with_its_place_unnamed() {
        assert_eq!(shown("^8Recharging at ^7[^5%l^7]"), "Recharging at […]");
    }
}
