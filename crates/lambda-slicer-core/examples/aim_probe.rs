//! Development check of other players' aim rebuilt from snapshots
//!
//! Usage is `aim_probe <demos...>`. Other players' shot events carry no aim,
//! so the shooter's origin and angles come from the snapshots around the
//! shot. Hits are blood sprites along that aim line. Two checks tell whether
//! the rebuilt aim holds up. Recall is the share of visible direct-hit kills
//! with a matched hit by the killer in the half second before. The control
//! shifts every blood sprite 1.5 s later, so matches it still finds are
//! chance
//!
//! Both pitch signs are tried, since the snapshot carries the model's pitch,
//! a third of the view pitch

use std::collections::BTreeMap;

use lambda_slicer_core::analysis::analyze_demo;
use lambda_slicer_core::analysis::weapons::{Actor, Weapon};
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::protocol::messages::Message;
use lambda_slicer_core::state::{DecodeOptions, decode_demo};

const TE_BLOODSPRITE: u8 = 115;

/// A shot's time, eye, direction, spread, whether it is the recorder's and
/// its entity
type Aimed = (f32, [f32; 3], [f32; 3], f32, bool, u16);

fn coord(b: &[u8]) -> f32 {
    f32::from(i16::from_le_bytes([b[0], b[1]])) / 8.
}

fn forward(pitch: f32, yaw: f32) -> [f32; 3] {
    let (p, y) = (pitch.to_radians(), yaw.to_radians());
    [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()]
}

fn spread(w: Weapon) -> f32 {
    match w {
        Weapon::Shotgun => 12.,
        Weapon::Mp5 | Weapon::Glock => 6.,
        _ => 3.,
    }
}

#[derive(Clone, Copy)]
struct Pose {
    time: f32,
    origin: [f32; 3],
    angles: [f32; 3],
}

