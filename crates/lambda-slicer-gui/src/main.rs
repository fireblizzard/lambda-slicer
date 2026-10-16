//! Lambda Slicer desktop app
//!
//! `lambda-slicer-gui [--locale TAG] [--settings] [--screenshot OUT.png]`.
//! `--locale` overrides the saved language for this run, `--settings` opens
//! the settings window, and `--screenshot` saves the window to a PNG and
//! exits, for checking how a language renders

mod app;
mod i18n;
mod settings;
mod text;

use std::path::PathBuf;

use eframe::egui;

use app::{App, Screenshot};
use i18n::Locale;
use settings::Settings;

fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let mut locale = None;
    let mut show_settings = false;
    let mut screenshot = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--locale" => locale = args.next().as_deref().and_then(Locale::from_tag),
            "--settings" => show_settings = true,
            "--screenshot" => {
                screenshot = args.next().map(|p| Screenshot {
                    path: PathBuf::from(p),
                    frames: 5,
                })
            }
            other => eprintln!("ignoring argument {other}"),
        }
    }

    let settings_path = settings::path();
    let mut settings = settings_path
        .as_deref()
        .map(Settings::load_from)
        .unwrap_or_default();
    // A screenshot run must not change the saved settings
    let save_to = if screenshot.is_some() {
        None
    } else {
        settings_path
    };
    if locale.is_some() {
        settings.locale = locale;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000., 640.]),
        ..Default::default()
    };
    eframe::run_native(
        "Lambda Slicer",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(App::new(
                settings,
                save_to,
                show_settings,
                screenshot,
            )))
        }),
    )
}
