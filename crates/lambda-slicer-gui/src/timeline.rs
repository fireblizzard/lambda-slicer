//! A demo's weapon activity arranged for the timeline (implementation plan
//! §8)
//!
//! One lane per player connection, keyed by slot and incarnation so a
//! reused slot is a new lane, plus one lane for actors that could not be
//! resolved. Marks and spans are sorted by time within their lane, so the
//! visible part of a lane is found by binary search and can be grouped into
//! bins at wide zoom. The full report is dropped once this is built

use lambda_slicer_core::analysis::AnalysisReport;
use lambda_slicer_core::analysis::context::{Coverage, KillCause, ScoreUpdate, TeamScoreUpdate};
use lambda_slicer_core::analysis::weapons::{Action, Actor, IntervalEnd, Weapon};
use lambda_slicer_core::message::Message;
use lambda_slicer_core::state::PlayerIdentity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LaneKey {
    Player {
        segment: u32,
        slot: u8,
        incarnation: u32,
    },
    /// Actors with no known player
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub key: LaneKey,
    /// Names used in this lane, first first
    pub names: Vec<String>,
    pub recorder: bool,
    /// Marks in time order
    pub marks: Vec<Mark>,
    /// Charge and beam spans in start order
    pub spans: Vec<Span>,
}

impl Lane {
    /// Marks whose time falls in `[from, to]`
    pub fn marks_in(&self, from: f32, to: f32) -> &[Mark] {
        let a = self.marks.partition_point(|m| m.time < from);
        let b = self.marks.partition_point(|m| m.time <= to);
        &self.marks[a..b.max(a)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkKind {
    Shot,
    /// A recorded action that is not a shot, such as a stop or an attempt
    Action,
    Kill,
    Death,
    /// The recorder changed weapon
    Switch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub time: f32,
    pub kind: MarkKind,
    pub weapon: Option<Weapon>,
    /// Index into [`Model::events`]
    pub event: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind {
    Charge,
    Beam,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub start: f32,
    pub end: f32,
    pub kind: SpanKind,
    pub weapon: Weapon,
    /// The start or end lies outside what the recording shows
    pub start_known: bool,
    pub end_known: bool,
}

/// What happened, for the event table
#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    Action {
        action: Action,
        mode: Option<&'static str>,
    },
    Kill {
        victim: Option<String>,
        weapon_name: String,
        cause: KillCause,
    },
    Switch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// The session, since times restart with each one
    pub segment: u32,
    pub time: f32,
    pub lane: usize,
    /// The actor's name at the time
    pub actor: Option<String>,
    pub weapon: Option<Weapon>,
    pub kind: EventKind,
}

/// A stretch of play by action score, best first in the model
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    pub start: f32,
    pub end: f32,
    pub score: f64,
    /// Frags per side, busiest first
    pub sides: Vec<(String, usize)>,
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub lanes: Vec<Lane>,
    /// Every event in time order
    pub events: Vec<Event>,
    pub start: f32,
    pub end: f32,
    pub coverage_notes: Vec<Message>,
    pub decode_error: Option<Message>,
    /// The best action windows, highest score first
    pub windows: Vec<Window>,
    /// Score and team changes, for the scores at any moment
    pub score_updates: Vec<ScoreUpdate>,
    pub team_score_updates: Vec<TeamScoreUpdate>,
}

fn key_of(p: &PlayerIdentity) -> LaneKey {
    LaneKey::Player {
        segment: p.segment,
        slot: p.slot,
        incarnation: p.incarnation,
    }
}

struct Builder {
    lanes: Vec<Lane>,
    events: Vec<Event>,
}

impl Builder {
    fn lane(&mut self, key: LaneKey, name: Option<&str>) -> usize {
        let at = match self.lanes.iter().position(|l| l.key == key) {
            Some(at) => at,
            None => {
                self.lanes.push(Lane {
                    key,
                    names: Vec::new(),
                    recorder: false,
                    marks: Vec::new(),
                    spans: Vec::new(),
                });
                self.lanes.len() - 1
            }
        };
        if let Some(n) = name
            && !self.lanes[at].names.iter().any(|x| x == n)
        {
            self.lanes[at].names.push(n.to_string());
        }
        at
    }

    fn actor_lane(&mut self, actor: &Actor) -> (usize, Option<String>) {
        match actor {
            Actor::Player { identity, .. } => (
                self.lane(key_of(identity), Some(&identity.name)),
                Some(identity.name.clone()),
            ),
            _ => (self.lane(LaneKey::Unknown, None), None),
        }
    }

