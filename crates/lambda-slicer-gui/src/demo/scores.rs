//! The Scoreboard tab, the standings at the selected event or the final
//! ones, a table per team or per player of a duel and one ranked table for
//! anything else (GUI redesign plan §10.3)
//!
//! Other players' kills and shots often happen outside what the recording
//! client receives, so their headshots and accuracy are incomplete. Only
//! the recording player's are shown, and in a duel the opponent's headshots
//! too, since every kill there is of the recording player, whose death the
//! recording always shows. Anything not shown is a dash, never a zero

use std::collections::HashMap;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rangef, Rect, Sense, Ui, Vec2};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::accuracy::accuracy;
use lambda_slicer_core::analysis::context::{PlayerScore, StandingSide, standing_at};
use lambda_slicer_core::timeline::time::DemoTime;

use super::{DemoAction, DemoView};
use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::timeline::{EventKind, Model};
use crate::widgets::{self, Variant};

/// A table row's height, the least width of each number column from the
/// trailing end, accuracy first, and the gap a column keeps before the next
const ROW_HEIGHT: f32 = 24.;
const NUMBER_COLUMNS: [f32; 4] = [52., 40., 48., 48.];
const COLUMN_GAP: f32 = theme::GAP_S;

/// The two sides' totals over their comparison strip, and the strip
const TOTAL_SIZE: f32 = 26.;
const STRIP_HEIGHT: f32 = 4.;

/// The gap between tables, and the recorder's mark beside their name
const TABLE_GAP: f32 = theme::GAP_M;
const RECORDER_MARK: f32 = 3.;

/// What a value nobody can know shows as
const UNKNOWN: &str = "—";

/// One player's line of a table
struct PlayerRow {
    name: String,
    recorder: bool,
    frags: i32,
    deaths: i32,
    headshots: Option<u32>,
    accuracy: Option<f64>,
}

/// A team's or a duel player's table, or the one table of a game with
/// neither
struct SideTable {
    title: Option<String>,
    colour: Color32,
    frags: i32,
    players: Vec<PlayerRow>,
}

/// Whether two sides' frags can be compared as shares of one strip, which
/// only nonnegative scores with something scored can
fn comparable(frags: &[i32]) -> bool {
    let [a, b] = frags else {
        return false;
    };

    *a >= 0 && *b >= 0 && a + b > 0
}

/// A time as the heading gives it
fn clock(time: f32) -> String {
    DemoTime::from_f32(time)
        .map(|t| t.to_string())
        .unwrap_or_default()
}

