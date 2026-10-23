//! What the game's client does itself for an event
//!
//! The server and the recording client send weapon events, and the game's
//! client turns each into sounds and effects in its event handlers. The
//! handlers here make the same sounds, with the same samples, channels,
//! volumes and pitches as the SDK's (`ev_hldm.cpp`), and start the same
//! weapon animation when the event is the local player's, throw out the
//! same shells and fire the same bullets (`crate::bullets`), not yet
//! compared with the game (docs/viewer-profile.md, "Client events"). The
//! crossbow's bolts and the beams come later.
//!
//! An event with a delay waits in a queue until its time, and a handler can
//! take queued events back, as the gauss's stop takes back its spin. The
//! game draws its random choices from the client's own generator, which no
//! recording keeps, so each event seeds its own from where it was recorded.
//! A seek then hears the same choices every time

use std::collections::BTreeMap;
use std::sync::Arc;

use lambda_slicer_core::analysis::events::{EventArgs, RecordedEvent};
use lambda_slicer_core::state::{ActorResolution, EventSource};

use crate::bullets::{Bullet, Volley};
use crate::gauss::GaussShot;
use crate::tempents::{Bounce, Throw};
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
    /// The game's player slots
    pub max_clients: u8,
}

/// The server fills a player's event with the angles of the player's model,
/// whose pitch is the view's times this. Only the recording player's own
/// events, which its game wrote into the demo, carry the view's
const MODEL_PITCH: f32 = -1. / 3.;

