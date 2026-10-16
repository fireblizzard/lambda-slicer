//! Players' movement around kills, for highlights such as midair shots
//!
//! Each player's recent origins are kept from the decoded snapshots while
//! they are in the recording client's view. For the recording player the
//! client data gives the exact on-ground flag and velocity. For other players
//! the state comes from the trajectory. In the air their height follows the
//! server's gravity, so a quadratic fit over a short window shows a vertical
//! acceleration of about minus gravity, and on the ground about zero.
//! Anything between (stairs, a landing inside the window, too few samples)
//! stays unknown. Players out of view have no state at all

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use serde::Serialize;

use crate::protocol::delta::Value;
use crate::state::{Decoder, Snapshot};

/// `FL_ONGROUND`
const FL_ONGROUND: i64 = 1 << 9;

/// How much history each player keeps, enough to reach a projectile's launch
pub const HISTORY: f32 = 5.0;

/// The trajectory window. Long enough that 1/8 unit position steps do not
/// swamp the fit, short enough to fit inside one jump
const WINDOW: f32 = 0.2;
const MIN_SAMPLES: usize = 4;
/// Horizontal speed is measured over the last this many seconds
const SPEED_SPAN: f32 = 0.1;
/// Snapshots further apart than this break the trajectory
const MAX_GAP: f32 = 0.1;
/// Vertical acceleration within this share of gravity counts as falling,
/// and below this share as level
const FALLING: f32 = 0.25;
const LEVEL: f32 = 0.2;
/// Largest RMS distance from the fitted curve, in units
const MAX_RESIDUAL: f32 = 0.5;

const DEFAULT_GRAVITY: f32 = 800.;

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub time: f32,
    pub origin: [f32; 3],
    /// The recording player's exact state, from client data
    pub exact: Option<Exact>,
}

#[derive(Clone, Copy, Debug)]
pub struct Exact {
    pub on_ground: bool,
    pub velocity: [f32; 3],
}

