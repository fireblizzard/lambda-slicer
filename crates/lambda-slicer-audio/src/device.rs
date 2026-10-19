//! The mix sent to the system's default output device
//!
//! The window publishes what is playing in a [`Feed`] every frame, the demo
//! the worker has open, the playback clock and whether sound is heard. A
//! mixing thread runs a playback of its own of the same demo, sharing the
//! worker's checkpoints so a seek finds its place quickly, and moves it to
//! the end of each block of 10 ms before mixing the channels it then has.
//! Every block hears exactly the sounds playing during it, however late
//! the window's frames come. The device's callback copies blocks out. It
//! takes no lock the mixer holds for long, reads no file and frees nothing,
//! as the blocks go back to the mixer to be filled again.
//!
//! The mixer keeps its own place in source time and follows the clock,
//! starting again from it after a seek, a load, a pause or when it falls
//! more than [`DRIFT`] away. While paused, seeking or stepping it plays
//! silence, and the device's own latency comes on top.
//!
//! Each block is marked with the feed it was mixed for. A seek, load, pause
//! or resume changes the feed's mark at once, and the callback drops every
//! block queued under the old one, as does the mixer for a block it was
//! still mixing. A seek the mixer is making stops when the feed changes or
//! the device closes. Closing releases the stream first, so a mixer waiting
//! on a device that stopped taking blocks ends

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lambda_slicer_playback::controller::Stamp;
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::time::{Clock, SourceTime};
use lambda_slicer_playback::worker::Loaded;

use crate::library::Library;
use crate::mixer::{Listener, Output, follow, mix, write_wav};

/// How far the mixer may drift from the clock before it starts again
pub const DRIFT: f64 = 0.04;

/// Blocks of 10 ms queued ahead of the device
const QUEUED: usize = 4;

/// What is playing, as the window last saw it
#[derive(Clone)]
pub struct Feed {
    pub stamp: Stamp,
    /// The demo open in the worker, which must be of the stamp's generation
    pub demo: Loaded,
    pub clock: Clock,
    /// The instant the clock's wall time counts from
    pub clock_base: Instant,
    /// Sound is heard, playing forward
    pub audible: bool,
    /// Where playing goes no further, a loop's or a played range's end.
    /// Nothing past it is mixed, as the playhead goes back or pauses there
    /// before the window's next frame says so
    pub until: Option<SourceTime>,
}

/// A block of samples, the mark of the feed it was mixed for, and where in
/// the demo it starts as that feed's clock ran
struct Block {
    mark: u64,
    samples: Vec<[f32; 2]>,
    timing: Option<Timing>,
}

/// Where a block starts in the demo, and the clock it was mixed against
#[derive(Clone, Copy)]
struct Timing {
    start: SourceTime,
    clock: Clock,
    base: Instant,
}

/// How far the sound heard runs ahead of the playhead, in seconds, when a
/// block begins to be heard at `heard`
fn skew(timing: &Timing, heard: Instant) -> f64 {
    timing.start.0 - clock_at_instant(&timing.clock, timing.base, heard).0
}

fn clock_at_instant(clock: &Clock, base: Instant, at: Instant) -> SourceTime {
    clock.at(at.saturating_duration_since(base).as_secs_f64())
}

/// Sound playing on the default output device until dropped
pub struct Device {
    stream: Option<cpal::Stream>,
    feed: Arc<Mutex<Option<Feed>>>,
    /// Grows whenever the feed's stamp changes or it starts or stops being
    /// heard
    mark: Arc<AtomicU64>,
    volume: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
    mixer: Option<JoinHandle<()>>,
    /// Writes the measured skew out when the device closes
    skew_log: Option<JoinHandle<()>>,
}

