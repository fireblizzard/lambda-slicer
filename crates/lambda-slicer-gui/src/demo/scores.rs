//! The scoreboard over the events, the final standings until the playhead
//! moves and then the standings where it is, or at the selected event
//! without a viewer. A table per team or per player of a duel and one
//! ranked table for anything else, and the spectators under them
//!
//! Other players' kills and shots often happen outside what the recording
//! client receives, so their headshots and accuracy are incomplete. Only
//! the recording player's are shown, and in a duel the opponent's headshots
//! too, since every kill there is of the recording player, whose death the
//! recording always shows. Anything not shown is a dash, never a zero

use std::collections::{BTreeMap, HashMap};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rangef, Rect, Sense, Ui, Vec2};
use fluent_bundle::FluentValue;
use lambda_slicer_core::analysis::accuracy::accuracy;
use lambda_slicer_core::analysis::context::{PlayerScore, StandingSide, standing_at};
use lambda_slicer_core::analysis::match_events::MatchChange;
use lambda_slicer_core::timeline::time::DemoTime;

use super::{DemoAction, DemoView};
use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::timeline::{EventKind, LaneKey, Model};
use crate::widgets::{self, Variant};

/// How many number columns a table has, accuracy, headshots, kills per
/// death, deaths and frags from the trailing end
const COLUMNS: usize = 5;

/// A table row's height, the least width of each number column from the
/// trailing end, accuracy first, and the gap a column keeps before the next
const ROW_HEIGHT: f32 = 24.;
const NUMBER_COLUMNS: [f32; COLUMNS] = [52., 40., 52., 48., 48.];
const COLUMN_GAP: f32 = theme::GAP_S;

/// The two sides' totals over their comparison strip, and the strip
const TOTAL_SIZE: f32 = 26.;
const STRIP_HEIGHT: f32 = 4.;

/// The gap between tables, and the recorder's mark beside their name
const TABLE_GAP: f32 = theme::GAP_M;
const RECORDER_MARK: f32 = 3.;

/// What a value nobody can know shows as
const UNKNOWN: &str = "—";

/// Decimals a player's kills per death shows
const KD_DECIMALS: usize = 2;

