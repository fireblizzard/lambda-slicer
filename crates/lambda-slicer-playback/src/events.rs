//! What the game's client does itself for an event (implementation plan
//! §13.6)
//!
//! The server and the recording client send weapon events, and the game's
//! client turns each into sounds and effects in its event handlers. The
//! handlers here make the same sounds, with the same samples, channels,
//! volumes and pitches as the SDK's (`ev_hldm.cpp`), and start the same
//! weapon animation when the event is the local player's, not yet compared
//! with the game (docs/viewer-profile.md, "Client events"). The impacts of
//! bullets and bolts, beams and the shells weapons throw out come later.
//!
//! An event with a delay waits in a queue until its time, and a handler can
//! take queued events back, as the gauss's stop takes back its spin. The
//! game draws its random choices from the client's own generator, which no
//! recording keeps, so each event seeds its own from where it was recorded.
//! A seek then hears the same choices every time

use std::sync::Arc;

use lambda_slicer_core::analysis::events::{EventArgs, RecordedEvent};
use lambda_slicer_core::state::{ActorResolution, EventSource};

use crate::time::SourceTime;

/// `ATTN_NORM`
pub const ATTN_NORM: f32 = 0.8;
/// `PITCH_NORM`
pub const PITCH_NORM: u8 = 100;

pub const CHAN_WEAPON: u8 = 1;
pub const CHAN_ITEM: u8 = 3;
pub const CHAN_STATIC: u8 = 6;

/// One event as the client's handler receives it
#[derive(Clone, Debug, PartialEq)]
pub struct ClientEvent {
    /// When it fires, its delay included
    pub time: SourceTime,
    /// The event script, `events/glock1.sc` for example
    pub script: Arc<str>,
    pub entity: u16,
    pub args: EventArgs,
    pub seed: u64,
    /// The game has more than one player slot, which the python's body
    /// depends on
    pub multiplayer: bool,
}

impl ClientEvent {
    /// `event` as it arrived at `time`, with its actor resolved as the
    /// client resolves it
    pub fn new(event: &RecordedEvent, time: SourceTime, multiplayer: bool) -> Option<Self> {
        let script = event.resource.as_deref()?;
        let entity = match event.actor {
            ActorResolution::PacketEntity { entity, .. } => i32::from(entity),
            ActorResolution::Explicit { entity } => entity,
            _ => event.args.entindex,
        };
        let source = match event.source {
            EventSource::Demo => 0,
            EventSource::Svc => 1,
            EventSource::SvcReliable => 2,
        };
        let seed = mix(
            mix(mix(event.ordinal, u64::from(event.index)), source),
            u64::from(event.message_ordinal.unwrap_or(u32::MAX)),
        );
        Some(Self {
            time: SourceTime(time.0 + f64::from(event.delay.unwrap_or(0.).max(0.))),
            script: Arc::from(script),
            entity: u16::try_from(entity).unwrap_or(0),
            args: event.args.clone(),
            seed,
            multiplayer,
        })
    }
}

/// A sound a handler starts, changes or stops (`EV_PlaySound`,
/// `EV_StopSound`)
#[derive(Clone, Debug, PartialEq)]
pub struct SoundCall {
    pub entity: u16,
    pub channel: u8,
    pub sample: Arc<str>,
    pub volume: f32,
    pub attenuation: f32,
    /// `SND_` flags
    pub flags: u16,
    pub pitch: u8,
    pub origin: [f32; 3],
}

/// What a handler does
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Sound(SoundCall),
    /// Take back the queued events of this script for the entity
    /// (`EV_KillEvents`)
    Kill {
        entity: u16,
        script: &'static str,
    },
    /// Play this animation on the viewmodel (`EV_WeaponAnimation`)
    WeaponAnimation {
        sequence: i32,
        body: i32,
    },
}

/// Events waiting for their time
#[derive(Clone, Debug, Default)]
pub struct Events {
    pending: Vec<ClientEvent>,
    /// The crowbar's swings so far, which choose its next animation
    /// (`g_iSwing`)
    swings: u32,
}

impl Events {
    /// Queue `event` when it fires later than `now`, and give it back when
    /// it fires now
    pub fn arrive(&mut self, event: ClientEvent, now: SourceTime) -> Option<ClientEvent> {
        if event.time > now {
            self.pending.push(event);
            None
        } else {
            Some(event)
        }
    }

    /// The queued events due by `now`, in the order they fire
    pub fn due(&mut self, now: SourceTime) -> Vec<ClientEvent> {
        let mut due = Vec::new();
        self.pending.retain(|e| {
            let fire = e.time <= now;
            if fire {
                due.push(e.clone());
            }
            !fire
        });
        due.sort_by(|a, b| a.time.0.total_cmp(&b.time.0));
        due
    }

