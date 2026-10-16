//! The library list (implementation plan §8)
//!
//! Rows come from the cache a page at a time as they scroll into view, so a
//! library of any size costs only the visible pages. Changing a filter or
//! the order counts the matches again and drops the loaded pages. While a
//! scan stores files the list refreshes at most once a second

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText, Ui, WidgetText};
use egui_extras::{Column, TableBuilder};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::weapons::Weapon;
use lambda_slicer_core::library::cache::{Cache, FileQuery, FileRow, Sort};
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::timeline::time::DemoTime;

use crate::i18n::{Catalog, DateOrder, Ymd};
use crate::layout;
use crate::text::{self, Look};

/// A date bound with a checkbox to use it, its fields in the locale's order
fn date_filter(
    ui: &mut Ui,
    catalog: &Catalog,
    label: &WidgetText,
    value: &mut Option<Ymd>,
    salt: &str,
) {
    let mut on = value.is_some();
    if ui.checkbox(&mut on, label.clone()).changed() {
        *value = on.then(|| {
            let today = chrono::Local::now().date_naive();
            use chrono::Datelike;
            Ymd {
                year: today.year(),
                month: today.month(),
                day: today.day(),
            }
        });
    }
    let Some(d) = value else {
        return;
    };
    ui.push_id(salt, |ui| {
        let year = |ui: &mut Ui, d: &mut Ymd| {
            ui.add(
                egui::DragValue::new(&mut d.year)
                    .range(1998..=2100)
                    .speed(0.1),
            );
        };
        let month = |ui: &mut Ui, d: &mut Ymd| {
            ui.add(egui::DragValue::new(&mut d.month).range(1..=12).speed(0.1));
        };
        let day = |ui: &mut Ui, d: &mut Ymd| {
            let last = d.month_days();
            ui.add(egui::DragValue::new(&mut d.day).range(1..=last).speed(0.1));
        };
        let sep = |ui: &mut Ui, s: char| {
            ui.label(s.to_string());
        };
        match catalog.locale.date_order() {
            DateOrder::DayMonthYear(s) => {
                day(ui, d);
                sep(ui, s);
                month(ui, d);
                sep(ui, s);
                year(ui, d);
            }
            DateOrder::MonthDayYear(s) => {
                month(ui, d);
                sep(ui, s);
                day(ui, d);
                sep(ui, s);
                year(ui, d);
            }
            DateOrder::YearMonthDay(s) => {
                year(ui, d);
                sep(ui, s);
                month(ui, d);
                sep(ui, s);
                day(ui, d);
            }
        }
        d.day = d.day.min(d.month_days());
    });
}

/// A number typed with either a point or a comma before the decimals
fn parse_number(text: &str) -> Option<f64> {
    let cleaned: String = text
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}')
        .map(|c| if c == ',' { '.' } else { c })
        .collect();
    cleaned.parse().ok()
}

/// Rows loaded from the cache at a time
const PAGE: usize = 100;

/// Shortest time between refreshes while a scan stores files
const REFRESH: Duration = Duration::from_secs(1);

