//! The gauss's shot as the client draws it
//!
//! The gauss's handler traces its shot itself (`EV_FireGauss`). From the
//! eye along the aim the shot is drawn as a beam a tenth of a second long,
//! orange for a primary shot and white for a charged one, the first from
//! the gun. A brush met at a glancing angle, less than 60 degrees off its
//! plane, reflects the shot, which loses that much of its energy, leaves a
//! glow and throws out three balls. One met more squarely takes a shot
//! decal and a glow. A charged shot then tries to punch through, which it
//! does when the wall is thinner than its energy, losing that much, with
//! balls, a decal and a glow on the far side. A primary shot stops there,
//! with a smaller glow and eight balls. A shot that meets a player goes on
//! through it. The shot ends when its energy is 10 or less, or after ten
//! hits.
//!
//! Each piece is the SDK's, not yet compared with the game
//! (docs/viewer-profile.md, "Gauss and Egon"). The punch through traces
//! the world only, where the game traces brush entities and players too

use crate::bullets::{Hit, Impacts, gunshot_decal, trace};
use crate::events::Random;
use crate::scene::Scene;
use crate::trace::Hull;

/// `sprites/smoke.spr`, which the gauss's beams are drawn with
pub const BEAM_SPRITE: &str = "sprites/smoke.spr";
/// `sprites/hotglow.spr`, its glows and balls
pub const GLOW_SPRITE: &str = "sprites/hotglow.spr";

/// One gauss shot as its event gives it
#[derive(Clone, Debug, PartialEq)]
pub struct GaussShot {
    /// The shooter's eye
    pub source: [f32; 3],
    /// Forward, right and up of the aim
    pub axes: [[f32; 3]; 3],
    /// Its energy, the event's damage
    pub damage: f32,
    pub primary: bool,
    pub seed: u64,
}

/// A beam of the shot, from the gun when it is the first
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShotBeam {
    pub start: [f32; 3],
    pub from_gun: bool,
    pub end: [f32; 3],
    pub width: f32,
    pub color: [f32; 3],
}

/// A glow left where the shot met something (`R_TempSprite`)
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Glow {
    pub at: [f32; 3],
    pub scale: f32,
    /// 0 to 1
    pub brightness: f32,
    pub life: f32,
}