    pub fn kill(&mut self, entity: u16, script: &str) {
        self.pending
            .retain(|e| !(e.entity == entity && script_is(&e.script, script)));
    }

    pub fn pending(&self) -> &[ClientEvent] {
        &self.pending
    }

    /// The crowbar's count of swings, for its handler
    pub fn swings(&mut self) -> &mut u32 {
        &mut self.swings
    }
}

/// Whether a recorded script path names `script`, compared as the client
/// compares them, without regard to case or the slash's direction
fn script_is(recorded: &str, script: &str) -> bool {
    recorded.len() == script.len()
        && recorded
            .bytes()
            .zip(script.bytes())
            .all(|(a, b)| a.eq_ignore_ascii_case(&b) || (a == b'\\' && b == b'/'))
}

pub(crate) fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_add(0x9e37_79b9_7f4a_7c15).wrapping_add(a << 6);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The random choices of one event, in the order its handler makes them
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.0, 0)
    }

    /// `gEngfuncs.pfnRandomLong`, both ends included
    fn long(&mut self, low: i32, high: i32) -> i32 {
        let span = (i64::from(high) - i64::from(low) + 1).max(1) as u64;
        (i64::from(low) + (self.next() % span) as i64) as i32
    }

    /// `gEngfuncs.pfnRandomFloat`
    fn float(&mut self, low: f32, high: f32) -> f32 {
        let unit = (self.next() >> 40) as f32 / (1u64 << 24) as f32;
        low + (high - low) * unit
    }
}

/// What the client's handler for `event` does. `local` when its entity is
/// the player the recorder plays or watches through the eyes of
/// (`EV_IsLocal`), and `swings` the crowbar's count
pub fn handle(event: &ClientEvent, local: bool, swings: &mut u32) -> Vec<Action> {
    let mut animations = Vec::new();
    let mut animate = |sequence: i32, body: i32| {
        if local {
            animations.push(Action::WeaponAnimation { sequence, body });
        }
    };
    let sounds = handle_sounds(event, &mut animate, swings);
    animations.extend(sounds);
    animations
}

