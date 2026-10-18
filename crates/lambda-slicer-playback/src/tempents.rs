//! The client's temporary entities (implementation plan §13.6)
//!
//! A temporary entity is a model the client makes and moves itself, such as
//! a shell a weapon throws out (`R_TempModel`). Playback moves them as each
//! group moves time on, as the client's `HUD_TempEntUpdate` does each
//! frame. On along its velocity, turned by its spin, stopped where a trace
//! through the world meets something, its velocity reflected and halved
//! there, and pulled down by the server's gravity. Landing slowly on a
//! floor stops it for good. A shell plays a bounce sound on some of its
//! bounces (`CL_TempEntPlaySound`). Each lives 2.5 seconds.
//!
//! Its random spin and sounds come from a seed of its own, so a seek finds
//! the same shells in the same places

use std::sync::Arc;

use crate::audio::CHAN_AUTO;
use crate::events::{ATTN_NORM, PITCH_NORM, Random, SoundCall};
use crate::time::SourceTime;
use crate::trace::Hull;

/// How long a shell lies about, in seconds
const SHELL_LIFE: f64 = 2.5;

/// What a temporary model sounds like when it hits something
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bounce {
    /// `TE_BOUNCE_SHELL`
    Shell,
    /// `TE_BOUNCE_SHOTSHELL`
    ShotShell,
}

/// A shell as a handler throws it out
#[derive(Clone, Debug, PartialEq)]
pub struct Throw {
    pub model: &'static str,
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    /// Degrees it is turned about the vertical
    pub yaw: f32,
    pub bounce: Bounce,
    /// Chooses its spin and sounds
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TempModel {
    /// Unique within one playback, in the order they were made
    pub id: u64,
    pub model: Arc<str>,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    velocity: [f32; 3],
    /// Degrees a second about each axis
    spin: [f32; 3],
    /// Still turning, falling and colliding, until it comes to rest
    moving: bool,
    bounce: Bounce,
    die: SourceTime,
    random: u64,
}

/// Every temporary entity alive
#[derive(Clone, Debug, Default)]
pub struct TempEntities {
    models: Vec<TempModel>,
    next_id: u64,
    last: Option<SourceTime>,
}

impl TempEntities {
    pub fn models(&self) -> &[TempModel] {
        &self.models
    }

    /// Throw out a shell at `now` (`EV_EjectBrass`)
    pub fn shell(&mut self, throw: &Throw, now: SourceTime) {
        let Throw {
            model,
            origin,
            velocity,
            yaw,
            bounce,
            seed,
        } = *throw;
        let mut random = Random(seed);
        let spin = [0; 3].map(|_| random.float(-255., 255.));
        self.models.push(TempModel {
            id: self.next_id,
            model: Arc::from(model),
            origin,
            angles: [0., yaw, 0.],
            velocity,
            spin,
            moving: true,
            bounce,
            die: SourceTime(now.0 + SHELL_LIFE),
            random: random.0,
        });
        self.next_id += 1;
    }