/// One player's line of a table
struct PlayerRow {
    name: String,
    slot: u8,
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

/// A player's frags for each death, or their frags alone when they never
/// died, so a player with no deaths still has a number
fn kills_per_death(frags: i32, deaths: i32) -> f64 {
    f64::from(frags) / f64::from(deaths.max(1))
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
    /// totals when they compare, the tables and the spectators. Returns how
    /// much of them is scrolled out of view
    pub(super) fn scores_tab(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) -> f32 {
        let Some(model) = self.model.clone() else {
            return 0.;
        };

        // With a viewer the scores follow the playhead once it moves, and
        // without one they are at the selected event
        let live = self.live_moment(&model);
        let selected = match self.viewer {
            Some(_) => None,
            None => self.selected.and_then(|i| model.events.get(i)),
        };

        // The final scores are of the map played longest, since a demo may
        // end with a few moments of the next one
        let longest = model.groups.longest_segment();
        let last_kill = model.events.iter().rev().find(|e| {
            let kill = matches!(e.kind, EventKind::Kill { .. });
            kill && longest.is_none_or(|s| e.segment == s)
        });

        // The selected event's moment, or the last kill for the final scores,
        // since players often leave once a game is over
        let (segment, at) = match (live, selected, last_kill) {
            (Some(moment), _, _) => moment,
            (None, Some(e), _) | (None, None, Some(e)) => (e.segment, e.time),
            (None, None, None) => {
                let last = model.score_updates.last().map_or(0, |u| u.segment);
                (longest.unwrap_or(last), f32::INFINITY)
            }
        };
        let not_final = live.is_some() || selected.is_some();
        let heading = if not_final {
            let time = FluentValue::from(clock(at));
            catalog.text_with("scores-at", &[("time", time)])
        } else {
            let time = FluentValue::from(last_kill.map(|e| clock(e.time)).unwrap_or_default());
            catalog.text_with("scores-final", &[("time", time)])
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
            if not_final {
                let label = catalog.text("scores-show-final");
                let label = text::widget_text(ui.ctx(), &label, rtl, None, Look::Plain);
                let button = widgets::Button::new(None, label, Variant::Ghost, rtl);
                if ui.add(button).clicked() {
                    self.actions.push(DemoAction::ShowFinal);
                }
            }
        });
        ui.add_space(theme::GAP_S);

        let watching = spectators(&model, segment, at);
        let tables = score_tables(&model, segment, at, &watching);
        if tables.is_empty() && watching.is_empty() {
            let none = catalog.text("scores-none");
            ui.label(text::widget_text(
                ui.ctx(),
                &none,
                rtl,
                None,
                Look::Color(theme::MUTED),
            ));
            return 0.;
        }

        // As tall as the scores, up to the room there is
        // Names are never cut short, so tables too wide for the panel scroll
        // sideways to their last numbers
        let names = names_width(ui, catalog, &tables);
        let scrolled = egui::ScrollArea::both()
            .id_salt(("scores", self.generation))
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let frags: Vec<i32> = tables.iter().map(|t| t.frags).collect();
                if tables.len() == 2 && tables[0].title.is_some() && comparable(&frags) {
                    totals(ui, rtl, &tables);
                    ui.add_space(TABLE_GAP);
                }

                // With a viewer that has the game's files a row clicked
                // follows its player, the recorder's going back to their
                // own view
                let viewer = self.viewer.as_ref().filter(|v| v.has_assets());
                let following = viewer.map(|v| v.followed_slot());
                let mut chosen = None;
                for table in &tables {
                    let clicked = side_table(ui, catalog, rtl, table, (names, following));
                    chosen = chosen.or(clicked);
                    ui.add_space(TABLE_GAP);
                }
                spectator_list(ui, catalog, rtl, &watching, model.recorder);

                if let Some(slot) = chosen
                    && let Some(viewer) = &mut self.viewer
                {
                    viewer.follow_slot(slot);
                }
            });
        (scrolled.content_size.y - scrolled.inner_rect.height()).max(0.)
    }

    /// The playhead's session and time once the scoreboard follows it,
    /// from the first time the playhead plays or moves after the demo
    /// opened or the final scores were asked for. None without a viewer
    fn live_moment(&mut self, model: &Model) -> Option<(u32, f32)> {
        let viewer = self.viewer.as_ref()?;
        let (time, playing) = (viewer.time(), viewer.is_playing());
        if !self.scores_live {
            let moved = self.scores_still.is_some_and(|still| still != time);
            if !playing && !moved {
                self.scores_still = Some(time);
                return None;
            }
            self.scores_live = true;
        }

        let segment = DemoTime::from_f32(time).and_then(|t| model.groups.segment_at(t))?;
        Some((segment, time))
    }
}

/// The players spectating in `segment` at `at`, by slot and incarnation,
/// with their names, as their last switch before then left them. Those
/// who had left the server by then are left out
fn spectators(model: &Model, segment: u32, at: f32) -> BTreeMap<(u8, u32), String> {
    let mut watching = BTreeMap::new();
    for event in &model.events {
        if event.segment != segment || event.time > at {
            continue;
        }
        let spectating = match event.kind {
            EventKind::Match(MatchChange::Spectating) => true,
            EventKind::Match(MatchChange::Playing) => false,
            _ => continue,
        };
        let Some(LaneKey::Player {
            slot, incarnation, ..
        }) = model.lanes.get(event.lane).map(|l| l.key)
        else {
            continue;
        };

        let key = (slot, incarnation);
        if spectating {
            let name = event.actor.as_deref().unwrap_or_default();
            watching.insert(key, text::player_name(name));
        } else {
            watching.remove(&key);
        }
    }

    for update in &model.score_updates {
        if update.segment == segment && update.time <= at && update.left {
            watching.remove(&(update.player.slot, update.player.incarnation));
        }
    }
    watching
}

