//! Text the server places on the HUD, such as a bhop or KZ server's timer
//! and stats, where and how the game's client draws it (HLSDK's
//! `CHudMessage`)
//!
//! A message stands at its share of the view's width and height, -1 for
//! the middle and a negative share counting from the right or the bottom,
//! kept inside the view. Each line is centred on the middle, or else lines
//! start together. The text is added to the picture in its colour, fading
//! in and out over its times. A written out message shows a letter at a
//! time, each starting in the second colour and settling into the first.
//! The flicker of the flickering effect is left out. Text is sized to the
//! view as the game's 640 by 480 screen would be, as the chat is

use eframe::egui::{self, Color32, Pos2, Rect};
use lambda_slicer_playback::hud_messages::{HudMessage, ShownMessage, WRITE_OUT};

/// The game's screen height the HUD is laid out on
const SCREEN_HEIGHT: f32 = 480.;

/// A line's height and the text's size, in the game's rows
const LINE_HEIGHT: f32 = 16.;
const TEXT_SIZE: f32 = 13.;

/// The share that stands for the middle
const MIDDLE: f32 = -1.;

/// How far toward black, of 255, the whole message has faded at `since`
/// seconds (`MessageScanStart`)
fn faded(message: &HudMessage, since: f32) -> f32 {
    let letters = message.text.len() as f32;
    let fade_in = if message.effect == WRITE_OUT {
        message.fade_in * letters
    } else {
        message.fade_in
    };
    let fade_out_from = fade_in + message.hold;

    if message.effect != WRITE_OUT && since < message.fade_in {
        (message.fade_in - since) / message.fade_in * 255.
    } else if since > fade_out_from && message.fade_out > 0. {
        (since - fade_out_from) / message.fade_out * 255.
    } else if since > fade_out_from && message.effect != WRITE_OUT {
        255.
    } else {
        0.
    }
}

/// `from` blended `blend` of 255 toward `to`
fn blended(from: [u8; 3], to: [u8; 3], blend: f32) -> Color32 {
    let blend = blend.clamp(0., 255.);
    let mix = |a: u8, b: u8| ((f32::from(a) * (255. - blend) + f32::from(b) * blend) / 256.) as u8;
    let [r, g, b] = [0, 1, 2].map(|k| mix(from[k], to[k]));
    Color32::from_rgba_premultiplied(r, g, b, 0)
}

/// Each letter's colour at `since` seconds, line breaks left out, added to
/// the picture (`MessageScanNextChar`)
fn letter_colours(message: &HudMessage, since: f32) -> Vec<Color32> {
    let fade = faded(message, since);
    let black = [0; 3];
    let letters = message.text.chars().filter(|&c| c != '\n').count();
    if message.effect != WRITE_OUT {
        return vec![blended(message.colour, black, fade); letters];
    }

    // Each letter appears a fade in after the one before it
    let fade_out_from = message.fade_in * message.text.len() as f32 + message.hold;
    let mut colours = Vec::with_capacity(letters);
    let mut appears = 0.;
    for _ in 0..letters {
        appears += message.fade_in;
        let colour = if appears > since {
            blended(black, black, 0.)
        } else if since > fade_out_from {
            blended(message.colour, black, fade)
        } else if since - appears >= message.settle {
            blended(message.colour, black, 0.)
        } else {
            let settled = (since - appears) / message.settle;
            blended(message.colour, message.colour2, 255. - settled * 255.)
        };
        colours.push(colour);
    }
    colours
}

/// Where a run `size` long starts in a view `room` long at `share` of it,
/// `whole` being the longest run's size when counting from the far end, and
/// kept inside (`XPosition`, `YPosition`)
fn place(share: f32, size: f32, whole: f32, room: f32) -> f32 {
    let at = if share == MIDDLE {
        (room - size) / 2.
    } else if share < 0. {
        (1. + share) * room - whole
    } else {
        share * room
    };
    at.min(room - size).max(0.)
}

/// `messages` over the view at `rect`
pub fn draw(ui: &egui::Ui, rect: Rect, messages: &[ShownMessage]) {
    let scale = rect.height() / SCREEN_HEIGHT;
    let font = egui::FontId::proportional(TEXT_SIZE * scale);
    let line_height = LINE_HEIGHT * scale;
    let painter = ui.painter_at(rect);

    for shown in messages {
        let message = &shown.message;
        let colours = letter_colours(message, shown.since);

        // Each line laid out letter by letter in its colours
        let mut lines = Vec::new();
        let mut letter = 0;
        for text in message.text.split('\n') {
            let mut job = egui::text::LayoutJob::default();
            for c in text.chars() {
                let colour = colours.get(letter).copied().unwrap_or(Color32::TRANSPARENT);
                let format = egui::TextFormat::simple(font.clone(), colour);
                job.append(c.encode_utf8(&mut [0; 4]), 0., format);
                letter += 1;
            }
            lines.push(painter.layout_job(job));
        }

        let widest = lines.iter().map(|l| l.size().x).fold(0., f32::max);
        let height = line_height * lines.len() as f32;
        let mut top = rect.top() + place(message.y, height, height, rect.height());
        for line in lines {
            let width = line.size().x;
            let left = rect.left() + place(message.x, width, widest, rect.width());
            painter.galley(Pos2::new(left, top), line, Color32::WHITE);
            top += line_height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(effect: u8) -> HudMessage {
        HudMessage {
            x: MIDDLE,
            y: 0.5,
            effect,
            colour: [200, 100, 0],
            colour2: [0, 0, 200],
            fade_in: 0.5,
            fade_out: 1.,
            hold: 2.,
            settle: 0.25,
            text: "ab\nc".into(),
        }
    }

    #[test]
    fn a_message_fades_in_holds_and_fades_out() {
        let m = message(0);
        assert_eq!(faded(&m, 0.25), 127.5);
        assert_eq!(faded(&m, 1.), 0.);
        assert_eq!(faded(&m, 3.), 127.5);
        assert_eq!(letter_colours(&m, 1.).len(), 3);
    }

    #[test]
    fn written_out_letters_appear_in_turn_and_settle() {
        let m = message(WRITE_OUT);
        // At 0.6 the first letter has settled halfway, the rest wait
        let colours = letter_colours(&m, 0.625);
        assert_eq!(colours[0], blended([200, 100, 0], [0, 0, 200], 127.5));
        assert_eq!(colours[1], blended([0; 3], [0; 3], 0.));
        assert_eq!(colours[2], blended([0; 3], [0; 3], 0.));
    }

    #[test]
    fn places_count_from_either_end_and_stay_inside() {
        assert_eq!(place(MIDDLE, 100., 100., 640.), 270.);
        assert_eq!(place(0.25, 100., 100., 640.), 160.);
        assert_eq!(place(-0.25, 50., 100., 640.), 380.);
        assert_eq!(place(0.95, 100., 100., 640.), 540.);
    }
}
