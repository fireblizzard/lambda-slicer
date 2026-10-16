//! Right-to-left text for egui ([gui-text](../../../docs/gui-text.md))
//!
//! egui 0.37 shapes words with harfrust but places text in logical order,
//! so a line holding right-to-left text is ordered here. The Unicode bidi
//! algorithm gives the level runs in visual order, right-to-left runs are
//! split at spaces with their words reversed and brackets mirrored, and each
//! piece is laid out on its own so egui never reorders across pieces. Lines
//! are broken in logical order first and ordered one at a time

use eframe::egui::{
    self, Color32, Context, FontId, Galley, Pos2, Rect, RichText, Ui, Vec2, WidgetText,
};
use eframe::epaint::text::LayoutJob;
use std::sync::Arc;
use unicode_bidi::{BidiClass, BidiInfo, Level, bidi_class};

/// Whether `text` holds any right-to-left letter
pub fn has_rtl(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(bidi_class(c), BidiClass::R | BidiClass::AL))
}

/// A player name without the `^0` to `^9` colour codes Adrenaline Gamer
/// draws in colour, for display. The raw name stays in tooltips and data
pub fn player_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            continue;
        }
        out.push(c);
    }
    if out.trim().is_empty() {
        name.to_string()
    } else {
        out
    }
}

/// Whether `text` reads right to left by its first letter with a direction,
/// for names and paths that keep their own direction in any interface
pub fn starts_rtl(text: &str) -> bool {
    text.chars()
        .map(bidi_class)
        .find(|c| matches!(c, BidiClass::L | BidiClass::R | BidiClass::AL))
        .is_some_and(|c| c != BidiClass::L)
}