impl ClientEvent {
    /// `event` as it arrived at `time`, with its actor resolved as the
    /// client resolves it, and a player's aim as the player looked
    pub fn new(event: &RecordedEvent, time: SourceTime, max_clients: u8) -> Option<Self> {
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

        // The view's pitch back from the model's, for a player's event the
        // server sent
        let mut args = event.args.clone();
        let player = (1..=i32::from(max_clients)).contains(&entity);
        if player && event.source != EventSource::Demo {
            args.angles[0] /= MODEL_PITCH;
        }

        Some(Self {
            time: SourceTime(time.0 + f64::from(event.delay.unwrap_or(0.).max(0.))),
            script: Arc::from(script),
            entity: u16::try_from(entity).unwrap_or(0),
            args,
            seed,
            max_clients,
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
    /// Throw out a shell (`EV_EjectBrass`)
    Shell(Throw),
    /// Fire bullets (`EV_HLDM_FireBullets`)
    Bullets(Volley),
    /// Fire the gauss (`EV_FireGauss`)
    Gauss(GaussShot),
    /// Draw the Egon's beams for the local player, from the eye along its
    /// view
    EgonStart,
    /// End them
    EgonStop,
}

/// Events waiting for their time
#[derive(Clone, Debug, Default)]
pub struct Events {
    pending: Vec<ClientEvent>,
    /// The crowbar's swings so far, which choose its next animation
    /// (`g_iSwing`)
    swings: u32,
    /// Each shooter's bullets so far, which say which are tracers
    /// (`tracerCount`)
    tracers: BTreeMap<u16, u32>,
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

    /// Drop the queued events, as a new session does (`CL_ResetEvents`).
    /// The counts the client's handlers keep live on, as the game's do
    pub(crate) fn clear(&mut self) {
        self.pending.clear();
    }

    /// The crowbar's count of swings, for its handler
    pub fn swings(&mut self) -> &mut u32 {
        &mut self.swings
    }

    /// `shooter`'s count of bullets
    pub fn tracers(&mut self, shooter: u16) -> &mut u32 {
        self.tracers.entry(shooter).or_insert(0)
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
pub(crate) struct Random(pub(crate) u64);

impl Random {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.0, 0)
    }

    /// `gEngfuncs.pfnRandomLong`, both ends included
    pub(crate) fn long(&mut self, low: i32, high: i32) -> i32 {
        let span = (i64::from(high) - i64::from(low) + 1).max(1) as u64;
        (i64::from(low) + (self.next() % span) as i64) as i32
    }

    /// `gEngfuncs.pfnRandomFloat`
    pub(crate) fn float(&mut self, low: f32, high: f32) -> f32 {
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
    let sounds = handle_sounds(event, local, &mut animate, swings);
    animations.extend(sounds);
    animations
}

fn handle_sounds(
    event: &ClientEvent,
    local: bool,
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
        let brass = shell(event, &mut random, SHELL, Bounce::Shell, [20., -12., 4.]);
        let volume = random.float(0.92, 1.);
        let pitch = 98 + random.long(0, 3);
        let spread = [args.fparam1, args.fparam2];
        vec![
            brass,
            sound(CHAN_WEAPON, "weapons/pl_gun3.wav", volume, pitch),
            volley(event, &mut random, Bullet::Nine, 1, 8192., spread, 0),
        ]
    } else if is("events/shotgun1.sc") {
        // SHOTGUN_FIRE
        animate(1, 2);
        let brass = shell(
            event,
            &mut random,
            SHOTSHELL,
            Bounce::ShotShell,
            [32., -12., 6.],
        );
        let volume = random.float(0.95, 1.);
        let pitch = 93 + random.long(0, 0x1f);
        // Four pellets in a wide cone in multiplayer, six in a round one
        let (shots, spread) = match event.max_clients > 1 {
            true => (4, [0.08716, 0.04362]),
            false => (6, [0.08716, 0.08716]),
        };
        vec![
            brass,
            sound(CHAN_WEAPON, "weapons/sbarrel1.wav", volume, pitch),
            volley(
                event,
                &mut random,
                Bullet::Buckshot,
                shots,
                2048.,
                spread,
                0,
            ),
        ]
    } else if is("events/shotgun2.sc") {
        // SHOTGUN_FIRE2, and a shell from each barrel
        animate(2, 2);
        let mut actions: Vec<Action> = (0..2)
            .map(|_| {
                shell(
                    event,
                    &mut random,
                    SHOTSHELL,
                    Bounce::ShotShell,
                    [32., -12., 6.],
                )
            })
            .collect();
        let volume = random.float(0.98, 1.);
        let pitch = 85 + random.long(0, 0x1f);
        actions.push(sound(CHAN_WEAPON, "weapons/dbarrel1.wav", volume, pitch));
        // Eight pellets in a wider cone in multiplayer, twelve in a round one
        let (shots, spread) = match event.max_clients > 1 {
            true => (8, [0.17365, 0.04362]),
            false => (12, [0.08716, 0.08716]),
        };
        actions.push(volley(
            event,
            &mut random,
            Bullet::Buckshot,
            shots,
            2048.,
            spread,
            0,
        ));
        actions
    } else if is("events/mp5.sc") {
        // MP5_FIRE1 to MP5_FIRE3
        animate(5 + random.long(0, 2), 2);
        let brass = shell(event, &mut random, SHELL, Bounce::Shell, [20., -12., 4.]);
        let sample = match random.long(0, 1) {
            0 => "weapons/hks1.wav",
            _ => "weapons/hks2.wav",
        };
        let spread = [args.fparam1, args.fparam2];
        vec![
            brass,
            sound(CHAN_WEAPON, sample, 1., 94 + random.long(0, 0xf)),
            volley(event, &mut random, Bullet::Mp5, 1, 8192., spread, 2),
        ]
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
        animate(2, i32::from(event.max_clients > 1));
        let sample = match random.long(0, 1) {
            0 => "weapons/357_shot1.wav",
            _ => "weapons/357_shot2.wav",
        };
        let volume = random.float(0.8, 0.9);
        let spread = [args.fparam1, args.fparam2];
        vec![
            sound(CHAN_WEAPON, sample, volume, norm),
            volley(event, &mut random, Bullet::Magnum, 1, 8192., spread, 0),
        ]
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
        let shot = GaussShot {
            source: eye(event),
            axes: angle_vectors(args.angles),
            damage: args.fparam1,
            primary: args.bparam1 != 0,
            seed: random.next(),
        };
        vec![
            sound(CHAN_WEAPON, "weapons/gauss2.wav", volume, pitch),
            Action::Gauss(shot),
        ]
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
        let mut actions = vec![started];
        // Only the local player's beams are the client's own
        if args.bparam1 != 0 && local {
            actions.push(Action::EgonStart);
        }
        actions
    } else if is("events/egon_stop.sc") {
        let mut actions = vec![Action::Sound(SoundCall {
            flags: crate::audio::flags::STOP,
            ..stop_call(idx, CHAN_STATIC, "weapons/egon_run3.wav")
        })];
        if args.iparam1 != 0 {
            actions.push(sound(CHAN_WEAPON, "weapons/egon_off1.wav", 0.98, 100));
        }
        if local {
            actions.push(Action::EgonStop);
        }
        actions
    } else if is("events/train.sc") {
        train(idx, args)
    } else {
        Vec::new()
    }
}

const SHELL: &str = "models/shell.mdl";
const SHOTSHELL: &str = "models/shotgunshell.mdl";

/// The game's view vectors for angles in degrees (`AngleVectors`)
pub(crate) fn angle_vectors(angles: [f32; 3]) -> [[f32; 3]; 3] {
    let [p, y, r] = angles.map(f32::to_radians);
    let (sp, cp) = p.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sr, cr) = r.sin_cos();
    [
        [cp * cy, cp * sy, -sp],
        [-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp],
        [cr * sp * cy + sr * sy, cr * sp * sy - sr * cy, cr * cp],
    ]
}

/// Where the shooter's eye is, over the event's origin by the standing or
/// ducking view height (`EV_GetGunPosition`)
pub(crate) fn eye(event: &ClientEvent) -> [f32; 3] {
    let args = &event.args;
    let player = (1..=u16::from(event.max_clients)).contains(&event.entity);
    // VEC_DUCK_VIEW or DEFAULT_VIEWHEIGHT
    let height = if player && args.ducking == 1 {
        12.
    } else {
        28.
    };
    [args.origin[0], args.origin[1], args.origin[2] + height]
}

/// Bullets fired from the eye along the event's angles
fn volley(
    event: &ClientEvent,
    random: &mut Random,
    kind: Bullet,
    shots: u32,
    distance: f32,
    spread: [f32; 2],
    tracer_every: u32,
) -> Action {
    Action::Bullets(Volley {
        kind,
        shots,
        distance,
        spread,
        tracer_every,
        source: eye(event),
        axes: angle_vectors(event.args.angles),
        seed: random.next(),
    })
}

/// A shell thrown out of the gun, forward, up and right of the eye by
/// `scales`, flying right and up from the player's own velocity
/// (`EV_GetDefaultShellInfo` and `EV_EjectBrass`)
fn shell(
    event: &ClientEvent,
    random: &mut Random,
    model: &'static str,
    bounce: Bounce,
    [forward_scale, up_scale, right_scale]: [f32; 3],
) -> Action {
    let args = &event.args;
    let [forward, right, up] = angle_vectors(args.angles);
    let eye = eye(event);
    let across = random.float(50., 70.);
    let lift = random.float(100., 150.);
    let velocity = args.velocity.unwrap_or([0.; 3]);
    Action::Shell(Throw {
        model,
        origin: [0, 1, 2].map(|k| {
            eye[k] + up[k] * up_scale + forward[k] * forward_scale + right[k] * right_scale
        }),
        velocity: [0, 1, 2]
            .map(|k| velocity[k] + right[k] * across + up[k] * lift + forward[k] * 25.),
        yaw: args.angles[1],
        bounce,
        seed: random.next(),
    })
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

    /// A gauss shot by player 2, written by `source`, aiming at `angles`
    fn recorded(source: EventSource, angles: [f32; 3]) -> RecordedEvent {
        RecordedEvent {
            id: 0,
            segment: 1,
            entry: 1,
            ordinal: 0,
            time: 1.,
            server_time: 1.,
            source,
            message_ordinal: None,
            index: 0,
            resource: Some("events/gauss.sc".into()),
            dispatch_flags: Some(0),
            args: EventArgs {
                entindex: 2,
                angles,
                ..Default::default()
            },
            delay: Some(0.),
            actor: ActorResolution::Explicit { entity: 2 },
            player: None,
        }
    }

    #[test]
    fn a_shot_the_server_sent_aims_as_its_player_looked() {
        // The server sends the model's pitch, a third of the view's turned
        // round, and the recorder's own game the view's itself
        let sent = recorded(EventSource::Svc, [7.75, 7., 0.]);
        let sent = ClientEvent::new(&sent, SourceTime(1.), 32).unwrap();
        assert_eq!(sent.args.angles, [-23.25, 7., 0.]);

        let own = recorded(EventSource::Demo, [7.75, 7., 0.]);
        let own = ClientEvent::new(&own, SourceTime(1.), 32).unwrap();
        assert_eq!(own.args.angles, [7.75, 7., 0.]);
    }

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
            max_clients: 32,
        }
    }

