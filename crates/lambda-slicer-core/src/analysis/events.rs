//! Recorded event evidence
//!
//! One record per event the recording holds, whether a standalone demo
//! `Event` record or an `svc_event`/`svc_event_reliable` network event, with
//! its complete arguments, timing and actor resolution. Weapon actions are
//! derived from these and point back to them, so detector changes can be
//! audited against the raw evidence

use serde::Serialize;

use crate::protocol::delta::Value;
use crate::state::{ActorResolution, DecodedEvent, Decoder, EventSource, PlayerIdentity};

/// `event_args_t` as the client receives it. Network events send no args
/// flags or velocity, so those are only known for demo records
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct EventArgs {
    pub flags: Option<i32>,
    pub entindex: i32,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub velocity: Option<[f32; 3]>,
    pub ducking: i32,
    pub fparam1: f32,
    pub fparam2: f32,
    pub iparam1: i32,
    pub iparam2: i32,
    pub bparam1: i32,
    pub bparam2: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct RecordedEvent {
    /// Position among the recording's events, stable for one decoder version
    pub id: usize,
    pub segment: u32,
    pub entry: u32,
    /// Outer record ordinal and playback time of the frame that carried it
    pub ordinal: u64,
    pub time: f32,
    /// `svc_time` when it arrived
    pub server_time: f32,
    pub source: EventSource,
    /// Position of the network message in its packet
    pub message_ordinal: Option<u32>,
    pub index: u16,
    /// Event script path, lossily decoded for display. Matching uses the
    /// raw bytes
    pub resource: Option<String>,
    /// Outer dispatch flags of a demo record, separate from the args flags
    pub dispatch_flags: Option<i32>,
    pub args: EventArgs,
    /// Delay before the client plays it, in seconds
    pub delay: Option<f32>,
    pub actor: ActorResolution,
    /// The player behind the actor when the event arrived
    pub player: Option<PlayerIdentity>,
}

fn network_args(d: &Decoder, e: &DecodedEvent) -> EventArgs {
    let (Some(schema), Some(state)) = (d.schema("event_t"), &e.args) else {
        return EventArgs::default();
    };

    let float = |name: &str| match state.value(&schema, name) {
        Some(Value::Float(f)) => f,
        Some(Value::Int(i)) => i as f32,
        _ => 0.,
    };
    let int = |name: &str| match state.value(&schema, name) {
        Some(Value::Int(i)) => i as i32,
        Some(Value::Float(f)) => f as i32,
        _ => 0,
    };
    let vec3 = |base: &str| [0, 1, 2].map(|i| float(&format!("{base}[{i}]")));

    EventArgs {
        flags: None,
        entindex: int("entindex"),
        origin: vec3("origin"),
        angles: vec3("angles"),
        velocity: None,
        ducking: int("ducking"),
        fparam1: float("fparam1"),
        fparam2: float("fparam2"),
        iparam1: int("iparam1"),
        iparam2: int("iparam2"),
        bparam1: int("bparam1"),
        bparam2: int("bparam2"),
    }
}

/// Every event the decoder collected, in recording order
pub fn recorded_events(d: &Decoder) -> Vec<RecordedEvent> {
    d.events
        .iter()
        .enumerate()
        .map(|(id, e)| recorded_event(d, id, e))
        .collect()
}

/// One event the decoder collected, numbered `id`, read while the decoder
/// still holds the schema it arrived with
pub fn recorded_event(d: &Decoder, id: usize, e: &DecodedEvent) -> RecordedEvent {
    let (args, dispatch_flags, delay) = match &e.demo {
        Some(r) => (
            EventArgs {
                flags: Some(r.args_flags()),
                entindex: r.entity_index(),
                origin: r.origin(),
                angles: r.angles(),
                velocity: Some(r.velocity()),
                ducking: r.ducking(),
                fparam1: r.fparam1(),
                fparam2: r.fparam2(),
                iparam1: r.iparam1(),
                iparam2: r.iparam2(),
                bparam1: r.bparam1(),
                bparam2: r.bparam2(),
            },
            Some(r.flags),
            Some(r.delay()),
        ),
        None => (
            network_args(d, e),
            None,
            e.delay_raw.map(|v| v as f32 / 100.),
        ),
    };

    RecordedEvent {
        id,
        segment: e.segment,
        entry: e.location.entry,
        ordinal: e.location.ordinal,
        time: e.location.playback_time(),
        server_time: f32::from_bits(e.server_time_bits),
        source: e.source,
        message_ordinal: e.message_ordinal,
        index: e.index,
        resource: e
            .resource
            .as_ref()
            .map(|r| String::from_utf8_lossy(r).into_owned()),
        dispatch_flags,
        args,
        delay,
        actor: e.actor.clone(),
        player: e.player.clone(),
    }
}
