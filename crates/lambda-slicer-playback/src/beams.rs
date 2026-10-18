//! Beams, trails and tracers (implementation plan §13.6)
//!
//! The server asks for them with `svc_temp_entity`, and the client's MP5
//! handler fires tracers itself. How each is built and how long it lasts is
//! the engine's as Xash3D restates it, not yet compared with the game
//! (docs/viewer-profile.md, "Beams and tracers").
//!
//! A trail follows an entity, such as a rocket, leaving a point behind each
//! time the entity has moved 32 units (`R_BeamFollow`). Each point fades out
//! over the trail's life, and a trail whose entity is gone stops growing and
//! ends with its last point. A beam joins two points for its life. A
//! tracer is a streak flying from just past where it was fired towards
//! where it ends, at 6000 units a second (`R_TracerEffect`), drawn as long
//! as a tenth of a second of its flight
//!
//! Playback keeps where each is. Turning them into strips facing the view is
//! the renderer's

use std::collections::VecDeque;
use std::sync::Arc;

use crate::events::{Random, mix};
use crate::scene::{EntityId, Scene};
use crate::time::SourceTime;

/// How far an entity moves before its trail leaves a point
const TRAIL_STEP: f32 = 32.;
/// A tracer's speed (`tracerspeed`), how far past where it was fired it
/// starts (`traceroffset`), and how much of its flight it is drawn as
/// (`tracerlength`)
const TRACER_SPEED: f32 = 6000.;
const TRACER_OFFSET: f32 = 30.;
pub const TRACER_LENGTH: f32 = 0.8;
/// A trail's every point, at most, as the engine's particles run out
const MAX_TRAIL_POINTS: usize = 512;

