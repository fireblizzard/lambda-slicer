//! Decode state reconstructed from the message stream, including schemas,
//! resources, user messages, baselines, and sequence-indexed snapshots of the
//! complete entity set, clientdata, and weapon data

mod run;
mod world;

pub use run::{DECODE_REPORT_SCHEMA, DecodeOptions, DecodeReport, DecodeStatus, decode_demo};

pub use world::{
    ActorResolution, DecodeError, DecodeErrorKind, DecodeStats, DecodedEvent, Decoder, EntityState,
    EventSource, FrameLocation, NameChange, NameChangeKind, PlayerIdentity, PlayerInfo, Snapshot,
};