impl Device {
    /// Open the default output device, mixing `library`'s sounds. With
    /// `tap`, every block from the first heard on is written there as a WAV
    /// file when the device closes, for checking the mix, and the device
    /// plays silence at the same pace. With `skew_log`, how far each block
    /// heard runs ahead of the playhead is written there as text when the
    /// device closes, with the device's latency
    pub fn open(
        library: Arc<Library>,
        tap: Option<PathBuf>,
        skew_log: Option<PathBuf>,
    ) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "no output device".to_owned())?;
        let config = device.default_output_config().map_err(|e| e.to_string())?;
        let rate = config.sample_rate();
        let channels = usize::from(config.channels());
        let block = (rate / 100).max(1) as usize;

        let (filled_tx, filled_rx) = mpsc::sync_channel::<Block>(QUEUED);
        // The device's own latency, from when the callback runs to when its
        // first sample is heard
        let latency = Arc::new(AtomicU64::new(0));
        let measured = latency.clone();
        let (skew_tx, skew_rx) = mpsc::sync_channel::<(Instant, Timing, u64)>(4096);
        let skew_tx = skew_log.is_some().then_some(skew_tx);
        let (empty_tx, empty_rx) = mpsc::sync_channel::<Vec<[f32; 2]>>(QUEUED + 2);
        for _ in 0..QUEUED + 2 {
            let _ = empty_tx.try_send(Vec::with_capacity(block));
        }

        let muted = tap.is_some();
        let mark = Arc::new(AtomicU64::new(0));
        let live = mark.clone();
        let mut current: Option<Block> = None;
        let mut read = 0;
        let stream = device
            .build_output_stream(
                config.config(),
                move |out: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    let called = Instant::now();
                    let stamps = info.timestamp();
                    let delay = stamps.playback.duration_since(stamps.callback);
                    measured.store(delay.as_nanos() as u64, Ordering::Relaxed);
                    let now = live.load(Ordering::SeqCst);
                    for (k, frame) in out.chunks_mut(channels).enumerate() {
                        if current
                            .as_ref()
                            .is_none_or(|b: &Block| read >= b.samples.len() || b.mark != now)
                        {
                            if let Some(done) = current.take() {
                                let _ = empty_tx.try_send(done.samples);
                            }
                            // Blocks mixed for an older feed are not heard
                            current = loop {
                                match filled_rx.try_recv() {
                                    Ok(stale) if stale.mark != now => {
                                        let _ = empty_tx.try_send(stale.samples);
                                    }
                                    Ok(fresh) => break Some(fresh),
                                    Err(_) => break None,
                                }
                            };
                            read = 0;
                            // When this block's first sample is heard
                            if let (Some(tx), Some(timing)) =
                                (&skew_tx, current.as_ref().and_then(|b| b.timing))
                            {
                                let offset = Duration::from_secs_f64(k as f64 / f64::from(rate));
                                let _ = tx.try_send((
                                    called + delay + offset,
                                    timing,
                                    delay.as_micros() as u64,
                                ));
                            }
                        }
                        let sample = current
                            .as_ref()
                            .and_then(|b| b.samples.get(read))
                            .copied()
                            .unwrap_or([0.; 2]);
                        read += 1;
                        for (i, value) in frame.iter_mut().enumerate() {
                            *value = if i < 2 && !muted { sample[i] } else { 0. };
                        }
                    }
                },
                |e| eprintln!("sound: {e}"),
                None,
            )
            .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;

        let feed = Arc::new(Mutex::new(None));
        let volume = Arc::new(AtomicU32::new(0.5f32.to_bits()));
        let stop = Arc::new(AtomicBool::new(false));
        let mixer = {
            let shared = Shared {
                feed: feed.clone(),
                mark: mark.clone(),
                volume: volume.clone(),
                stop: stop.clone(),
                latency: latency.clone(),
            };
            std::thread::Builder::new()
                .name("sound mixer".into())
                .spawn(move || {
                    let tapped = run(
                        library,
                        shared,
                        rate,
                        block,
                        filled_tx,
                        empty_rx,
                        tap.is_some(),
                    );
                    if let Some(path) = tap
                        && let Err(e) = write_wav(&path, rate, &tapped)
                    {
                        eprintln!("sound: {}: {e}", path.display());
                    }
                })
                .map_err(|e| e.to_string())?
        };

        // Collected off the device's thread, written at the end
        let skew_log = skew_log.map(|path| {
            std::thread::spawn(move || {
                let lines: Vec<String> = skew_rx
                    .iter()
                    .map(|(heard, timing, delay)| {
                        format!(
                            "{:.4} {:.2} {:.2}",
                            timing.start.0,
                            skew(&timing, heard) * 1000.,
                            delay as f64 / 1000.
                        )
                    })
                    .collect();
                let text = format!(
                    "# source_s skew_ms device_latency_ms\n{}\n",
                    lines.join("\n")
                );
                if let Err(e) = std::fs::write(&path, text) {
                    eprintln!("sound: {}: {e}", path.display());
                }
            })
        });

        Ok(Self {
            stream: Some(stream),
            feed,
            mark,
            volume,
            stop,
            mixer: Some(mixer),
            skew_log,
        })
    }

    /// Publish what is playing now
    pub fn feed(&self, feed: Feed) {
        let mut slot = self.feed.lock().unwrap_or_else(|e| e.into_inner());
        let key = |f: &Feed| (f.stamp, f.audible && f.clock.playing());
        if slot.as_ref().map(key) != Some(key(&feed)) {
            self.mark.fetch_add(1, Ordering::SeqCst);
        }
        *slot = Some(feed);
    }

    /// The master volume, 0 to 1
    pub fn set_volume(&self, volume: f32) {
        self.volume
            .store(volume.clamp(0., 1.).to_bits(), Ordering::Relaxed);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // The stream's callback holds the queue's receiving end, so the
        // mixer's next send fails even if the device stopped taking blocks
        drop(self.stream.take());
        if let Some(mixer) = self.mixer.take() {
            let _ = mixer.join();
        }
        // The stream held the log's sending end, so the log ends now
        if let Some(log) = self.skew_log.take() {
            let _ = log.join();
        }
    }
}