/// `svc_temp_entity` kinds handled
mod kind {
    pub const BEAM_POINTS: u8 = 0;
    pub const TRACER: u8 = 6;
    pub const BEAM_FOLLOW: u8 = 22;
    pub const KILL_BEAM: u8 = 99;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trail {
    pub sprite: Arc<str>,
    /// Followed while it exists
    entity: Option<EntityId>,
    /// Where the entity is now, the trail's head
    pub head: Option<[f32; 3]>,
    /// Newest first, each with when it fades out
    pub points: VecDeque<([f32; 3], SourceTime)>,
    pub life: f32,
    /// Units either side of the line
    pub width: f32,
    /// 0 to 1, the brightness folded in
    pub color: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Beam {
    pub sprite: Arc<str>,
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub born: SourceTime,
    pub die: SourceTime,
    pub width: f32,
    pub color: [f32; 3],
    pub start_frame: f32,
    pub framerate: f32,
    /// How fast its texture slides along it, whole textures a second
    pub scroll: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tracer {
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub born: SourceTime,
    pub die: SourceTime,
}

impl Tracer {
    /// Where its streak runs at `at`, from its tail to its head
    pub fn streak(&self, at: SourceTime) -> ([f32; 3], [f32; 3]) {
        let flown = (at.0 - self.born.0).max(0.) as f32;
        let left = ((self.die.0 - at.0) as f32).clamp(0., 0.1);
        let tail = [0, 1, 2].map(|k| self.origin[k] + self.velocity[k] * flown);
        let head = [0, 1, 2].map(|k| tail[k] + self.velocity[k] * TRACER_LENGTH * left);
        (tail, head)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Beams {
    pub trails: Vec<Trail>,
    pub beams: Vec<Beam>,
    pub tracers: Vec<Tracer>,
    /// How many tracers the server has asked for, which seeds the next
    made: u64,
}

fn short(b: &[u8], at: usize) -> i16 {
    i16::from_le_bytes([
        b.get(at).copied().unwrap_or(0),
        b.get(at + 1).copied().unwrap_or(0),
    ])
}

fn coord(b: &[u8], at: usize) -> f32 {
    f32::from(short(b, at)) / 8.
}

fn vector(b: &[u8], at: usize) -> [f32; 3] {
    [coord(b, at), coord(b, at + 2), coord(b, at + 4)]
}

impl Beams {
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    /// A tracer from `start` towards `end` at `now`
    pub(crate) fn tracer(&mut self, start: [f32; 3], end: [f32; 3], now: SourceTime, seed: u64) {
        let d = [0, 1, 2].map(|k| end[k] - start[k]);
        let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if length == 0. {
            return;
        }
        let direction = d.map(|v| v / length);
        let offset = Random(seed).float(-10., 9.) + TRACER_OFFSET;
        self.tracers.push(Tracer {
            origin: [0, 1, 2].map(|k| start[k] + direction[k] * offset),
            velocity: direction.map(|v| v * TRACER_SPEED),
            born: now,
            die: SourceTime(now.0 + f64::from(length / TRACER_SPEED)),
        });
    }

    /// A beam, trail or tracer from an `svc_temp_entity` of `kind`, if it
    /// makes one
    pub(crate) fn message(
        &mut self,
        kind: u8,
        body: &[u8],
        now: SourceTime,
        scene: &Scene,
        model: &dyn Fn(i16) -> Option<Arc<str>>,
    ) {
        let byte = |at: usize| f32::from(body.get(at).copied().unwrap_or(0));
        let colour =
            |at: usize, brightness: f32| [0, 1, 2].map(|k| byte(at + k) / 255. * brightness);
        match kind {
            kind::BEAM_POINTS => {
                let Some(sprite) = model(short(body, 12)) else {
                    return;
                };
                let life = byte(16) * 0.1;
                self.beams.push(Beam {
                    sprite,
                    start: vector(body, 0),
                    end: vector(body, 6),
                    born: now,
                    die: SourceTime(now.0 + f64::from(life)),
                    width: byte(17) * 0.1,
                    color: colour(19, byte(22) / 255.),
                    start_frame: byte(14),
                    framerate: byte(15),
                    scroll: byte(23) * 0.1,
                });
            }
            kind::TRACER => {
                let seed = mix(now.0.to_bits(), self.made);
                self.made += 1;
                self.tracer(vector(body, 0), vector(body, 6), now, seed);
            }
            kind::BEAM_FOLLOW => {
                // The entity's number in the low 12 bits, an attachment
                // above them
                let number = (short(body, 0) as u16) & 0x0fff;
                let Some(sprite) = model(short(body, 2)) else {
                    return;
                };
                let entity = scene.entities.iter().find(|e| e.id.number == number);
                self.trails.push(Trail {
                    sprite,
                    entity: entity.map(|e| e.id),
                    head: entity.map(|e| e.origin),
                    points: VecDeque::new(),
                    life: byte(4) * 0.1,
                    width: byte(5),
                    color: colour(6, byte(9) / 255.),
                });
            }
            kind::KILL_BEAM => {
                let number = short(body, 0) as u16;
                self.trails
                    .retain(|t| t.entity.is_none_or(|e| e.number != number));
            }
            _ => {}
        }
    }

    /// Move everything on to `at`, trails following their entities
    pub(crate) fn step(&mut self, at: SourceTime, scene: &Scene) {
        self.tracers.retain(|t| t.die >= at);
        self.beams.retain(|b| b.die >= at);
        for trail in &mut self.trails {
            trail.points.retain(|(_, die)| *die >= at);
            let followed = trail
                .entity
                .and_then(|id| scene.entities.iter().find(|e| e.id == id));
            let Some(e) = followed else {
                trail.entity = None;
                trail.head = None;
                continue;
            };
            trail.head = Some(e.origin);
            let far = trail.points.front().is_none_or(|(p, _)| {
                let d = [0, 1, 2].map(|k| p[k] - e.origin[k]);
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() >= TRAIL_STEP
            });
            if far {
                let die = SourceTime(at.0 + f64::from(trail.life));
                trail.points.push_front((e.origin, die));
                trail.points.truncate(MAX_TRAIL_POINTS);
            }
        }
        self.trails
            .retain(|t| t.entity.is_some() || !t.points.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tracer_flies_its_length_and_is_gone() {
        let mut beams = Beams::default();
        beams.tracer([0.; 3], [600., 0., 0.], SourceTime(1.), 3);
        let t = beams.tracers[0];
        // Starting 20 to 39 units out, a tenth of a second of flight at most
        assert!((20. ..=39.).contains(&t.origin[0]));
        assert!((t.die.0 - 1.1).abs() < 1e-6);
        let (tail, head) = t.streak(SourceTime(1.05));
        assert!((tail[0] - (t.origin[0] + 300.)).abs() < 1e-2);
        assert!((head[0] - tail[0] - 6000. * 0.8 * 0.05).abs() < 1e-1);
        beams.step(SourceTime(1.2), &Scene::default());
        assert!(beams.tracers.is_empty());
    }

    #[test]
    fn a_beam_lasts_its_life() {
        let mut beams = Beams::default();
        let mut body = vec![0u8; 24];
        body[12] = 7;
        body[16] = 5;
        body[17] = 30;
        body[19..23].copy_from_slice(&[255, 128, 0, 255]);
        let model = |_| Some(Arc::from("sprites/laserbeam.spr"));
        beams.message(
            kind::BEAM_POINTS,
            &body,
            SourceTime(2.),
            &Scene::default(),
            &model,
        );
        let b = &beams.beams[0];
        assert_eq!((b.width, b.die), (3., SourceTime(2.5)));
        assert!((b.color[1] - 128. / 255.).abs() < 1e-6);
        beams.step(SourceTime(2.6), &Scene::default());
        assert!(beams.beams.is_empty());
    }
}
