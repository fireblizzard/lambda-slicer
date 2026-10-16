//! Weapon activity and actors (implementation plan §5)
//!
//! [`events`] keeps the recording's events as evidence, and [`weapons`]
//! derives weapon actions and activity intervals from them with a versioned
//! profile. Counts say whether they are raw records or actions

pub mod context;
pub mod events;
pub mod weapons;

use std::path::Path;

use serde::Serialize;

use crate::container::Limits;
use crate::state::{DecodeOptions, DecodeStatus, PlayerIdentity, decode_demo};
use context::{Context, Coverage, Kill, WeaponSwitch};
use events::recorded_events;
use weapons::{Action, ActivityInterval, PROFILE, Weapon, WeaponOccurrence, derive};

pub const ANALYSIS_SCHEMA: u32 = 1;

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
    pub decode_error: Option<String>,
    /// The recording client's player, at the end of the recording
    pub recorder: Option<PlayerIdentity>,
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
    /// The recorder's active weapon over time
    pub weapon_switches: Vec<WeaponSwitch>,
    pub coverage: Coverage,
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
    let options = DecodeOptions {
        limits,
        collect_events: true,
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
    let local = events
        .iter()
        .filter(|e| e.source == crate::state::EventSource::Demo)
        .count();
    let coverage = context.coverage(&d, local, events.len() - local);

    AnalysisReport {
        schema: ANALYSIS_SCHEMA,
        profile: PROFILE,
        path: path.display().to_string(),
        decode: report.status,
        decode_error: report.error,
        recorder,
        max_clients: report.max_clients,
        events: events.len(),
        weapons,
        actions,
        occurrences: derived.occurrences,
        intervals: derived.intervals,
        unclassified: derived.unclassified,
        kills: context.kills,
        weapon_switches: context.switches,
        coverage,
    }
}
