//! The bullets the client fires itself for a weapon event (implementation
//! plan §13.6)
//!
//! The glock's, MP5's, shotgun's and python's handlers trace each bullet
//! from the shooter's eye along its aim and the event's spread, and where it
//! hits make the impact the server leaves to them (`EV_HLDM_FireBullets`).
//! A bullet that hits a player sounds like flesh. One that hits the world or
//! a solid brush entity sounds like the material of the face it meets, as
//! `sound/materials.txt` names it, concrete when it names none, ricochets
//! half the time and leaves a shot decal. The MP5's every other bullet is a
//! tracer, which leaves no impact, and buckshot makes no material sound.
//!
//! The trace meets the world and solid brush entities through the map's
//! point hull, brushes unturned, and players as their standing boxes where
//! the latest snapshot has them. The game traces players through their
//! hitboxes where its prediction has them, so a shot past a player's box
//! but clear of the model hits the wall behind here and the player there.
//! The streaks of tracers and the dust of impacts are not drawn yet

use std::sync::Arc;

use crate::audio::CHAN_AUTO;
use crate::events::{ATTN_NORM, CHAN_STATIC, PITCH_NORM, Random, SoundCall};
use crate::scene::Scene;
use crate::trace::Hull;

/// `SOLID_BSP`
const SOLID_BSP: i64 = 4;
/// A standing player's box about its origin (`VEC_HULL_MIN`, `VEC_HULL_MAX`)
const PLAYER_MIN: [f32; 3] = [-16., -16., -36.];
const PLAYER_MAX: [f32; 3] = [16., 16., 36.];

/// `BULLET_PLAYER_` kinds
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bullet {
    Nine,
    Mp5,
    Buckshot,
    Magnum,
}

/// The bullets of one shot
#[derive(Clone, Debug, PartialEq)]
pub struct Volley {
    pub kind: Bullet,
    pub shots: u32,
    pub distance: f32,
    /// How far right and up of the aim each bullet goes, at most for
    /// buckshot and exactly for the rest
    pub spread: [f32; 2],
    /// Every this many bullets is a tracer, 0 for none
    pub tracer_every: u32,
    /// The shooter's eye
    pub source: [f32; 3],
    /// The view vectors of the aim, forward, right and up
    pub axes: [[f32; 3]; 3],
    pub seed: u64,
}

/// What a volley's bullets leave
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Impacts {
    pub sounds: Vec<SoundCall>,
    /// A decal by its name in `decals.wad`, the entity it is on and where
    pub decals: Vec<(&'static str, u16, [f32; 3])>,
    /// A tracer's start and end, and its seed
    pub tracers: Vec<([f32; 3], [f32; 3], u64)>,
}

/// What a bullet hit
#[derive(Clone, Copy, Debug, PartialEq)]
enum Hit {
    /// The world or a brush entity, by entity number, with the material
    /// of the face met when one is found
    Brush {
        entity: u16,
        material: Option<u8>,
        render_mode: i64,
    },
    Player,
}

/// How far along the line from `start` to `end` it enters the box, if it
/// does
fn enter_box(start: [f32; 3], end: [f32; 3], min: [f32; 3], max: [f32; 3]) -> Option<f32> {
    let (mut near, mut far) = (0f32, 1f32);
    for k in 0..3 {
        let d = end[k] - start[k];
        if d.abs() < 1e-9 {
            if start[k] < min[k] || start[k] > max[k] {
                return None;
            }
            continue;
        }
        let (a, b) = ((min[k] - start[k]) / d, (max[k] - start[k]) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return None;
        }
    }
    Some(near)
}

/// The first thing the line from `start` to `end` meets, past the shooter
fn trace(
    start: [f32; 3],
    end: [f32; 3],
    shooter: u16,
    max_clients: u8,
    scene: &Scene,
    world: Option<&Hull>,
) -> Option<(f32, [f32; 3], Hit)> {
    let mut best: Option<(f32, [f32; 3], Hit)> = None;
    let mut offer = |fraction: f32, at: [f32; 3], hit: Hit| {
        if fraction < 1. && best.is_none_or(|(f, ..)| fraction < f) {
            best = Some((fraction, at, hit));
        }
    };
    if let Some(world) = world {
        let t = world.trace(start, end);
        let material = world.material(0, start, end);
        offer(
            t.fraction,
            t.end,
            Hit::Brush {
                entity: 0,
                material,
                render_mode: 0,
            },
        );
    }
    for e in scene.entities.iter() {
        let number = e.id.number;
        let player = (1..=u16::from(max_clients)).contains(&number);
        if player {
            if number == shooter || e.solid == 0 || e.iuser1 != 0 {
                continue;
            }
            let min = [0, 1, 2].map(|k| e.origin[k] + PLAYER_MIN[k]);
            let max = [0, 1, 2].map(|k| e.origin[k] + PLAYER_MAX[k]);
            if let Some(f) = enter_box(start, end, min, max) {
                let at = [0, 1, 2].map(|k| start[k] + (end[k] - start[k]) * f);
                offer(f, at, Hit::Player);
            }
        } else if e.solid == SOLID_BSP
            && let Some(world) = world
            && let Some(model) = e.model.as_deref().and_then(|m| m.strip_prefix('*'))
            && let Ok(model) = model.parse::<usize>()
        {
            let local = |p: [f32; 3]| [0, 1, 2].map(|k| p[k] - e.origin[k]);
            let t = world.trace_model(model, local(start), local(end));
            let at = [0, 1, 2].map(|k| t.end[k] + e.origin[k]);
            let hit = Hit::Brush {
                entity: number,
                material: None,
                render_mode: e.render_mode,
            };
            offer(t.fraction, at, hit);
        }
    }
    best
}

/// The samples of a material, and the volume they play at
/// (`EV_HLDM_PlayTextureSound`)
fn material_sound(material: u8) -> (&'static [&'static str], f32) {
    match material {
        b'M' => (&["player/pl_metal1.wav", "player/pl_metal2.wav"], 0.9),
        b'D' => (
            &[
                "player/pl_dirt1.wav",
                "player/pl_dirt2.wav",
                "player/pl_dirt3.wav",
            ],
            0.9,
        ),
        b'V' => (&["player/pl_duct1.wav", "player/pl_duct1.wav"], 0.5),
        b'G' => (&["player/pl_grate1.wav", "player/pl_grate4.wav"], 0.9),
        b'T' => (
            &[
                "player/pl_tile1.wav",
                "player/pl_tile3.wav",
                "player/pl_tile2.wav",
                "player/pl_tile4.wav",
            ],
            0.8,
        ),
        b'S' => (
            &[
                "player/pl_slosh1.wav",
                "player/pl_slosh3.wav",
                "player/pl_slosh2.wav",
                "player/pl_slosh4.wav",
            ],
            0.9,
        ),
        b'W' => (
            &["debris/wood1.wav", "debris/wood2.wav", "debris/wood3.wav"],
            0.9,
        ),
        b'Y' | b'P' => (
            &[
                "debris/glass1.wav",
                "debris/glass2.wav",
                "debris/glass3.wav",
            ],
            0.8,
        ),
        _ => (&["player/pl_step1.wav", "player/pl_step2.wav"], 0.9),
    }
}