/// Balls flying out of a point towards another (`R_Sprite_Trail`)
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Balls {
    pub start: [f32; 3],
    pub towards: [f32; 3],
    pub count: u32,
    pub life: f32,
    pub size: f32,
    pub spread: f32,
    pub brightness: f32,
    pub speed: f32,
    pub seed: u64,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct GaussEffects {
    pub impacts: Impacts,
    pub beams: Vec<ShotBeam>,
    pub glows: Vec<Glow>,
    pub balls: Vec<Balls>,
}

/// A beam's colour from a handler's colour and brightness, their product
/// at most full. The handlers give the gauss's colour out of 255 at a
/// brightness of 128 or more, and the Egon's under 1 at 0.7 once divided
/// for hardware drawing, which only this reading draws as the game shows
/// them, the gauss's saturated and the Egon's a pale blue
pub(crate) fn beam_colour(rgb: [f32; 3], brightness: f32) -> [f32; 3] {
    rgb.map(|c| (c * brightness).clamp(0., 1.))
}

fn add(a: [f32; 3], b: [f32; 3], scale: f32) -> [f32; 3] {
    [0, 1, 2].map(|k| a[k] + b[k] * scale)
}

/// Fire `shot` for `shooter`
pub(crate) fn fire(
    shot: &GaussShot,
    shooter: u16,
    max_clients: u8,
    scene: &Scene,
    world: Option<&Hull>,
) -> GaussEffects {
    let mut random = Random(shot.seed);
    let mut out = GaussEffects::default();
    let mut damage = shot.damage;
    let mut forward = shot.axes[0];
    let mut source = shot.source;
    let mut dest = add(source, forward, 8192.);
    let mut punched = false;
    let mut first = true;
    // The player the shot last went through, which the next trace passes
    // as the server's does (`pentIgnore`), as it starts inside its box
    let mut passed: Option<u16> = None;
    // Orange at brightness 128 for a primary shot, white at the shot's
    // energy for a charged one
    let (width, color) = if shot.primary {
        (1., beam_colour([255., 128., 0.], 128.))
    } else {
        (2.5, beam_colour([255.; 3], damage))
    };
    let balls =
        |random: &mut Random, start, towards, count, life, spread, brightness, speed| Balls {
            start,
            towards,
            count,
            life,
            size: random.float(10., 20.) / 100.,
            spread,
            brightness,
            speed,
            seed: random.next(),
        };

    for _ in 0..10 {
        if damage <= 10. {
            break;
        }
        if world.is_some_and(|w| w.trace(source, dest).all_solid) {
            break;
        }
        let skip: Vec<u16> = std::iter::once(shooter).chain(passed).collect();
        let met = trace(source, dest, &skip, max_clients, scene, world);
        let end = met.map_or(dest, |(_, at, _)| at);
        out.beams.push(ShotBeam {
            start: source,
            from_gun: first,
            end,
            width,
            color,
        });
        first = false;
        let Some((_, at, hit)) = met else {
            break;
        };
        let (entity, render_mode, normal) = match hit {
            Hit::Brush {
                entity,
                render_mode,
                normal,
                ..
            } => (entity, render_mode, normal),
            Hit::Player(number) => {
                // On through the player
                source = add(at, forward, 1.);
                passed = Some(number);
                continue;
            }
        };
        passed = None;

        let n = -(normal[0] * forward[0] + normal[1] * forward[1] + normal[2] * forward[2]);
        if n < 0.5 {
            // Glancing, so it reflects and loses energy
            forward = add(forward, normal, 2. * n);
            source = add(at, forward, 8.);
            dest = add(source, forward, 8192.);
            out.glows.push(Glow {
                at,
                scale: 0.2,
                brightness: damage * n / 255.,
                life: damage * n * 0.5 * 0.1,
            });
            let ball = balls(
                &mut random,
                at,
                add(at, normal, 1.),
                3,
                0.1,
                100.,
                255.,
                100.,
            );
            out.balls.push(ball);
            let n = if n == 0. { 0.1 } else { n };
            damage *= 1. - n;
            continue;
        }

        // Square on, a decal and a glow, then one try at punching through
        gunshot_decal(entity, render_mode, at, &mut random, &mut out.impacts);
        out.glows.push(Glow {
            at,
            scale: 1.,
            brightness: damage / 255.,
            life: 6.,
        });
        if punched {
            break;
        }
        punched = true;
        if shot.primary {
            out.glows.push(Glow {
                at,
                scale: 0.2,
                brightness: 200. / 255.,
                life: 0.3,
            });
            let ball = balls(
                &mut random,
                at,
                add(at, normal, 1.),
                8,
                0.6,
                100.,
                255.,
                200.,
            );
            out.balls.push(ball);
            break;
        }
        let Some(world) = world else {
            break;
        };
        let start = add(at, forward, 8.);
        let beyond = world.trace(start, dest);
        if beyond.all_solid {
            break;
        }
        // Back from where it went on to, to find the wall's far side
        let exit = world.trace(beyond.end, at).end;
        let thickness = {
            let d = [0, 1, 2].map(|k| exit[k] - at[k]);
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        };
        if thickness < damage {
            damage -= thickness.max(1.);
            let ball = balls(
                &mut random,
                at,
                add(at, forward, -1.),
                3,
                0.1,
                100.,
                255.,
                100.,
            );
            out.balls.push(ball);
            gunshot_decal(0, 0, exit, &mut random, &mut out.impacts);
            out.glows.push(Glow {
                at: exit,
                scale: 0.1,
                brightness: damage / 255.,
                life: 6.,
            });
            let count = (damage * 0.3) as u32;
            let ball = balls(
                &mut random,
                exit,
                add(exit, forward, -1.),
                count,
                0.1,
                200.,
                255.,
                40.,
            );
            out.balls.push(ball);
            source = add(exit, forward, 1.);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::{Child, SOLID};

    /// A floor at height 0, solid under it
    fn floor() -> Hull {
        Hull {
            planes: vec![([0., 0., 1.], 0.)],
            nodes: vec![(0, [Child::Contents(-1), Child::Contents(SOLID)])],
            models: vec![Child::Node(0)],
            node_faces: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// A wall 16 units thick, from x = 100 to 116
    fn thin_wall() -> Hull {
        Hull {
            planes: vec![([1., 0., 0.], 100.), ([1., 0., 0.], 116.)],
            nodes: vec![
                (0, [Child::Node(1), Child::Contents(-1)]),
                (1, [Child::Contents(-1), Child::Contents(SOLID)]),
            ],
            models: vec![Child::Node(0)],
            node_faces: Vec::new(),
            faces: Vec::new(),
        }
    }

    fn shot(forward: [f32; 3], damage: f32, primary: bool) -> GaussShot {
        GaussShot {
            source: [0., 0., 50.],
            axes: [forward, [0., -1., 0.], [0., 0., 1.]],
            damage,
            primary,
            seed: 4,
        }
    }

    #[test]
    fn a_glancing_shot_reflects_off_the_floor_and_loses_energy() {
        let hull = floor();
        // 30 degrees down, so 0.5 into the floor's normal is not reached
        let down = [0.9, 0., -0.43];
        let out = fire(
            &shot(down, 200., false),
            1,
            32,
            &Scene::default(),
            Some(&hull),
        );
        assert!(out.beams.len() >= 2);
        assert!(out.beams[0].from_gun && !out.beams[1].from_gun);
        // The second beam leaves the floor going up
        assert!(out.beams[1].end[2] > out.beams[1].start[2]);
        assert_eq!(out.balls[0].count, 3);
        assert!(out.impacts.decals.is_empty());
    }

    #[test]
    fn a_charged_shot_punches_through_a_thin_wall_and_a_primary_one_stops() {
        let hull = thin_wall();
        let ahead = [1., 0., 0.];
        let charged = fire(
            &shot(ahead, 200., false),
            1,
            32,
            &Scene::default(),
            Some(&hull),
        );
        // In and out, a decal on each side, and the last beam beyond it
        assert_eq!(charged.impacts.decals.len(), 2);
        let exit = charged.impacts.decals[1].2;
        assert!((exit[0] - 116.).abs() < 0.1);
        assert!(charged.beams.last().unwrap().start[0] > 116.);

        let primary = fire(
            &shot(ahead, 200., true),
            1,
            32,
            &Scene::default(),
            Some(&hull),
        );
        assert_eq!(primary.impacts.decals.len(), 1);
        assert_eq!(primary.beams.len(), 1);
        assert_eq!(primary.balls.last().unwrap().count, 8);
    }

    #[test]
    fn a_shot_goes_through_a_player_to_the_wall_behind() {
        let hull = thin_wall();
        let player = crate::scene::Entity {
            id: crate::scene::EntityId {
                number: 2,
                ..Default::default()
            },
            origin: [50., 0., 50.],
            solid: 3,
            ..Default::default()
        };
        let mut scene = Scene::default();
        scene.entities = std::sync::Arc::new(vec![player]);
        let out = fire(&shot([1., 0., 0.], 200., true), 1, 32, &scene, Some(&hull));
        // Into the player's box, then on from inside it to the wall
        assert_eq!(out.beams.len(), 2);
        assert!((out.beams[0].end[0] - 34.).abs() < 0.1);
        assert!((out.beams[1].end[0] - 100.).abs() < 0.1);
        assert_eq!(out.impacts.decals.len(), 1);
    }
}
