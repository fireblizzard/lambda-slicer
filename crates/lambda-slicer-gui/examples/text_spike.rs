//! M6 spike, how egui renders each interface language's text
//!
//! Usage is `text_spike OUT.png`. Shows sample lines as plain egui labels
//! and as separately shaped pieces placed in bidi order, saves a screenshot
//! and exits
//!
//! egui 0.37 shapes each font run with harfrust, so single Arabic words
//! join correctly, but it places font runs left to right in logical order
//! and merges runs across layout sections. Several Arabic words, or Arabic
//! mixed with Latin, therefore come out in the wrong order unless each
//! piece is laid out on its own and placed by the bidi algorithm

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, Color32, FontId};
use unicode_bidi::{BidiInfo, Level};

const LINES: &[(&str, &str)] = &[
    ("ar", "تم العثور على 3 عمليات قتل بسلاح الغاوس"),
    (
        "ar name",
        "قتل \u{2068}-<[ATK]>-naz\u{2069} اللاعب \u{2068}Worry\u{2069} عند 01:36.250",
    ),
    ("ar mixed", "قتل naz اللاعب Worry عند 01:36.250"),
    ("ar digits", "قتل 125 لاعبا في 1536 ثانية"),
    ("ar punct", "هل قتلت (5) لاعبين؟ نعم!"),
    (
        "name in en",
        "Player \u{2068}محمد الغاوس\u{2069} killed naz",
    ),
    ("zh", "找到 3 次高斯击杀，时长 00:03:12，玩家：李小龍"),
    ("cjk names", "李小龍, 김민수, 山田太郎 vs naz"),
    ("ru", "Найдено 5 убийств из гаусса"),
    ("uk", "Знайдено 22 вбивства з гауса, ґ є ї"),
    ("tr", "İstanbul ışık Şş Ğğ, 3 öldürme"),
    ("pl", "Zażółć gęślą jaźń, 5 zabójstw"),
    ("lt", "ąčęėįšųūž Ąžuolas, 21 nužudymas"),
    ("de", "Straße Größe Übung, äöüß"),
    ("pt fr it fi", "ação œuvre élève ça perché città hyvää yötä"),
];

/// Bidi isolates and marks, which only steer ordering
fn is_control(c: char) -> bool {
    matches!(c, '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}')
}

/// The mirrored form of a bracket shown right to left
fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        other => other,
    }
}

/// Split into alternating whitespace and non-whitespace pieces
fn pieces(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut space = None;
    for (i, c) in s.char_indices() {
        let w = c.is_whitespace();
        if space.is_some_and(|s| s != w) {
            out.push(&s[start..i]);
            start = i;
        }
        space = Some(w);
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

/// Pieces of one line in visual order. Level runs come in visual order,
/// and right-to-left runs are split at spaces with their words reversed
/// and brackets mirrored, so egui only ever shapes a single word of one
/// direction
fn visual_pieces(text: &str, rtl_base: bool) -> Vec<String> {
    let info = BidiInfo::new(text, rtl_base.then(Level::rtl));
    let mut out = Vec::new();
    for para in &info.paragraphs {
        let (levels, runs) = info.visual_runs(para, para.range.clone());
        for run in runs {
            let rtl = levels[run.start].is_rtl();
            let piece: String = text[run]
                .chars()
                .filter(|c| !is_control(*c))
                .map(|c| if rtl { mirror(c) } else { c })
                .collect();
            if rtl {
                out.extend(pieces(&piece).into_iter().rev().map(str::to_string));
            } else {
                out.push(piece);
            }
        }
    }
    out
}

/// A label drawn from separately laid out pieces, placed left to right
fn bidi_label(ui: &mut egui::Ui, text: &str, rtl_base: bool, size: f32) {
    let galleys: Vec<_> = visual_pieces(text, rtl_base)
        .into_iter()
        .map(|p| {
            ui.painter()
                .layout_no_wrap(p, FontId::proportional(size), Color32::WHITE)
        })
        .collect();
    let width: f32 = galleys.iter().map(|g| g.size().x).sum();
    let height = galleys.iter().map(|g| g.size().y).fold(0., f32::max);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let mut x = rect.min.x;
    for g in galleys {
        let w = g.size().x;
        ui.painter()
            .galley(egui::pos2(x, rect.min.y), g, Color32::WHITE);
        x += w;
    }
}

struct Spike {
    out: PathBuf,
    frame: u32,
}

impl eframe::App for Spike {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let size = 20.;
        egui::Grid::new("lines")
            .num_columns(3)
            .spacing([24., 10.])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("case");
                ui.strong("plain egui");
                ui.strong("pieces in bidi order");
                ui.end_row();
                for (name, text) in LINES {
                    ui.label(*name);
                    ui.label(egui::RichText::new(*text).size(size));
                    bidi_label(ui, text, name.starts_with("ar"), size);
                    ui.end_row();
                }
            });

        self.frame += 1;
        if self.frame == 10 {
            let out = self.out.clone();
            let ctx = ui.ctx().clone();
            ui.ctx()
                .request_screenshot(move |image: Arc<egui::ColorImage>| {
                    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                    image::save_buffer(
                        &out,
                        &bytes,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                });
        }
        ui.ctx().request_repaint();
    }
}

fn main() -> eframe::Result {
    let out = PathBuf::from(std::env::args().nth(1).expect("usage: text_spike OUT.png"));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1150., 600.]),
        ..Default::default()
    };
    eframe::run_native(
        "text spike",
        options,
        Box::new(|_cc| Ok(Box::new(Spike { out, frame: 0 }))),
    )
}