fn handle_sounds(
    event: &ClientEvent,
    animate: &mut dyn FnMut(i32, i32),
    swings: &mut u32,
) -> Vec<Action> {
    let mut random = Random(event.seed);
    let args = &event.args;
    let idx = event.entity;
    let origin = args.origin;
    let sound = |channel, sample, volume, pitch: i32| {
        Action::Sound(SoundCall {
            entity: idx,
            channel,
            sample: Arc::from(sample),
            volume,
            attenuation: ATTN_NORM,
            flags: 0,
            pitch: pitch.clamp(0, 255) as u8,
            origin,
        })
    };
    let script = event.script.as_ref();
    let is = |name: &str| script_is(script, name);
    let norm = i32::from(PITCH_NORM);

    if is("events/glock1.sc") || is("events/glock2.sc") {
        // GLOCK_SHOOT_EMPTY or GLOCK_SHOOT
        animate(if args.bparam1 != 0 { 4 } else { 3 }, 2);
        let volume = random.float(0.92, 1.);
        let pitch = 98 + random.long(0, 3);
        vec![sound(CHAN_WEAPON, "weapons/pl_gun3.wav", volume, pitch)]
    } else if is("events/shotgun1.sc") {
        // SHOTGUN_FIRE
        animate(1, 2);
        let volume = random.float(0.95, 1.);
        let pitch = 93 + random.long(0, 0x1f);
        vec![sound(CHAN_WEAPON, "weapons/sbarrel1.wav", volume, pitch)]
    } else if is("events/shotgun2.sc") {
        // SHOTGUN_FIRE2
        animate(2, 2);
        let volume = random.float(0.98, 1.);
        let pitch = 85 + random.long(0, 0x1f);
        vec![sound(CHAN_WEAPON, "weapons/dbarrel1.wav", volume, pitch)]
    } else if is("events/mp5.sc") {
        // MP5_FIRE1 to MP5_FIRE3
        animate(5 + random.long(0, 2), 2);
        let sample = match random.long(0, 1) {
            0 => "weapons/hks1.wav",
            _ => "weapons/hks2.wav",
        };
        vec![sound(CHAN_WEAPON, sample, 1., 94 + random.long(0, 0xf))]
    } else if is("events/mp52.sc") {
        // MP5_LAUNCH
        animate(2, 2);
        let sample = match random.long(0, 1) {
            0 => "weapons/glauncher.wav",
            _ => "weapons/glauncher2.wav",
        };
        vec![sound(CHAN_WEAPON, sample, 1., 94 + random.long(0, 0xf))]
    } else if is("events/python.sc") {
        // PYTHON_FIRE1, with the multiplayer body when the game has more
        // than one player slot
        animate(2, i32::from(event.multiplayer));
        let sample = match random.long(0, 1) {
            0 => "weapons/357_shot1.wav",
            _ => "weapons/357_shot2.wav",
        };
        vec![sound(CHAN_WEAPON, sample, random.float(0.8, 0.9), norm)]
    } else if is("events/crowbar.sc") {
        // CROWBAR_ATTACK1MISS, then the swing's own miss
        animate(4, 1);
        animate([4, 5, 7][(*swings % 3) as usize], 1);
        *swings = swings.wrapping_add(1);
        vec![sound(CHAN_WEAPON, "weapons/cbar_miss1.wav", 1., norm)]
    } else if is("events/crossbow1.sc") || is("events/crossbow2.sc") {
        // CROSSBOW_FIRE1 with bolts left, CROSSBOW_FIRE3 with the last
        if args.iparam1 != 0 {
            animate(4, 1);
        } else if args.iparam2 != 0 {
            animate(6, 1);
        }
        let fire = sound(
            CHAN_WEAPON,
            "weapons/xbow_fire1.wav",
            1.,
            93 + random.long(0, 0xf),
        );
        let volume = random.float(0.95, 1.);
        let reload = sound(
            CHAN_ITEM,
            "weapons/xbow_reload1.wav",
            volume,
            93 + random.long(0, 0xf),
        );
        vec![fire, reload]
    } else if is("events/rpg.sc") {
        // RPG_FIRE2
        animate(3, 1);
        vec![
            sound(CHAN_WEAPON, "weapons/rocketfire1.wav", 0.9, norm),
            sound(CHAN_ITEM, "weapons/glauncher.wav", 0.7, norm),
        ]
    } else if is("events/firehornet.sc") {
        // HGUN_SHOOT
        animate(6, 1);
        let sample = match random.long(0, 2) {
            0 => "agrunt/ag_fire1.wav",
            1 => "agrunt/ag_fire2.wav",
            _ => "agrunt/ag_fire3.wav",
        };
        vec![sound(CHAN_WEAPON, sample, 1., 100)]
    } else if is("events/gauss.sc") {
        // A stop takes back the spin still to come and silences the charge
        if args.bparam2 != 0 {
            return vec![
                Action::Kill {
                    entity: idx,
                    script: "events/gaussspin.sc",
                },
                Action::Sound(SoundCall {
                    flags: crate::audio::flags::STOP,
                    ..stop_call(idx, CHAN_WEAPON, "ambience/pulsemachine.wav")
                }),
            ];
        }
        // GAUSS_FIRE2
        animate(6, 2);
        let pitch = 85 + random.long(0, 0x1f);
        let volume = 0.5 + args.fparam1 / 400.;
        vec![sound(CHAN_WEAPON, "weapons/gauss2.wav", volume, pitch)]
    } else if is("events/gaussspin.sc") {
        let flags = if args.bparam1 != 0 {
            crate::audio::flags::CHANGE_PITCH
        } else {
            0
        };
        vec![Action::Sound(SoundCall {
            flags,
            ..sound_call(
                idx,
                CHAN_WEAPON,
                "ambience/pulsemachine.wav",
                1.,
                args.iparam1,
                origin,
            )
        })]
    } else if is("events/egon_fire.sc") {
        let wide = args.iparam2 == 1;
        let (volume, pitch) = if wide { (0.98, 125) } else { (0.9, 100) };
        let started = if args.bparam1 != 0 {
            sound(CHAN_WEAPON, "weapons/egon_windup2.wav", volume, pitch)
        } else {
            sound(CHAN_STATIC, "weapons/egon_run3.wav", volume, pitch)
        };
        // EGON_FIRE1 to EGON_FIRE4
        animate(5 + random.long(0, 3), 1);
        vec![started]
    } else if is("events/egon_stop.sc") {
        let mut actions = vec![Action::Sound(SoundCall {
            flags: crate::audio::flags::STOP,
            ..stop_call(idx, CHAN_STATIC, "weapons/egon_run3.wav")
        })];
        if args.iparam1 != 0 {
            actions.push(sound(CHAN_WEAPON, "weapons/egon_off1.wav", 0.98, 100));
        }
        actions
    } else if is("events/train.sc") {
        train(idx, args)
    } else {
        Vec::new()
    }
}

fn sound_call(
    entity: u16,
    channel: u8,
    sample: &'static str,
    volume: f32,
    pitch: i32,
    origin: [f32; 3],
) -> SoundCall {
    SoundCall {
        entity,
        channel,
        sample: Arc::from(sample),
        volume,
        attenuation: ATTN_NORM,
        flags: 0,
        pitch: pitch.clamp(0, 255) as u8,
        origin,
    }
}

