//! Lambda Slicer desktop app
//!
//! `lambda-slicer-gui [PATH...] [--locale TAG] [--settings [--section NAME] [--restore] [--clear]]
//! [--timeline]
//! [--dates] [--pseudo] [--size WxH] [--cache FILE] [--clips FILE]
//! [--demo FILE [--window N] [--event N] [--show LIST] [--refine LIST] [--player TEXT] [--tab TAB]
//! [--export DIR]]
//! [--screenshot OUT.png [--frames N]]
//! [--game DIR [--viewer-at SECONDS] [--viewer-bench SECONDS OUT.txt]
//! [--viewer-capture TIMES FOLDER] [--viewer-size WxH] [--viewer-play SECONDS]
//! [--viewer-gamma GAMMA,BRIGHTNESS,TEXGAMMA,LIGHTGAMMA[,OVERBRIGHT]]
//! [--viewer-see-through]]`
//!
//! Demos and folders given as `PATH` are opened for this session. `--locale`
//! overrides the saved language, `--cache` the library cache and `--clips`
//! the clips file for this run. `--demo` shows a cached demo at start, on
//! its `N`th golden minute candidate with `--window`, with its `N`th shown
//! event selected with `--event`, the kinds of event in the comma-separated
//! `LIST` shown with `--show`, from `kills`, `shots`, `switches`, `actions`,
//! `chat` and `match`, and with the kill refinements in the
//! comma-separated `LIST` on with `--refine`, from `midair`, `both`, `one`,
//! `speed`, `killer`, `victim` and `wallgauss`, and only events of players
//! with `TEXT` in their name with `--player`. `--tab` shows the inspector's
//! `events`, `clips` or `scoreboard` tab, and `--export` exports its clips
//! into `DIR`. `--settings` opens the settings window, on its `general`,
//! `library`, `viewer` or `clips` section with `--section` and asking
//! whether to restore the defaults with `--restore` or to clear the
//! library with `--clear`, `--timeline` opens
//! the player lanes under the timeline, `--dates` turns on both date
//! filters, `--pseudo` lengthens and accents every message to find text that does
//! not fit, and `--size` sets the window size. `--screenshot` saves the window to a
//! PNG `N` frames after any demo has loaded and its exports finished, and
//! exits, for checking how a language renders. `--game` names the game's
//! installation folder, the one that holds `valve`, and plays the demo in
//! the demo view, from `--viewer-at` seconds if given. `--viewer-bench`
//! turns vsync off, plays and times the view's frames for `SECONDS`, writes
//! the times and how long loading took to `OUT.txt` and exits.
//! `--viewer-capture` saves the view at each of the comma-separated `TIMES`
//! into `FOLDER` as `<seconds>.png` and exits. Both draw the view
//! `--viewer-size` pixels, a capture 1280x720 and a bench the room the
//! window gives unless it is given. `--viewer-play` plays for `SECONDS` with
//! sound, says how often the sound device went without sound to play, and
//! exits, and with `LAMBDA_SLICER_SOUND_TAP` set to a file the sound sent
//! to the device is written there as a WAV file. `--viewer-gamma` draws
//! with the game's gamma settings for this run only, overbright when the
//! fifth is 1, for comparing with frames the game recorded.
//! `--viewer-see-through` shows players through walls for this run

// A release build opens no console window beside the app
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod announcements;
mod app;
mod chat;
mod clips;
mod crosshair;
mod decals;
mod demo;
mod export;
mod floors;
mod game_setup;
mod i18n;
mod jobs;
mod kill_analysis;
mod kill_feed;
mod kill_filters;
mod layout;
mod library;
mod platform;
mod settings;
mod text;
mod theme;
mod timeline;
mod viewer;
mod widgets;

use std::path::PathBuf;

use eframe::egui;

use app::{App, Launch, Screenshot, SettingsTab};
use demo::InspectorTab;
use i18n::Locale;
use kill_filters::{KillFilters, MidairMode, SpeedSubject};
use settings::Settings;

/// Kill refinements named in a comma-separated list, for showing them from
/// the command line
fn refinements(list: &str) -> KillFilters {
    let mut filters = KillFilters::default();
    for name in list.split(',') {
        match name.trim() {
            "midair" => filters.midair = MidairMode::Either,
            "both" => filters.midair = MidairMode::Both,
            "one" => filters.midair = MidairMode::ExactlyOne,
            "speed" => filters.speed = true,
            "killer" => filters.speed_subject = SpeedSubject::Killer,
            "victim" => filters.speed_subject = SpeedSubject::Victim,
            "wallgauss" => filters.wallgauss = true,
            other => eprintln!("ignoring refinement {other}"),
        }
    }

    filters
}

