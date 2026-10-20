//! A demo's weapon activity arranged for the timeline (implementation plan
//! §8)
//!
//! One lane per player connection, keyed by slot and incarnation so a
//! reused slot is a new lane, plus one lane for actors that could not be
//! resolved. Marks and spans are sorted by time within their lane, so the
//! visible part of a lane is found by binary search and can be grouped into
//! bins at wide zoom. The full report is dropped once this is built

use lambda_slicer_core::analysis::AnalysisReport;
use lambda_slicer_core::analysis::accuracy::ShotOutcome;
use lambda_slicer_core::analysis::context::{
    Coverage, Kill, KillCause, ScoreUpdate, TeamScoreUpdate,
};
use lambda_slicer_core::analysis::motion::{MotionEvidence, MotionState};
use lambda_slicer_core::analysis::weapons::{Action, Actor, IntervalEnd, Weapon};
use lambda_slicer_core::message::Message;
use lambda_slicer_core::slice::range::GroupIndex;
use lambda_slicer_core::state::PlayerIdentity;
use lambda_slicer_playback::kill_features::GaussKill;

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
        /// The killer's slot and incarnation
        killer: Option<(u8, u32)>,
        headshot: Option<bool>,
        /// What kill filters and badges read, boxed as kills are a small
        /// share of the events
        features: Box<KillFeatures>,
    },
    Switch,
}

/// A kill's place among the analysis's kills, the same however the events
/// are sorted or filtered. With the demo view's generation it names one
/// kill of one opened demo
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KillKey {
    pub segment: u32,
    /// Its position in the report's kills
    pub ordinal: u32,
}

/// When a player's movement around a kill was sampled
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SampledAt {
    /// At the shot the analysis linked to the kill, by playback time
    Shot(f32),
    /// When the death message arrived
    Kill,
}

/// One player's movement around a kill as the analysis estimated it. Only
/// the recording player's is exact, and a value that is absent is unknown,
/// never grounded or standing still
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KillMotion {
    pub airborne: Option<bool>,
    /// Units per second across the ground, leaving out vertical speed. A
    /// value that is not a finite speed is left out too
    pub horizontal_speed: Option<f32>,
    pub evidence: MotionEvidence,
    pub sampled_at: SampledAt,
}

impl KillMotion {
    fn new(state: &MotionState, sampled_at: SampledAt) -> Self {
        let horizontal_speed = state
            .horizontal_speed
            .filter(|speed| speed.is_finite() && *speed >= 0.);

        Self {
            airborne: state.airborne,
            horizontal_speed,
            evidence: state.evidence,
            sampled_at,
        }
    }
}

/// What a kill's filters and badges read of it, kept from the analysis
/// before its report is dropped
#[derive(Clone, Debug, PartialEq)]
pub struct KillFeatures {
    pub key: KillKey,
    /// Both players' movement, none when the analysis gave the kill no
    /// context. The killer is sampled at the linked shot, or at the kill
    /// when no shot was linked, and the victim at the kill
    pub killer_motion: Option<KillMotion>,
    pub victim_motion: Option<KillMotion>,
    /// The occurrence the analysis linked, a likely shot rather than proof
    /// of what did the damage
    pub shot: Option<usize>,
    /// The victim's connection
    pub victim: Option<LaneKey>,
    /// Teams from `TeamInfo` when the death message arrived
    pub killer_team: Option<String>,
    pub victim_team: Option<String>,
    /// A player killed with the Gauss, the only kill that can be through a
    /// wall
    pub gauss: bool,
    /// The Gauss kill with the killer's shots that could have made it, for
    /// telling whether it went through a wall
    pub gauss_kill: Option<GaussKill>,
}

impl KillFeatures {
    fn new(kill: &Kill, ordinal: usize, gauss_kill: Option<GaussKill>) -> Self {
        let key = KillKey {
            segment: kill.segment,
            ordinal: u32::try_from(ordinal).unwrap_or(u32::MAX),
        };

        let context = kill.context.as_ref();
        let killer_at = match context.and_then(|c| c.shot.as_ref()) {
            Some(shot) => SampledAt::Shot(shot.time),
            None => SampledAt::Kill,
        };
        let killer_motion = context.map(|c| KillMotion::new(&c.killer, killer_at));
        let victim_motion = context.map(|c| KillMotion::new(&c.victim, SampledAt::Kill));
        let shot = context.and_then(|c| c.shot.as_ref()).map(|s| s.occurrence);

        Self {
            key,
            killer_motion,
            victim_motion,
            shot,
            victim: kill.victim.as_ref().map(key_of),
            killer_team: kill.killer_team.clone(),
            victim_team: kill.victim_team.clone(),
            gauss: kill.weapon == Some(Weapon::Gauss),
            gauss_kill,
        }
    }
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
    pub segment: u32,
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
    /// Every counted shot's outcome, for accuracy at any moment
    pub shots: Vec<ShotOutcome>,
    /// The recording player's slot and incarnation
    pub recorder: Option<(u8, u32)>,
    /// Frame groups, for the range a clip's export would cut
    pub groups: GroupIndex,
    /// Sessions whose scoreboard names teams, where a player's team is a
    /// side rather than a model name
    pub team_segments: Vec<u32>,
}

