//! Decode state reconstructed from the message stream: schemas, resources,
//! user messages, baselines, and sequence-indexed snapshots of the complete
//! entity set, clientdata, and weapon data.

mod run;
mod world;

pub use run::{DecodeOptions, DecodeReport, DecodeStatus, decode_demo};

pub use world::{
    ActorResolution, DecodeError, DecodeErrorKind, DecodeStats, DecodedEvent, Decoder, EntityState,
    EventSource, FrameLocation, PlayerInfo, Snapshot,
};
