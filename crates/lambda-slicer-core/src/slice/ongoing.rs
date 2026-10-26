//! Effects under way at a restart's start
//!
//! Events, sounds and temp entities before the start already happened, so
//! they are not replayed. Some of them start something that lasts past the
//! start, which the client would then lack. Those with a known lifecycle are
//! carried over, gibs and thrown models are left out, and the others refuse
//! the export while they are active.
//!
//! - The recorder's gauss charge and egon fire need their starting demo
//!   events, which create the spin sound and the local egon beam
//!   (`cl_dll/ev_hldm.cpp`, `EV_SpinGauss`, `EV_EgonFire`)
//! - Server sound loops (doors, platforms, chargers, a rocket's flight) play
//!   until stopped. A sound started before the start whose stop arrives
//!   after it was still playing, so it is started again. A map change or
//!   reconnect stops every sound, so one not stopped before it is not
//! - Beams, lights and HUD text last for a set time. Those still alive are
//!   sent again with their remaining life
//! - Gibs and thrown models move under physics and cannot be restarted
//!   where they are. They are only debris, so a clip leaves out those still
//!   in the air at its start and its report says so
//!
//! Event loops whose updates restart a stopped sound need nothing. The
//! engine starts a sound that a pitch or volume change finds not playing,
//! and trains send such an update every think. Decals are lasting world
//! state rather than an effect, and clips don't carry them yet
//! (`docs/limitations.md`)

use crate::container::{Frame, FrameBody};
use crate::protocol::delta::StructState;
use crate::protocol::messages::{Message, ParsedMessage, Resource};
use crate::state::Decoder;

/// Spin events come every frame while charging, so a charge whose latest
/// spin is older than this has ended some other way
const CHARGE_GAP: f32 = 0.1;

/// Event scripts whose effects end by themselves
const ONE_SHOT_EVENTS: &[&[u8]] = &[
    b"events/crossbow1.sc",
    b"events/crossbow2.sc",
    b"events/crowbar.sc",
    b"events/firehornet.sc",
    b"events/gauss.sc",
    b"events/glock1.sc",
    b"events/glock2.sc",
    b"events/mp5.sc",
    b"events/mp52.sc",
    b"events/python.sc",
    b"events/rpg.sc",
    b"events/shotgun1.sc",
    b"events/shotgun2.sc",
    b"events/snarkfire.sc",
    b"events/tripfire.sc",
];

/// Event scripts with a lifecycle followed here or restored by their own
/// updates
const LIFECYCLE_EVENTS: &[&[u8]] = &[
    b"events/egon_fire.sc",
    b"events/egon_stop.sc",
    b"events/gaussspin.sc",
    b"events/train.sc",
];

/// `svc_sound` flags
const SND_STOP: u16 = 1 << 5;
const SND_CHANGE: u16 = (1 << 6) | (1 << 7);
/// Several static sounds play at once on one entity, so they are told
/// apart by sample
const CHAN_STATIC: u8 = 6;

/// How a temp entity kind lasts
enum Lasting {
    /// Over within its own short animation
    Instant,
    /// Lasting world state, not carried yet (`docs/limitations.md`)
    Decal,
    /// A life byte at this body offset, in tenths of a second, so it can be
    /// sent again with the remaining life. With `entity`, the effect is
    /// attached to the entity in the short at that offset
    Byte { at: usize, entity: Option<usize> },
    /// A life short at this offset, in tenths of a second
    Short { at: usize },
    /// HUD text with fade-in, hold and fade-out times
    Text,
    /// Moves under physics, with a life byte at this offset
    Physics { at: usize },
    /// Removes the beams attached to an entity
    KillBeam,
}

/// Body layouts from `common/const.h`, with coords written as 2 bytes
fn lasting(kind: u8) -> Option<Lasting> {
    use Lasting::*;

    Some(match kind {
        2..=6
        | 9..=12
        | 17
        | 18
        | 23
        | 25
        | 100..=103
        | 105
        | 110
        | 111
        | 113..=115
        | 120
        | 121
        | 125
        | 126 => Instant,
        13 | 104 | 109 | 112 | 116..=118 => Decal,
        0 | 19..=21 => Byte {
            at: 16,
            entity: None,
        },
        1 => Byte {
            at: 12,
            entity: Some(0),
        },
        7 => Byte {
            at: 12,
            entity: None,
        },
        8 | 24 => Byte {
            at: 8,
            entity: Some(0),
        },
        22 => Byte {
            at: 4,
            entity: Some(0),
        },
        27 => Byte {
            at: 11,
            entity: None,
        },
        28 => Byte {
            at: 13,
            entity: Some(0),
        },
        30 | 31 => Short { at: 12 },
        29 => Text,
        14 => Physics { at: 8 },
        15 => Physics { at: 15 },
        106 => Physics { at: 16 },
        107 => Physics { at: 12 },
        108 => Physics { at: 22 },
        119 => Physics { at: 14 },
        124 => Physics { at: 5 },
        99 => KillBeam,
        _ => return None,
    })
}

