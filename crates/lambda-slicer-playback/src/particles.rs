//! The client's particles, the dots and short streaks its effects throw out
//!
//! A dot flies along its velocity in a colour of the game's palette, pulled
//! down by some share of the server's gravity, and fades out over the last
//! sixteenth of a second of its life. An explosion's dots fly out faster or
//! slow down while they go through a ramp of fiery colours. A flickering
//! dot cycles through a ramp of sparks and shows in about one step in four.
//! A streak is a spark flying out and falling under the whole gravity, its
//! sideways speed slowing, drawn back along its velocity in the sparks'
//! orange and fading out over its last half second
//!
//! Effects make them as the engine does, as Xash3D restates it (`R_FlickerParticles`,
//! `R_BulletImpactParticles`, `R_SparkStreaks`, `R_RunParticleEffect`,
//! `R_ParticleExplosion`, `CL_ThinkParticle` and `CL_DrawTracers`), not yet
//! compared with the game. They move as each group moves time on, so how
//! they look depends a little on the steps, as the game's depend on its
//! frames. Each one's random choices come from a seed of its own, so a seek
//! finds the same ones in the same places

use crate::events::Random;
use crate::time::SourceTime;

/// As many dots and streaks as the engine keeps at once, Quake's default
const MAX_PARTICLES: usize = 2048;

/// The colours an explosion's dots go through, as places in the palette
const RAMP1: [u8; 8] = [0x6f, 0x6d, 0x6b, 0x69, 0x67, 0x65, 0x63, 0x61];
const RAMP2: [u8; 8] = [0x6f, 0x6e, 0x6d, 0x6c, 0x6b, 0x6a, 0x68, 0x66];
/// The colours a flickering dot cycles through (`gSparkRamp`)
const SPARK_RAMP: [u8; 9] = [0xfe, 0xfd, 0xfc, 0x6f, 0x6e, 0x6d, 0x6c, 0x67, 0x60];

/// A streak's colour (`gTracerColors`, the yellow orange of sparks) and its
/// width either side of its line (`gTracerSize` of a falling one)
pub const STREAK_COLOR: [u8; 3] = [255, 167, 17];
pub const STREAK_WIDTH: f32 = 0.5;
/// How much of a streak's flight it is drawn as (`ramp` of a spark)
const STREAK_LENGTH: f32 = 0.5;

/// How a dot moves (`ptype_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Motion {
    /// `pt_grav`, the whole gravity
    Gravity,
    /// `pt_slowgrav`, a twentieth of it
    SlowGravity,
    /// `pt_explode`, faster and faster through `RAMP1`
    Explode,
    /// `pt_explode2`, slower and slower through `RAMP2`
    Explode2,
    /// `pt_blob` and `pt_blob2` as the game makes them, through
    /// `SPARK_RAMP` and hidden three steps in four
    Flicker,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    pub origin: [f32; 3],
    /// Its place in the game's palette
    pub color: u8,
    velocity: [f32; 3],
    motion: Motion,
    /// How far through its colours it is
    ramp: f32,
    die: SourceTime,
    /// A flickering dot hidden this step
    hidden: bool,
    random: u64,
}

impl Particle {
    /// How opaque it is at `at`, 0 to 1, none when it is not drawn
    pub fn alpha(&self, at: SourceTime) -> Option<f32> {
        if self.hidden {
            return None;
        }

        let alpha = ((self.die.0 - at.0) * 16.).min(1.) as f32;
        (alpha > 0.).then_some(alpha)
    }
}

/// A spark's streak (`R_AllocTracer`)
#[derive(Clone, Debug, PartialEq)]
pub struct Streak {
    pub origin: [f32; 3],
    velocity: [f32; 3],
    /// 0 to 1
    pub alpha: f32,
    die: SourceTime,
}