/// The spectators under the tables, the recorder marked as in the tables,
/// nothing when nobody spectates
fn spectator_list(
    ui: &mut Ui,
    catalog: &Catalog,
    rtl: bool,
    watching: &BTreeMap<(u8, u32), String>,
    recorder: Option<(u8, u32)>,
) {
    if watching.is_empty() {
        return;
    }

    let title = catalog.text("scores-spectators");
    let title = text::styled_text(
        ui.ctx(),
        &title,
        rtl,
        Look::Color(theme::MUTED),
        &theme::secondary(),
    );
    layout::column(ui, rtl, |ui| ui.label(title));
    for (key, name) in watching {
        let line = if Some(*key) == recorder {
            let name = FluentValue::from(name.clone());
            catalog.text_with("lane-recorder", &[("name", name)])
        } else {
            name.clone()
        };
        let line = text::widget_text(ui.ctx(), &line, text::starts_rtl(name), None, Look::Plain);
        layout::column(ui, rtl, |ui| ui.label(line));
    }
}

/// The tables of the standings in `segment` at `at`, the players among
/// `watching` left out, as they show among the spectators
fn score_tables(
    model: &Model,
    segment: u32,
    at: f32,
    watching: &BTreeMap<(u8, u32), String>,
) -> Vec<SideTable> {
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
            slot: player.slot,
            recorder: own,
            frags: player.frags,
            deaths: player.deaths,
            headshots,
            accuracy,
        }
    };
    let playing = |p: &&PlayerScore| !watching.contains_key(&(p.slot, p.incarnation));
    let table = |side: &StandingSide, title: bool| SideTable {
        title: title.then(|| text::player_name(&side.name)),
        colour: theme::team_colour(&side.name),
        frags: side.frags,
        players: side.players.iter().filter(playing).map(row).collect(),
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
        players.extend(side.players.iter().filter(playing).map(row));
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
/// The table of one side, its names' column `names` wide. With `following`,
/// the slot of the player the viewer follows or none for the recorder,
/// each row is a button that follows its player, the one followed marked.
/// Returns the row clicked, by its player's slot or none for the recorder
fn side_table(
    ui: &mut Ui,
    catalog: &Catalog,
    rtl: bool,
    table: &SideTable,
    (names, following): (f32, Option<Option<u8>>),
) -> Option<Option<u8>> {
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
        (catalog.text("scores-kd"), Some(catalog.text("help-kd"))),
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

    // The names' column `names` wide, and the numbers' after it, or the
    // whole panel when it is wider
    let numbers_width: f32 = widths.iter().sum();
    let row_width = (names + numbers_width).max(ui.available_width());

    let (_, cells) = table_row(
        ui,
        rtl,
        &catalog.text("scores-player"),
        false,
        &header,
        (&widths, row_width),
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

    let mut clicked = None;
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
        let kd = kills_per_death(player.frags, player.deaths);
        let numbers = [
            accuracy,
            headshots,
            catalog.number(kd, KD_DECIMALS),
            player.deaths.to_string(),
            player.frags.to_string(),
        ];
        // The row followed and the row under the pointer are marked under
        // their text, so it is painted first
        let own = if player.recorder {
            None
        } else {
            Some(player.slot)
        };
        let next = Rect::from_min_size(ui.cursor().min, Vec2::new(row_width, ROW_HEIGHT));
        let fill = match following {
            Some(followed) if followed == own => Some(theme::ACCENT_SOFT),
            Some(_) if ui.rect_contains_pointer(next) => Some(theme::RAISED),
            _ => None,
        };
        if let Some(fill) = fill {
            ui.painter().rect_filled(next, theme::RADIUS_SMALL, fill);
        }

        let (row, cells) = table_row(
            ui,
            rtl,
            &player.name,
            player.recorder,
            &numbers,
            (&widths, row_width),
            theme::FOREGROUND,
        );

        // The recorder's name says who they are, and a row of a viewer
        // follows its player
        let id = ui.scope_id().with(("score row", player.slot));
        let sense = if following.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let mut response = ui.interact(row, id, sense);
        if player.recorder {
            let name = FluentValue::from(player.name.clone());
            let mine = catalog.text_with("lane-recorder", &[("name", name)]);
            let name_cell = row_name_cell(row, rtl, &cells);
            if ui.rect_contains_pointer(name_cell) {
                response = response.on_hover_text(mine);
            }
        }
        if following.is_some() {
            response = response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(catalog.text("scores-follow"));
        }
        if response.clicked() {
            clicked = Some(own);
        }
    }
    clicked
}

/// How wide the names' column must be for every player's whole name in
/// `tables`, the recorder's with their mark, and for its heading
fn names_width(ui: &Ui, catalog: &Catalog, tables: &[SideTable]) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let width = |name: String| {
        let galley = ui.fonts_mut(|f| f.layout_no_wrap(name, font.clone(), Color32::PLACEHOLDER));
        galley.size().x
    };

    let mut widest = width(catalog.text("scores-player"));
    for table in tables {
        for player in &table.players {
            let mark = if player.recorder {
                RECORDER_MARK * 2. + theme::GAP_XS
            } else {
                0.
            };
            widest = widest.max(width(player.name.clone()) + mark);
        }
    }
    widest + theme::GAP_S
}