fn main() -> eframe::Result {
    let mut args = std::env::args().skip(1);
    let mut locale = None;
    let mut show_settings = false;
    let mut confirm_restore = false;
    let mut confirm_clear = false;
    let mut show_timeline = false;
    let mut screenshot = None;
    let mut frames = 5;
    let mut cache = None;
    let mut clips = None;
    let mut open = Vec::new();
    let mut demo = None;
    let mut window = None;
    let mut event = None;
    let mut refine = KillFilters::default();
    let mut player = String::new();
    let mut show = None;
    let mut tab = InspectorTab::default();
    let mut section = SettingsTab::default();
    let mut size = [1280., 800.];
    let mut dates = false;
    let mut export = None;
    let mut pseudo = false;
    let mut game = None;
    let mut viewer_at = None;
    let mut capture_times: Option<(Vec<f64>, PathBuf)> = None;
    let mut viewer_size = None;
    let mut viewer_play = None;
    let mut viewer_gamma = None;
    let mut viewer_see_through = false;
    let mut viewer_bench = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--locale" => locale = args.next().as_deref().and_then(Locale::from_tag),
            "--settings" => show_settings = true,
            "--restore" => confirm_restore = true,
            "--clear" => confirm_clear = true,
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
            "--event" => {
                event = args
                    .next()
                    .and_then(|n| n.parse::<usize>().ok())
                    .map(|n| n.saturating_sub(1))
            }
            "--refine" => {
                if let Some(list) = args.next() {
                    refine = refinements(&list);
                }
            }
            "--player" => player = args.next().unwrap_or_default(),
            "--show" => show = args.next(),
            "--section" => {
                section = match args.next().as_deref() {
                    Some("library") => SettingsTab::Library,
                    Some("viewer") => SettingsTab::Viewer,
                    Some("clips") => SettingsTab::Clips,
                    _ => SettingsTab::General,
                }
            }
            "--tab" => {
                tab = match args.next().as_deref() {
                    Some("clips") => InspectorTab::Clips,
                    Some("scoreboard") => InspectorTab::Scoreboard,
                    _ => InspectorTab::Events,
                }
            }
            "--size" => {
                if let Some((w, h)) = args.next().as_deref().and_then(|s| s.split_once('x'))
                    && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
                {
                    size = [w, h];
                }
            }
            "--dates" => dates = true,
            "--pseudo" => pseudo = true,
            "--export" => export = args.next().map(PathBuf::from),
            "--game" => game = args.next().map(PathBuf::from),
            "--viewer-at" => viewer_at = args.next().and_then(|s| s.parse().ok()),
            "--viewer-capture" => {
                if let (Some(times), Some(folder)) = (args.next(), args.next()) {
                    let times = times
                        .split(',')
                        .filter_map(|t| t.trim().parse().ok())
                        .collect();
                    capture_times = Some((times, PathBuf::from(folder)));
                }
            }
            "--viewer-play" => viewer_play = args.next().and_then(|s| s.parse().ok()),
            "--viewer-see-through" => viewer_see_through = true,
            "--viewer-gamma" => {
                let values: Vec<f32> = args
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
                if let [gamma, brightness, texgamma, lightgamma, ref rest @ ..] = values[..] {
                    viewer_gamma = Some(lambda_slicer_render::gamma::Gamma {
                        gamma,
                        brightness,
                        texgamma,
                        lightgamma,
                        overbright: rest.first() == Some(&1.),
                    });
                }
            }
            "--viewer-size" => {
                if let Some((w, h)) = args.next().as_deref().and_then(|s| s.split_once('x'))
                    && let (Ok(w), Ok(h)) = (w.parse(), h.parse())
                {
                    viewer_size = Some([w, h]);
                }
            }
            "--viewer-bench" => {
                if let (Some(seconds), Some(out)) =
                    (args.next().and_then(|s| s.parse().ok()), args.next())
                {
                    viewer_bench = Some(viewer::Bench {
                        seconds,
                        out: PathBuf::from(out),
                        size: None,
                    });
                }
            }
            flag if flag.starts_with("--") => eprintln!("ignoring argument {flag}"),
            path => open.push(PathBuf::from(path)),
        }
    }

    if let Some(bench) = &mut viewer_bench {
        bench.size = viewer_size;
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
        settings.player_lanes = true;
    }
    if export.is_some() {
        settings.export_folder = export.clone();
    }

    let icon = eframe::icon_data::from_png_bytes(include_bytes!(
        "../../../assets/branding/icons/icon-256.png"
    ))
    .expect("the bundled application icon must be a valid PNG");
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_app_id("lambda-slicer")
            .with_icon(icon),
        ..Default::default()
    };
    if viewer_bench.is_some() {
        options.wgpu_options.surface.present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
    }
    eframe::run_native(
        "Lambda Slicer",
        options,
        Box::new(move |cc| {
            theme::install(&cc.egui_ctx);

            let mut app = App::new(
                settings,
                save_to,
                Launch {
                    show_settings,
                    confirm_restore,
                    confirm_clear,
                    screenshot,
                    cache,
                    clips,
                    open,
                    demo,
                    window,
                    event,
                    refine,
                    player,
                    show,
                    dates,
                    export: export.is_some(),
                    pseudo,
                    game,
                    viewer_at,
                    viewer_bench,
                    viewer_play,
                    viewer_gamma,
                    viewer_see_through,
                    tab,
                    section,
                    viewer_capture: capture_times.map(|(times, folder)| viewer::Capture {
                        times,
                        folder,
                        size: viewer_size.unwrap_or([1280, 720]),
                    }),
                },
            );
            app.render_state = cc.wgpu_render_state.clone();
            app.start_scan(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
