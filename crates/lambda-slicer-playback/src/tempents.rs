//! The client's temporary entities (implementation plan §13.6)
//!
//! A temporary entity is a sprite or model the client makes and moves
//! itself, such as a shell a weapon throws out or an explosion's sprite.
//! The server asks for most of them with `svc_temp_entity`, and the
//! client's event handlers make others. Playback moves them as each group
//! moves time on, as the client's `HUD_TempEntUpdate` does each frame. On
//! along its velocity, turned by its spin, its sprite animated, stopped
//! where a trace through the world meets something, its velocity reflected
//! and halved there, and pulled down by the server's gravity or half of it.
//! Landing slowly on a floor stops it for good. One that bounces plays a
//! sound on some of its bounces (`CL_TempEntPlaySound`), and one that fades
//! out loses its brightness after its time is up.
//!
//! How the server's kinds are built is the engine's, as Xash3D restates it
//! (`R_Sprite_Explode`, `R_BloodSprite` and the rest), not yet compared with
//! the game (docs/viewer-profile.md, "Temporary entities"). Each entity's
//! random choices come from a seed of its own, so a seek finds the same
//! ones in the same places

use std::sync::Arc;

use crate::audio::CHAN_AUTO;
use crate::events::{ATTN_NORM, PITCH_NORM, Random, SoundCall, mix};
use crate::time::SourceTime;
use crate::trace::Hull;

/// How long a shell lies about, in seconds
const SHELL_LIFE: f32 = 2.5;

/// As many as the engine keeps at once (`MAX_TEMP_ENTITIES`)
const MAX_TEMP_ENTITIES: usize = 500;

/// The game's render modes, as entities give them
pub mod render {
    pub const NORMAL: i64 = 0;
    pub const TEXTURE: i64 = 2;
    pub const GLOW: i64 = 3;
    pub const ALPHA: i64 = 4;
    pub const ADDITIVE: i64 = 5;
}

/// `FTENT_` flags
mod flag {
    pub const ROTATE: u16 = 1;
    pub const GRAVITY: u16 = 2;
    pub const SLOW_GRAVITY: u16 = 4;
    pub const COLLIDE: u16 = 8;
    pub const ANIMATE: u16 = 16;
    pub const FADE: u16 = 32;
    pub const SPARKS: u16 = 64;
    /// Its frames go round at ten a second
    pub const CYCLE: u16 = 128;
}

/// What a temporary model sounds like when it hits something
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bounce {
    /// `TE_BOUNCE_SHELL`
    Shell,
    /// `TE_BOUNCE_SHOTSHELL`
    ShotShell,
    /// A breakable's gibs, by its material (`BOUNCE_GLASS` and the rest)
    Glass,
    Metal,
    Flesh,
    Wood,
    Concrete,
}

/// A sprite's colour
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    /// Its own colours
    None,
    Rgb([u8; 3]),
    /// An index into the game's palette (`gfx/palette.lmp`)
    Palette(u8),
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
pub struct TempEntity {
    /// Unique within one playback, in the order they were made
    pub id: u64,
    /// A sprite or studio model, none for one that only makes others
    pub model: Option<Arc<str>>,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    /// The sprite's frame
    pub frame: f32,
    /// 0 for the sprite's own size
    pub scale: f32,
    /// The game's render mode and amount, 0 to 255
    pub render_mode: i64,
    pub render_amount: f32,
    pub tint: Tint,
    /// A glow that keeps its brightness at any distance
    /// (`kRenderFxNoDissipation`)
    pub steady: bool,
    velocity: [f32; 3],
    /// Degrees a second about each axis
    spin: [f32; 3],
    flags: u16,
    /// The last frame, after which an animation ends
    frame_max: f32,
    framerate: f32,
    base_amount: f32,
    fade_speed: f32,
    bounce: Option<Bounce>,
    /// How much of its speed a bounce keeps, before gravity halves it
    bounce_factor: f32,
    die: SourceTime,
    /// A spark shower's next shower and the life it has left
    sparks: (SourceTime, f32),
    random: u64,
}

impl TempEntity {
    fn new(id: u64, model: Option<&str>, origin: [f32; 3], now: SourceTime, seed: u64) -> Self {
        Self {
            id,
            model: model.map(Arc::from),
            origin,
            angles: [0.; 3],
            frame: 0.,
            scale: 1.,
            render_mode: render::NORMAL,
            render_amount: 255.,
            tint: Tint::None,
            steady: false,
            velocity: [0.; 3],
            spin: [0.; 3],
            flags: 0,
            frame_max: 0.,
            framerate: 0.,
            base_amount: 255.,
            fade_speed: 0.5,
            bounce: None,
            bounce_factor: 1.,
            // The engine's default life
            die: SourceTime(now.0 + 0.75),
            sparks: (now, 0.),
            random: seed,
        }
    }