fn hits(
    eye: [f32; 3],
    dir: [f32; 3],
    tan: f32,
    blood: &[(f32, [f32; 3])],
    t: f32,
    shift: f32,
) -> bool {
    blood
        .iter()
        .filter(|(bt, _)| *bt + shift >= t - 0.05 && *bt + shift <= t + 0.25)
        .any(|(_, p)| {
            let v = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
            let along = v[0] * dir[0] + v[1] * dir[1] + v[2] * dir[2];
            if along < 16. {
                return false;
            }
            let perp = [
                v[0] - along * dir[0],
                v[1] - along * dir[1],
                v[2] - along * dir[2],
            ];
            let off = (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
            off <= 16. + along * tan
        })
}

fn main() {
    for path in std::env::args().skip(1) {
        let mut blood: Vec<(f32, [f32; 3])> = Vec::new();
        let mut poses: BTreeMap<u16, Vec<Pose>> = BTreeMap::new();
        let mut last_time = f32::NAN;
        let options = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };
        decode_demo(path.as_ref(), &options, |loc, d, m| {
            if let Message::TempEntity { kind, body } = &m.message
                && *kind == TE_BLOODSPRITE
                && body.len() >= 6
            {
                blood.push((
                    loc.time(),
                    [coord(&body[0..2]), coord(&body[2..4]), coord(&body[4..6])],
                ));
            }
            if loc.time() == last_time {
                return;
            }
            last_time = loc.time();
            let (Some(snap), Some(schema)) =
                (d.latest_snapshot(), d.schema("entity_state_player_t"))
            else {
                return;
            };
            let Some(entities) = &snap.entities else {
                return;
            };
            for e in entities
                .iter()
                .filter(|e| e.number >= 1 && e.number <= u16::from(d.max_clients))
            {
                let f = |n: &str| {
                    e.fields
                        .value(&schema, n)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.) as f32
                };
                poses.entry(e.number).or_default().push(Pose {
                    time: loc.time(),
                    origin: [f("origin[0]"), f("origin[1]"), f("origin[2]")],
                    angles: [f("angles[0]"), f("angles[1]"), f("angles[2]")],
                });
            }
        });

        let r = analyze_demo(path.as_ref(), Limits::default());
        let direct = [
            Weapon::Glock,
            Weapon::Python,
            Weapon::Mp5,
            Weapon::Shotgun,
            Weapon::Crossbow,
            Weapon::Gauss,
        ];
        let recorder = u16::from(r.recorder.as_ref().map_or(0, |p| p.slot)) + 1;

        let _others: Vec<_> = r
            .occurrences
            .iter()
            .filter(|o| o.shot && direct.contains(&o.weapon))
            .filter_map(|o| match &o.actor {
                Actor::Player { entity, .. } if *entity as u16 != recorder => {
                    Some((o, *entity as u16))
                }
                _ => None,
            })
            .collect();
        println!("{path}");
        for (o, _) in _others.iter().take(3) {
            println!(
                "  other player's event args origin {:?} angles {:?}",
                o.event.args.origin, o.event.args.angles
            );
        }

        // Aim sources, each giving eye and direction for a shot, or none
        type Aim<'a> = Box<
            dyn Fn(
                    &lambda_slicer_core::analysis::weapons::WeaponOccurrence,
                    u16,
                ) -> Option<([f32; 3], [f32; 3])>
                + 'a,
        >;
        let snapshot = |sign: f32| -> Aim {
            let poses = &poses;
            Box::new(move |o, entity| {
                let list = poses.get(&entity)?;
                let at = list.partition_point(|p| p.time <= o.time);
                let pose = list[at.checked_sub(1)?];
                (o.time - pose.time <= 0.1).then(|| {
                    (
                        [pose.origin[0], pose.origin[1], pose.origin[2] + 28.],
                        forward(pose.angles[0] * sign, pose.angles[1]),
                    )
                })
            })
        };
        let event = |sign: f32| -> Aim {
            Box::new(move |o, _| {
                let a = &o.event.args;
                let eye = if a.ducking != 0 { 12. } else { 28. };
                Some((
                    [a.origin[0], a.origin[1], a.origin[2] + eye],
                    forward(a.angles[0] * sign, a.angles[1]),
                ))
            })
        };
        let sources: Vec<(&str, Aim, bool)> = vec![
            ("snapshot, pitch x-3", snapshot(-3.), false),
            ("event, pitch x-3", event(-3.), false),
            ("event, pitch x1 (recorder)", event(1.), true),
        ];

        // Each blood sprite given to the single shot that points at it most
        // precisely, the recorder's shots by their exact aim and other
        // players' by their rebuilt aim
        {
            let rec = event(1.);
            let snap = snapshot(-3.);
            // (time, eye, dir, tan, recorder, entity, weapon kills recall key)
            let mut all: Vec<Aimed> = Vec::new();
            for o in r
                .occurrences
                .iter()
                .filter(|o| o.shot && direct.contains(&o.weapon))
            {
                let Actor::Player { entity, .. } = &o.actor else {
                    continue;
                };
                let entity = *entity as u16;
                let own = entity == recorder;
                let aim = if own { rec(o, entity) } else { snap(o, entity) };
                if let Some((eye, dir)) = aim {
                    all.push((
                        o.time,
                        eye,
                        dir,
                        spread(o.weapon).to_radians().tan(),
                        own,
                        entity,
                    ));
                }
            }
            for shift in [0.0f32, 1.5] {
                let mut hit = vec![false; all.len()];
                for (bt, p) in &blood {
                    let bt = bt + shift;
                    let mut best: Option<(f32, usize)> = None;
                    for (i, (t, eye, dir, tan, _, _)) in all.iter().enumerate() {
                        if bt < t - 0.05 || bt > t + 0.25 {
                            continue;
                        }
                        let v = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
                        let along = v[0] * dir[0] + v[1] * dir[1] + v[2] * dir[2];
                        if along < 16. {
                            continue;
                        }
                        let perp = [
                            v[0] - along * dir[0],
                            v[1] - along * dir[1],
                            v[2] - along * dir[2],
                        ];
                        let off =
                            (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
                        if off > 16. + along * tan {
                            continue;
                        }
                        // Angular miss, so near and far shots compare fairly
                        let miss = off / along;
                        if best.is_none_or(|b| miss < b.0) {
                            best = Some((miss, i));
                        }
                    }
                    if let Some((_, i)) = best {
                        hit[i] = true;
                    }
                }
                for own in [true, false] {
                    let idx: Vec<usize> = (0..all.len()).filter(|i| all[*i].4 == own).collect();
                    let n = idx.len();
                    let h = idx.iter().filter(|i| hit[**i]).count();
                    let (mut kills, mut found) = (0, 0);
                    if shift == 0. {
                        for k in &r.kills {
                            let (Some(w), Some(killer)) = (k.weapon, &k.killer) else {
                                continue;
                            };
                            let entity = u16::from(killer.slot) + 1;
                            if (entity == recorder) != own
                                || !direct.contains(&w)
                                || k.headshot.is_none()
                            {
                                continue;
                            }
                            kills += 1;
                            if idx.iter().any(|i| {
                                let a = &all[*i];
                                hit[*i] && a.5 == entity && a.0 <= k.time && k.time - a.0 <= 0.5
                            }) {
                                found += 1;
                            }
                        }
                    }
                    println!(
                        "  assigned, {} shift {shift}: {n} shots, {h} hits ({:.1}%){}",
                        if own { "recorder" } else { "others  " },
                        100. * h as f64 / n.max(1) as f64,
                        if shift == 0. {
                            format!(", kills with a hit {found}/{kills}")
                        } else {
                            String::new()
                        }
                    );
                }
            }
        }

        for (label, aim, own) in &sources {
            let (mut shots, mut hit_n, mut control) = (0u32, 0u32, 0u32);
            let mut hit_times: BTreeMap<u16, Vec<f32>> = BTreeMap::new();
            let chosen: Vec<_> = r
                .occurrences
                .iter()
                .filter(|o| o.shot && direct.contains(&o.weapon))
                .filter_map(|o| match &o.actor {
                    Actor::Player { entity, .. } if (*entity as u16 == recorder) == *own => {
                        Some((o, *entity as u16))
                    }
                    _ => None,
                })
                .collect();
            for (o, entity) in &chosen {
                let Some((eye, dir)) = aim(o, *entity) else {
                    continue;
                };
                let tan = spread(o.weapon).to_radians().tan();
                shots += 1;
                if hits(eye, dir, tan, &blood, o.time, 0.) {
                    hit_n += 1;
                    hit_times.entry(*entity).or_default().push(o.time);
                }
                if hits(eye, dir, tan, &blood, o.time, 1.5) {
                    control += 1;
                }
            }
            let (mut kills, mut found) = (0, 0);
            for k in &r.kills {
                let (Some(w), Some(killer)) = (k.weapon, &k.killer) else {
                    continue;
                };
                let entity = u16::from(killer.slot) + 1;
                if (entity == recorder) != *own || !direct.contains(&w) || k.headshot.is_none() {
                    continue;
                }
                kills += 1;
                if hit_times
                    .get(&entity)
                    .is_some_and(|ts| ts.iter().any(|t| *t <= k.time && k.time - t <= 0.5))
                {
                    found += 1;
                }
            }
            println!(
                "  {label:28} {shots} shots, {hit_n} hits ({:.1}%), control {control} ({:.1}%), kills with a hit {found}/{kills}",
                100. * f64::from(hit_n) / f64::from(shots.max(1)),
                100. * f64::from(control) / f64::from(shots.max(1)),
            );
        }
    }
}
