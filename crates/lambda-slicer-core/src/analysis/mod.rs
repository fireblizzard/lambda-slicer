//! Weapon activity and actors
//!
//! [`events`] keeps the recording's events as evidence, and [`weapons`]
//! derives weapon actions and activity intervals from them with a versioned
//! profile. Counts say whether they are raw records or actions

pub mod accuracy;
pub mod context;
pub mod events;
pub mod grenades;
pub mod highlights;
pub mod match_events;
pub mod motion;
pub mod parts;
pub mod roster;
pub mod streaks;
pub mod weapons;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use crate::container::Limits;
use crate::message::Message;
use crate::slice::range::GroupIndex;
use crate::state::{DecodeOptions, DecodeStatus, PlayerIdentity, decode_demo};
use accuracy::{PlayerAccuracy, ShotOutcome};
use context::{
    ChatMessage, Context, Coverage, Kill, ScoreUpdate, Scoreboard, SessionMap, TeamScoreUpdate,
    WeaponSwitch, final_moment, standing_at,
};
use events::recorded_events;
use highlights::{ActionWindow, action_windows, link_kills};
use parts::{Played, PlayedPart, demo_counted, played_parts};
use weapons::{Action, ActivityInterval, PROFILE, Weapon, WeaponOccurrence, derive};

pub const ANALYSIS_SCHEMA: u32 = 20;

/// Length of an action window in seconds, and how many are reported
pub const ACTION_WINDOW: f32 = 60.;
pub const ACTION_WINDOWS: usize = 5;

/// Actions of one weapon, action and mode
#[derive(Clone, Debug, Serialize)]
pub struct ActionCount {
    pub weapon: Weapon,
    pub action: Action,
    pub mode: Option<&'static str>,
    pub count: usize,
    /// Of those, with a known player
    pub attributed: usize,
}