    /// Whether it is drawn
    pub fn visible(&self) -> bool {
        self.model.is_some() && self.render_amount > 0.
    }
}

/// Every temporary entity alive
#[derive(Clone, Debug, Default)]
pub struct TempEntities {
    entities: Vec<TempEntity>,
    next_id: u64,
    last: Option<SourceTime>,
}

impl TempEntities {
    pub fn entities(&self) -> &[TempEntity] {
        &self.entities
    }

    /// A new entity at `origin`, none when the engine would have no room
    pub(crate) fn alloc(
        &mut self,
        model: Option<&str>,
        origin: [f32; 3],
        now: SourceTime,
    ) -> Option<&mut TempEntity> {
        if self.entities.len() >= MAX_TEMP_ENTITIES {
            return None;
        }
        let seed = mix(now.0.to_bits(), self.next_id);
        self.entities
            .push(TempEntity::new(self.next_id, model, origin, now, seed));
        self.next_id += 1;
        self.entities.last_mut()
    }

    /// A seed for a message's random choices, from when it came and how
    /// many entities came before
    pub(crate) fn seed(&self, now: SourceTime) -> u64 {
        mix(mix(now.0.to_bits(), self.next_id), 0x7e)
    }

    /// Forget everything, as a new session does
    pub(crate) fn clear(&mut self) {
        self.entities.clear();
    }