    fn sounds(event: &ClientEvent) -> Vec<Action> {
        handle(event, false, &mut 0)
    }

    fn sound_of(actions: &[Action]) -> Option<&SoundCall> {
        actions.iter().find_map(|a| match a {
            Action::Sound(c) => Some(c),
            _ => None,
        })
    }

    #[test]
    fn handlers_choose_the_same_way_for_the_same_event() {
        let first = sounds(&event("events/mp5.sc", 7));
        assert_eq!(first, sounds(&event("events/mp5.sc", 7)));
        let Some(call) = sound_of(&first) else {
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
            .map(|seed| {
                let actions = sounds(&event("events/mp5.sc", seed));
                sound_of(&actions).map(|c| c.sample.clone())
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
    fn a_double_shot_throws_out_two_shells_to_the_right() {
        let mut shot = event("events/shotgun2.sc", 5);
        shot.args.angles = [0., 90., 0.];
        let shells: Vec<_> = sounds(&shot)
            .into_iter()
            .filter_map(|a| match a {
                Action::Shell(Throw {
                    model,
                    origin,
                    velocity,
                    yaw,
                    ..
                }) => Some((model, origin, velocity, yaw)),
                _ => None,
            })
            .collect();
        assert_eq!(shells.len(), 2);
        let (model, origin, velocity, yaw) = shells[0];
        assert_eq!((model, yaw), ("models/shotgunshell.mdl", 90.));
        // Facing +y, right is +x, so 32 forward, 6 right and 12 under the
        // eye
        assert!((origin[0] - 7.).abs() < 1e-4 && (origin[1] - 34.).abs() < 1e-4);
        assert!((origin[2] - 19.).abs() < 1e-4);
        assert!((50. ..=70.).contains(&velocity[0]) && (100. ..=150.).contains(&velocity[2]));
        assert!((velocity[1] - 25.).abs() < 1e-4);
        assert_ne!(shells[0], shells[1]);
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
