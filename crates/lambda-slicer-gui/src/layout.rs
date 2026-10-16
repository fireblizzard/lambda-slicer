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

/// A group kept on one line inside a wrapping row, such as a checkbox with
/// its fields. The group moves to the next line whole when it does not fit
/// what is left of this one. Its width is only known once laid out, so the
/// last frame's width is remembered
pub fn unbroken<R>(ui: &mut Ui, id_salt: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    let id = ui.make_persistent_id(id_salt);
    let width: f32 = ui.ctx().data(|d| d.get_temp(id)).unwrap_or(0.);
    let left = ui.available_size_before_wrap().x;
    let at_line_start = left >= ui.max_rect().width() - 1.;
    if width > left && !at_line_start {
        ui.end_row();
    }
    let r = ui.horizontal(add);
    ui.ctx()
        .data_mut(|d| d.insert_temp(id, r.response.rect.width()));
    r.inner
}