fn sound(sample: &str, volume: f32, attenuation: f32, pitch: i32, origin: [f32; 3]) -> SoundCall {
    SoundCall {
        entity: 0,
        channel: CHAN_STATIC,
        sample: Arc::from(sample),
        volume,
        attenuation,
        flags: 0,
        pitch: pitch.clamp(0, 255) as u8,
        origin,
    }
}

/// Fire `volley` for `shooter`, counting its bullets in `tracers`
pub(crate) fn fire(
    volley: &Volley,
    shooter: u16,
    max_clients: u8,
    tracers: &mut u32,
    scene: &Scene,
    world: Option<&Hull>,
) -> Impacts {
    let mut random = Random(volley.seed);
    let mut impacts = Impacts::default();
    let [forward, right, up] = volley.axes;
    for _ in 0..volley.shots {
        let [x, y] = if volley.kind == Bullet::Buckshot {
            // Spread over a disc, more of it near the middle
            loop {
                let x = random.float(-0.5, 0.5) + random.float(-0.5, 0.5);
                let y = random.float(-0.5, 0.5) + random.float(-0.5, 0.5);
                if x * x + y * y <= 1. {
                    break [x, y];
                }
            }
        } else {
            [1., 1.]
        };
        let direction = [0, 1, 2]
            .map(|k| forward[k] + x * volley.spread[0] * right[k] + y * volley.spread[1] * up[k]);
        let end = [0, 1, 2].map(|k| volley.source[k] + volley.distance * direction[k]);

        // `EV_HLDM_CheckTracer`, every so many a tracer that leaves nothing
        let drawn = volley.tracer_every != 0 && {
            let due = tracers.is_multiple_of(volley.tracer_every);
            *tracers = tracers.wrapping_add(1);
            due
        };
        // A gun that draws every bullet's tracer still leaves its impacts
        let tracer = drawn && volley.tracer_every != 1;

        let met = trace(volley.source, end, shooter, max_clients, scene, world);
        if drawn {
            // A player's tracer starts a little below, right of and ahead of
            // the eye
            let player = (1..=u16::from(max_clients)).contains(&shooter);
            let start = match player {
                true => [0, 1, 2].map(|k| {
                    let below = if k == 2 { -4. } else { 0. };
                    volley.source[k] + below + right[k] * 2. + forward[k] * 16.
                }),
                false => volley.source,
            };
            let reached = met.map_or(end, |(_, at, _)| at);
            impacts.tracers.push((start, reached, random.next()));
        }
        let Some((_, at, hit)) = met else {
            continue;
        };
        let material = match (volley.kind, tracer) {
            (Bullet::Buckshot, _) | (Bullet::Mp5, true) => None,
            _ => Some(match hit {
                Hit::Player => b'F',
                Hit::Brush { material, .. } => material.unwrap_or(b'C'),
            }),
        };
        if let Some(material) = material {
            if material == b'F' {
                let samples = ["weapons/bullet_hit1.wav", "weapons/bullet_hit2.wav"];
                let sample = samples[random.long(0, 1) as usize];
                let pitch = 96 + random.long(0, 0xf);
                impacts.sounds.push(sound(sample, 1., 1., pitch, at));
            } else {
                let (samples, volume) = material_sound(material);
                let sample = samples[random.long(0, samples.len() as i32 - 1) as usize];
                let pitch = 96 + random.long(0, 0xf);
                impacts
                    .sounds
                    .push(sound(sample, volume, ATTN_NORM, pitch, at));
            }
        }

        // `EV_HLDM_DecalGunshot`, on brushes only
        let Hit::Brush {
            entity,
            render_mode,
            ..
        } = hit
        else {
            continue;
        };
        if volley.kind == Bullet::Mp5 && tracer {
            continue;
        }
        let decal = if render_mode != 0 {
            "{bproof1"
        } else {
            ["{shot1", "{shot2", "{shot3", "{shot4", "{shot5"][random.long(0, 4) as usize]
        };
        let roll = random.long(0, 0x7fff);
        if roll < 0x7fff / 2 {
            let samples = [
                "weapons/ric1.wav",
                "weapons/ric2.wav",
                "weapons/ric3.wav",
                "weapons/ric4.wav",
                "weapons/ric5.wav",
            ];
            let mut ricochet = sound(
                samples[(roll % 5) as usize],
                1.,
                ATTN_NORM,
                i32::from(PITCH_NORM),
                at,
            );
            ricochet.channel = CHAN_AUTO;
            impacts.sounds.push(ricochet);
        }
        impacts.decals.push((decal, entity, at));
    }
    impacts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::{Child, HullFace, SOLID};

    /// A wall at x = 100, solid beyond, its face metal
    fn wall() -> Hull {
        Hull {
            planes: vec![([-1., 0., 0.], -100.)],
            nodes: vec![(0, [Child::Contents(-1), Child::Contents(SOLID)])],
            models: vec![Child::Node(0)],
            node_faces: std::iter::once(0..1).collect(),
            faces: vec![HullFace {
                axes: [[0., 1., 0., 0.], [0., 0., -1., 0.]],
                mins: [-512., -512.],
                extents: [1024., 1024.],
                material: b'M',
            }],
        }
    }

    fn volley(kind: Bullet) -> Volley {
        Volley {
            kind,
            shots: 1,
            distance: 8192.,
            spread: [0., 0.],
            tracer_every: 0,
            source: [0.; 3],
            axes: [[1., 0., 0.], [0., -1., 0.], [0., 0., 1.]],
            seed: 9,
        }
    }

    #[test]
    fn a_bullet_into_a_wall_sounds_of_its_metal_and_leaves_a_shot() {
        let scene = Scene::default();
        let hull = wall();
        let impacts = fire(&volley(Bullet::Nine), 1, 32, &mut 0, &scene, Some(&hull));
        assert!(impacts.sounds[0].sample.starts_with("player/pl_metal"));
        assert_eq!(impacts.sounds[0].channel, CHAN_STATIC);
        let (name, entity, at) = impacts.decals[0];
        assert!(name.starts_with("{shot"));
        assert_eq!(entity, 0);
        assert!(at[0] > 99.9 && at[0] < 100.);
    }

    #[test]
    fn buckshot_spreads_and_mp5_tracers_leave_no_impact() {
        let scene = Scene::default();
        let hull = wall();
        let mut shot = volley(Bullet::Buckshot);
        shot.shots = 8;
        shot.spread = [0.17365, 0.04362];
        let impacts = fire(&shot, 1, 32, &mut 0, &scene, Some(&hull));
        // Every pellet leaves a decal and none a material sound
        assert_eq!(impacts.decals.len(), 8);
        assert!(
            impacts
                .sounds
                .iter()
                .all(|s| s.sample.starts_with("weapons/ric"))
        );
        let ys: Vec<f32> = impacts.decals.iter().map(|d| d.2[1]).collect();
        assert!(ys.iter().any(|y| (y - ys[0]).abs() > 1.));

        // The first of every two MP5 bullets is a tracer
        let mut tracers = 0;
        let mp5 = Volley {
            tracer_every: 2,
            ..volley(Bullet::Mp5)
        };
        let first = fire(&mp5, 1, 32, &mut tracers, &scene, Some(&hull));
        let second = fire(&mp5, 1, 32, &mut tracers, &scene, Some(&hull));
        assert!(first.decals.is_empty() && first.sounds.is_empty());
        assert_eq!(second.decals.len(), 1);
    }

    #[test]
    fn a_line_enters_a_box_where_it_crosses_its_near_face() {
        let f = enter_box(
            [0., 0., 0.],
            [100., 0., 0.],
            [40., -16., -36.],
            [72., 16., 36.],
        );
        assert_eq!(f, Some(0.4));
        assert_eq!(
            enter_box(
                [0., 0., 0.],
                [100., 0., 0.],
                [40., 20., -36.],
                [72., 52., 36.]
            ),
            None
        );
    }
}