/// Shots per weapon, and the raw records behind every action
#[derive(Clone, Debug, Serialize)]
pub struct WeaponSummary {
    pub weapon: Weapon,
    /// Fire and launch actions
    pub shots: usize,
    /// Kills of other players with this weapon, which are not shots
    pub kills: usize,
    /// Every recorded event of this weapon, activity updates included
    pub records: usize,
    pub intervals: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct AnalysisReport {
    pub schema: u32,
    pub profile: &'static str,
    pub path: String,
    pub decode: DecodeStatus,
    pub decode_error: Option<Message>,
    /// The recording client's player, at the end of the recording
    pub recorder: Option<PlayerIdentity>,
    /// Every player seen, once per name they used
    pub players: Vec<PlayerIdentity>,
    pub max_clients: u8,
    /// Recorded events of any kind
    pub events: usize,
    pub weapons: Vec<WeaponSummary>,
    pub actions: Vec<ActionCount>,
    pub occurrences: Vec<WeaponOccurrence>,
    pub intervals: Vec<ActivityInterval>,
    /// Event scripts the profile does not know, with their record counts
    pub unclassified: Vec<(String, usize)>,
    pub kills: Vec<Kill>,
    /// The best stretches of play by action score
    pub action_windows: Vec<ActionWindow>,
    /// The recorder's active weapon over time
    pub weapon_switches: Vec<WeaponSwitch>,
    pub coverage: Coverage,
    /// Each session's map, in decode order
    pub maps: Vec<SessionMap>,
    /// The chat, in decode order
    pub chat: Vec<ChatMessage>,
    /// Votes, timeouts, pauses and sudden death, each change once
    pub match_events: Vec<match_events::MatchEvent>,
    /// How many players and teams the demo's complete matches had at most,
    /// or the most at any moment without one
    pub counted: roster::Counted,
    /// Each match, and each stretch of free play long enough to list, in
    /// the order played
    pub parts: Vec<PlayedPart>,
    /// The final score of the session played longest
    pub scoreboard: Scoreboard,
    /// Every score and team change, for the scores at any moment
    pub score_updates: Vec<ScoreUpdate>,
    pub team_score_updates: Vec<TeamScoreUpdate>,
    /// Whether each counted shot hit, and each player's accuracy
    /// ([`accuracy`])
    #[serde(skip)]
    pub shot_outcomes: Vec<ShotOutcome>,
    pub accuracy: Vec<PlayerAccuracy>,
    /// The recording player's accuracy, from its exact aim, when it fired
    /// enough counted shots
    pub recorder_accuracy: Option<f64>,
    /// The recording player's headshot kills. Other players' are often out
    /// of view and so incomplete
    pub recorder_headshots: Option<u32>,
    /// The playback entry's frame groups, for snapping a clip's range
    #[serde(skip)]
    pub groups: GroupIndex,
}

fn summary_of(weapons: &mut Vec<WeaponSummary>, weapon: Weapon) -> &mut WeaponSummary {
    let at = match weapons.iter().position(|w| w.weapon == weapon) {
        Some(at) => at,
        None => {
            weapons.push(WeaponSummary {
                weapon,
                shots: 0,
                kills: 0,
                records: 0,
                intervals: 0,
            });
            weapons.len() - 1
        }
    };

    &mut weapons[at]
}

pub fn analyze_demo(path: &Path, limits: Limits) -> AnalysisReport {
    analyze_demo_with(path, limits, None, None)
}

/// [`analyze_demo`] that stops when `cancel` is set, reporting
/// [`DecodeStatus::Cancelled`], and that keeps players just off the floor
/// out of the air when `floors` can read the maps
pub fn analyze_demo_with(
    path: &Path,
    limits: Limits,
    cancel: Option<Arc<AtomicBool>>,
    floors: Option<&motion::Floors>,
) -> AnalysisReport {
    let options = DecodeOptions {
        limits,
        collect_events: true,
        cancel,
    };
    let mut context = Context::default();
    let (report, d) = decode_demo(path, &options, |loc, d, m| context.observe(loc, d, m));
    let events = recorded_events(&d);
    let derived = derive(&events, report.max_clients);

    let mut actions: Vec<ActionCount> = Vec::new();
    for o in &derived.occurrences {
        let attributed = usize::from(matches!(o.actor, weapons::Actor::Player { .. }));
        match actions
            .iter_mut()
            .find(|a| (a.weapon, a.action, a.mode) == (o.weapon, o.action, o.mode))
        {
            Some(a) => {
                a.count += 1;
                a.attributed += attributed;
            }
            None => actions.push(ActionCount {
                weapon: o.weapon,
                action: o.action,
                mode: o.mode,
                count: 1,
                attributed,
            }),
        }
    }
    actions.sort_by_key(|a| (a.weapon, a.action, a.mode));

    let mut weapons: Vec<WeaponSummary> = Vec::new();
    for e in &events {
        if let Some((weapon, _, _)) = weapons::classify(e) {
            summary_of(&mut weapons, weapon).records += 1;
        }
    }
    for o in derived.occurrences.iter().filter(|o| o.shot) {
        summary_of(&mut weapons, o.weapon).shots += 1;
    }
    for iv in &derived.intervals {
        summary_of(&mut weapons, iv.weapon).intervals += 1;
    }
    for k in &context.kills {
        if let (context::KillCause::PlayerWeapon, Some(w)) = (k.cause, k.weapon) {
            summary_of(&mut weapons, w).kills += 1;
        }
    }
    weapons.sort_by_key(|w| w.weapon);

    let recorder = d.player_for_entity(i64::from(report.player_num) + 1);
    let recorder_entity = (!report.hltv).then(|| u16::from(report.player_num) + 1);
    let (shot_outcomes, accuracy) = accuracy::assess(
        &derived.occurrences,
        &context.blood,
        &context.poses,
        recorder_entity,
    );
    let recorder_accuracy = accuracy
        .iter()
        .filter(|a| a.exact)
        .map(|a| (a.shots, a.hits, a.chance))
        .reduce(|x, y| (x.0 + y.0, x.1 + y.1, x.2 + y.2))
        .and_then(|(s, h, c)| accuracy::accuracy(s, h, c));
    let mut players: Vec<PlayerIdentity> = Vec::new();
    for change in &report.name_history {
        if !players.contains(&change.player) {
            players.push(change.player.clone());
        }
    }
    let local = events
        .iter()
        .filter(|e| e.source == crate::state::EventSource::Demo)
        .count();
    let coverage = context.coverage(
        &d,
        local,
        events.len() - local,
        report.playback_seconds,
        report.spectator_seconds,
    );

    // Departures from the name history, so scores at a moment leave out
    // players who had gone
    let mut score_updates = std::mem::take(&mut context.score_updates);
    for change in &report.name_history {
        if change.kind == crate::state::NameChangeKind::Left {
            score_updates.push(ScoreUpdate {
                segment: change.player.segment,
                time: change.location.playback_time(),
                player: change.player.clone(),
                team: String::new(),
                frags: 0,
                deaths: 0,
                left: true,
            });
        }
    }
    // In decode order, each session's changes in time order
    score_updates.sort_by(|a, b| a.segment.cmp(&b.segment).then(a.time.total_cmp(&b.time)));
    let mut kills = std::mem::take(&mut context.kills);
    let recorder_headshots = recorder.as_ref().map(|rec| {
        kills
            .iter()
            .filter(|k| k.headshot == Some(true))
            .filter(|k| {
                k.killer.as_ref().is_some_and(|p| {
                    (p.segment, p.slot, p.incarnation) == (rec.segment, rec.slot, rec.incarnation)
                })
            })
            .count() as u32
    });
    let team_score_updates = std::mem::take(&mut context.team_score_updates);
    let (segment, time) = final_moment(&kills, &score_updates, &report.groups);
    let scoreboard = Scoreboard::from(&standing_at(
        &score_updates,
        &team_score_updates,
        segment,
        time,
    ));
    link_kills(
        &mut kills,
        &derived.occurrences,
        motion::gravity(&d),
        floors,
    );
    // A grenade kill's weapon, once kills are linked to the mp5's launches,
    // counted with that weapon's kills of other players
    grenades::tell_grenades(&mut kills, &context.held);
    for k in &kills {
        let other = k.cause == context::KillCause::PlayerOther;
        if let (true, Some(w)) = (other && k.weapon_name == "grenade", k.weapon) {
            summary_of(&mut weapons, w).kills += 1;
        }
    }
    weapons.sort_by_key(|w| w.weapon);
    let action_windows = action_windows(&kills, ACTION_WINDOW, ACTION_WINDOWS);

    context.roster.finish();
    let parts = played_parts(&Played {
        matches: &context.roster.matches,
        history: &context.roster.history,
        maps: &context.maps,
        kills: &kills,
        updates: &score_updates,
        team_updates: &team_score_updates,
        spans: &report.groups.segment_spans(),
    });

    AnalysisReport {
        schema: ANALYSIS_SCHEMA,
        profile: PROFILE,
        path: path.display().to_string(),
        decode: report.status,
        decode_error: report.error,
        recorder,
        players,
        max_clients: report.max_clients,
        events: events.len(),
        weapons,
        actions,
        occurrences: derived.occurrences,
        intervals: derived.intervals,
        unclassified: derived.unclassified,
        kills,
        action_windows,
        weapon_switches: context.switches,
        coverage,
        maps: std::mem::take(&mut context.maps),
        chat: std::mem::take(&mut context.chat),
        match_events: {
            context.match_events.finish();
            std::mem::take(&mut context.match_events.events)
        },
        counted: demo_counted(&parts, context.roster.any_moment),
        parts,
        scoreboard,
        score_updates,
        team_score_updates,
        shot_outcomes,
        accuracy,
        recorder_accuracy,
        recorder_headshots,
        groups: report.groups,
    }
}
