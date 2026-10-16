//! Lambda Slicer desktop app
//!
//! `lambda-slicer-gui [PATH...] [--locale TAG] [--settings] [--cache FILE]
//! [--demo FILE [--window N]] [--screenshot OUT.png [--frames N]]`. Demos and folders
//! given as `PATH` are opened for this session, and `--demo` shows a cached
//! demo at start, on its `N`th best action window with `--window`. `--locale` overrides the saved
//! language and `--cache` the library cache for this run, `--settings` opens
//! the settings window, and `--screenshot` saves the window to a PNG after
//! `N` frames and exits, for checking how a language renders

mod app;
mod demo;
mod i18n;
mod jobs;
mod layout;
mod library;
mod settings;
mod text;
mod timeline;

use std::path::PathBuf;

use eframe::egui;

use app::{App, Launch, Screenshot};
use i18n::Locale;
use settings::Settings;

fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let mut locale = None;
    let mut show_settings = false;
    let mut screenshot = None;
    let mut frames = 5;
    let mut cache = None;
    let mut open = Vec::new();
    let mut demo = None;
    let mut window = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--locale" => locale = args.next().as_deref().and_then(Locale::from_tag),
            "--settings" => show_settings = true,
            "--screenshot" => screenshot = args.next().map(PathBuf::from),
            "--frames" => frames = args.next().and_then(|n| n.parse().ok()).unwrap_or(frames),
            "--cache" => cache = args.next().map(PathBuf::from),
            "--demo" => demo = args.next().map(PathBuf::from),
            "--window" => {
                window = args
                    .next()
                    .and_then(|n| n.parse::<usize>().ok())
                    .map(|n| n.saturating_sub(1))
            }
            flag if flag.starts_with("--") => eprintln!("ignoring argument {flag}"),
            path => open.push(PathBuf::from(path)),
        }
    }

    let settings_path = settings::path();
    let mut settings = settings_path
        .as_deref()
        .map(Settings::load_from)
        .unwrap_or_default();
    let screenshot = screenshot.map(|path| Screenshot { path, frames });
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
        viewport: egui::ViewportBuilder::default().with_inner_size([1280., 800.]),
        ..Default::default()
    };
    eframe::run_native(
        "Lambda Slicer",
        options,
        Box::new(move |cc| {
            let mut app = App::new(
                settings,
                save_to,
                Launch {
                    show_settings,
                    screenshot,
                    cache,
                    open,
                    demo,
                    window,
                },
            );
            app.start_scan(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
