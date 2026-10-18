//! The client events in studio models' animations (implementation plan
//! §13.6)
//!
//! A sequence can name client events on its frames, muzzle flashes and
//! sounds among them, which the engine fires while it draws the model, for
//! the frames passed since it last drew it (`R_StudioClientEvents`).
//! Playback fires them as each group moves time on instead, so the same
//! events fire however often the window draws, and a seek finds the same
//! ones. From the frame at the group before to the frame at this group's
//! client time, or from just before the first frame when an animation that
//! does not loop has just begun.
//!
//! An entity's animation begins when its sequence changes, which is when
//! the engine marks one, or when its frame goes back with a new animation
//! time, as the server sets it for a shot that repeats the sequence. That second case is a reading of the server's code, not yet
//! compared with the game. The viewmodel's begins with each weapon
//! animation. Within one animation each frame's events fire once, where
//! the engine can fire a first frame's events again on the frames it draws
//! soon after. Only models the game would draw fire events, so not the
//! entity the recorder views from, which the viewmodel stands for

use std::collections::HashMap;
use std::sync::Arc;

use crate::events::mix;
use crate::scene::{EntityId, Scene};
use crate::time::SourceTime;

/// `EF_NODRAW`
const NODRAW: i64 = 128;
/// `OBS_IN_EYE`
pub(crate) const IN_EYE: i64 = 4;
/// The first event number the client handles
const CLIENT_EVENTS: i32 = 5000;
/// How long a muzzle flash stays in the list, for a window drawing less
/// often than groups come
const FLASH_KEPT: f64 = 0.1;

/// What playback needs of one of a model's sequences
#[derive(Clone, Debug, PartialEq)]
pub struct Sequence {
    pub fps: f32,
    pub frames: u32,
    pub looping: bool,
    /// Its client events
    pub events: Vec<StudioEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StudioEvent {
    pub frame: i32,
    pub event: i32,
    pub options: Arc<str>,
}

/// Where playback learns a model's sequences, by its path, shared like
/// sound lengths so every playback of a demo fires the same events
pub type ModelSequences = Arc<dyn Fn(&str) -> Option<Arc<[Sequence]>> + Send + Sync>;

/// `StudioEstimateFrame`, the frame `advance` frames on from `cycle`, which
/// counts 0 to 256 through the sequence
pub fn estimate_frame(sequence: &Sequence, cycle: f64, advance: f64) -> f64 {
    let frames = f64::from(sequence.frames);
    let last = frames - 1.;
    let mut frame = if sequence.frames <= 1 {
        0.
    } else {
        cycle * last / 256.
    };
    frame += advance;
    if sequence.looping {
        if sequence.frames > 1 {
            frame -= (frame / last).trunc() * last;
        }
        if frame < 0. {
            frame += last;
        }
    } else {
        frame = frame.min(frames - 1.001).max(0.);
    }
    frame
}

/// The model that shows a muzzle flash
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Holder {
    Viewmodel,
    Entity(EntityId),
}

/// A muzzle flash, a sprite at one of a model's attachments for a moment
/// (`R_MuzzleFlash`)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flash {
    pub holder: Holder,
    pub attachment: u8,
    /// `sprites/muzzleflash1.spr` to `muzzleflash3.spr`, 0 to 2
    pub sprite: u8,
    pub scale: f32,
    /// Degrees about the view
    pub roll: f32,
    /// Picks the sprite's frame, modulo its frame count
    pub frame: u32,
    pub time: SourceTime,
}