impl Streak {
    /// Where it runs at `at`, from its tail to its head
    pub fn line(&self, at: SourceTime) -> ([f32; 3], [f32; 3]) {
        let left = ((self.die.0 - at.0) as f32).clamp(0., 0.1);
        let reach = STREAK_LENGTH * left;
        let head = [0, 1, 2].map(|k| self.origin[k] + self.velocity[k] * reach);
        (self.origin, head)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Particles {
    pub dots: Vec<Particle>,
    pub streaks: Vec<Streak>,
}

impl Particles {
    /// Whether the engine has room for another
    fn full(&self) -> bool {
        self.dots.len() + self.streaks.len() >= MAX_PARTICLES
    }

    /// A new dot at `origin`, none when the engine would have no room
    fn dot(
        &mut self,
        origin: [f32; 3],
        die: SourceTime,
        motion: Motion,
        random: &mut Random,
    ) -> Option<&mut Particle> {
        if self.full() {
            return None;
        }
        self.dots.push(Particle {
            origin,
            color: 0,
            velocity: [0.; 3],
            motion,
            ramp: 0.,
            die,
            hidden: false,
            random: random.next(),
        });
        self.dots.last_mut()
    }

    /// `count` sparks flying out of `origin`, each way at a speed from
    /// `low` to `high` (`R_SparkStreaks`)
    pub(crate) fn sparks(
        &mut self,
        origin: [f32; 3],
        count: u32,
        low: f32,
        high: f32,
        now: SourceTime,
        seed: u64,
    ) {
        let mut random = Random(seed);
        for _ in 0..count {
            if self.full() {
                return;
            }
            let velocity = [0; 3].map(|_| random.float(low, high));
            let life = random.float(0.1, 0.5);
            self.streaks.push(Streak {
                origin,
                velocity,
                alpha: 1.,
                die: SourceTime(now.0 + f64::from(life)),
            });
        }
    }

    /// Fifteen sparks rising out of an explosion and flickering as they
    /// fall (`R_FlickerParticles`)
    pub(crate) fn flicker(&mut self, origin: [f32; 3], now: SourceTime, seed: u64) {
        let mut random = Random(seed);
        let die = SourceTime(now.0 + 2.);
        for _ in 0..15 {
            let velocity = [
                random.float(-32., 32.),
                random.float(-32., 32.),
                random.float(80., 143.),
            ];
            let Some(p) = self.dot(origin, die, Motion::Flicker, &mut random) else {
                return;
            };
            p.velocity = velocity;
            p.color = 254;
        }
    }

    /// A bullet's dust where it hit a wall, more of it the nearer the
    /// recorder's `eye`, and two sparks (`R_BulletImpactParticles`)
    pub(crate) fn bullet_impact(
        &mut self,
        origin: [f32; 3],
        eye: [f32; 3],
        now: SourceTime,
        seed: u64,
    ) {
        let mut random = Random(seed);
        let to = [0, 1, 2].map(|k| origin[k] - eye[k]);
        let distance = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2])
            .sqrt()
            .min(1000.);
        let quantity = (((1000. - distance) / 100.) as u32).max(1);
        // Darker the more there is, from the palette's first few greys
        let color = (30 * quantity / 100) as u8;
        self.sparks(origin, 2, -200., 200., now, random.next());

        let die = SourceTime(now.0 + 0.5);
        for _ in 0..quantity * 4 {
            let heading = [0; 3].map(|_| random.float(-1., 1.));
            let speed = random.float(50., 100.);
            let Some(p) = self.dot(origin, die, Motion::Gravity, &mut random) else {
                return;
            };
            p.velocity = heading.map(|v| v * speed);
            p.color = color;
        }
    }

    /// `count` dots of `color` and the seven after it in the palette,
    /// drifting along `direction` and falling slowly. A count of 1024 is
    /// an explosion instead (`R_RunParticleEffect`)
    pub(crate) fn effect(
        &mut self,
        origin: [f32; 3],
        direction: [f32; 3],
        color: u8,
        count: u32,
        now: SourceTime,
        seed: u64,
    ) {
        if count == 1024 {
            self.explosion(origin, now, seed);
            return;
        }

        let mut random = Random(seed);
        for _ in 0..count {
            let shade = (color & !7) + random.long(0, 7) as u8;
            let die = SourceTime(now.0 + f64::from(random.float(0.1, 0.4)));
            let at = origin.map(|v| v + random.float(-8., 8.));
            let Some(p) = self.dot(at, die, Motion::SlowGravity, &mut random) else {
                return;
            };
            p.velocity = direction.map(|v| v * 15.);
            p.color = shade;
        }
    }

    /// Quake's rocket explosion, 1024 dots flying out through fiery
    /// colours (`R_ParticleExplosion`)
    fn explosion(&mut self, origin: [f32; 3], now: SourceTime, seed: u64) {
        let mut random = Random(seed);
        let die = SourceTime(now.0 + 5.);
        for i in 0..1024 {
            let motion = if i % 2 == 1 {
                Motion::Explode
            } else {
                Motion::Explode2
            };
            let ramp = random.long(0, 3) as f32;
            let at = origin.map(|v| v + random.float(-16., 16.));
            let velocity = [0; 3].map(|_| random.float(-256., 256.));
            let Some(p) = self.dot(at, die, motion, &mut random) else {
                return;
            };
            p.ramp = ramp;
            p.color = RAMP1[0];
            p.velocity = velocity;
        }
    }