/// The part of the `row` left of its numbers, or right of them in a
/// right-to-left interface
fn row_name_cell(row: Rect, rtl: bool, cells: &[Rect; COLUMNS]) -> Rect {
    let last = &cells[COLUMNS - 1];
    let across = if rtl {
        Rangef::new(last.max.x, row.max.x)
    } else {
        Rangef::new(row.min.x, last.min.x)
    };
    Rect::from_x_y_ranges(across, row.y_range())
}

/// One row of a table `width` wide, the numbers in their columns `widths`
/// wide and the name in what they leave, cut short only when that is too
/// little, with the recorder marked beside their name. Returns the row and
/// where each number was drawn, accuracy first
fn table_row(
    ui: &mut Ui,
    rtl: bool,
    name: &str,
    recorder: bool,
    numbers: &[String; COLUMNS],
    (widths, width): (&[f32; COLUMNS], f32),
    colour: Color32,
) -> (Rect, [Rect; COLUMNS]) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::hover());
    let painter = ui.painter();

    // Numbers from the trailing end, each right-aligned in its column, or
    // left-aligned from the left in a right-to-left interface
    let mut cells = [Rect::NOTHING; COLUMNS];
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

    (rect, cells)
}

#[cfg(test)]
mod tests {
    use lambda_slicer_core::analysis::context::ScoreUpdate;
    use lambda_slicer_core::state::PlayerIdentity;

    use super::*;
    use crate::timeline::{Event, Lane};

    /// A lane of the player in `slot`
    fn lane(slot: u8) -> Lane {
        Lane {
            key: LaneKey::Player {
                segment: 0,
                slot,
                incarnation: 0,
            },
            names: Vec::new(),
            recorder: false,
            marks: Vec::new(),
            spans: Vec::new(),
        }
    }

    /// The player in `lane` starting to spectate, or to play again, at `time`
    fn switch(lane: usize, name: &str, time: f32, spectating: bool) -> Event {
        let change = if spectating {
            MatchChange::Spectating
        } else {
            MatchChange::Playing
        };
        Event {
            segment: 0,
            time,
            lane,
            actor: Some(name.into()),
            weapon: None,
            kind: EventKind::Match(change),
        }
    }

    #[test]
    fn spectators_are_those_whose_last_switch_was_to_spectate_and_who_stayed() {
        let left = ScoreUpdate {
            segment: 0,
            time: 30.,
            player: PlayerIdentity {
                segment: 0,
                slot: 3,
                user_id: 3,
                incarnation: 0,
                name: "gone".into(),
            },
            team: String::new(),
            frags: 0,
            deaths: 0,
            left: true,
        };
        let model = Model {
            lanes: vec![lane(1), lane(2), lane(3)],
            events: vec![
                switch(0, "watcher", 5., true),
                switch(1, "back", 5., true),
                switch(1, "back", 20., false),
                switch(2, "gone", 10., true),
            ],
            score_updates: vec![left],
            ..Model::default()
        };

        // Before the second player plays again and the third leaves, all
        // three watch
        let names = |at| -> Vec<String> { spectators(&model, 0, at).into_values().collect() };
        assert_eq!(names(15.), ["watcher", "back", "gone"]);
        assert_eq!(names(40.), ["watcher"]);
        assert!(names(1.).is_empty());
    }

    #[test]
    fn kills_per_death_divide_by_one_for_a_player_who_never_died() {
        assert_eq!(kills_per_death(30, 20), 1.5);
        assert_eq!(kills_per_death(7, 0), 7.);
        assert_eq!(kills_per_death(-2, 4), -0.5);
    }

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
