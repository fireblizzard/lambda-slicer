//! Rows and columns that follow the interface's direction
//!
//! `Ui::with_layout` claims all the remaining space, which pushes later
//! widgets away. These size to their content like `Ui::horizontal`, and run
//! right to left in a right-to-left interface

use eframe::egui::{self, Align, Layout, Ui};

fn row_layout(rtl: bool) -> Layout {
    if rtl {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    }
}

/// A row one widget high, as wide as the space
pub fn row<R>(ui: &mut Ui, rtl: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, row_layout(rtl), add).inner
}

/// A row that wraps onto more lines when its widgets do not fit
pub fn wrapping_row<R>(ui: &mut Ui, rtl: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(size, row_layout(rtl).with_main_wrap(true), add)
        .inner
}

/// A column aligned to the interface's leading side
pub fn column<R>(ui: &mut Ui, rtl: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let layout = Layout::top_down(if rtl { Align::Max } else { Align::Min });
    let size = egui::vec2(ui.available_width(), 0.);
    ui.allocate_ui_with_layout(size, layout, add).inner
}
