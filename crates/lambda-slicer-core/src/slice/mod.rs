//! Export. M1 provides only the S0 control exports used to qualify the
//! container writer in the game. Arbitrary-start slicing (S1) needs the
//! audited state decoder and is not implemented yet

pub mod controls;
pub mod ongoing;
pub mod output;
pub mod persistent;
pub mod restart;

use std::path::PathBuf;

use crate::container::ContainerError;
use crate::protocol::profile::UnsupportedProfile;
use crate::timeline::time::DemoTime;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Container(#[from] ContainerError),
    #[error("destination already exists: {0}")]
    DestinationExists(PathBuf),
    #[error("destination is the source file: {0}")]
    SameAsSource(PathBuf),
    #[error("destination directory does not exist: {0}")]
    MissingParent(PathBuf),
    #[error("source changed during export: {0}")]
    SourceChanged(PathBuf),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error(transparent)]
    UnsupportedProtocol(#[from] UnsupportedProfile),
    #[error("selected range retains no network frames")]
    EmptyRange,
    #[error("end {end} is beyond the segment end {segment_end}")]
    RangeBeyondEnd {
        end: DemoTime,
        segment_end: DemoTime,
    },
    #[error("output failed validation: {0}")]
    Validation(String),
    #[error("source does not decode: {0}")]
    Decode(String),
    #[error("the source failed its cross-check, so its reconstructed state is not trusted")]
    SourceFailedCrossCheck,
    #[error("state at {0} rests on an inferred delta base")]
    UnprovenState(String),
    #[error("the client dropped packets as too old at {0}, which a restart cannot reproduce yet")]
    ClientDroppedPackets(String),
    #[error("a new session starts at {0} inside the range")]
    CrossSegmentRange(String),
    #[error("delta base {0} before the start is not in the recording")]
    MissingDeltaBase(u32),
    #[error("{0} before the start sets state a restart cannot carry over yet")]
    UnknownPersistentMessage(String),
    #[error("{0} is under way at the start and cannot be carried over it yet")]
    ActiveEffect(String),
}