impl DemoView {
    /// The heading saying whose moment the scores are of, the two sides'
    /// totals when they compare, and the tables
    pub(super) fn scores_tab(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let Some(model) = self.model.clone() else {
            return;
        };

        let selected = self.selected.and_then(|i| model.events.get(i));

        // The final scores are of the map played longest, since a demo may
        // end with a few moments of the next one
        let longest = model.groups.longest_segment();
        let last_kill = model.events.iter().rev().find(|e| {
            let kill = matches!(e.kind, EventKind::Kill { .. });
            kill && longest.is_none_or(|s| e.segment == s)
        });

        // The selected event's moment, or the last kill for the final scores,
        // since players often leave once a game is over
        let (segment, at) = match (selected, last_kill) {
            (Some(e), _) | (None, Some(e)) => (e.segment, e.time),
            (None, None) => {
                let last = model.score_updates.last().map_or(0, |u| u.segment);
                (longest.unwrap_or(last), f32::INFINITY)
            }
        };
        let heading = match (selected, last_kill) {
            (Some(e), _) => {
                let time = FluentValue::from(clock(e.time));
                catalog.text_with("scores-at", &[("time", time)])
            }
            (None, kill) => {
                let time = FluentValue::from(kill.map(|e| clock(e.time)).unwrap_or_default());
                catalog.text_with("scores-final", &[("time", time)])
            }
        };

        layout::wrapping_row(ui, rtl, |ui| {
            let look = Look::Color(theme::MUTED);
            ui.label(text::styled_text(
                ui.ctx(),
                &heading,
                rtl,
                look,
                &theme::secondary(),
            ));
            if selected.is_some() {
                let label = catalog.text("scores-show-final");
                let label = text::widget_text(ui.ctx(), &label, rtl, None, Look::Plain);
                let button = widgets::Button::new(None, label, Variant::Ghost, rtl);
                if ui.add(button).clicked() {
                    self.actions.push(DemoAction::ShowFinal);
                }
            }
        });
        ui.add_space(theme::GAP_S);

        let tables = score_tables(&model, segment, at);
        if tables.is_empty() {
            let none = catalog.text("scores-none");
            ui.label(text::widget_text(
                ui.ctx(),
                &none,
                rtl,
                None,
                Look::Color(theme::MUTED),
            ));
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt(("scores", self.generation))
            .auto_shrink(false)
            .show(ui, |ui| {
                let frags: Vec<i32> = tables.iter().map(|t| t.frags).collect();
                if tables.len() == 2 && tables[0].title.is_some() && comparable(&frags) {
                    totals(ui, rtl, &tables);
                    ui.add_space(TABLE_GAP);
                }

                for table in &tables {
                    side_table(ui, catalog, rtl, table);
                    ui.add_space(TABLE_GAP);
                }
            });
    }
}

/// The tables of the standings in `segment` at `at`
fn score_tables(model: &Model, segment: u32, at: f32) -> Vec<SideTable> {
    let standing = standing_at(&model.score_updates, &model.team_score_updates, segment, at);
    let recorder = model.recorder;

    // Two players, as teams of one or without teams, are a duel when one
    // of them recorded it
    let mut keys = Vec::new();
    for side in &standing.sides {
        for player in &side.players {
            keys.push((player.slot, player.incarnation));
        }
    }
    let duel = keys.len() == 2 && recorder.is_some_and(|r| keys.contains(&r));

    // Shots, hits and chance matches up to the moment shown
    let mut aims: HashMap<(u8, u32), (u32, u32, u32)> = HashMap::new();
    for shot in &model.shots {
        if shot.segment != segment || shot.time > at {
            continue;
        }
        let aim = aims.entry((shot.slot, shot.incarnation)).or_default();
        aim.0 += 1;
        aim.1 += u32::from(shot.hit);
        aim.2 += u32::from(shot.chance);
    }

    // Headshot kills up to the moment shown
    let mut heads: HashMap<(u8, u32), u32> = HashMap::new();
    for event in &model.events {
        if event.segment != segment || event.time > at {
            continue;
        }
        if let EventKind::Kill {
            killer: Some(killer),
            headshot: Some(true),
            ..
        } = event.kind
        {
            *heads.entry(killer).or_default() += 1;
        }
    }

    let row = |player: &PlayerScore| {
        let key = (player.slot, player.incarnation);
        let own = Some(key) == recorder;
        let headshots = (duel || own).then(|| heads.get(&key).copied().unwrap_or(0));
        let accuracy = aims
            .get(&key)
            .filter(|_| own)
            .and_then(|&(shots, hits, chance)| accuracy(shots, hits, chance));
        PlayerRow {
            name: text::player_name(&player.name),
            recorder: own,
            frags: player.frags,
            deaths: player.deaths,
            headshots,
            accuracy,
        }
    };
    let table = |side: &StandingSide, title: bool| SideTable {
        title: title.then(|| text::player_name(&side.name)),
        colour: theme::team_colour(&side.name),
        frags: side.frags,
        players: side.players.iter().map(row).collect(),
    };

    // Teams and the two sides of a duel get a table each, and anything else
    // one ranked table
    if standing.teams || standing.sides.len() == 2 {
        return standing.sides.iter().map(|s| table(s, true)).collect();
    }
    if standing.sides.is_empty() {
        return Vec::new();
    }

    let mut players = Vec::new();
    for side in &standing.sides {
        players.extend(side.players.iter().map(row));
    }
    vec![SideTable {
        title: None,
        colour: theme::FOREGROUND,
        frags: 0,
        players,
    }]
}

/// The two sides' totals at either end over a strip sharing their frags
fn totals(ui: &mut Ui, rtl: bool, tables: &[SideTable]) {
    let (first, second) = (&tables[0], &tables[1]);
    let width = ui.available_width();
    let height = TOTAL_SIZE + theme::GAP_XS + STRIP_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // The first side leads in the interface's direction
    let (lead, trail) = if rtl {
        (Align2::RIGHT_TOP, Align2::LEFT_TOP)
    } else {
        (Align2::LEFT_TOP, Align2::RIGHT_TOP)
    };
    let (lead_x, trail_x) = if rtl {
        (rect.max.x, rect.min.x)
    } else {
        (rect.min.x, rect.max.x)
    };
    let font = FontId::monospace(TOTAL_SIZE);
    painter.text(
        Pos2::new(lead_x, rect.min.y),
        lead,
        first.frags.to_string(),
        font.clone(),
        first.colour,
    );
    painter.text(
        Pos2::new(trail_x, rect.min.y),
        trail,
        second.frags.to_string(),
        font,
        second.colour,
    );

    // Each side's share of the strip from its own end
    let share = first.frags as f32 / (first.frags + second.frags) as f32;
    let strip = Rangef::new(rect.max.y - STRIP_HEIGHT, rect.max.y);
    let split = if rtl {
        rect.max.x - width * share
    } else {
        rect.min.x + width * share
    };
    let lead_part = Rangef::new(lead_x.min(split), lead_x.max(split));
    let trail_part = Rangef::new(trail_x.min(split), trail_x.max(split));
    painter.rect_filled(Rect::from_x_y_ranges(lead_part, strip), 0., first.colour);
    painter.rect_filled(Rect::from_x_y_ranges(trail_part, strip), 0., second.colour);
}

/// A side's title, then its players under a heading of what each column
/// holds
fn side_table(ui: &mut Ui, catalog: &Catalog, rtl: bool, table: &SideTable) {
    if let Some(title) = &table.title {
        let line = format!("{title}  {}", table.frags);
        let look = Look::Color(table.colour);
        let line = text::styled_text(
            ui.ctx(),
            &line,
            text::starts_rtl(title),
            look,
            &theme::secondary(),
        );
        layout::column(ui, rtl, |ui| ui.label(line));
    }

    let columns = [
        (
            catalog.text("scores-accuracy"),
            Some(catalog.text("help-accuracy")),
        ),
        (
            catalog.text("scores-headshots"),
            Some(catalog.text("help-headshots")),
        ),
        (catalog.text("scores-deaths"), None),
        (catalog.text("scores-frags"), None),
    ];
    let header = columns.clone().map(|(title, _)| title);

    // Each column as wide as its heading, however long a language makes it
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let mut widths = NUMBER_COLUMNS;
    for (width, title) in widths.iter_mut().zip(&header) {
        let title =
            ui.fonts_mut(|f| f.layout_no_wrap(title.clone(), font.clone(), Color32::PLACEHOLDER));
        *width = width.max(title.size().x + COLUMN_GAP);
    }

    let cells = table_row(
        ui,
        rtl,
        &catalog.text("scores-player"),
        false,
        &header,
        &widths,
        theme::FAINT,
    );

    // The headings whose numbers need explaining explain them on hover
    for (cell, (_, help)) in cells.iter().zip(&columns) {
        if let Some(help) = help {
            let id = ui.scope_id().with(("score heading", cell.min.x as i32));
            let hover = ui.interact(*cell, id, Sense::hover());
            hover.on_hover_text(text::help(ui.ctx(), help, rtl));
        }
    }

    for player in &table.players {
        let accuracy = player.accuracy.map_or_else(
            || UNKNOWN.to_string(),
            |a| {
                let percent = FluentValue::from((a * 100.).round());
                catalog.text_with("library-percent", &[("percent", percent)])
            },
        );
        let headshots = player
            .headshots
            .map_or_else(|| UNKNOWN.to_string(), |h| h.to_string());
        let numbers = [
            accuracy,
            headshots,
            player.deaths.to_string(),
            player.frags.to_string(),
        ];
        let cells = table_row(
            ui,
            rtl,
            &player.name,
            player.recorder,
            &numbers,
            &widths,
            theme::FOREGROUND,
        );

        if player.recorder {
            let name = FluentValue::from(player.name.clone());
            let mine = catalog.text_with("lane-recorder", &[("name", name)]);
            let id = ui.scope_id().with(("recorder row", player.name.as_str()));
            let name_cell = row_name_cell(ui, rtl, &cells);
            ui.interact(name_cell, id, Sense::hover())
                .on_hover_text(mine);
        }
    }
}

/// The part of a row left of its numbers, or right of them in a
/// right-to-left interface
fn row_name_cell(ui: &Ui, rtl: bool, cells: &[Rect; 4]) -> Rect {
    let row = cells[0].y_range();
    let last = &cells[3];
    let across = if rtl {
        Rangef::new(last.max.x, ui.max_rect().max.x)
    } else {
        Rangef::new(ui.max_rect().min.x, last.min.x)
    };
    Rect::from_x_y_ranges(across, row)
}

/// One row of a table, the name cut short to leave the numbers their
/// columns `widths` wide, and the recorder marked beside their name.
/// Returns where each number was drawn, accuracy first
fn table_row(
    ui: &mut Ui,
    rtl: bool,
    name: &str,
    recorder: bool,
    numbers: &[String; 4],
    widths: &[f32; 4],
    colour: Color32,
) -> [Rect; 4] {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::hover());
    let painter = ui.painter();