    /// A model thrown out at `origin`, turned `yaw` degrees and spinning,
    /// living `life` seconds (`R_TempModel`)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn model(
        &mut self,
        model: &str,
        origin: [f32; 3],
        velocity: [f32; 3],
        yaw: f32,
        life: f32,
        bounce: Option<Bounce>,
        now: SourceTime,
        seed: u64,
    ) {
        let Some(e) = self.alloc(Some(model), origin, now) else {
            return;
        };
        let mut random = Random(seed);
        e.spin = [0; 3].map(|_| random.float(-255., 255.));
        e.random = random.0;
        e.angles = [0., yaw, 0.];
        e.velocity = velocity;
        e.bounce = bounce;
        e.flags = flag::COLLIDE | flag::GRAVITY | flag::ROTATE;
        e.die = SourceTime(now.0 + f64::from(life));
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
        self.model(
            model,
            origin,
            velocity,
            yaw,
            SHELL_LIFE,
            Some(bounce),
            now,
            seed,
        );
    }

    /// A sprite playing through its `frames` once at `framerate`, 10 for
    /// none (`R_DefaultSprite`)
    pub(crate) fn sprite(
        &mut self,
        model: &str,
        origin: [f32; 3],
        frames: u32,
        framerate: f32,
        now: SourceTime,
    ) -> Option<&mut TempEntity> {
        let framerate = if framerate == 0. { 10. } else { framerate };
        let e = self.alloc(Some(model), origin, now)?;
        e.frame_max = frames.saturating_sub(1) as f32;
        e.framerate = framerate;
        e.flags |= flag::ANIMATE;
        e.die = SourceTime(now.0 + f64::from(frames as f32 / framerate));
        Some(e)
    }

    /// A steady glow of `model` at `origin`, fading out after `life`
    /// seconds, or after its frames at ten a second and one more when none
    /// (`R_TempSprite` with `kRenderGlow` and `kRenderFxNoDissipation`)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn glow(
        &mut self,
        model: &str,
        frames: u32,
        origin: [f32; 3],
        scale: f32,
        brightness: f32,
        life: f32,
        now: SourceTime,
    ) {
        let Some(e) = self.alloc(Some(model), origin, now) else {
            return;
        };
        e.render_mode = render::GLOW;
        e.steady = true;
        e.scale = scale;
        e.render_amount = brightness * 255.;
        e.base_amount = e.render_amount;
        e.frame_max = frames.saturating_sub(1) as f32;
        e.flags = flag::FADE;
        let life = if life != 0. {
            life
        } else {
            frames as f32 * 0.1 + 1.
        };
        e.die = SourceTime(now.0 + f64::from(life));
    }

    /// `count` steady glows of `model` thrown from along the line from
    /// `start` towards `towards` at `speed` with some spread, cycling their
    /// frames, bouncing under half gravity and fading out after `life`
    /// seconds and up to four more (`R_Sprite_Trail`)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sprite_trail(
        &mut self,
        model: &str,
        frames: u32,
        start: [f32; 3],
        towards: [f32; 3],
        count: u32,
        life: f32,
        size: f32,
        spread: f32,
        brightness: f32,
        speed: f32,
        now: SourceTime,
        seed: u64,
    ) {
        let delta = [0, 1, 2].map(|k| towards[k] - start[k]);
        let length = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        let direction = delta.map(|v| if length > 0. { v / length } else { 0. });
        let spread = spread / 256.;
        let mut random = Random(seed);
        for i in 0..count {
            let along = if i == 0 {
                0.
            } else {
                i as f32 / (count as f32 - 1.)
            };
            let origin = [0, 1, 2].map(|k| start[k] + delta[k] * along);
            let Some(e) = self.alloc(Some(model), origin, now) else {
                return;
            };
            e.flags = flag::COLLIDE | flag::CYCLE | flag::FADE | flag::SLOW_GRAVITY;
            e.velocity = direction.map(|d| d * speed + random.float(-127., 128.) * spread);
            e.scale = size;
            e.render_mode = render::GLOW;
            e.steady = true;
            e.render_amount = brightness;
            e.base_amount = brightness;
            e.frame = random.long(0, frames as i32 - 1).max(0) as f32;
            e.frame_max = frames.saturating_sub(1) as f32;
            e.die = SourceTime(now.0 + f64::from(life + random.float(0., 4.)));
        }
    }

    /// A ricochet's glint, fading away at once (`R_RicochetSprite`)
    pub(crate) fn ricochet_sprite(&mut self, origin: [f32; 3], scale: f32, now: SourceTime) {
        let Some(e) = self.alloc(Some(RICOCHET), origin, now) else {
            return;
        };
        let mut random = Random(e.random);
        e.render_mode = render::GLOW;
        e.render_amount = 200.;
        e.base_amount = 200.;
        e.steady = true;
        e.scale = scale;
        e.flags = flag::FADE;
        e.fade_speed = 8.;
        e.die = now;
        e.angles[2] = 45. * random.long(0, 7) as f32;
        e.random = random.0;
    }

    /// Sparks flying out of `origin`, glinting every tenth of a second as
    /// they go (`R_SparkShower`)
    pub(crate) fn spark_shower(&mut self, origin: [f32; 3], now: SourceTime) {
        let Some(e) = self.alloc(None, origin, now) else {
            return;
        };
        let mut random = Random(e.random);
        e.velocity = [
            random.float(-300., 300.),
            random.float(-300., 300.),
            random.float(-200., 200.),
        ];
        e.flags = flag::SLOW_GRAVITY | flag::COLLIDE | flag::SPARKS;
        e.sparks = (now, random.float(0.5, 1.5));
        e.die = SourceTime(now.0 + 0.5);
        e.random = random.0;
    }

    /// Move everything on to `at`, colliding with `world` under `gravity`,
    /// and the bounce sounds on the way
    pub fn step(&mut self, at: SourceTime, world: Option<&Hull>, gravity: f32) -> Vec<SoundCall> {
        let dt = self.last.map_or(0., |last| (at.0 - last.0).clamp(0., 1.)) as f32;
        self.last = Some(at);
        if dt <= 0. {
            return Vec::new();
        }
        let pull = -dt * gravity;
        let mut sounds = Vec::new();
        let mut glints = Vec::new();
        self.entities.retain_mut(|e| {
            let life = (e.die.0 - at.0) as f32;
            if life < 0. {
                if e.flags & flag::FADE == 0 {
                    return false;
                }
                if e.render_mode == render::NORMAL {
                    e.render_mode = render::TEXTURE;
                }
                e.render_amount = e.base_amount * (1. + life * e.fade_speed);
                if e.render_amount <= 0. {
                    return false;
                }
            }

            if e.flags & flag::SPARKS != 0 && at >= e.sparks.0 {
                glints.push(e.origin);
                e.sparks.1 -= 0.1;
                if e.sparks.1 <= 0. {
                    e.die = at;
                } else {
                    e.die = SourceTime(at.0 + 0.5);
                    e.sparks.0 = SourceTime(at.0 + 0.1);
                }
            }

            let from = e.origin;
            e.origin = [0, 1, 2].map(|k| e.origin[k] + e.velocity[k] * dt);
            if e.flags & flag::ANIMATE != 0 {
                e.frame += dt * e.framerate;
                if e.frame >= e.frame_max {
                    // Played through once, so it is gone
                    return false;
                }
            } else if e.flags & flag::CYCLE != 0 {
                e.frame += dt * 10.;
                if e.frame >= e.frame_max {
                    e.frame = e.frame.fract();
                }
            }
            if e.flags & flag::ROTATE != 0 {
                e.angles = [0, 1, 2].map(|k| e.angles[k] + e.spin[k] * dt);
            }

            if e.flags & flag::COLLIDE != 0
                && let Some(world) = world
            {
                let trace = world.trace(from, e.origin);
                if trace.fraction < 1. {
                    // At the contact point, with the velocity damped
                    let f = trace.fraction * dt;
                    e.origin = [0, 1, 2].map(|k| from[k] + e.velocity[k] * f);
                    let mut damp = e.bounce_factor;
                    if e.flags & (flag::GRAVITY | flag::SLOW_GRAVITY) != 0 {
                        damp *= 0.5;
                        if trace.normal[2] > 0.9
                            && e.velocity[2] <= 0.
                            && e.velocity[2] >= pull * 3.
                        {
                            // Landed slowly on a floor, so it stays there
                            damp = 0.;
                            e.flags &= !(flag::ROTATE
                                | flag::GRAVITY
                                | flag::SLOW_GRAVITY
                                | flag::COLLIDE);
                            e.angles[0] = 0.;
                            e.angles[2] = 0.;
                        }
                    }
                    if let Some(sound) = bounce_sound(e, damp) {
                        sounds.push(sound);
                    }
                    if damp != 0. {
                        let proj = e.velocity[0] * trace.normal[0]
                            + e.velocity[1] * trace.normal[1]
                            + e.velocity[2] * trace.normal[2];
                        e.velocity = [0, 1, 2].map(|k| e.velocity[k] - proj * 2. * trace.normal[k]);
                        e.angles[1] = -e.angles[1];
                    }
                    if damp != 1. {
                        e.velocity = e.velocity.map(|v| v * damp);
                        e.angles = e.angles.map(|a| a * 0.9);
                    }
                }
            }
            if e.flags & flag::GRAVITY != 0 {
                e.velocity[2] += pull;
            } else if e.flags & flag::SLOW_GRAVITY != 0 {
                e.velocity[2] += pull * 0.5;
            }
            true
        });
        // A spark shower's glints, `R_SparkEffect`'s sprite without its
        // streaks
        for origin in glints {
            let scale = Random(mix(at.0.to_bits(), self.next_id)).float(0.5, 1.);
            self.ricochet_sprite(origin, scale, at);
        }
        sounds
    }
}