    /// Move everything on to `at`, colliding with `world` under `gravity`,
    /// and the bounce sounds on the way
    pub fn step(&mut self, at: SourceTime, world: Option<&Hull>, gravity: f32) -> Vec<SoundCall> {
        let dt = self.last.map_or(0., |last| (at.0 - last.0).clamp(0., 1.)) as f32;
        self.last = Some(at);
        self.models.retain(|m| m.die >= at);
        if dt <= 0. {
            return Vec::new();
        }
        let pull = -dt * gravity;
        let mut sounds = Vec::new();
        for m in &mut self.models {
            if !m.moving {
                continue;
            }
            let from = m.origin;
            m.origin = [0, 1, 2].map(|k| m.origin[k] + m.velocity[k] * dt);
            m.angles = [0, 1, 2].map(|k| m.angles[k] + m.spin[k] * dt);

            if let Some(world) = world {
                let trace = world.trace(from, m.origin);
                if trace.fraction < 1. {
                    // At the contact point, with the velocity damped
                    let f = trace.fraction * dt;
                    m.origin = [0, 1, 2].map(|k| from[k] + m.velocity[k] * f);
                    let mut damp = 0.5;
                    if trace.normal[2] > 0.9 && m.velocity[2] <= 0. && m.velocity[2] >= pull * 3. {
                        // Landed slowly on a floor, so it stays there
                        damp = 0.;
                        m.moving = false;
                        m.angles[0] = 0.;
                        m.angles[2] = 0.;
                    }
                    let mut random = Random(m.random);
                    if let Some(sound) = bounce_sound(m, damp, &mut random) {
                        sounds.push(sound);
                    }
                    m.random = random.0;
                    if damp != 0. {
                        let proj = m.velocity[0] * trace.normal[0]
                            + m.velocity[1] * trace.normal[1]
                            + m.velocity[2] * trace.normal[2];
                        m.velocity = [0, 1, 2].map(|k| m.velocity[k] - proj * 2. * trace.normal[k]);
                        m.angles[1] = -m.angles[1];
                    }
                    m.velocity = m.velocity.map(|v| v * damp);
                    m.angles = m.angles.map(|a| a * 0.9);
                }
            }
            if m.moving {
                m.velocity[2] += pull;
            }
        }
        sounds
    }
}

/// The sound of a shell hitting something, for the first bounce that
/// comes down fast and then one in four, louder the faster it falls. None
/// when it comes to rest
fn bounce_sound(m: &TempModel, damp: f32, random: &mut Random) -> Option<SoundCall> {
    let n = random.long(1, 3);
    let sample = match (m.bounce, n) {
        (Bounce::Shell, 1) => "player/pl_shell1.wav",
        (Bounce::Shell, 2) => "player/pl_shell2.wav",
        (Bounce::Shell, _) => "player/pl_shell3.wav",
        (Bounce::ShotShell, 1) => "weapons/sshell1.wav",
        (Bounce::ShotShell, 2) => "weapons/sshell2.wav",
        (Bounce::ShotShell, _) => "weapons/sshell3.wav",
    };
    let fall = m.velocity[2].abs().trunc();
    if fall < 200. && random.long(0, 3) != 0 {
        return None;
    }
    if damp <= 0. {
        return None;
    }
    Some(SoundCall {
        entity: 0,
        channel: CHAN_AUTO,
        sample: Arc::from(sample),
        volume: (fall / 350.).min(1.),
        attenuation: ATTN_NORM,
        flags: 0,
        pitch: PITCH_NORM,
        origin: m.origin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::{Child, SOLID};

    fn floor() -> Hull {
        Hull {
            planes: vec![([0., 0., 1.], 0.)],
            nodes: vec![(0, [Child::Contents(-1), Child::Contents(SOLID)])],
            root: Child::Node(0),
        }
    }

    #[test]
    fn a_shell_falls_bounces_and_comes_to_rest_on_the_floor() {
        let world = floor();
        let mut t = TempEntities::default();
        let start = SourceTime(0.);
        t.step(start, Some(&world), 800.);
        let throw = Throw {
            model: "models/shell.mdl",
            origin: [0., 0., 40.],
            velocity: [60., 0., 120.],
            yaw: 90.,
            bounce: Bounce::Shell,
            seed: 7,
        };
        t.shell(&throw, start);
        assert_eq!(t.models()[0].angles, [0., 90., 0.]);

        let mut lowest = f32::MAX;
        let mut bounced = 0;
        for i in 1..=200 {
            let sounds = t.step(SourceTime(f64::from(i) * 0.01), Some(&world), 800.);
            bounced += sounds.len();
            lowest = lowest.min(t.models()[0].origin[2]);
        }
        let shell = &t.models()[0];
        // Never through the floor, and lying flat on it at the end
        assert!(lowest > 0.);
        assert!(!shell.moving);
        assert!(shell.origin[2] < 0.1);
        assert_eq!((shell.angles[0], shell.angles[2]), (0., 0.));
        assert!(bounced >= 1);

        // Gone once its life is over
        t.step(SourceTime(2.6), Some(&world), 800.);
        assert!(t.models().is_empty());
    }

    #[test]
    fn the_same_seed_throws_the_same_shell() {
        let run = || {
            let mut t = TempEntities::default();
            t.step(SourceTime(0.), None, 800.);
            let throw = Throw {
                model: "models/shell.mdl",
                origin: [0.; 3],
                velocity: [10., 0., 100.],
                yaw: 0.,
                bounce: Bounce::Shell,
                seed: 3,
            };
            t.shell(&throw, SourceTime(0.));
            t.step(SourceTime(0.5), None, 800.);
            t.models()[0].clone()
        };
        assert_eq!(run(), run());
    }
}