/// How a motion state is known
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionEvidence {
    /// The recording player's client data
    Exact,
    /// Fitted to the player's recent trajectory
    Trajectory,
    /// In view, but the trajectory fits neither falling nor level movement,
    /// or there are too few recent samples
    Unclear,
    /// Not in the recording client's view
    NotVisible,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct MotionState {
    pub evidence: MotionEvidence,
    pub airborne: Option<bool>,
    /// Units per second, total and horizontal
    pub speed: Option<f32>,
    pub horizontal_speed: Option<f32>,
    pub origin: Option<[f32; 3]>,
}

impl MotionState {
    /// The state of a player out of view
    pub fn not_visible() -> Self {
        Self::unknown(MotionEvidence::NotVisible, None)
    }

    fn unknown(evidence: MotionEvidence, origin: Option<[f32; 3]>) -> Self {
        Self {
            evidence,
            airborne: None,
            speed: None,
            horizontal_speed: None,
            origin,
        }
    }
}

fn speeds(v: [f32; 3]) -> (f32, f32) {
    let horizontal = (v[0] * v[0] + v[1] * v[1]).sqrt();

    ((horizontal * horizontal + v[2] * v[2]).sqrt(), horizontal)
}

/// Least-squares fit of `y = a t² + b t + c`, returning (a, b, c, RMS
/// residual). `t` is relative to the last sample
fn quadratic(ts: &[f32], ys: &[f32]) -> Option<(f32, f32, f32, f32)> {
    let n = ts.len() as f64;
    let (mut s1, mut s2, mut s3, mut s4) = (0f64, 0f64, 0f64, 0f64);
    let (mut y0, mut y1, mut y2) = (0f64, 0f64, 0f64);
    for (&t, &y) in ts.iter().zip(ys) {
        let (t, y) = (f64::from(t), f64::from(y));
        s1 += t;
        s2 += t * t;
        s3 += t * t * t;
        s4 += t * t * t * t;
        y0 += y;
        y1 += t * y;
        y2 += t * t * y;
    }

    // Normal equations with rows (s4 s3 s2), (s3 s2 s1), (s2 s1 n) times
    // (a b c) equal to (y2 y1 y0)
    let m = [[s4, s3, s2], [s3, s2, s1], [s2, s1, n]];
    let det = |m: [[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(m);
    if d.abs() < 1e-12 {
        return None;
    }

    let with = |col: usize| {
        let mut r = m;
        for (row, v) in [y2, y1, y0].into_iter().enumerate() {
            r[row][col] = v;
        }
        det(r) / d
    };
    let (a, b, c) = (with(0), with(1), with(2));

    let residual = ts
        .iter()
        .zip(ys)
        .map(|(&t, &y)| {
            let t = f64::from(t);
            let e = a * t * t + b * t + c - f64::from(y);
            e * e
        })
        .sum::<f64>()
        / n;

    Some((a as f32, b as f32, c as f32, residual.sqrt() as f32))
}

/// A player's state at time `t`, from samples in time order
pub fn state_at(samples: &[Sample], t: f32, gravity: f32) -> MotionState {
    let upto: Vec<&Sample> = samples.iter().filter(|s| s.time <= t).collect();
    let Some(last) = upto.last() else {
        return MotionState::unknown(MotionEvidence::NotVisible, None);
    };
    if t - last.time > MAX_GAP {
        return MotionState::unknown(MotionEvidence::NotVisible, None);
    }

    if let Some(e) = last.exact {
        let (speed, horizontal) = speeds(e.velocity);
        return MotionState {
            evidence: MotionEvidence::Exact,
            airborne: Some(!e.on_ground),
            speed: Some(speed),
            horizontal_speed: Some(horizontal),
            origin: Some(last.origin),
        };
    }

    // The unbroken run of samples inside the window
    let mut window: Vec<&Sample> = Vec::new();
    for s in upto.iter().rev() {
        let next = window.last().map_or(last.time, |n| n.time);
        if last.time - s.time > WINDOW || next - s.time > MAX_GAP {
            break;
        }
        window.push(s);
    }
    window.reverse();
    let span = window.last().map_or(0., |l| l.time) - window.first().map_or(0., |f| f.time);
    if window.len() < MIN_SAMPLES || span < WINDOW / 2. {
        return MotionState::unknown(MotionEvidence::Unclear, Some(last.origin));
    }

    let ts: Vec<f32> = window.iter().map(|s| s.time - last.time).collect();
    let axis = |i: usize| -> Option<(f32, f32, f32, f32)> {
        let ys: Vec<f32> = window.iter().map(|s| s.origin[i]).collect();
        quadratic(&ts, &ys)
    };
    let (Some(x), Some(y), Some(z)) = (axis(0), axis(1), axis(2)) else {
        return MotionState::unknown(MotionEvidence::Unclear, Some(last.origin));
    };

    // Horizontal velocity from the displacement over the last part of the
    // window, which follows air strafing and collisions better than the
    // whole-window fit. Vertical velocity from the fit at the last sample
    let recent = window
        .iter()
        .find(|s| last.time - s.time <= SPEED_SPAN)
        .filter(|s| last.time - s.time > 0.)
        .copied();
    let velocity = match recent {
        Some(s) => {
            let dt = last.time - s.time;
            [
                (last.origin[0] - s.origin[0]) / dt,
                (last.origin[1] - s.origin[1]) / dt,
                z.1,
            ]
        }
        None => [x.1, y.1, z.1],
    };
    let (speed, horizontal) = speeds(velocity);
    let acceleration = 2. * z.0;
    let airborne = if z.3 > MAX_RESIDUAL {
        None
    } else if (acceleration + gravity).abs() <= FALLING * gravity {
        Some(true)
    } else if acceleration.abs() <= LEVEL * gravity {
        Some(false)
    } else {
        None
    };

    MotionState {
        evidence: if airborne.is_some() {
            MotionEvidence::Trajectory
        } else {
            MotionEvidence::Unclear
        },
        airborne,
        speed: Some(speed),
        horizontal_speed: Some(horizontal),
        origin: Some(last.origin),
    }
}

/// Recent samples of every player entity, from the snapshots as they arrive
#[derive(Default)]
pub struct Motion {
    history: BTreeMap<u16, VecDeque<Sample>>,
    last: Option<Arc<Snapshot>>,
    segment: u32,
}

impl Motion {
    /// Call for every decoded message. Each new snapshot is taken once,
    /// timed by its server time, since positions belong to the server's
    /// simulation and the client plays packets in bursts
    pub fn observe(&mut self, d: &Decoder) {
        let Some(snap) = d.latest_snapshot() else {
            return;
        };
        if self.last.as_ref().is_some_and(|l| Arc::ptr_eq(l, snap)) {
            return;
        }
        self.last = Some(snap.clone());
        if d.segment != self.segment {
            self.history.clear();
            self.segment = d.segment;
        }

        let time = f32::from_bits(snap.server_time_bits);
        let recorder = u16::from(d.player_num) + 1;
        let exact = recorder_exact(d, snap);
        if let Some(entities) = &snap.entities {
            for e in entities
                .iter()
                .filter(|e| e.number >= 1 && e.number <= u16::from(d.max_clients))
            {
                let Some(schema) = d.entity_schema(e.number, e.custom) else {
                    continue;
                };
                let coord = |name: &str| match e.fields.value(&schema, name) {
                    Some(Value::Float(f)) => f,
                    _ => 0.,
                };
                let mut origin = [coord("origin[0]"), coord("origin[1]"), coord("origin[2]")];
                let mut sample_exact = None;
                if e.number == recorder
                    && let Some((o, x)) = exact
                {
                    origin = o;
                    sample_exact = Some(x);
                }

                let ring = self.history.entry(e.number).or_default();
                if ring.back().is_some_and(|s| s.time >= time) {
                    continue;
                }
                ring.push_back(Sample {
                    time,
                    origin,
                    exact: sample_exact,
                });
                while ring.front().is_some_and(|s| time - s.time > HISTORY) {
                    ring.pop_front();
                }
            }
        }
    }

    /// The recent samples of a player entity, oldest first
    pub fn samples(&self, entity: u16) -> Vec<Sample> {
        self.history
            .get(&entity)
            .map(|r| r.iter().copied().collect())
            .unwrap_or_default()
    }
}

/// Origin, on-ground flag and velocity of the recording player
fn recorder_exact(d: &Decoder, snap: &Snapshot) -> Option<([f32; 3], Exact)> {
    let schema = d.schema("clientdata_t")?;
    let client = snap.clientdata.as_ref()?;
    let float = |name: &str| {
        client
            .value(&schema, name)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    };
    let origin = [
        float("origin[0]")?,
        float("origin[1]")?,
        float("origin[2]")?,
    ];
    let velocity = [
        float("velocity[0]")?,
        float("velocity[1]")?,
        float("velocity[2]")?,
    ];
    let flags = client.value(&schema, "flags")?.as_i64()?;

    Some((
        origin,
        Exact {
            on_ground: flags & FL_ONGROUND != 0,
            velocity,
        },
    ))
}

/// The server's gravity from the last `svc_newmovevars`, whose body starts
/// with it as a float
pub fn gravity(d: &Decoder) -> f32 {
    d.movevars
        .as_deref()
        .and_then(|m| m.get(..4))
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .filter(|g| g.is_finite() && *g > 0.)
        .unwrap_or(DEFAULT_GRAVITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(times: &[f32], z: impl Fn(f32) -> f32) -> Vec<Sample> {
        times
            .iter()
            .map(|&t| Sample {
                time: t,
                origin: [t * 300., 0., (z(t) * 8.).round() / 8.],
                exact: None,
            })
            .collect()
    }

    fn frames(from: f32, to: f32) -> Vec<f32> {
        let mut t = from;
        let mut out = Vec::new();
        while t <= to {
            out.push(t);
            t += 0.01;
        }
        out
    }

    #[test]
    fn a_fall_is_airborne_and_level_running_is_not() {
        let times = frames(0., 0.5);
        let falling = path(&times, |t| 100. + 268. * t - 400. * t * t);
        let s = state_at(&falling, 0.5, 800.);
        assert_eq!(
            (s.evidence, s.airborne),
            (MotionEvidence::Trajectory, Some(true))
        );
        assert!((s.horizontal_speed.unwrap() - 300.).abs() < 5., "{s:?}");

        let level = path(&times, |_| 100.);
        let s = state_at(&level, 0.5, 800.);
        assert_eq!(s.airborne, Some(false));
    }

    #[test]
    fn a_landing_inside_the_window_and_gaps_stay_unknown() {
        // Falls until 0.4 s, then runs on the ground
        let times = frames(0., 0.5);
        let landing = path(&times, |t| {
            if t < 0.4 {
                100. + 160. * t - 400. * t * t
            } else {
                100. + 160. * 0.4 - 400. * 0.16
            }
        });
        assert_eq!(state_at(&landing, 0.5, 800.).airborne, None);

        // Not seen for a second before the kill
        let old = path(&frames(0., 0.3), |_| 100.);
        assert_eq!(
            state_at(&old, 1.3, 800.).evidence,
            MotionEvidence::NotVisible
        );
    }

    #[test]
    fn the_recorder_state_is_exact() {
        let samples = [Sample {
            time: 1.,
            origin: [0.; 3],
            exact: Some(Exact {
                on_ground: false,
                velocity: [300., 400., 0.],
            }),
        }];
        let s = state_at(&samples, 1.0, 800.);
        assert_eq!(
            (s.evidence, s.airborne, s.speed),
            (MotionEvidence::Exact, Some(true), Some(500.))
        );
    }
}