/// `svc_temp_entity` kinds handled
pub mod kind {
    pub const EXPLOSION: u8 = 3;
    pub const SMOKE: u8 = 5;
    pub const SPARKS: u8 = 9;
    pub const SPRITE: u8 = 17;
    pub const MODEL: u8 = 106;
    pub const BREAK_MODEL: u8 = 108;
    pub const GUNSHOT_DECAL: u8 = 109;
    pub const SPRITE_SPRAY: u8 = 110;
    pub const ARMOR_RICOCHET: u8 = 111;
    pub const BLOOD_SPRITE: u8 = 115;
}

/// `TE_EXPLFLAG_` flags
const NO_ADDITIVE: u8 = 1;
const NO_SOUND: u8 = 4;

/// A temporary entity message's body, read in the order it was written
struct Body<'a>(&'a [u8], usize);

impl Body<'_> {
    fn byte(&mut self) -> u8 {
        let b = self.0.get(self.1).copied().unwrap_or(0);
        self.1 += 1;
        b
    }

    fn short(&mut self) -> i16 {
        i16::from_le_bytes([self.byte(), self.byte()])
    }

    /// `WRITE_COORD`, an eighth of a unit
    fn coord(&mut self) -> f32 {
        f32::from(self.short()) / 8.
    }

    fn vector(&mut self) -> [f32; 3] {
        [self.coord(), self.coord(), self.coord()]
    }
}

/// What a message needs of the session and the game's files
pub(crate) struct Lookup<'a> {
    /// A model's path by its precache index
    pub model: &'a dyn Fn(i16) -> Option<Arc<str>>,
    /// A sprite's frame count by its path
    pub frames: &'a dyn Fn(&str) -> Option<u32>,
    /// The map, where gibs are not put inside solid
    pub world: Option<&'a Hull>,
}