    /// Move everything on by `dt` seconds to `at` under `gravity`, dropping
    /// what has died (`CL_ThinkParticle`, `CL_DrawTracers`)
    pub(crate) fn step(&mut self, at: SourceTime, dt: f32, gravity: f32) {
        self.dots.retain(|p| p.die >= at);
        self.streaks.retain(|s| s.die >= at);

        let pull = dt * gravity * 0.05;
        for p in &mut self.dots {
            p.origin = [0, 1, 2].map(|k| p.origin[k] + p.velocity[k] * dt);
            match p.motion {
                Motion::Gravity => p.velocity[2] -= pull * 20.,
                Motion::SlowGravity => p.velocity[2] -= pull,
                Motion::Explode => {
                    p.ramp += dt * 10.;
                    match RAMP1.get(p.ramp as usize) {
                        Some(&color) => p.color = color,
                        None => p.die = at,
                    }
                    p.velocity = p.velocity.map(|v| v + v * dt * 4.);
                    p.velocity[2] -= pull;
                }
                Motion::Explode2 => {
                    p.ramp += dt * 15.;
                    match RAMP2.get(p.ramp as usize) {
                        Some(&color) => p.color = color,
                        None => p.die = at,
                    }
                    p.velocity = p.velocity.map(|v| v - v * dt);
                    p.velocity[2] -= pull;
                }
                Motion::Flicker => {
                    p.ramp += dt * 10.;
                    if p.ramp >= SPARK_RAMP.len() as f32 {
                        p.ramp = 0.;
                    }
                    p.color = SPARK_RAMP[p.ramp as usize];
                    p.velocity = p.velocity.map(|v| v - v * dt * 0.5);
                    p.velocity[2] -= pull * 5.;
                    let mut random = Random(p.random);
                    p.hidden = random.long(0, 3) != 0;
                    p.random = random.0;
                }
            }
        }

        let slowing = (1. - dt * 0.9).max(0.);
        for s in &mut self.streaks {
            s.origin = [0, 1, 2].map(|k| s.origin[k] + s.velocity[k] * dt);
            s.velocity[0] *= slowing;
            s.velocity[1] *= slowing;
            s.velocity[2] -= dt * gravity;
            s.alpha = ((s.die.0 - at.0) * 2.).min(1.) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explosions_sparks_rise_flicker_and_fall() {
        let mut particles = Particles::default();
        particles.flicker([0.; 3], SourceTime(0.), 7);
        assert_eq!(particles.dots.len(), 15);
        assert!(
            particles
                .dots
                .iter()
                .all(|p| p.alpha(SourceTime(0.)) == Some(1.))
        );

        // A hundred steps a second for a second, each shown about one step
        // in four
        let mut shown = 0;
        for step in 1..=100 {
            let at = SourceTime(f64::from(step) * 0.01);
            particles.step(at, 0.01, 800.);
            shown += particles
                .dots
                .iter()
                .filter(|p| p.alpha(at).is_some())
                .count();
        }
        assert!((200..550).contains(&shown), "{shown}");
        // Rising at first, then pulled back down
        assert!(particles.dots.iter().all(|p| p.velocity[2] < 80.));

        particles.step(SourceTime(2.01), 1.01, 800.);
        assert!(particles.dots.is_empty());
    }

    #[test]
    fn a_bullet_leaves_more_dust_nearer_the_eye() {
        let mut near = Particles::default();
        near.bullet_impact([0.; 3], [100., 0., 0.], SourceTime(0.), 1);
        let mut far = Particles::default();
        far.bullet_impact([0.; 3], [2000., 0., 0.], SourceTime(0.), 1);
        assert_eq!((near.dots.len(), near.streaks.len()), (36, 2));
        assert_eq!((far.dots.len(), far.streaks.len()), (4, 2));
        assert_eq!(near.dots[0].color, 2);
        assert_eq!(far.dots[0].color, 0);
    }

    #[test]
    fn a_streak_shortens_and_fades_as_it_ends() {
        let mut particles = Particles::default();
        particles.sparks([0.; 3], 1, 100., 100., SourceTime(0.), 3);
        let streak = &particles.streaks[0];
        let life = streak.die.0;
        let (tail, head) = streak.line(SourceTime(0.));
        assert_eq!(tail, [0.; 3]);
        assert_eq!(head, [5.; 3]);

        let end = SourceTime(life - 0.05);
        particles.step(end, end.0 as f32, 800.);
        let streak = &particles.streaks[0];
        assert!((streak.alpha - 0.1).abs() < 1e-3);
        let (tail, head) = streak.line(end);
        assert!((head[0] - tail[0]) < 2.5);
    }

    #[test]
    fn the_engine_keeps_so_many_at_once() {
        let mut particles = Particles::default();
        particles.effect([0.; 3], [0.; 3], 0, 1024, SourceTime(0.), 1);
        particles.effect([0.; 3], [0.; 3], 0, 1024, SourceTime(0.), 2);
        particles.sparks([0.; 3], 10, -1., 1., SourceTime(0.), 3);
        assert_eq!(particles.dots.len(), MAX_PARTICLES);
        assert!(particles.streaks.is_empty());
    }
}