/// Isolates and marks, which only steer ordering
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
fn words(s: &str) -> Vec<&str> {
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

/// The pieces of `text[range]`, one line of `info`, in visual order
fn order_line(info: &BidiInfo, text: &str, range: std::ops::Range<usize>) -> Vec<String> {
    let mut out = Vec::new();
    let Some(para) = info
        .paragraphs
        .iter()
        .find(|p| p.range.start <= range.start && range.end <= p.range.end)
    else {
        return out;
    };
    let (levels, runs) = info.visual_runs(para, range);
    for run in runs {
        let rtl = levels[run.start].is_rtl();
        let piece: String = text[run]
            .chars()
            .filter(|c| !is_control(*c) && *c != '\n')
            .map(|c| if rtl { mirror(c) } else { c })
            .collect();
        if piece.is_empty() {
            continue;
        }
        if rtl {
            out.extend(rtl_pieces(&piece).into_iter().rev().map(str::to_string));
        } else {
            out.push(piece);
        }
    }
    out
}

/// Whether `c` belongs inside a right-to-left word, a letter or a mark or
/// joiner attached to one
fn joins_rtl_word(c: char) -> bool {
    matches!(
        bidi_class(c),
        BidiClass::R | BidiClass::AL | BidiClass::NSM | BidiClass::BN
    )
}

/// A right-to-left run in logical order, split into words of right-to-left
/// letters and single other characters. Punctuation such as an ellipsis
/// comes from another font than the letters, so egui would place it as a
/// run of its own in logical order unless it is a piece of its own
fn rtl_pieces(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut word_start = None;
    for (i, c) in s.char_indices() {
        if joins_rtl_word(c) {
            word_start.get_or_insert(i);
            continue;
        }
        if let Some(start) = word_start.take() {
            out.push(&s[start..i]);
        }
        out.push(&s[i..i + c.len_utf8()]);
    }
    if let Some(start) = word_start {
        out.push(&s[start..]);
    }
    out
}

/// One line of `text` as pieces in visual order
pub fn visual_pieces(text: &str, rtl_base: bool) -> Vec<String> {
    let info = BidiInfo::new(
        text,
        Some(if rtl_base { Level::rtl() } else { Level::ltr() }),
    );
    order_line(&info, text, 0..text.len())
}

/// Lines of `text` no wider than `width` where possible, each as pieces in
/// visual order. `measure` gives a piece's width. Lines break at spaces in
/// logical order, and a word wider than `width` gets a line of its own
pub fn wrapped_lines(
    text: &str,
    rtl_base: bool,
    width: f32,
    measure: impl Fn(&str) -> f32,
) -> Vec<Vec<String>> {
    let info = BidiInfo::new(
        text,
        Some(if rtl_base { Level::rtl() } else { Level::ltr() }),
    );
    let mut lines = Vec::new();
    for para in &info.paragraphs {
        let ptext = &text[para.range.clone()];
        let mut start = para.range.start;
        let mut offset = start;
        // End of the line's last word, so a line never ends or starts with
        // the space it broke at
        let mut word_end = None;
        let mut line_width = 0.;
        for w in words(ptext) {
            let ww = measure(&w.chars().filter(|c| !is_control(*c)).collect::<String>());
            if w.chars().all(char::is_whitespace) {
                offset += w.len();
                match word_end {
                    Some(_) => line_width += ww,
                    None => start = offset,
                }
                continue;
            }
            if let Some(end) = word_end
                && line_width + ww > width
            {
                lines.push(order_line(&info, text, start..end));
                start = offset;
                line_width = 0.;
            }
            line_width += ww;
            offset += w.len();
            word_end = Some(offset);
        }
        if let Some(end) = word_end {
            lines.push(order_line(&info, text, start..end));
        }
    }
    lines
}

/// One galley holding `lines` of pieces, each piece shaped on its own and
/// placed in visual order, lines aligned right when `align_right`. With the
/// placeholder colour a widget paints the text in its own state's colour
fn merge(
    ctx: &Context,
    text: &str,
    lines: Vec<Vec<String>>,
    font: &FontId,
    color: Color32,
    align_right: bool,
) -> Arc<Galley> {
    let layout = |s: String| ctx.fonts_mut(|f| f.layout_no_wrap(s, font.clone(), color));
    let row_height = ctx.fonts_mut(|f| f.row_height(font));
    let laid: Vec<(Vec<Arc<Galley>>, f32)> = lines
        .into_iter()
        .map(|line| {
            let galleys: Vec<_> = line.into_iter().map(layout).collect();
            let w = galleys.iter().map(|g| g.size().x).sum();
            (galleys, w)
        })
        .collect();
    let width = laid.iter().map(|(_, w)| *w).fold(0., f32::max);

    let mut merged = (*layout(String::new())).clone();
    merged.job = Arc::new(LayoutJob::simple_singleline(
        text.to_string(),
        font.clone(),
        color,
    ));
    merged.rows.clear();
    merged.rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 0.));
    merged.mesh_bounds = Rect::NOTHING;
    merged.num_vertices = 0;
    merged.num_indices = 0;

    let mut y = 0.;
    for (galleys, w) in laid {
        let mut x = if align_right { width - w } else { 0. };
        let mut h: f32 = row_height;
        for g in galleys {
            for placed in &g.rows {
                let mut placed = placed.clone();
                let offset = Vec2::new(x, y);
                placed.pos += offset;
                merged.mesh_bounds |= placed.visuals.mesh_bounds.translate(placed.pos.to_vec2());
                merged.rect |= Rect::from_min_size(placed.pos, placed.size);
                merged.rows.push(placed);
            }
            merged.num_vertices += g.num_vertices;
            merged.num_indices += g.num_indices;
            h = h.max(g.size().y);
            x += g.size().x;
        }
        y += h;
    }
    merged.rect.max.y = merged.rect.max.y.max(y);

    Arc::new(merged)
}

/// How a piece of widget text looks
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Look {
    /// The widget's own colour
    #[default]
    Plain,
    /// The theme's strong text colour, for headings
    Strong,
}

/// Text for any egui widget. Text with right-to-left letters, or any text in
/// a right-to-left interface, becomes one galley of pieces in visual order.
/// Other text goes to egui unchanged. `wrap` wraps to that width
pub fn widget_text(
    ctx: &Context,
    text: &str,
    rtl_base: bool,
    wrap: Option<f32>,
    look: Look,
) -> WidgetText {
    if !rtl_base && !has_rtl(text) {
        return match look {
            Look::Plain => WidgetText::from(text),
            Look::Strong => WidgetText::from(RichText::new(text).strong()),
        };
    }
    let style = ctx.global_style();
    let font = egui::TextStyle::Body.resolve(&style);
    let color = match look {
        Look::Plain => Color32::PLACEHOLDER,
        Look::Strong => style.visuals.strong_text_color(),
    };
    let lines = match wrap {
        Some(width) => {
            let measure = |s: &str| {
                ctx.fonts_mut(|f| f.layout_no_wrap(s.to_string(), font.clone(), color))
                    .size()
                    .x
            };
            wrapped_lines(text, rtl_base, width.max(1.), measure)
        }
        None => vec![visual_pieces(text, rtl_base)],
    };
    WidgetText::Galley(merge(ctx, text, lines, &font, color, rtl_base))
}