impl TempEntities {
    /// An `svc_temp_entity` of `kind` that arrived at `now`, and the sounds
    /// it starts
    pub(crate) fn message(
        &mut self,
        kind: u8,
        body: &[u8],
        now: SourceTime,
        lookup: &Lookup,
    ) -> Vec<SoundCall> {
        let mut b = Body(body, 0);
        let mut random = Random(self.seed(now));
        let mut sounds = Vec::new();
        let sprite_of = |index: i16| {
            let model = (lookup.model)(index).filter(|m| m.ends_with(".spr"))?;
            let frames = (lookup.frames)(&model)?;
            Some((model, frames))
        };
        match kind {
            kind::EXPLOSION => {
                let origin = b.vector();
                let index = b.short();
                let scale = f32::from(b.byte()) * 0.1;
                let framerate = f32::from(b.byte());
                let flags = b.byte();
                if let Some((model, frames)) = sprite_of(index)
                    && let Some(e) = self.sprite(&model, origin, frames, framerate, now)
                {
                    // `R_Sprite_Explode`, rising out of where it went off
                    (e.render_mode, e.render_amount) = if flags & NO_ADDITIVE != 0 {
                        (render::NORMAL, 255.)
                    } else {
                        (render::ADDITIVE, 180.)
                    };
                    e.base_amount = e.render_amount;
                    e.velocity[2] = 8.;
                    e.origin[2] += 10.;
                    e.scale = scale;
                }
                if flags & NO_SOUND == 0 {
                    let samples = [
                        "weapons/explode3.wav",
                        "weapons/explode4.wav",
                        "weapons/explode5.wav",
                    ];
                    sounds.push(world_sound(samples[random.long(0, 2) as usize], origin));
                }
            }
            kind::SMOKE => {
                let origin = b.vector();
                let index = b.short();
                let scale = f32::from(b.byte()) * 0.1;
                let framerate = f32::from(b.byte());
                if let Some((model, frames)) = sprite_of(index)
                    && let Some(e) = self.sprite(&model, origin, frames, framerate, now)
                {
                    // `R_Sprite_Smoke`, dark grey and rising
                    let grey = random.long(20, 35) as u8;
                    e.render_mode = render::ALPHA;
                    e.tint = Tint::Rgb([grey; 3]);
                    e.velocity[2] = 30.;
                    e.origin[2] += 20.;
                    e.scale = scale;
                }
            }
            kind::SPARKS => {
                let origin = b.vector();
                self.spark_shower(origin, now);
            }
            kind::SPRITE => {
                let origin = b.vector();
                let index = b.short();
                let scale = f32::from(b.byte()) * 0.1;
                let brightness = f32::from(b.byte());
                if let Some((model, frames)) = sprite_of(index)
                    && let Some(e) = self.sprite(&model, origin, frames, 0., now)
                {
                    e.scale = scale;
                    e.render_mode = render::ADDITIVE;
                    e.render_amount = brightness;
                    e.base_amount = brightness;
                }
            }
            kind::MODEL => {
                let origin = b.vector();
                let velocity = b.vector();
                let yaw = f32::from(b.byte()) * 360. / 256.;
                let index = b.short();
                let bounce = match b.byte() {
                    1 => Some(Bounce::Shell),
                    2 => Some(Bounce::ShotShell),
                    _ => None,
                };
                let life = f32::from(b.byte()) * 0.1;
                if let Some(model) = (lookup.model)(index) {
                    let seed = random.next();
                    self.model(&model, origin, velocity, yaw, life, bounce, now, seed);
                }
            }
            kind::BREAK_MODEL => {
                let origin = b.vector();
                let size = b.vector();
                let velocity = b.vector();
                let spread = f32::from(b.byte()) * 10.;
                let index = b.short();
                let count = b.byte();
                let life = f32::from(b.byte()) * 0.1;
                let flags = b.byte();
                if let Some(model) = (lookup.model)(index) {
                    let frames = (lookup.frames)(&model).unwrap_or(1);
                    let gibs = Gibs {
                        model: &model,
                        frames,
                        origin,
                        size,
                        velocity,
                        spread,
                        count,
                        life,
                        flags,
                    };
                    self.break_model(&gibs, lookup.world, now, random.next());
                }
            }
            kind::GUNSHOT_DECAL => {
                // The decal is laid with the others and the dust is not
                // drawn yet. Half the shots ricochet
                let origin = b.vector();
                if random.long(0, 0x7fff) < 0x3fff {
                    sounds.push(ricochet_sound(origin, &mut random));
                }
            }
            kind::SPRITE_SPRAY => {
                let origin = b.vector();
                let direction = b.vector();
                let index = b.short();
                let count = b.byte();
                let speed = f32::from(b.byte());
                let noise = (f32::from(b.byte()) / 100.).min(1.);
                if let Some((model, frames)) = sprite_of(index) {
                    let spray = Spray {
                        model: &model,
                        frames,
                        direction,
                        count,
                        speed,
                        noise,
                    };
                    self.spray(&spray, origin, now);
                }
            }
            kind::ARMOR_RICOCHET => {
                let origin = b.vector();
                let scale = f32::from(b.byte()) * 0.1;
                self.ricochet_sprite(origin, scale, now);
                sounds.push(ricochet_sound(origin, &mut random));
            }
            kind::BLOOD_SPRITE => {
                let origin = b.vector();
                let spray = b.short();
                let drop = b.short();
                let colour = b.byte();
                let size = f32::from(b.byte());
                self.blood(origin, sprite_of(spray), sprite_of(drop), colour, size, now);
            }
            _ => {}
        }
        sounds
    }