fn stop_call(entity: u16, channel: u8, sample: &'static str) -> SoundCall {
    sound_call(entity, channel, sample, 0., i32::from(PITCH_NORM), [0.; 3])
}

/// `EV_TrainPitchAdjust`, the moving train's sound with its volume, sample
/// and pitch packed into one parameter
fn train(idx: u16, args: &EventArgs) -> Vec<Action> {
    let packed = args.iparam1 as u16;
    let volume = f32::from(packed & 0x3f) / 40.;
    let pitch = (10. * f32::from((packed >> 6) & 0x3f)) as i32;
    let sample = match (packed >> 12) & 7 {
        1 => "plats/ttrain1.wav",
        2 => "plats/ttrain2.wav",
        3 => "plats/ttrain3.wav",
        4 => "plats/ttrain4.wav",
        5 => "plats/ttrain6.wav",
        6 => "plats/ttrain7.wav",
        _ => return Vec::new(),
    };
    let call = if args.bparam1 != 0 {
        SoundCall {
            flags: crate::audio::flags::STOP,
            ..stop_call(idx, CHAN_STATIC, sample)
        }
    } else {
        SoundCall {
            flags: crate::audio::flags::CHANGE_PITCH,
            ..sound_call(idx, CHAN_STATIC, sample, volume, pitch, args.origin)
        }
    };
    vec![Action::Sound(call)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(script: &str, seed: u64) -> ClientEvent {
        ClientEvent {
            time: SourceTime(1.),
            script: Arc::from(script),
            entity: 3,
            args: EventArgs {
                origin: [1., 2., 3.],
                ..EventArgs::default()
            },
            seed,
            multiplayer: true,
        }
    }

    fn sounds(event: &ClientEvent) -> Vec<Action> {
        handle(event, false, &mut 0)
    }

    #[test]
    fn handlers_choose_the_same_way_for_the_same_event() {
        let first = sounds(&event("events/mp5.sc", 7));
        assert_eq!(first, sounds(&event("events/mp5.sc", 7)));
        let Action::Sound(call) = &first[0] else {
            panic!("{first:?}");
        };
        assert!(call.sample.starts_with("weapons/hks"));
        assert!((94..=109).contains(&call.pitch));
        assert_eq!(
            (call.entity, call.channel, call.origin),
            (3, CHAN_WEAPON, [1., 2., 3.])
        );

        // Over many events every choice comes up
        let samples: std::collections::BTreeSet<_> = (0..64)
            .map(|seed| match &sounds(&event("events/mp5.sc", seed))[0] {
                Action::Sound(c) => c.sample.clone(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(samples.len(), 2);
    }

    #[test]
    fn a_gauss_stop_takes_back_the_spin_and_stops_its_sound() {
        let mut stop = event("events\\GAUSS.sc", 1);
        stop.args.bparam2 = 1;
        let actions = sounds(&stop);
        assert_eq!(
            actions[0],
            Action::Kill {
                entity: 3,
                script: "events/gaussspin.sc"
            }
        );
        let Action::Sound(call) = &actions[1] else {
            panic!("{actions:?}");
        };
        assert_eq!(call.flags, crate::audio::flags::STOP);

        let mut events = Events::default();
        let mut spin = event("events/gaussspin.sc", 2);
        spin.time = SourceTime(2.);
        assert_eq!(events.arrive(spin, SourceTime(1.)), None);
        events.kill(3, "events/gaussspin.sc");
        assert!(events.due(SourceTime(3.)).is_empty());
    }

    #[test]
    fn the_local_players_shots_animate_the_viewmodel() {
        let shot = event("events/gauss.sc", 1);
        assert!(
            !sounds(&shot)
                .iter()
                .any(|a| matches!(a, Action::WeaponAnimation { .. }))
        );
        assert_eq!(
            handle(&shot, true, &mut 0)[0],
            Action::WeaponAnimation {
                sequence: 6,
                body: 2
            }
        );

        // The crowbar's swings go through its three misses in turn
        let mut swings = 0;
        let swing = event("events/crowbar.sc", 1);
        let misses: Vec<i32> = (0..4)
            .map(|_| match handle(&swing, true, &mut swings)[1] {
                Action::WeaponAnimation { sequence, .. } => sequence,
                ref other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(misses, [4, 5, 7, 4]);
    }

    #[test]
    fn delayed_events_wait_for_their_time() {
        let mut events = Events::default();
        let mut later = event("events/glock1.sc", 1);
        later.time = SourceTime(1.5);
        let now = event("events/glock1.sc", 2);
        assert_eq!(events.arrive(later.clone(), SourceTime(1.)), None);
        assert_eq!(events.arrive(now.clone(), SourceTime(1.)), Some(now));
        assert!(events.due(SourceTime(1.4)).is_empty());
        assert_eq!(events.due(SourceTime(1.5)), vec![later]);
    }
}