/// Where the next block starts in source time. The mixer goes on from where
/// it is unless a seek or load came in between or it drifted more than
/// [`DRIFT`] from `wanted`, where the clock says it should be
fn block_start(place: Option<(Stamp, SourceTime)>, stamp: Stamp, wanted: SourceTime) -> SourceTime {
    match place {
        Some((at_stamp, at)) if at_stamp == stamp && (at.0 - wanted.0).abs() <= DRIFT => at,
        _ => wanted,
    }
}

/// The source time `feed`'s clock gives at `instant`
fn clock_at(feed: &Feed, instant: Instant) -> SourceTime {
    let wall = instant
        .saturating_duration_since(feed.clock_base)
        .as_secs_f64();
    feed.clock.at(wall)
}

/// What the mixer shares with the [`Device`]
struct Shared {
    feed: Arc<Mutex<Option<Feed>>>,
    mark: Arc<AtomicU64>,
    volume: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
    /// The device's latency in nanoseconds, as its callback last measured
    latency: Arc<AtomicU64>,
}

/// The mixer's own playback, for one load generation. None when the demo
/// could not be opened
type Own = Option<(u64, Option<Playback>)>;

/// Move the mixer's playback of `demo` to `time`, opening it on a new
/// generation, and the channels and listener it then has
fn advance<'a>(
    own: &'a mut Own,
    demo: &Loaded,
    library: &Arc<Library>,
    time: SourceTime,
    interrupt: impl Fn() -> bool + Send + 'static,
) -> Option<&'a Playback> {
    if own
        .as_ref()
        .is_none_or(|(generation, _)| *generation != demo.generation)
    {
        let opened = Playback::open_with(&demo.demo, demo.limits, demo.checkpoints.clone())
            .inspect_err(|e| eprintln!("sound: {e:?}"))
            .ok()
            .map(|mut playback| {
                playback.set_game_files(&library.game_files());
                playback.set_interrupt(interrupt);
                playback
            });
        *own = Some((demo.generation, opened));
    }
    let playback = own.as_mut()?.1.as_mut()?;
    playback.advance_to(time).ok()?;
    Some(playback)
}