    /// `R_BreakModel`, a breakable's pieces scattered through its box,
    /// flying out, bouncing and fading, see-through for glass
    fn break_model(&mut self, gibs: &Gibs, world: Option<&Hull>, now: SourceTime, seed: u64) {
        // Enough to cover the box's faces when the message gives none
        let [x, y, z] = gibs.size;
        let count = match gibs.count {
            0 => ((x * y + y * z + z * x) / (3. * SHARD * SHARD)) as u32,
            n => u32::from(n),
        }
        .min(100);
        let kind = gibs.flags & 0x4f;
        let bounce = match kind {
            0x01 => Some(Bounce::Glass),
            0x02 => Some(Bounce::Metal),
            0x04 => Some(Bounce::Flesh),
            0x08 => Some(Bounce::Wood),
            0x40 => Some(Bounce::Concrete),
            _ => None,
        };
        let see_through = kind == 0x01 || gibs.flags & 0x20 != 0;
        let mut random = Random(seed);
        for _ in 0..count {
            // A spot in the box out of the solid, after four tries
            let spot = (0..4).find_map(|_| {
                let spot =
                    [0, 1, 2].map(|k| gibs.origin[k] + random.float(-0.5, 0.5) * gibs.size[k]);
                world
                    .is_none_or(|w| w.contents(spot) != crate::trace::SOLID)
                    .then_some(spot)
            });
            let Some(spot) = spot else {
                continue;
            };
            let Some(e) = self.alloc(Some(gibs.model), spot, now) else {
                return;
            };
            e.bounce = bounce;
            e.frame = random.long(0, gibs.frames as i32 - 1).max(0) as f32;
            e.flags = flag::COLLIDE | flag::FADE | flag::SLOW_GRAVITY;
            if random.long(0, 255) < 200 {
                e.flags |= flag::ROTATE;
                e.spin = [0; 3].map(|_| random.float(-256., 255.));
            }
            (e.render_mode, e.render_amount) = if see_through {
                (render::TEXTURE, 128.)
            } else {
                (render::NORMAL, 255.)
            };
            e.base_amount = e.render_amount;
            let r = gibs.spread;
            e.velocity = [
                gibs.velocity[0] + random.float(-r, r),
                gibs.velocity[1] + random.float(-r, r),
                gibs.velocity[2] + random.float(0., r),
            ];
            e.die = SourceTime(now.0 + f64::from(gibs.life + random.float(0., 1.)));
            e.random = random.next();
        }
    }

    /// `R_Sprite_Spray`, sprites flying out along a direction with some
    /// noise, fading in a third of a second
    fn spray(&mut self, spray: &Spray, origin: [f32; 3], now: SourceTime) {
        for _ in 0..spray.count {
            let Some(e) = self.alloc(Some(spray.model), origin, now) else {
                return;
            };
            let mut random = Random(e.random);
            e.render_mode = render::ALPHA;
            e.steady = true;
            e.scale = 0.5;
            e.flags = flag::FADE | flag::SLOW_GRAVITY;
            e.fade_speed = 2.;
            let noise = spray.noise;
            let d = spray.direction;
            let heading = [
                d[0] + random.float(-noise, noise),
                d[1] + random.float(-noise, noise),
                d[2] + random.float(0., noise),
            ];
            let speed = random.float(spray.speed * 0.8, spray.speed * 1.2);
            e.velocity = heading.map(|v| v * speed);
            e.die = SourceTime(now.0 + 0.35);
            e.frame = random.long(0, spray.frames as i32 - 1).max(0) as f32;
            e.random = random.0;
        }
    }