/// A sound started before the start, keyed the way a later sound on the
/// same channel replaces it
struct Started {
    entity: u16,
    channel: u8,
    sample: u16,
    bytes: Vec<u8>,
}

impl Started {
    fn same_key(&self, entity: u16, channel: u8, sample: u16) -> bool {
        self.entity == entity
            && self.channel == channel
            && (channel != CHAN_STATIC || self.sample == sample)
    }
}

/// A temp entity whose effect may outlast the start
struct Temp {
    kind: u8,
    time: f32,
    bytes: Vec<u8>,
}

/// What a restart carries over the start
#[derive(Default)]
pub struct Carried {
    /// Demo records for the end of the bootstrap group
    pub records: Vec<Frame>,
    /// Network messages for the record after it
    pub messages: Vec<u8>,
    /// One line per carried effect, for the report
    pub notes: Vec<String>,
    /// One line per effect still under way that the clip leaves out, for
    /// the report
    pub left_out: Vec<String>,
}

/// What becomes of a temp entity under way at the start
enum Carry {
    /// Sent again as these bytes, with a line for the report
    Sent(Vec<u8>, String),
    /// Left out of the clip, with a line for the report
    LeftOut(String),
}

/// Sounds that may still be playing at the start, confirmed by a stop
/// after it or ruled out by a sound replacing them
pub struct PendingSounds {
    candidates: Vec<(Started, Option<bool>)>,
}

impl PendingSounds {
    pub fn unresolved(&self) -> bool {
        self.candidates.iter().any(|(_, state)| state.is_none())
    }

    /// Follow sounds after the start
    pub fn observe(&mut self, parsed: &ParsedMessage) {
        let Message::Sound(s) = &parsed.message else {
            return;
        };

        for (c, state) in &mut self.candidates {
            if state.is_some() || !c.same_key(s.entity, s.channel, s.sound_index) {
                continue;
            }

            // A stop names its sample, and one for another sample leaves
            // this one playing. A new sound on the channel replaces it
            if s.flags & SND_STOP != 0 {
                if c.sample == s.sound_index {
                    *state = Some(true);
                }
            } else if s.flags & SND_CHANGE == 0 {
                *state = Some(false);
            }
        }
    }

    /// A map change or reconnect stops every sound, so those not stopped
    /// before it are not confirmed, and the next session's are not followed
    pub fn end_session(&mut self) {
        for (_, state) in &mut self.candidates {
            if state.is_none() {
                *state = Some(false);
            }
        }
    }

    /// The confirmed loops, as messages and report lines, their sounds
    /// named from the start's `resources`
    pub fn confirmed(&self, resources: &[Resource]) -> (Vec<u8>, Vec<String>) {
        let mut bytes = Vec::new();
        let mut notes = Vec::new();
        for (c, state) in &self.candidates {
            if *state != Some(true) {
                continue;
            }

            bytes.extend_from_slice(&c.bytes);
            notes.push(format!(
                "sound {} on entity {}",
                sound_name(resources, c.sample),
                c.entity
            ));
        }

        (bytes, notes)
    }
}

fn sound_name(resources: &[Resource], index: u16) -> String {
    resources
        .iter()
        .find(|r| r.kind == 0 && r.index == index)
        .map_or_else(
            || format!("#{index}"),
            |r| String::from_utf8_lossy(&r.name).into_owned(),
        )
}

#[derive(Default)]
pub struct Ongoing {
    last_anim: Option<Frame>,
    spin_up: Option<Frame>,
    spin: Option<Frame>,
    egon: Option<Frame>,
    sounds: Vec<Started>,
    temps: Vec<Temp>,
    /// The first effect with no restart handling, or of unknown kind
    pub blocker: Option<String>,
}