/// Weapons that leave events, so a shot count can show their use
pub const WEAPONS: [Weapon; 12] = [
    Weapon::Gauss,
    Weapon::Crossbow,
    Weapon::Rpg,
    Weapon::Mp5,
    Weapon::Shotgun,
    Weapon::Python,
    Weapon::Glock,
    Weapon::Crowbar,
    Weapon::Egon,
    Weapon::Hivehand,
    Weapon::Tripmine,
    Weapon::Snark,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spectating {
    #[default]
    Any,
    /// Spectating for more than half the playback
    Mostly,
    /// Spectating for at most half the playback
    Playing,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filters {
    pub text: String,
    pub map: String,
    pub player: String,
    pub weapon: Option<Weapon>,
    pub min_minutes: f32,
    pub spectating: Spectating,
    /// Recording date bounds, inclusive
    pub from: Option<Ymd>,
    pub to: Option<Ymd>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Col {
    Status,
    Name,
    Date,
    Map,
    Duration,
    Score,
    Gauss,
    Kills,
    Action,
    Spectating,
}

impl Col {
    const ALL: [Col; 10] = [
        Col::Status,
        Col::Name,
        Col::Date,
        Col::Map,
        Col::Duration,
        Col::Score,
        Col::Gauss,
        Col::Kills,
        Col::Action,
        Col::Spectating,
    ];

    /// The name kept in settings
    fn key(self) -> &'static str {
        match self {
            Col::Status => "status",
            Col::Name => "name",
            Col::Date => "date",
            Col::Map => "map",
            Col::Duration => "duration",
            Col::Score => "score",
            Col::Gauss => "gauss",
            Col::Kills => "kills",
            Col::Action => "action",
            Col::Spectating => "spectating",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Col::Status => "column-status",
            Col::Name => "column-name",
            Col::Date => "column-date",
            Col::Map => "column-map",
            Col::Duration => "column-duration",
            Col::Score => "column-score",
            Col::Gauss => "column-gauss",
            Col::Kills => "column-kills",
            Col::Action => "column-action",
            Col::Spectating => "column-spectating",
        }
    }

    fn sort(self) -> Option<Sort> {
        match self {
            Col::Status | Col::Score => None,
            Col::Name => Some(Sort::Path),
            Col::Date => Some(Sort::Date),
            Col::Map => Some(Sort::Map),
            Col::Duration => Some(Sort::Duration),
            Col::Gauss => Some(Sort::Gauss),
            Col::Kills => Some(Sort::Kills),
            Col::Action => Some(Sort::Action),
            Col::Spectating => Some(Sort::Spectating),
        }
    }

    fn column(self) -> Column {
        match self {
            Col::Status => Column::initial(150.).at_least(60.),
            Col::Name => Column::remainder().at_least(160.),
            Col::Date => Column::initial(95.).at_least(70.),
            Col::Map => Column::initial(120.).at_least(60.),
            Col::Duration => Column::initial(90.).at_least(70.),
            Col::Score => Column::initial(110.).at_least(60.),
            _ => Column::initial(80.).at_least(50.),
        }
    }

    /// Columns kept beside an open demo
    fn compact(self) -> bool {
        matches!(
            self,
            Col::Name | Col::Date | Col::Map | Col::Duration | Col::Score | Col::Gauss
        )
    }
}

pub struct LibraryView {
    cache_path: PathBuf,
    cache: Option<Cache>,
    pub filters: Filters,
    applied: Option<(Filters, Sort, bool)>,
    sort: Sort,
    descending: bool,
    total: u64,
    pages: HashMap<usize, Vec<FileRow>>,
    stale: bool,
    last_refresh: Instant,
    pub selected: Option<FileRow>,
    error: Option<Message>,
    /// Column keys left out, and whether that changed this frame
    pub hidden: Vec<String>,
    pub columns_changed: bool,
}

impl LibraryView {
    pub fn new(cache_path: PathBuf, hidden: Vec<String>) -> Self {
        Self {
            hidden,
            columns_changed: false,
            cache_path,
            cache: None,
            filters: Filters::default(),
            applied: None,
            sort: Sort::Path,
            descending: false,
            total: 0,
            pages: HashMap::new(),
            stale: true,
            last_refresh: Instant::now() - REFRESH,
            selected: None,
            error: None,
        }
    }

    /// Select the cached file at `path`, as if its row were clicked
    pub fn select_path(&mut self, path: &Path) {
        let wanted = path.display().to_string();
        let query = FileQuery {
            text: Some(wanted.clone()),
            limit: 50,
            ..Default::default()
        };
        if let Some(Ok(rows)) = self.cache().map(|c| c.query(&query)) {
            self.selected = rows.into_iter().find(|r| r.path == wanted);
        }
    }

    /// Stored files may have changed what the list shows
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    fn query(&self) -> FileQuery {
        let f = &self.filters;
        let text = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_string());
        FileQuery {
            text: text(&f.text),
            map: text(&f.map),
            player: text(&f.player),
            weapon_used: f.weapon.map(|w| w.name().to_string()),
            min_duration: (f.min_minutes > 0.).then_some(f.min_minutes * 60.),
            min_spectating: (f.spectating == Spectating::Mostly).then_some(0.5000001),
            max_spectating: (f.spectating == Spectating::Playing).then_some(0.5),
            from_date: f.from.map(Ymd::iso),
            to_date: f.to.map(Ymd::iso),
            sort: self.sort,
            descending: self.descending,
            limit: PAGE,
            ..Default::default()
        }
    }

    fn cache(&mut self) -> Option<&Cache> {
        if self.cache.is_none() && self.cache_path.exists() {
            match Cache::open(&self.cache_path) {
                Ok(c) => self.cache = Some(c),
                Err(e) => self.error = Some(e.message()),
            }
        }
        self.cache.as_ref()
    }

    /// Count again and drop loaded pages when the filters, the order or the
    /// stored files changed
    fn refresh(&mut self) {
        let wanted = (self.filters.clone(), self.sort, self.descending);
        let changed = self.applied.as_ref() != Some(&wanted);
        let due = self.stale && self.last_refresh.elapsed() >= REFRESH;
        if !changed && !due {
            return;
        }
        let query = self.query();
        let count = self.cache().map(|c| c.count(&query));
        match count {
            Some(Ok(n)) => {
                self.total = n;
                self.error = None;
            }
            Some(Err(e)) => self.error = Some(e.message()),
            None => self.total = 0,
        }
        self.pages.clear();
        self.applied = Some(wanted);
        self.stale = false;
        self.last_refresh = Instant::now();
    }

    fn row(&mut self, index: usize) -> Option<FileRow> {
        let page = index / PAGE;
        if !self.pages.contains_key(&page) {
            let query = FileQuery {
                offset: page * PAGE,
                ..self.query()
            };
            let rows = self.cache().map(|c| c.query(&query));
            match rows {
                Some(Ok(rows)) => {
                    self.pages.insert(page, rows);
                }
                Some(Err(e)) => self.error = Some(e.message()),
                None => return None,
            }
        }
        self.pages.get(&page)?.get(index % PAGE).cloned()
    }

    fn filters_ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        let t = |id: &str| text::widget_text(&ctx, &catalog.text(id), rtl, None, Look::Plain);
        layout::wrapping_row(ui, rtl, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.filters.text)
                    .hint_text(t("filter-search"))
                    .desired_width(180.),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.filters.map)
                    .hint_text(t("filter-map"))
                    .desired_width(110.),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.filters.player)
                    .hint_text(t("filter-player"))
                    .desired_width(130.),
            );

            let weapon = self
                .filters
                .weapon
                .map_or_else(|| t("filter-weapon-any"), |w| WidgetText::from(w.name()));
            egui::ComboBox::from_id_salt("weapon")
                .selected_text(weapon)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filters.weapon, None, t("filter-weapon-any"));
                    for w in WEAPONS {
                        ui.selectable_value(&mut self.filters.weapon, Some(w), w.name());
                    }
                });

            ui.label(t("filter-min-minutes"));
            ui.add(
                egui::DragValue::new(&mut self.filters.min_minutes)
                    .range(0.0..=120.0)
                    .speed(0.5)
                    .custom_formatter(|v, _| catalog.number(v, 1))
                    .custom_parser(parse_number),
            );

            let spectating = |s: Spectating| match s {
                Spectating::Any => "filter-spectating-any",
                Spectating::Mostly => "filter-spectating-mostly",
                Spectating::Playing => "filter-spectating-playing",
            };
            egui::ComboBox::from_id_salt("spectating")
                .selected_text(t(spectating(self.filters.spectating)))
                .show_ui(ui, |ui| {
                    for s in [Spectating::Any, Spectating::Mostly, Spectating::Playing] {
                        ui.selectable_value(&mut self.filters.spectating, s, t(spectating(s)));
                    }
                });

            date_filter(
                ui,
                catalog,
                &t("filter-from"),
                &mut self.filters.from,
                "from",
            );
            date_filter(ui, catalog, &t("filter-to"), &mut self.filters.to, "to");

            if self.filters != Filters::default() && ui.button(t("filter-clear")).clicked() {
                self.filters = Filters::default();
            }
        });
    }

    /// The status column's text for a row, and its tooltip
    fn status(row: &FileRow, catalog: &Catalog) -> (String, Option<String>) {
        let id = match row.status.as_str() {
            "complete" if row.shots.unwrap_or(0) == 0 => "status-no-use",
            "complete" => "status-complete",
            "partial" => "status-partial",
            "unsupported" => "status-unsupported",
            _ => "status-failed",
        };
        (
            catalog.text(id),
            row.error.as_ref().map(|m| catalog.stored(m)),
        )
    }

    fn cell(
        ui: &mut Ui,
        ctx: &egui::Context,
        text: &str,
        rtl: bool,
        color: Option<Color32>,
    ) -> egui::Response {
        let w = match color {
            Some(c) if !rtl && !text::has_rtl(text) => RichText::new(text).color(c).into(),
            _ => text::widget_text(ctx, text, rtl, None, Look::Plain),
        };
        ui.add(egui::Label::new(w).selectable(false).truncate())
    }

    /// The list with its filters. `compact` keeps the columns that fit
    /// beside an open demo
    pub fn ui(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool, compact: bool) {
        self.filters_ui(ui, catalog, rtl);
        self.refresh();
        ui.add_space(4.);

        let ctx = ui.ctx().clone();
        let total = self.total;
        let empty = if self.filters == Filters::default() {
            "library-empty"
        } else {
            "library-no-matches"
        };
        layout::column(ui, rtl, |ui| {
            if let Some(e) = &self.error {
                text::label(ui, &catalog.message(e), rtl);
            }
            let line = match total {
                0 => catalog.text(empty),
                n => catalog.text_with("library-demos", &[("count", (n as i64).into())]),
            };
            text::label(ui, &line, rtl);
        });
        if total == 0 {
            return;
        }

        // Columns run right to left in a right-to-left interface
        let mut cols: Vec<Col> = Col::ALL
            .into_iter()
            .filter(|c| *c == Col::Name || !self.hidden.iter().any(|h| h == c.key()))
            .filter(|c| !compact || c.compact())
            .collect();
        if rtl {
            cols.reverse();
        }
        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 4.;
        let mut clicked = None;
        let mut sort_by = None;
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .max_scroll_height(f32::INFINITY)
            .min_scrolled_height(0.)
            .auto_shrink(false);
        for c in &cols {
            table = table.column(c.column().clip(true));
        }
        table
            .header(row_height + 4., |mut header| {
                for c in &cols {
                    header.col(|ui| {
                        let mut title = catalog.text(c.title());
                        if c.sort() == Some(self.sort) {
                            title.push_str(if self.descending { " ▼" } else { " ▲" });
                        }
                        let w = text::widget_text(&ctx, &title, rtl, None, Look::Strong);
                        let r = ui.add(
                            egui::Label::new(w)
                                .selectable(false)
                                .sense(egui::Sense::click()),
                        );
                        if r.clicked() {
                            sort_by = c.sort();
                        }
                        // Right-click any heading to choose the columns
                        r.context_menu(|ui| {
                            for col in Col::ALL.into_iter().filter(|c| *c != Col::Name) {
                                let mut shown = !self.hidden.iter().any(|h| h == col.key());
                                let label = text::widget_text(
                                    &ctx,
                                    &catalog.text(col.title()),
                                    rtl,
                                    None,
                                    Look::Plain,
                                );
                                if ui.checkbox(&mut shown, label).changed() {
                                    self.hidden.retain(|h| h != col.key());
                                    if !shown {
                                        self.hidden.push(col.key().to_string());
                                    }
                                    self.columns_changed = true;
                                }
                            }
                        });
                    });
                }
            })
            .body(|body| {
                body.rows(row_height, self.total as usize, |mut row| {
                    let index = row.index();
                    let Some(file) = self.row(index) else {
                        return;
                    };
                    let selected = self.selected.as_ref().is_some_and(|s| s.id == file.id);
                    row.set_selected(selected);
                    let name = Path::new(&file.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| file.path.clone());
                    for c in &cols {
                        row.col(|ui| match c {
                            Col::Status => {
                                let (status, tip) = Self::status(&file, catalog);
                                let muted = file.status == "complete";
                                let color = (!muted).then_some(ui.visuals().warn_fg_color);
                                let r = Self::cell(ui, &ctx, &status, rtl, color);
                                if let Some(tip) = tip {
                                    r.on_hover_text(tip);
                                }
                            }
                            // Names keep their own direction in any interface
                            Col::Name => {
                                Self::cell(ui, &ctx, &name, text::starts_rtl(&name), None)
                                    .on_hover_text(&file.path);
                            }
                            Col::Map => {
                                let map = file.map.as_deref().unwrap_or("");
                                Self::cell(ui, &ctx, map, text::starts_rtl(map), None);
                            }
                            Col::Date => {
                                if let Some(d) = file.recorded_at.as_deref().and_then(Ymd::parse) {
                                    let tip = match file.date_source.as_deref() {
                                        Some("file_name") => catalog.text("date-from-name"),
                                        _ => catalog.text("date-from-modified"),
                                    };
                                    let time = file
                                        .recorded_at
                                        .as_deref()
                                        .and_then(|t| t.get(11..))
                                        .unwrap_or("");
                                    let tip = format!("{} {time}\n{tip}", catalog.date(d));
                                    Self::cell(ui, &ctx, &catalog.date(d), false, None)
                                        .on_hover_text(text::widget_text(
                                            &ctx,
                                            &tip,
                                            rtl,
                                            None,
                                            Look::Plain,
                                        ));
                                }
                            }
                            Col::Score => {
                                Self::cell(
                                    ui,
                                    &ctx,
                                    file.score.as_deref().unwrap_or(""),
                                    false,
                                    None,
                                );
                            }
                            Col::Duration => {
                                let d = file
                                    .duration
                                    .and_then(DemoTime::from_f32)
                                    .map(|d| d.to_string())
                                    .unwrap_or_default();
                                let d = d.split('.').next().unwrap_or("").to_string();
                                Self::cell(ui, &ctx, &d, false, None);
                            }
                            Col::Gauss => {
                                let n = file.gauss.unwrap_or(0);
                                let text = if n > 0 {
                                    catalog.number(n as f64, 0)
                                } else {
                                    String::new()
                                };
                                let color = Some(ui.visuals().hyperlink_color);
                                Self::cell(ui, &ctx, &text, false, color);
                            }
                            Col::Kills => {
                                let text = file
                                    .kills
                                    .filter(|k| *k > 0)
                                    .map(|k| catalog.number(k as f64, 0))
                                    .unwrap_or_default();
                                Self::cell(ui, &ctx, &text, false, None);
                            }
                            Col::Action => {
                                let text = file
                                    .best_window
                                    .map(|s| catalog.number(s, 2))
                                    .unwrap_or_default();
                                Self::cell(ui, &ctx, &text, false, None);
                            }
                            Col::Spectating => {
                                let text = file
                                    .spectator_fraction
                                    .filter(|s| *s > 0.)
                                    .map(|s| {
                                        catalog.text_with(
                                            "library-percent",
                                            &[("percent", FluentValue::from((s * 100.).round()))],
                                        )
                                    })
                                    .unwrap_or_default();
                                Self::cell(ui, &ctx, &text, rtl, None);
                            }
                        });
                    }
                    if row.response().clicked() {
                        clicked = Some(file);
                    }
                });
            });

        if let Some(s) = sort_by {
            if s == self.sort {
                self.descending = !self.descending;
            } else {
                self.sort = s;
                // Counts and scores read best from the top
                self.descending = !matches!(s, Sort::Path | Sort::Map);
            }
        }
        if let Some(f) = clicked {
            self.selected = Some(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_number;

    #[test]
    fn numbers_parse_with_either_decimal_mark() {
        assert_eq!(parse_number("2,5"), Some(2.5));
        assert_eq!(parse_number(" 2.5 "), Some(2.5));
        assert_eq!(parse_number("1\u{a0}000,5"), Some(1000.5));
        assert_eq!(parse_number("x"), None);
    }
}