    /// `R_BloodSprite`, a splash playing through quickly in the blood's
    /// colour, and drops flying out and falling
    fn blood(
        &mut self,
        origin: [f32; 3],
        spray: Option<(Arc<str>, u32)>,
        drop: Option<(Arc<str>, u32)>,
        colour: u8,
        size: f32,
        now: SourceTime,
    ) {
        let mut random = Random(self.seed(now));
        let impact = colour.wrapping_add(random.long(1, 3) as u8);
        if let Some((model, frames)) = spray {
            let lift = random.float(2., 4.);
            let at = [origin[0], origin[1], origin[2] + lift];
            if let Some(e) = self.alloc(Some(&model), at, now) {
                let mut r = Random(e.random);
                e.render_mode = render::TEXTURE;
                e.scale = r.float(size / 25., size / 35.);
                e.flags = flag::ANIMATE;
                e.tint = Tint::Palette(impact);
                e.render_amount = 250.;
                e.base_amount = 250.;
                // Over in a quarter of a second
                e.framerate = frames as f32 * 4.;
                e.frame_max = frames.saturating_sub(1) as f32;
                e.die = SourceTime(now.0 + 0.25);
                e.angles[2] = r.long(0, 360) as f32;
                e.bounce_factor = 0.;
                e.random = r.0;
            }
        }
        if let Some((model, frames)) = drop {
            let splatter = size as i32 + random.long(1, 8) + random.long(1, 8);
            for _ in 0..splatter {
                let Some(e) = self.alloc(Some(&model), origin, now) else {
                    return;
                };
                let mut r = Random(e.random);
                e.render_mode = render::TEXTURE;
                e.scale = r.float(size / 15., size / 25.);
                e.flags = flag::ROTATE | flag::SLOW_GRAVITY | flag::COLLIDE;
                e.tint = Tint::Palette(impact.wrapping_sub(1));
                e.render_amount = 250.;
                e.base_amount = 250.;
                e.velocity = [r.float(-96., 95.), r.float(-96., 95.), r.float(-32., 95.)];
                e.spin = [0; 3].map(|_| r.float(-256., -255.));
                e.frame = r.long(1, frames as i32 - 1).max(0) as f32;
                e.die = SourceTime(now.0 + f64::from(r.float(1., 3.)));
                e.angles[2] = r.long(0, 360) as f32;
                e.bounce_factor = 0.;
                e.random = r.0;
            }
        }
    }
}

/// The side of a cube a gib stands for when a breakable gives no count
/// (`SHARD_VOLUME`)
const SHARD: f32 = 12.;

/// A breakable's pieces, as `TE_BREAKMODEL` gives them
struct Gibs<'a> {
    model: &'a str,
    frames: u32,
    origin: [f32; 3],
    size: [f32; 3],
    velocity: [f32; 3],
    spread: f32,
    count: u8,
    life: f32,
    flags: u8,
}

/// A sprite spray's sprite and how it flies
struct Spray<'a> {
    model: &'a str,
    frames: u32,
    direction: [f32; 3],
    count: u8,
    speed: f32,
    noise: f32,
}

/// A sound from no entity at `origin`, at full volume
fn world_sound(sample: &str, origin: [f32; 3]) -> SoundCall {
    SoundCall {
        entity: 0,
        channel: CHAN_AUTO,
        sample: Arc::from(sample),
        volume: 1.,
        attenuation: ATTN_NORM,
        flags: 0,
        pitch: PITCH_NORM,
        origin,
    }
}

/// One of the five ricochets (`R_RicochetSound`)
fn ricochet_sound(origin: [f32; 3], random: &mut Random) -> SoundCall {
    let samples = [
        "weapons/ric1.wav",
        "weapons/ric2.wav",
        "weapons/ric3.wav",
        "weapons/ric4.wav",
        "weapons/ric5.wav",
    ];
    world_sound(samples[random.long(0, 4) as usize], origin)
}

/// `sprites/richo1.spr`, the ricochet's glint
pub(crate) const RICOCHET: &str = "sprites/richo1.spr";

