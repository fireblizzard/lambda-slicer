//! Development check of trajectory-based airborne detection against the
//! recording player's exact on-ground flag
//!
//! Usage is `motion_check <demos...>`. The recorder's client data origins
//! are classified as a trajectory at every snapshot and compared with the
//! client data on-ground flag at the same snapshot

use std::collections::VecDeque;
use std::sync::Arc;

use lambda_slicer_core::analysis::motion::{Sample, gravity, state_at};
use lambda_slicer_core::protocol::delta::Value;
use lambda_slicer_core::state::{DecodeOptions, Snapshot, decode_demo};

const FL_ONGROUND: i64 = 1 << 9;

#[derive(Default)]
struct Confusion {
    /// (exact airborne, inferred) counts
    air_as_air: u64,
    air_as_ground: u64,
    air_unknown: u64,
    ground_as_air: u64,
    ground_as_ground: u64,
    ground_unknown: u64,
    speed_error_sum: f64,
    speed_samples: u64,
}

fn main() {
    let mut c = Confusion::default();
    let mut errors: Vec<f32> = Vec::new();
    for path in std::env::args().skip(1) {
        let options = DecodeOptions {
            collect_events: false,
            ..Default::default()
        };
        let mut ring: VecDeque<Sample> = VecDeque::new();
        let mut last: Option<Arc<Snapshot>> = None;
        decode_demo(path.as_ref(), &options, |loc, d, _| {
            let Some(snap) = d.latest_snapshot() else {
                return;
            };
            if loc.entry == 0 || last.as_ref().is_some_and(|l| Arc::ptr_eq(l, snap)) {
                return;
            }
            last = Some(snap.clone());

            let recorder = u16::from(d.player_num) + 1;
            let (Some(entities), Some(client), Some(cschema)) =
                (&snap.entities, &snap.clientdata, d.schema("clientdata_t"))
            else {
                return;
            };
            // The recorder's own entity carries no origin, so its client data
            // origin stands in as the trajectory
            if !entities.iter().any(|e| e.number == recorder) {
                ring.clear();
                return;
            }
            let coord = |name: &str| match client.value(&cschema, name) {
                Some(Value::Float(f)) => f,
                _ => 0.,
            };
            // Positions belong to the server's simulation, so they are timed
            // by its clock, not by when the client happened to play them
            let time = f32::from_bits(snap.server_time_bits);
            if ring.back().is_some_and(|b| b.time >= time) {
                return;
            }
            ring.push_back(Sample {
                time,
                origin: [coord("origin[0]"), coord("origin[1]"), coord("origin[2]")],
                exact: None,
            });
            while ring.front().is_some_and(|s| time - s.time > 1.) {
                ring.pop_front();
            }

            let Some(flags) = client.value(&cschema, "flags").and_then(|v| v.as_i64()) else {
                return;
            };
            let velocity: Vec<f32> = ["velocity[0]", "velocity[1]", "velocity[2]"]
                .iter()
                .filter_map(|n| client.value(&cschema, n).and_then(|v| v.as_f64()))
                .map(|v| v as f32)
                .collect();
            let exact_air = flags & FL_ONGROUND == 0;

            let samples: Vec<Sample> = ring.iter().copied().collect();
            let s = state_at(&samples, time, gravity(d));
            match (exact_air, s.airborne) {
                (true, Some(true)) => c.air_as_air += 1,
                (true, Some(false)) => c.air_as_ground += 1,
                (true, None) => c.air_unknown += 1,
                (false, Some(true)) => c.ground_as_air += 1,
                (false, Some(false)) => c.ground_as_ground += 1,
                (false, None) => c.ground_unknown += 1,
            }
            if let (Some(speed), 3) = (s.horizontal_speed, velocity.len()) {
                let exact = (velocity[0] * velocity[0] + velocity[1] * velocity[1]).sqrt();
                c.speed_error_sum += f64::from((speed - exact).abs());
                c.speed_samples += 1;
                errors.push((speed - exact).abs());
            }
        });
    }

    errors.sort_by(f32::total_cmp);
    let at = |q: f64| {
        errors
            .get(((errors.len() as f64 - 1.) * q) as usize)
            .copied()
            .unwrap_or(0.)
    };
    println!(
        "horizontal speed error percentiles 50% {:.1}, 90% {:.1}, 99% {:.1} u/s",
        at(0.5),
        at(0.9),
        at(0.99)
    );
    let air = c.air_as_air + c.air_as_ground + c.air_unknown;
    let ground = c.ground_as_air + c.ground_as_ground + c.ground_unknown;
    let pct = |n: u64, of: u64| 100. * n as f64 / of.max(1) as f64;
    println!(
        "exact airborne {air}: inferred airborne {:.1}%, ground {:.1}%, unknown {:.1}%",
        pct(c.air_as_air, air),
        pct(c.air_as_ground, air),
        pct(c.air_unknown, air)
    );
    println!(
        "exact on ground {ground}: inferred airborne {:.1}%, ground {:.1}%, unknown {:.1}%",
        pct(c.ground_as_air, ground),
        pct(c.ground_as_ground, ground),
        pct(c.ground_unknown, ground)
    );
    let decided_air = c.air_as_air + c.ground_as_air;
    println!(
        "when inferred airborne, exact airborne {:.2}% of {decided_air}; mean horizontal speed error {:.1} u/s",
        pct(c.air_as_air, decided_air),
        c.speed_error_sum / c.speed_samples.max(1) as f64
    );
}
