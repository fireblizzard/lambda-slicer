//! The Player filter. Typing finds demos where anyone used a name with that
//! text in it, and the library's players who used it are offered under the
//! field. Picking one finds the demos they played in under any name, as
//! their SteamID tells them apart, and players picked must all have played

use eframe::egui::{self, Key, Modifiers, Ui};
use fluent_bundle::FluentValue;
use lambda_slicer_core::library::cache::{Cache, PersonRow, fold_name};
use lambda_slicer_core::message::Coded;

use super::LibraryView;
use crate::i18n::Catalog;
use crate::layout;
use crate::text::{self, Look};
use crate::theme;
use crate::widgets::{self, Variant};

/// The most players offered under the field
const OFFERED: usize = 8;

/// The other names shown beside a player's own
const OTHER_NAMES: usize = 2;

/// A player picked to filter by, by SteamID, with the name they show under
#[derive(Clone, Debug, PartialEq)]
pub struct Chosen {
    pub steam_id: i64,
    pub name: String,
}

/// Whether `person` used a name with `typed` in it, both folded and without
/// colours. Nothing typed is in every name
fn used_name(person: &PersonRow, typed: &str) -> bool {
    for name in &person.names {
        if fold_name(name).contains(typed) {
            return true;
        }
    }
    false
}

impl LibraryView {
    /// The players picked, each a button that takes them off, then the name
    /// typed, with the players who used it offered under it while it has
    /// focus
    pub(super) fn player_field(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        let ctx = ui.ctx().clone();
        self.chosen_players(ui, catalog, rtl);

        // The arrow keys move through the players offered, before the field
        // takes them
        let id = ui.make_persistent_id("player field");
        let focused = ctx.memory(|m| m.has_focus(id));
        let clicking = ctx.input(|i| i.pointer.any_down() || i.pointer.any_released());
        let shown = focused || (self.offering && clicking);
        let offered = if shown { self.offered() } else { Vec::new() };
        if focused && !offered.is_empty() {
            let down = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));
            let up = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp));
            if down {
                self.highlighted += 1;
            }
            if up {
                self.highlighted = self.highlighted.saturating_sub(1);
            }
        }
        self.highlighted = self.highlighted.min(offered.len().saturating_sub(1));

        let hint = text::widget_text(&ctx, &catalog.text("filter-player"), rtl, None, Look::Plain);
        let field = egui::TextEdit::singleline(&mut self.filters.player)
            .id(id)
            .hint_text(hint)
            .desired_width(f32::INFINITY);
        let help = text::help(&ctx, &catalog.text("help-player-filter"), rtl);
        let response = ui.add(field).on_hover_text(help);
        if response.changed() {
            self.highlighted = 0;
        }

        // Enter picks the player lit, and a click the one clicked
        let entered = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        let mut picked = if entered && !offered.is_empty() {
            Some(self.highlighted)
        } else {
            None
        };
        if let Some(i) = self.offer_list(ui, &offered, catalog, rtl) {
            picked = Some(i);
        }
        self.offering = shown && !offered.is_empty();

        if let Some(person) = picked.and_then(|i| offered.get(i)) {
            self.filters.players_chosen.push(Chosen {
                steam_id: person.steam_id,
                name: person.names.first().cloned().unwrap_or_default(),
            });
            self.filters.player.clear();
            self.highlighted = 0;
            response.request_focus();
        }
    }

    /// The players picked, each a button that takes them off the filter
    fn chosen_players(&mut self, ui: &mut Ui, catalog: &Catalog, rtl: bool) {
        if self.filters.players_chosen.is_empty() {
            return;
        }

        let ctx = ui.ctx().clone();
        let mut removed = None;
        layout::wrapping_row(ui, rtl, |ui| {
            for (i, chosen) in self.filters.players_chosen.iter().enumerate() {
                let label = format!("{}  ×", text::player_name(&chosen.name));
                let label = text::widget_text(&ctx, &label, rtl, None, Look::Plain);
                let button = widgets::Button::new(None, label, Variant::Default, rtl);
                let tip = catalog.text("player-remove");
                if ui.add(button).on_hover_text(tip).clicked() {
                    removed = Some(i);
                }
            }
        });
        if let Some(i) = removed {
            self.filters.players_chosen.remove(i);
        }
    }

    /// The players to offer for the name typed, those in the most demos
    /// first, leaving out those already picked. The library's players are
    /// read once, and again after a scan stored more demos
    fn offered(&mut self) -> Vec<PersonRow> {
        if self.people.is_none() {
            let people = self.cache().map(Cache::people);
            self.people = match people {
                Some(Ok(people)) => Some(people),
                Some(Err(e)) => {
                    self.error = Some(e.message());
                    Some(Vec::new())
                }
                None => Some(Vec::new()),
            };
        }

        let typed = fold_name(self.filters.player.trim());
        let chosen = &self.filters.players_chosen;
        let mut out = Vec::new();
        for person in self.people.iter().flatten() {
            let picked = chosen.iter().any(|c| c.steam_id == person.steam_id);
            if !picked && used_name(person, &typed) {
                out.push(person.clone());
            }
            if out.len() == OFFERED {
                break;
            }
        }
        out
    }

    /// The players `offered`, each with their most used name, others they
    /// used and how many demos they played in, and which was clicked
    fn offer_list(
        &self,
        ui: &mut Ui,
        offered: &[PersonRow],
        catalog: &Catalog,
        rtl: bool,
    ) -> Option<usize> {
        if offered.is_empty() {
            return None;
        }

        let ctx = ui.ctx().clone();
        let mut clicked = None;
        let frame = egui::Frame::new()
            .fill(theme::SURFACE)
            .corner_radius(theme::RADIUS_CONTROL)
            .inner_margin(theme::GAP_XS as i8);
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (i, person) in offered.iter().enumerate() {
                let name = person.names.first().map_or("", String::as_str);
                let name =
                    text::widget_text(&ctx, &text::player_name(name), rtl, None, Look::Plain);
                let count = FluentValue::from(person.demos);
                let demos = catalog.text_with("player-demos", &[("count", count)]);

                // Every name they used, on hover
                let mut every = Vec::new();
                for name in &person.names {
                    every.push(text::player_name(name));
                }
                let every = text::help(&ctx, &every.join("\n"), rtl);

                layout::row(ui, rtl, |ui| {
                    let row = egui::Button::selectable(i == self.highlighted, name);
                    if ui.add(row).on_hover_text(every).clicked() {
                        clicked = Some(i);
                    }
                    layout::trailing(ui, rtl, |ui| {
                        let look = Look::Color(theme::MUTED);
                        ui.label(text::widget_text(&ctx, &demos, rtl, None, look));
                    });
                });

                // A couple of the other names, so a renamed player is known
                let others: Vec<String> = person
                    .names
                    .iter()
                    .skip(1)
                    .take(OTHER_NAMES)
                    .map(|n| text::player_name(n))
                    .collect();
                if !others.is_empty() {
                    let names = FluentValue::from(others.join(", "));
                    let also = catalog.text_with("player-also", &[("names", names)]);
                    let look = Look::Color(theme::MUTED);
                    let also = text::widget_text(&ctx, &also, rtl, None, look);

                    // In line with the name above it, inside its button
                    let indent = ui.spacing().button_padding.x;
                    layout::row(ui, rtl, |ui| {
                        ui.add_space(indent);
                        ui.add(egui::Label::new(also).truncate());
                    });
                }
            }
        });
        clicked
    }
}