impl Model {
    /// Whether `segment` was played in teams
    pub fn team_play(&self, segment: u32) -> bool {
        self.team_segments.contains(&segment)
    }
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

        // The recording player's events aim as they looked, other players'
        // as their model pitched
        let recorder = r.recorder.as_ref().map(|p| u16::from(p.slot) + 1);
        for (ordinal, k) in r.kills.iter().enumerate() {
            let killer = k.killer.as_ref();
            let lane = match killer {
                Some(p) => b.lane(key_of(p), Some(&p.name)),
                None => b.lane(LaneKey::Unknown, None),
            };
            let victim = k.victim.as_ref().map(|p| p.name.clone());
            let gauss_kill = GaussKill::of(k, &r.occurrences, r.max_clients, recorder);
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
                    killer: killer.map(|p| (p.slot, p.incarnation)),
                    headshot: k.headshot,
                    features: Box::new(KillFeatures::new(k, ordinal, gauss_kill)),
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

        // The playback entry's own extent, its last group or terminal record,
        // so a quiet tail stays in view and nothing reaches past the end.
        // Without frame groups, the library's duration or the last event
        let playback = r
            .groups
            .groups
            .iter()
            .map(|g| g.time)
            .chain(r.groups.end)
            .fold(0f32, f32::max);
        let last = events.last().map_or(0., |e| e.time);
        let end = if playback > 0. {
            playback
        } else {
            duration.max(last)
        };

        // A scoreboard names teams only in team play
        let mut team_segments = Vec::new();
        for update in &r.score_updates {
            if !update.team.is_empty() && !team_segments.contains(&update.segment) {
                team_segments.push(update.segment);
            }
        }

        Model {
            lanes: b.lanes,
            events,
            start: 0.,
            end,
            coverage_notes: coverage_notes(&r.coverage),
            decode_error: r.decode_error.clone(),
            windows: r
                .action_windows
                .iter()
                .map(|w| Window {
                    segment: w.segment,
                    start: w.start,
                    end: w.end,
                    score: w.score,
                    sides: w.sides.clone(),
                })
                .collect(),
            score_updates: r.score_updates.clone(),
            team_score_updates: r.team_score_updates.clone(),
            shots: r.shot_outcomes.clone(),
            recorder: r.recorder.as_ref().map(|p| (p.slot, p.incarnation)),
            groups: r.groups.clone(),
            team_segments,
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
pub fn bins(
    lane: &Lane,
    from: f32,
    to: f32,
    width: f32,
    visible: impl Fn(&Mark) -> bool,
) -> Vec<Bin> {
    let width = width.max(f32::EPSILON);
    let mut out: Vec<Bin> = Vec::new();
    // Hidden marks are left out first, so a bin never mixes in a weapon the
    // filters hide
    for m in lane.marks_in(from, to).iter().filter(|m| visible(m)) {
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

/// The visible mark nearest `time` in `lane` within `tolerance` seconds,
/// so a hidden mark never stands in front of a visible one
pub fn nearest(
    lane: &Lane,
    time: f32,
    tolerance: f32,
    visible: impl Fn(&Mark) -> bool,
) -> Option<&Mark> {
    lane.marks_in(time - tolerance, time + tolerance)
        .iter()
        .filter(|m| visible(m))
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
    use lambda_slicer_core::analysis::context::RecordingKind;
    use lambda_slicer_core::analysis::highlights::{KillContext, LinkedShot};
    use lambda_slicer_core::state::DecodeStatus;

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
        let all = |_: &Mark| true;
        assert_eq!(nearest(&l, 2.9, 0.5, all).unwrap().kind, MarkKind::Kill);
        assert!(nearest(&l, 6., 0.5, all).is_none());
        // A hidden kill nearer the pointer does not hide the shot behind it
        let shots = |m: &Mark| m.kind == MarkKind::Shot;
        assert_eq!(nearest(&l, 2.9, 1., shots).unwrap().time, 2.);
    }

    #[test]
    fn filtered_bins_hold_only_visible_weapons() {
        let mut l = lane(&[(0.1, MarkKind::Shot), (0.2, MarkKind::Shot)]);
        l.marks[0].weapon = Some(Weapon::Glock);
        l.marks[1].weapon = Some(Weapon::Mp5);
        let mixed = bins(&l, 0., 1., 1., |_| true);
        assert_eq!((mixed[0].count, mixed[0].weapon), (2, None));
        let gauss = bins(&l, 0., 1., 1., |m| m.weapon == Some(Weapon::Gauss));
        assert!(gauss.is_empty());
        let mp5 = bins(&l, 0., 1., 1., |m| m.weapon == Some(Weapon::Mp5));
        assert_eq!((mp5[0].count, mp5[0].weapon), (1, Some(Weapon::Mp5)));
    }

    #[test]
    fn wide_zoom_groups_marks_per_bin_and_kind() {
        let l = lane(&[
            (0.1, MarkKind::Shot),
            (0.2, MarkKind::Shot),
            (0.3, MarkKind::Kill),
            (1.5, MarkKind::Shot),
        ]);
        let b = bins(&l, 0., 2., 1., |_| true);
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

    fn player(slot: u8, incarnation: u32, name: &str) -> PlayerIdentity {
        PlayerIdentity {
            segment: 0,
            slot,
            user_id: u32::from(slot),
            incarnation,
            name: name.into(),
        }
    }

    fn motion(airborne: Option<bool>, horizontal_speed: Option<f32>) -> MotionState {
        MotionState {
            evidence: MotionEvidence::Trajectory,
            airborne,
            speed: horizontal_speed,
            horizontal_speed,
            origin: None,
        }
    }

    fn kill(time: f32, context: Option<KillContext>) -> Kill {
        Kill {
            segment: 0,
            time,
            server_time: time,
            killer_entity: 1,
            victim_entity: 2,
            killer: Some(player(0, 0, "killer")),
            victim: Some(player(1, 3, "victim")),
            killer_team: Some("blue".into()),
            victim_team: None,
            weapon_name: "gauss".into(),
            cause: KillCause::PlayerWeapon,
            weapon: Some(Weapon::Gauss),
            victim_state: MotionState::not_visible(),
            killer_samples: Vec::new(),
            context,
            headshot: None,
        }
    }

    fn report(kills: Vec<Kill>) -> AnalysisReport {
        AnalysisReport {
            schema: 0,
            profile: "",
            path: String::new(),
            decode: DecodeStatus::Complete,
            decode_error: None,
            recorder: None,
            players: Vec::new(),
            max_clients: 2,
            events: 0,
            weapons: Vec::new(),
            actions: Vec::new(),
            occurrences: Vec::new(),
            intervals: Vec::new(),
            unclassified: Vec::new(),
            kills,
            action_windows: Vec::new(),
            weapon_switches: Vec::new(),
            coverage: Coverage {
                kind: RecordingKind::MultiplayerView,
                local_events: 0,
                network_events: 0,
                kills: 0,
                playback_seconds: 0.,
                spectator_seconds: 0.,
                spectator_fraction: 0.,
                notes: Vec::new(),
            },
            scoreboard: Default::default(),
            score_updates: Vec::new(),
            team_score_updates: Vec::new(),
            shot_outcomes: Vec::new(),
            accuracy: Vec::new(),
            recorder_accuracy: None,
            recorder_headshots: None,
            groups: Default::default(),
        }
    }

    fn features(model: &Model, event: usize) -> &KillFeatures {
        match &model.events[event].kind {
            EventKind::Kill { features, .. } => features,
            other => panic!("not a kill: {other:?}"),
        }
    }

    #[test]
    fn kill_features_survive_the_model_with_their_own_kill() {
        let linked = KillContext {
            killer: motion(Some(true), Some(1200.)),
            victim: motion(None, Some(f32::NAN)),
            shot: Some(LinkedShot {
                occurrence: 7,
                weapon: Weapon::Gauss,
                action: Action::Fire,
                mode: Some("charged"),
                time: 9.5,
                flight: 0.,
            }),
            distance: None,
        };

        // The second kill comes first in time, so sorting moves it
        let r = report(vec![kill(10., Some(linked)), kill(4., None)]);
        let model = Model::build(&r, 20.);

        let first = features(&model, 1);
        assert_eq!((first.key.segment, first.key.ordinal), (0, 0));
        assert_eq!(first.shot, Some(7));

        let killer = first.killer_motion.unwrap();
        assert_eq!(killer.airborne, Some(true));
        assert_eq!(killer.horizontal_speed, Some(1200.));
        assert_eq!(killer.sampled_at, SampledAt::Shot(9.5));

        // An unknown state and a speed that is not finite stay unknown
        let victim = first.victim_motion.unwrap();
        assert_eq!((victim.airborne, victim.horizontal_speed), (None, None));
        assert_eq!(victim.sampled_at, SampledAt::Kill);

        assert_eq!(
            first.victim,
            Some(LaneKey::Player {
                segment: 0,
                slot: 1,
                incarnation: 3
            })
        );
        assert_eq!(first.killer_team.as_deref(), Some("blue"));

        // A kill the analysis gave no context knows nothing of movement
        let second = features(&model, 0);
        assert_eq!((second.key.segment, second.key.ordinal), (0, 1));
        assert_eq!((second.killer_motion, second.victim_motion), (None, None));
        assert_eq!(second.shot, None);
    }

    #[test]
    fn tick_steps_leave_room_for_labels() {
        assert_eq!(tick_step(0.3), 0.5);
        assert_eq!(tick_step(7.), 10.);
        assert_eq!(tick_step(5000.), 3600.);
    }
}