impl Ongoing {
    /// Follow a demo record before the start
    pub fn observe_record(&mut self, d: &Decoder, f: &Frame) {
        match &f.body {
            FrameBody::WeaponAnim { .. } => self.last_anim = Some(f.clone()),
            FrameBody::Event(e) => {
                let Some(name) = u16::try_from(e.index).ok().and_then(|i| d.event_name(i)) else {
                    self.blocker
                        .get_or_insert(format!("demo event #{}", e.index));
                    return;
                };

                match name {
                    b"events/gaussspin.sc" => {
                        if e.bparam1() == 0 {
                            let same_group = |a: &&Frame| a.frame_number == f.frame_number;
                            self.spin_up = self.last_anim.as_ref().filter(same_group).cloned();
                        }
                        self.spin = Some(f.clone());
                    }
                    b"events/gauss.sc" => {
                        self.spin_up = None;
                        self.spin = None;
                    }
                    b"events/egon_fire.sc" => self.egon = Some(f.clone()),
                    b"events/egon_stop.sc" => self.egon = None,
                    n => self.check_known(n),
                }
            }
            _ => {}
        }
    }

    fn check_known(&mut self, name: &[u8]) {
        if !ONE_SHOT_EVENTS.contains(&name) && !LIFECYCLE_EVENTS.contains(&name) {
            self.blocker
                .get_or_insert(format!("event script {}", String::from_utf8_lossy(name)));
        }
    }

    /// Follow a network message before the start. `payload` is its record's
    /// message bytes and `time` the record's outer time
    pub fn observe_message(
        &mut self,
        d: &Decoder,
        parsed: &ParsedMessage,
        payload: &[u8],
        time: f32,
    ) {
        let bytes = || {
            payload
                .get(parsed.start as usize..parsed.end as usize)
                .map(<[u8]>::to_vec)
        };

        match &parsed.message {
            Message::Sound(s) => {
                self.sounds
                    .retain(|c| !c.same_key(s.entity, s.channel, s.sound_index));
                if s.flags & (SND_STOP | SND_CHANGE) == 0
                    && let Some(b) = bytes()
                {
                    self.sounds.push(Started {
                        entity: s.entity,
                        channel: s.channel,
                        sample: s.sound_index,
                        bytes: b,
                    });
                }
            }
            Message::TempEntity { kind, body } => match lasting(*kind) {
                None => {
                    self.blocker
                        .get_or_insert(format!("temp entity kind {kind}"));
                }
                Some(Lasting::Instant | Lasting::Decal) => {}
                Some(Lasting::KillBeam) => {
                    let entity = body.get(..2).map(|b| u16::from_le_bytes([b[0], b[1]]));
                    self.temps.retain(|t| attached(t) != entity);
                }
                Some(_) => {
                    // A newer text on the same channel replaces the older
                    if *kind == 29 {
                        let channel = body.first();
                        self.temps
                            .retain(|t| !(t.kind == 29 && t.bytes.get(2) == channel));
                    }
                    if let Some(b) = bytes() {
                        self.temps.push(Temp {
                            kind: *kind,
                            time,
                            bytes: b,
                        });
                    }
                }
            },
            Message::Event(entries) => {
                for e in entries {
                    let args = e.args.as_ref();
                    self.network_event(d, e.index, args);
                }
            }
            Message::EventReliable(e) => self.network_event(d, e.index, Some(&e.args)),
            _ => {}
        }
    }

    fn network_event(
        &mut self,
        d: &Decoder,
        index: u16,
        args: Option<&crate::protocol::delta::DeltaUpdate>,
    ) {
        let Some(name) = d.event_name(index) else {
            self.blocker.get_or_insert(format!("event #{index}"));
            return;
        };

        if name == b"events/egon_stop.sc" {
            // The recorder's stop arrives from the server
            let recorder = i64::from(d.player_num) + 1;
            let entity = d.schema("event_t").and_then(|s| {
                let state = StructState::apply(&StructState::default(), &s, args?);
                state.value(&s, "entindex")?.as_i64()
            });
            if entity.is_none_or(|e| e == recorder) {
                self.egon = None;
            }
        } else {
            self.check_known(name);
        }
    }

    /// Effects under way at `start` (outer time). `present` tells whether
    /// an entity is in the bootstrap state
    pub fn at_start(
        &mut self,
        start: f32,
        present: impl Fn(u16) -> bool,
    ) -> Result<(Carried, PendingSounds), String> {
        if let Some(b) = self.blocker.take() {
            return Err(b);
        }

        let mut carried = Carried::default();
        self.carry_charge(start, &mut carried);
        if let Some(fire) = &self.egon {
            let mut startup = fire.clone();
            if let FrameBody::Event(e) = &mut startup.body {
                e.args[64..68].copy_from_slice(&1i32.to_le_bytes());
            }
            carried.records.push(startup);
            carried.notes.push("egon fire (startup event)".into());
        }

        for t in &self.temps {
            let elapsed = (start - t.time).max(0.);
            match carry_temp(t, elapsed, &present) {
                Some(Carry::Sent(bytes, note)) => {
                    carried.messages.extend_from_slice(&bytes);
                    carried.notes.push(note);
                }
                Some(Carry::LeftOut(note)) => carried.left_out.push(note),
                None => {}
            }
        }

        let pending = PendingSounds {
            candidates: std::mem::take(&mut self.sounds)
                .into_iter()
                .map(|s| (s, None))
                .collect(),
        };

        Ok((carried, pending))
    }

