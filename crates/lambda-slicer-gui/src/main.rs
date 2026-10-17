//! Lambda Slicer desktop app
//!
//! `lambda-slicer-gui [PATH...] [--locale TAG] [--settings] [--timeline]
//! [--dates] [--size WxH] [--cache FILE] [--clips FILE]
//! [--demo FILE [--window N]] [--screenshot OUT.png [--frames N]]`
//!
//! Demos and folders given as `PATH` are opened for this session. `--locale`
//! overrides the saved language, `--cache` the library cache and `--clips`
//! the clips file for this run. `--demo` shows a cached demo at start, on its `N`th golden minute
//! candidate with `--window`. `--settings` opens the settings window,
//! `--timeline` shows demo timelines, `--dates` turns on both date filters
//! and `--size` sets the window size. `--screenshot` saves the window to a
//! PNG `N` frames after any demo has loaded and exits, for checking how a
//! language renders

mod app;
mod clips;
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
    let mut show_timeline = false;
    let mut screenshot = None;
    let mut frames = 5;
    let mut cache = None;
    let mut clips = None;
    let mut open = Vec::new();
    let mut demo = None;
    let mut window = None;
    let mut size = [1280., 800.];
    let mut dates = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--locale" => locale = args.next().as_deref().and_then(Locale::from_tag),
            "--settings" => show_settings = true,
            "--timeline" => show_timeline = true,
            "--screenshot" => screenshot = args.next().map(PathBuf::from),
            "--frames" => frames = args.next().and_then(|n| n.parse().ok()).unwrap_or(frames),
            "--cache" => cache = args.next().map(PathBuf::from),
            "--clips" => clips = args.next().map(PathBuf::from),
            "--demo" => demo = args.next().map(PathBuf::from),
            "--window" => {
                window = args
                    .next()
                    .and_then(|n| n.parse::<usize>().ok())
                    .map(|n| n.saturating_sub(1))
            }
            "--size" => {
                if let Some((w, h)) = args.next().as_deref().and_then(|s| s.split_once('x'))
                    && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
                {
                    size = [w, h];
                }
            }
            "--dates" => dates = true,
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
    let clips = clips.or_else(|| {
        settings_path
            .as_ref()
            .map(|p| p.with_file_name("clips.json"))
    });
    // A screenshot run must not change the saved settings
    let save_to = if screenshot.is_some() {
        None
    } else {
        settings_path
    };
    if locale.is_some() {
        settings.locale = locale;
    }
    if show_timeline {
        settings.show_timeline = true;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(size),
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
                    clips,
                    open,
                    demo,
                    window,
                    dates,
                },
            );
            app.start_scan(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