/// What an animation's client events do
#[derive(Clone, Debug, PartialEq)]
pub enum Fired {
    Flash(Flash),
    /// A sound by name at a place, heard as the listener's own
    /// (`pfnPlaySoundByNameAtLocation`)
    Sound {
        sample: Arc<str>,
        origin: [f32; 3],
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Latched {
    life: u32,
    sequence: i64,
    animtime: f32,
    cycle: f32,
    /// The frame its events have fired up to
    end: Option<f64>,
}

/// Where the events to fire start, for an animation now at frame `end` that
/// moved `step` frames since the group before. From just before the first
/// frame when one that does not loop has `begun`, as the engine does, and
/// otherwise never before the frame its events already fired up to, so a
/// frame fires once in one animation however the groups fall. Only a
/// looping one wraps back to its start
fn window(sequence: &Sequence, begun: bool, fired_to: Option<f64>, end: f64, step: f64) -> f64 {
    if begun && !sequence.looping {
        return -0.01;
    }
    let start = end - step;
    match fired_to {
        // A frame estimate can fall back when a late update brings the
        // animation time, and nothing fires again until it passes on
        Some(to) if to <= end || !sequence.looping => start.max(to),
        _ => start,
    }
}

/// Where every model's animation was when time last moved on
#[derive(Clone, Debug, Default)]
pub struct Animations {
    latched: HashMap<u16, Latched>,
    /// When the viewmodel's animation began, and the frame its events have
    /// fired up to
    viewmodel: Option<(SourceTime, f64)>,
    last: Option<SourceTime>,
    flashes: Vec<Flash>,
}

impl Animations {
    /// Recent muzzle flashes, oldest first
    pub fn flashes(&self) -> &[Flash] {
        &self.flashes
    }

    /// Move every animation on to the group at `at`, and what their client
    /// events did on the way
    pub fn step(
        &mut self,
        scene: &Scene,
        sequences: &dyn Fn(&str) -> Option<Arc<[Sequence]>>,
        at: SourceTime,
    ) -> Vec<Fired> {
        let dt = self.last.map_or(0., |last| (at.0 - last.0).clamp(0., 1.));
        self.last = Some(at);
        self.flashes.retain(|f| at.0 - f.time.0 < FLASH_KEPT);
        let mut fired = Vec::new();

        let camera = &scene.camera;
        let observing = camera.observer_mode != 0 && camera.observer_mode != IN_EYE;
        let shown = camera.health > 0 && !camera.intermission && !observing;
        match (&camera.viewmodel, scene.viewmodel, shown) {
            (Some(model), Some(animation), true) => {
                let before = self
                    .viewmodel
                    .filter(|(started, _)| *started == animation.started)
                    .map(|(_, end)| end);
                let sequence = sequences(model)
                    .and_then(|list| list.get(animation.sequence.max(0) as usize).cloned());
                let end = sequence.as_ref().map_or(0., |sequence| {
                    let fps = f64::from(sequence.fps);
                    let elapsed = (at.0 - animation.started.0) * fps;
                    let end = estimate_frame(sequence, 0., elapsed);
                    let start = window(sequence, before.is_none(), before, end, dt * fps);
                    if dt > 0. || before.is_none() {
                        let origin = camera.origin;
                        let holder = Holder::Viewmodel;
                        self.fire(sequence, start, end, holder, origin, at, &mut fired);
                    }
                    end
                });
                self.viewmodel = Some((animation.started, end));
            }
            _ => self.viewmodel = None,
        }

        let view = camera.view_entity;
        let client_time = scene.client_time(at);
        let mut latched = HashMap::with_capacity(self.latched.len());
        for e in scene.entities.iter() {
            let hidden = Some(e.id.number) == view
                || (camera.observer_mode == IN_EYE
                    && i64::from(e.id.number) == camera.observer_target)
                || e.effects & NODRAW != 0;
            let Some(model) = e.model.as_deref().filter(|m| m.ends_with(".mdl")) else {
                continue;
            };
            if hidden {
                continue;
            }
            let before = self
                .latched
                .get(&e.id.number)
                .filter(|l| l.life == e.id.life)
                .copied();
            let begun = before.is_some_and(|l| {
                l.sequence != e.sequence || (e.frame < l.cycle && l.animtime != e.animtime)
            });
            let mut now = Latched {
                life: e.id.life,
                sequence: e.sequence,
                animtime: e.animtime,
                cycle: e.frame,
                end: None,
            };
            let sequence = sequences(model)
                .and_then(|list| list.get(e.sequence.max(0) as usize).cloned())
                .filter(|sequence| !sequence.events.is_empty());
            if let Some(sequence) = sequence {
                let rate = f64::from(e.framerate) * f64::from(sequence.fps);
                let elapsed = (client_time - f64::from(e.animtime)).max(0.);
                let end = estimate_frame(&sequence, f64::from(e.frame), elapsed * rate);
                let fired_to = before.filter(|_| !begun).and_then(|l| l.end);
                let start = window(&sequence, begun, fired_to, end, dt * rate);
                if dt > 0. || begun {
                    let holder = Holder::Entity(e.id);
                    self.fire(&sequence, start, end, holder, e.origin, at, &mut fired);
                }
                now.end = Some(end);
            }
            latched.insert(e.id.number, now);
        }
        self.latched = latched;
        fired
    }

    /// Fire `sequence`'s client events on frames after `start` up to `end`
    #[allow(clippy::too_many_arguments)]
    fn fire(
        &mut self,
        sequence: &Sequence,
        start: f64,
        end: f64,
        holder: Holder,
        origin: [f32; 3],
        at: SourceTime,
        fired: &mut Vec<Fired>,
    ) {
        let number = match holder {
            Holder::Viewmodel => 0,
            Holder::Entity(id) => u64::from(id.number) + 1,
        };
        for (i, event) in sequence.events.iter().enumerate() {
            let frame = f64::from(event.frame);
            if event.event < CLIENT_EVENTS || frame <= start || frame > end {
                continue;
            }
            let attachment = match event.event {
                5001 => 0,
                5011 => 1,
                5021 => 2,
                5031 => 3,
                5004 => {
                    fired.push(Fired::Sound {
                        sample: event.options.clone(),
                        origin,
                    });
                    continue;
                }
                _ => continue,
            };
            let kind: i32 = event.options.trim().parse().unwrap_or(0);
            let sprite = ((kind % 10) % 3).unsigned_abs() as u8;
            let scale = match (kind / 10) as f32 * 0.1 {
                0. => 0.5,
                scale => scale,
            };
            let seed = mix(mix(at.0.to_bits(), number), i as u64);
            let roll = if sprite == 0 {
                (seed % 21) as f32
            } else {
                (seed % 360) as f32
            };
            let flash = Flash {
                holder,
                attachment,
                sprite,
                scale,
                roll,
                frame: (seed >> 32) as u32,
                time: at,
            };
            self.flashes.push(flash);
            fired.push(Fired::Flash(flash));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shoot() -> Sequence {
        Sequence {
            fps: 30.,
            frames: 10,
            looping: false,
            events: vec![
                StudioEvent {
                    frame: 0,
                    event: 5001,
                    options: Arc::from("20"),
                },
                StudioEvent {
                    frame: 5,
                    event: 5004,
                    options: Arc::from("items/clipinsert1.wav"),
                },
            ],
        }
    }

    #[test]
    fn events_fire_once_as_their_frame_passes() {
        let mut a = Animations::default();
        let mut fired = Vec::new();
        let at = SourceTime(1.);
        // Just begun, the first frame's flash fires
        a.fire(
            &shoot(),
            -0.01,
            0.3,
            Holder::Viewmodel,
            [0.; 3],
            at,
            &mut fired,
        );
        assert!(
            matches!(fired.as_slice(), [Fired::Flash(f)] if f.sprite == 0 && (f.scale - 0.2).abs() < 1e-6 && f.roll <= 20.)
        );
        // Later steps pass frame 5 once and frame 0 no more
        fired.clear();
        a.fire(
            &shoot(),
            0.3,
            4.9,
            Holder::Viewmodel,
            [0.; 3],
            at,
            &mut fired,
        );
        assert!(fired.is_empty());
        a.fire(
            &shoot(),
            4.9,
            5.2,
            Holder::Viewmodel,
            [0.; 3],
            at,
            &mut fired,
        );
        assert!(
            matches!(fired.as_slice(), [Fired::Sound { sample, .. }] if &**sample == "items/clipinsert1.wav")
        );
    }

    #[test]
    fn frames_stop_at_the_end_or_wrap_when_looping() {
        let mut s = shoot();
        assert!((estimate_frame(&s, 0., 20.) - 8.999).abs() < 1e-9);
        s.looping = true;
        assert!((estimate_frame(&s, 0., 10.) - 1.).abs() < 1e-9);
        assert!((estimate_frame(&s, 128., 0.) - 4.5).abs() < 1e-9);
    }
}