/// A label wrapping to the available width, ordered and aligned for the
/// interface's direction
pub fn label(ui: &mut Ui, text: &str, rtl_base: bool) -> egui::Response {
    let width = ui.available_width();
    let t = widget_text(ui.ctx(), text, rtl_base, Some(width), Look::Plain);
    ui.add(egui::Label::new(t).selectable(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str, rtl: bool) -> Vec<String> {
        visual_pieces(text, rtl)
    }

    #[test]
    fn arabic_words_come_right_to_left() {
        assert_eq!(one("قتل اللاعب", true), ["اللاعب", " ", "قتل"]);
    }

    #[test]
    fn latin_names_and_times_keep_their_order_inside_arabic() {
        let text = "قتل \u{2068}-<[ATK]>-naz\u{2069} عند 01:36.250";
        assert_eq!(
            one(text, true),
            ["01:36.250", " ", "عند", " ", "-<[ATK]>-naz", " ", "قتل"]
        );
    }

    #[test]
    fn numbers_stay_left_to_right() {
        assert_eq!(one("قتل 125 لاعبا", true), ["لاعبا", " ", "125", " ", "قتل"]);
    }

    #[test]
    fn brackets_mirror_in_right_to_left_runs() {
        // The opening bracket sits on the right and faces left, which the
        // mirrored glyph does, so visual order reads ( 5 ) left to right
        assert_eq!(
            one("هل (5) نعم", true),
            ["نعم", " ", "(", "5", ")", " ", "هل"]
        );
    }

    #[test]
    fn arabic_names_inside_english_read_right_to_left() {
        assert_eq!(
            one("Player \u{2068}محمد الغاوس\u{2069} won", false),
            ["Player ", "الغاوس", " ", "محمد", " won"]
        );
    }

    #[test]
    fn punctuation_inside_arabic_takes_its_place() {
        assert_eq!(one("فتح تسجيل…", true), ["…", "تسجيل", " ", "فتح"]);
        // The Arabic question mark is an Arabic letter to the bidi algorithm
        // and shapes with its word
        assert_eq!(one("لاعبين؟ نعم!", true), ["!", "نعم", " ", "لاعبين؟"]);
        // Marks stay on their letter
        assert_eq!(one("تسجيلًا", true), ["تسجيلًا"]);
    }

    #[test]
    fn plain_text_is_one_piece() {
        assert_eq!(one("Найдено 5 убийств", false), ["Найдено 5 убийств"]);
        assert!(!has_rtl("İstanbul 李小龍"));
        assert!(has_rtl("Player محمد"));
    }

    #[test]
    fn lines_break_in_logical_order_then_reorder() {
        // Four letters per unit of width, so each word fits a line of one
        let measure = |s: &str| s.chars().count() as f32;
        let lines = wrapped_lines("واحد اثنان ثلاثة", true, 6., measure);
        assert_eq!(lines, [vec!["واحد"], vec!["اثنان"], vec!["ثلاثة"]]);
        let lines = wrapped_lines("one two three", false, 8., measure);
        assert_eq!(lines, [vec!["one two"], vec!["three"]]);
    }
}

#[cfg(test)]
mod name_tests {
    use super::player_name;

    #[test]
    fn colour_codes_leave_names() {
        assert_eq!(player_name("^8*^4Fe^8D^1oR^8*"), "*FeDoR*");
        assert_eq!(player_name("-<[ATK]>-naz"), "-<[ATK]>-naz");
        assert_eq!(player_name("a^b^"), "a^b^");
        assert_eq!(player_name("^1^2"), "^1^2");
    }
}