    fn mark(
        &mut self,
        lane: usize,
        time: f32,
        kind: MarkKind,
        weapon: Option<Weapon>,
        event: Event,
    ) {
        self.events.push(event);
        let event = self.events.len() - 1;
        self.lanes[lane].marks.push(Mark {
            time,
            kind,
            weapon,
            event,
        });
    }
}

impl Model {
    pub fn build(r: &AnalysisReport, duration: f32) -> Self {
        let mut b = Builder {
            lanes: Vec::new(),
            events: Vec::new(),
        };
        // Players in the order they appeared, so lanes read like the scoreboard
        for p in &r.players {
            b.lane(key_of(p), Some(&p.name));
        }

        for o in r.occurrences.iter().filter(|o| !o.action.is_activity()) {
            let (lane, actor) = b.actor_lane(&o.actor);
            let kind = if o.shot {
                MarkKind::Shot
            } else {
                MarkKind::Action
            };
            b.mark(
                lane,
                o.time,
                kind,
                Some(o.weapon),
                Event {
                    segment: o.segment,
                    time: o.time,
                    lane,
                    actor,
                    weapon: Some(o.weapon),
                    kind: EventKind::Action {
                        action: o.action,
                        mode: o.mode,
                    },
                },
            );
        }

        for iv in &r.intervals {
            let (lane, _) = b.actor_lane(&iv.actor);
            let kind = if iv.weapon == Weapon::Egon {
                SpanKind::Beam
            } else {
                SpanKind::Charge
            };
            b.lanes[lane].spans.push(Span {
                start: iv.start,
                end: iv.end,
                kind,
                weapon: iv.weapon,
                start_known: iv.start_known,
                end_known: iv.end_reason != IntervalEnd::Unknown,
            });
        }

        for k in &r.kills {
            let killer = k.killer.as_ref();
            let lane = match killer {
                Some(p) => b.lane(key_of(p), Some(&p.name)),
                None => b.lane(LaneKey::Unknown, None),
            };
            let victim = k.victim.as_ref().map(|p| p.name.clone());
            let event = Event {
                segment: k.segment,
                time: k.time,
                lane,
                actor: killer.map(|p| p.name.clone()),
                weapon: k.weapon,
                kind: EventKind::Kill {
                    victim: victim.clone(),
                    weapon_name: k.weapon_name.clone(),
                    cause: k.cause,
                },
            };
            b.mark(lane, k.time, MarkKind::Kill, k.weapon, event);
            // The victim's lane shows the death, pointing at the same event
            if let Some(v) = &k.victim {
                let vl = b.lane(key_of(v), Some(&v.name));
                let event = b.events.len() - 1;
                b.lanes[vl].marks.push(Mark {
                    time: k.time,
                    kind: MarkKind::Death,
                    weapon: k.weapon,
                    event,
                });
            }
        }

        if let Some(rec) = &r.recorder {
            let lane = b.lane(key_of(rec), Some(&rec.name));
            b.lanes[lane].recorder = true;
            for s in &r.weapon_switches {
                b.mark(
                    lane,
                    s.time,
                    MarkKind::Switch,
                    s.weapon,
                    Event {
                        segment: s.segment,
                        time: s.time,
                        lane,
                        actor: Some(rec.name.clone()),
                        weapon: s.weapon,
                        kind: EventKind::Switch,
                    },
                );
            }
        }

        // Events in time order, with marks pointing at their new places
        let mut order: Vec<usize> = (0..b.events.len()).collect();
        order.sort_by(|x, y| b.events[*x].time.total_cmp(&b.events[*y].time));
        let mut new_index = vec![0; order.len()];
        for (new, old) in order.iter().enumerate() {
            new_index[*old] = new;
        }
        let events: Vec<Event> = order.iter().map(|i| b.events[*i].clone()).collect();
        for lane in &mut b.lanes {
            for m in &mut lane.marks {
                m.event = new_index[m.event];
            }
            lane.marks.sort_by(|x, y| x.time.total_cmp(&y.time));
            lane.spans.sort_by(|x, y| x.start.total_cmp(&y.start));
        }

        // Lanes with nothing to show are left out, the unknown lane last
        b.lanes
            .retain(|l| !l.marks.is_empty() || !l.spans.is_empty());
        let unknown = b.lanes.iter().position(|l| l.key == LaneKey::Unknown);
        if let Some(at) = unknown {
            let lane = b.lanes.remove(at);
            b.lanes.push(lane);
        }
        let mut events = events;
        let lane_of: std::collections::HashMap<LaneKey, usize> = b
            .lanes
            .iter()
            .enumerate()
            .map(|(i, l)| (l.key, i))
            .collect();
        for lane in &b.lanes {
            for m in &lane.marks {
                if m.kind != MarkKind::Death {
                    events[m.event].lane = lane_of[&lane.key];
                }
            }
        }

        let last = events.last().map_or(0., |e| e.time);
        Model {
            lanes: b.lanes,
            events,
            start: 0.,
            end: duration.max(last),
            coverage_notes: coverage_notes(&r.coverage),
            decode_error: r.decode_error.clone(),
            windows: r
                .action_windows
                .iter()
                .map(|w| Window {
                    start: w.start,
                    end: w.end,
                    score: w.score,
                    sides: w.sides.clone(),
                })
                .collect(),
            score_updates: r.score_updates.clone(),
            team_score_updates: r.team_score_updates.clone(),
        }
    }
}

fn coverage_notes(c: &Coverage) -> Vec<Message> {
    c.notes.clone()
}

/// One bin of marks at wide zoom
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bin {
    pub start: f32,
    pub kind: MarkKind,
    pub weapon: Option<Weapon>,
    pub count: usize,
    /// The first mark's event, for selecting by click
    pub event: usize,
}

/// Marks of `lane` in `[from, to]` grouped into bins `width` seconds wide,
/// one per kind and bin, in time order
pub fn bins(lane: &Lane, from: f32, to: f32, width: f32) -> Vec<Bin> {
    let width = width.max(f32::EPSILON);
    let mut out: Vec<Bin> = Vec::new();
    for m in lane.marks_in(from, to) {
        let start = from + ((m.time - from) / width).floor() * width;
        match out
            .iter_mut()
            .rev()
            .take_while(|b| b.start == start)
            .find(|b| b.kind == m.kind)
        {
            Some(b) => {
                b.count += 1;
                if b.weapon != m.weapon {
                    b.weapon = None;
                }
            }
            None => out.push(Bin {
                start,
                kind: m.kind,
                weapon: m.weapon,
                count: 1,
                event: m.event,
            }),
        }
    }
    out
}

/// The mark nearest `time` in `lane` within `tolerance` seconds
pub fn nearest(lane: &Lane, time: f32, tolerance: f32) -> Option<&Mark> {
    lane.marks_in(time - tolerance, time + tolerance)
        .iter()
        .min_by(|a, b| (a.time - time).abs().total_cmp(&(b.time - time).abs()))
}

/// A tick spacing in seconds that leaves at least `min_gap` seconds between
/// labels
pub fn tick_step(min_gap: f32) -> f32 {
    const STEPS: [f32; 14] = [
        0.1, 0.25, 0.5, 1., 2., 5., 10., 15., 30., 60., 120., 300., 600., 1800.,
    ];
    STEPS
        .iter()
        .copied()
        .find(|s| *s >= min_gap)
        .unwrap_or(3600.)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane(times: &[(f32, MarkKind)]) -> Lane {
        Lane {
            key: LaneKey::Unknown,
            names: Vec::new(),
            recorder: false,
            marks: times
                .iter()
                .enumerate()
                .map(|(i, (t, k))| Mark {
                    time: *t,
                    kind: *k,
                    weapon: Some(Weapon::Gauss),
                    event: i,
                })
                .collect(),
            spans: Vec::new(),
        }
    }

    #[test]
    fn visible_marks_are_found_by_time() {
        let l = lane(&[
            (1., MarkKind::Shot),
            (2., MarkKind::Shot),
            (3., MarkKind::Kill),
            (9., MarkKind::Shot),
        ]);
        assert_eq!(l.marks_in(1.5, 3.).len(), 2);
        assert_eq!(l.marks_in(10., 20.).len(), 0);
        assert_eq!(nearest(&l, 2.9, 0.5).unwrap().kind, MarkKind::Kill);
        assert!(nearest(&l, 6., 0.5).is_none());
    }

    #[test]
    fn wide_zoom_groups_marks_per_bin_and_kind() {
        let l = lane(&[
            (0.1, MarkKind::Shot),
            (0.2, MarkKind::Shot),
            (0.3, MarkKind::Kill),
            (1.5, MarkKind::Shot),
        ]);
        let b = bins(&l, 0., 2., 1.);
        assert_eq!(
            b.iter()
                .map(|b| (b.start, b.kind, b.count))
                .collect::<Vec<_>>(),
            [
                (0., MarkKind::Shot, 2),
                (0., MarkKind::Kill, 1),
                (1., MarkKind::Shot, 1)
            ]
        );
    }

    #[test]
    fn tick_steps_leave_room_for_labels() {
        assert_eq!(tick_step(0.3), 0.5);
        assert_eq!(tick_step(7.), 10.);
        assert_eq!(tick_step(5000.), 3600.);
    }
}