    // Numbers from the trailing end, each right-aligned in its column, or
    // left-aligned from the left in a right-to-left interface
    let mut cells = [Rect::NOTHING; 4];
    let mut edge = if rtl { rect.min.x } else { rect.max.x };
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    for (k, number) in numbers.iter().enumerate() {
        let column = widths[k];
        let across = if rtl {
            Rangef::new(edge, edge + column)
        } else {
            Rangef::new(edge - column, edge)
        };
        cells[k] = Rect::from_x_y_ranges(across, rect.y_range());

        let (anchor, x) = if rtl {
            (Align2::LEFT_CENTER, across.min)
        } else {
            (Align2::RIGHT_CENTER, across.max)
        };
        let look = if number == UNKNOWN {
            theme::FAINT
        } else {
            colour
        };
        painter.text(
            Pos2::new(x, rect.center().y),
            anchor,
            number,
            font.clone(),
            look,
        );
        edge = if rtl { across.max } else { across.min };
    }

    // The name in what the numbers leave, with the recorder's mark
    let room = if rtl {
        rect.max.x - edge
    } else {
        edge - rect.min.x
    };
    let mark_room = if recorder {
        RECORDER_MARK * 2. + theme::GAP_XS
    } else {
        0.
    };
    let name = text::widget_text(
        ui.ctx(),
        name,
        text::starts_rtl(name),
        None,
        Look::Color(colour),
    );
    let galley = name.into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        (room - mark_room - theme::GAP_S).max(0.),
        egui::TextStyle::Body,
    );
    let name_x = if rtl {
        rect.max.x - galley.size().x
    } else {
        rect.min.x
    };
    let top = rect.center().y - galley.size().y / 2.;
    let after = if rtl {
        name_x - theme::GAP_XS - RECORDER_MARK
    } else {
        name_x + galley.size().x + theme::GAP_XS + RECORDER_MARK
    };
    painter.galley(Pos2::new(name_x, top), galley, colour);
    if recorder {
        painter.circle_filled(
            Pos2::new(after, rect.center().y),
            RECORDER_MARK,
            theme::ACCENT,
        );
    }

    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_two_nonnegative_sides_with_a_score_compare() {
        assert!(comparable(&[10, 5]));
        assert!(comparable(&[0, 3]));

        // Nothing scored, a negative score, or not two sides
        assert!(!comparable(&[0, 0]));
        assert!(!comparable(&[-2, 5]));
        assert!(!comparable(&[4]));
        assert!(!comparable(&[1, 2, 3]));
    }
}