    fn carry_charge(&self, start: f32, carried: &mut Carried) {
        let Some(spin) = &self.spin else {
            return;
        };
        if start - spin.time() > CHARGE_GAP {
            return;
        }

        let mut restart = spin.clone();
        if let FrameBody::Event(e) = &mut restart.body {
            e.args[64..68].copy_from_slice(&0i32.to_le_bytes());
        }

        carried.records.extend(self.spin_up.iter().cloned());
        carried.records.push(restart);
        carried
            .notes
            .push("gauss charge (spin-up and spin sound)".into());
    }
}

/// The entity a beam is attached to, from the message bytes (opcode and
/// kind come first)
fn attached(t: &Temp) -> Option<u16> {
    match lasting(t.kind) {
        Some(Lasting::Byte {
            entity: Some(at), ..
        }) => t
            .bytes
            .get(2 + at..4 + at)
            .map(|b| u16::from_le_bytes([b[0], b[1]]) & 0x0FFF),
        _ => None,
    }
}

/// The temp entity sent again with its remaining life, left out when it is
/// still moving under physics, or nothing when it has ended
fn carry_temp(t: &Temp, elapsed: f32, present: &impl Fn(u16) -> bool) -> Option<Carry> {
    let mut bytes = t.bytes.clone();
    let tenths = |b: &[u8], at: usize| b.get(2 + at).map(|v| f32::from(*v) / 10.);
    let note = |left: f32| format!("temp entity {} ({left:.1} s left)", t.kind);

    match lasting(t.kind) {
        Some(Lasting::Byte { at, entity }) => {
            let life = tenths(&bytes, at)?;
            if let Some(e) = entity.and_then(|_| attached(t))
                && !present(e)
            {
                return None;
            }

            // A life of zero lasts until the beam is killed
            let left = if life == 0. { 0. } else { life - elapsed };
            if life != 0. && left <= 0. {
                return None;
            }

            if life != 0. {
                bytes[2 + at] = ((left * 10.).ceil() as u8).max(1);
            }
            Some(Carry::Sent(bytes, note(left)))
        }
        Some(Lasting::Short { at }) => {
            let b = bytes.get(2 + at..4 + at)?;
            let left = f32::from(u16::from_le_bytes([b[0], b[1]])) / 10. - elapsed;
            if left <= 0. {
                return None;
            }

            let v = ((left * 10.).ceil() as u16).max(1).to_le_bytes();
            bytes[2 + at..4 + at].copy_from_slice(&v);
            Some(Carry::Sent(bytes, note(left)))
        }
        Some(Lasting::Text) => {
            carry_text(bytes, elapsed).map(|(b, left)| Carry::Sent(b, note(left)))
        }
        Some(Lasting::Physics { at }) => match tenths(&bytes, at) {
            Some(life) if life > elapsed => Some(Carry::LeftOut(note(life - elapsed))),
            _ => None,
        },
        _ => None,
    }
}

/// HUD text with its timing cut to what remains. Times are 8.8 fixed point
/// after channel, x, y, effect and two colours. The fade-in already
/// happened, so it becomes zero
fn carry_text(mut bytes: Vec<u8>, elapsed: f32) -> Option<(Vec<u8>, f32)> {
    let at = 2 + 1 + 2 + 2 + 1 + 4 + 4;
    let time = |b: &[u8], i: usize| -> Option<f32> {
        let v = b.get(at + 2 * i..at + 2 * i + 2)?;
        Some(f32::from(u16::from_le_bytes([v[0], v[1]])) / 256.)
    };
    let (fade_in, fade_out, hold) = (time(&bytes, 0)?, time(&bytes, 1)?, time(&bytes, 2)?);

    let left = fade_in + hold + fade_out - elapsed;
    if left <= 0. {
        return None;
    }

    let fade_out = fade_out.min(left);
    let hold = left - fade_out;
    let fixed = |v: f32| ((v * 256.).round() as u16).to_le_bytes();
    bytes[at..at + 2].copy_from_slice(&fixed(0.));
    bytes[at + 2..at + 4].copy_from_slice(&fixed(fade_out));
    bytes[at + 4..at + 6].copy_from_slice(&fixed(hold));

    Some((bytes, left))
}