/// The sound of a model hitting something, for the first bounce that comes
/// down fast and then one in four, louder the faster it falls. None when it
/// comes to rest or makes no sound
fn bounce_sound(e: &mut TempEntity, damp: f32) -> Option<SoundCall> {
    let bounce = e.bounce?;
    let mut random = Random(e.random);
    let (name, count) = match bounce {
        Bounce::Shell => ("player/pl_shell", 3),
        Bounce::ShotShell => ("weapons/sshell", 3),
        Bounce::Glass => ("debris/glass", 4),
        Bounce::Metal => ("debris/metal", 6),
        Bounce::Flesh => ("debris/flesh", 7),
        Bounce::Wood => ("debris/wood", 4),
        Bounce::Concrete => ("debris/concrete", 3),
    };
    let sample = format!("{name}{}.wav", random.long(1, count));
    let shell = matches!(bounce, Bounce::Shell | Bounce::ShotShell);
    let fall = e.velocity[2].abs().trunc();
    // A shell's first bounce falling fast and one in four after, and
    // one in six of anything else's
    let quiet = if shell {
        fall < 200. && random.long(0, 3) != 0
    } else {
        random.long(0, 5) != 0
    };
    let (volume, pitch) = if shell {
        ((fall / 350.).min(1.), PITCH_NORM)
    } else if random.long(0, 3) == 0 {
        ((fall / 450.).min(1.), random.long(95, 105) as u8)
    } else {
        ((fall / 450.).min(1.), PITCH_NORM)
    };
    e.random = random.0;
    if quiet || damp <= 0. {
        return None;
    }
    Some(SoundCall {
        entity: 0,
        channel: CHAN_AUTO,
        sample: Arc::from(sample),
        volume,
        attenuation: ATTN_NORM,
        flags: 0,
        pitch,
        origin: e.origin,
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
            models: vec![Child::Node(0)],
            node_faces: Vec::new(),
            faces: Vec::new(),
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
        assert_eq!(t.entities()[0].angles, [0., 90., 0.]);

        let mut lowest = f32::MAX;
        let mut bounced = 0;
        for i in 1..=200 {
            let sounds = t.step(SourceTime(f64::from(i) * 0.01), Some(&world), 800.);
            bounced += sounds.len();
            lowest = lowest.min(t.entities()[0].origin[2]);
        }
        let shell = &t.entities()[0];
        // Never through the floor, and lying flat on it at the end
        assert!(lowest > 0.);
        assert_eq!(shell.flags & flag::COLLIDE, 0);
        assert!(shell.origin[2] < 0.1);
        assert_eq!((shell.angles[0], shell.angles[2]), (0., 0.));
        assert!(bounced >= 1);

        // Gone once its life is over
        t.step(SourceTime(2.6), Some(&world), 800.);
        assert!(t.entities().is_empty());
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
            t.entities()[0].clone()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn a_sprite_plays_once_and_a_glint_fades() {
        let mut t = TempEntities::default();
        t.step(SourceTime(0.), None, 800.);
        t.sprite("sprites/zerogxplode.spr", [0.; 3], 15, 15., SourceTime(0.));
        t.ricochet_sprite([0.; 3], 1., SourceTime(0.));
        t.step(SourceTime(0.05), None, 800.);
        let glint = &t.entities()[1];
        // Faded by eight times the time past its death
        assert!((glint.render_amount - 200. * (1. - 0.05 * 8.)).abs() < 1e-3);
        assert!((t.entities()[0].frame - 0.75).abs() < 1e-4);
        t.step(SourceTime(0.2), None, 800.);
        assert_eq!(t.entities().len(), 1);
        // 15 frames at 15 a second, gone after the 14th
        t.step(SourceTime(0.94), None, 800.);
        assert!(t.entities().is_empty());
    }

    #[test]
    fn a_spark_shower_glints_until_its_life_runs_out() {
        let mut t = TempEntities::default();
        t.step(SourceTime(0.), None, 800.);
        t.spark_shower([0.; 3], SourceTime(0.));
        let mut glints = 0;
        for i in 1..=300 {
            t.step(SourceTime(f64::from(i) * 0.01), None, 800.);
            glints += t
                .entities()
                .iter()
                .filter(|e| e.model.is_some() && e.render_amount == 200.)
                .count();
        }
        // Between 5 and 15 showers a tenth of a second apart, none left
        assert!((5..=16).contains(&glints), "{glints}");
        assert!(t.entities().is_empty());
    }

    #[test]
    fn a_window_breaks_into_see_through_shards_out_of_the_floor() {
        let world = floor();
        let mut t = TempEntities::default();
        t.step(SourceTime(0.), Some(&world), 800.);
        // A 48 by 4 by 48 pane half sunk in the floor, with no count given
        let gibs = Gibs {
            model: "models/glassgibs.mdl",
            frames: 1,
            origin: [0., 0., 0.],
            size: [48., 4., 48.],
            velocity: [0., 0., 50.],
            spread: 20.,
            count: 0,
            life: 2.5,
            flags: 0x01,
        };
        t.break_model(&gibs, Some(&world), SourceTime(0.), 5);
        let shards = t.entities();
        // (48*4 + 4*48 + 48*48) / (3*12*12) is 6, fewer when four tries
        // all land in the floor
        assert!((1..=6).contains(&shards.len()), "{}", shards.len());
        for s in shards {
            assert!(s.origin[2] >= 0.);
            assert_eq!(s.bounce, Some(Bounce::Glass));
            assert_eq!((s.render_mode, s.render_amount), (render::TEXTURE, 128.));
            assert!((2.5..3.5).contains(&s.die.0));
        }
    }
}