fn run(
    library: Arc<Library>,
    shared: Shared,
    rate: u32,
    block: usize,
    filled: SyncSender<Block>,
    empty: Receiver<Vec<[f32; 2]>>,
    tapping: bool,
) -> Vec<[f32; 2]> {
    let Shared {
        feed,
        mark,
        volume,
        stop,
        latency,
    } = shared;
    // The mark of the feed being mixed, which a seek stops for once the
    // feed's changes or the device closes
    let serving = Arc::new(AtomicU64::new(0));
    let interrupt = {
        let (mark, serving, stop) = (mark.clone(), serving.clone(), stop.clone());
        move || {
            stop.load(Ordering::SeqCst)
                || mark.load(Ordering::SeqCst) != serving.load(Ordering::SeqCst)
        }
    };
    let mut tapped = Vec::new();
    // Where the mixer is in source time, for which load and seek, none
    // while silent
    let mut place: Option<(Stamp, SourceTime)> = None;
    let mut own: Own = None;
    let sound = |name: &_| library.sound(name);

    while !stop.load(Ordering::Relaxed) {
        let mut buffer = match empty.try_recv() {
            Ok(b) => b,
            Err(TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            Err(TryRecvError::Disconnected) => break,
        };
        buffer.clear();

        let (now, tag) = {
            let feed = feed.lock().unwrap_or_else(|e| e.into_inner());
            (feed.clone(), mark.load(Ordering::SeqCst))
        };
        serving.store(tag, Ordering::SeqCst);
        let playing = now
            .filter(|f| f.audible && f.clock.playing() && f.demo.generation == f.stamp.generation);
        let mixed = playing.and_then(|f| {
            // The block is heard once the queued blocks before it are, the
            // one playing half done on average, and the device's latency
            // has passed
            let queued = (QUEUED as f64 + 0.5) * block as f64 / f64::from(rate);
            let device = Duration::from_nanos(latency.load(Ordering::Relaxed));
            let heard = Instant::now() + Duration::from_secs_f64(queued) + device;
            let from = block_start(place, f.stamp, clock_at(&f, heard));
            if f.until.is_some_and(|until| from.0 >= until.0) {
                return None;
            }
            let speed = f.clock.rate().factor();
            let to = SourceTime(from.0 + block as f64 * speed / f64::from(rate));
            let playback = advance(&mut own, &f.demo, &library, to, interrupt.clone())?;
            let output = Output {
                rate,
                speed,
                volume: f32::from_bits(volume.load(Ordering::Relaxed)),
            };
            let channels = follow(playback.audio().channels(), playback.scene());
            buffer.extend(mix(
                &channels,
                &sound,
                &Listener::of(playback),
                from,
                block,
                &output,
            ));
            Some((f.stamp, to, from, f.clock, f.clock_base))
        });
        // A block whose feed changed while it was mixed is not heard, and
        // the next starts at the clock
        let mixed = mixed.filter(|_| mark.load(Ordering::SeqCst) == tag);
        place = mixed.map(|(stamp, to, ..)| (stamp, to));
        let timing = mixed.map(|(_, _, start, clock, base)| Timing { start, clock, base });
        if place.is_none() {
            buffer.clear();
            buffer.resize(block, [0.; 2]);
        }
        if tapping && (place.is_some() || !tapped.is_empty()) {
            tapped.extend_from_slice(&buffer);
        }
        let block = Block {
            mark: tag,
            samples: buffer,
            timing,
        };
        if !send(&filled, block, &stop) {
            break;
        }
    }
    tapped
}

/// Queue `block` for the device, false once the device closed or the
/// stream is gone
fn send(filled: &SyncSender<Block>, mut block: Block, stop: &AtomicBool) -> bool {
    loop {
        match filled.try_send(block) {
            Ok(()) => return true,
            Err(TrySendError::Disconnected(_)) => return false,
            Err(TrySendError::Full(back)) => {
                if stop.load(Ordering::SeqCst) {
                    return false;
                }
                block = back;
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_follow_on_until_a_seek_or_drift() {
        let stamp = Stamp {
            generation: 1,
            revision: 2,
        };
        let at = SourceTime(10.);
        // Close to the clock, the mixer goes on from where it is
        assert_eq!(block_start(Some((stamp, at)), stamp, SourceTime(10.03)), at);
        // Too far, after a seek, or from silence, it starts at the clock
        assert_eq!(
            block_start(Some((stamp, at)), stamp, SourceTime(10.05)),
            SourceTime(10.05)
        );
        let seek = Stamp {
            revision: 3,
            ..stamp
        };
        assert_eq!(
            block_start(Some((stamp, at)), seek, SourceTime(10.01)),
            SourceTime(10.01)
        );
        assert_eq!(block_start(None, stamp, SourceTime(3.)), SourceTime(3.));
    }
}
