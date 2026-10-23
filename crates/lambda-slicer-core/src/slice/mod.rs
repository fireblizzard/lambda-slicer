//! Export. The control exports test the container writer in the game
//! ([`controls`]), and the restart export writes a range that plays on its
//! own ([`restart`]).
//! [`range`] tells where a restart would cut without running it, and
//! [`naming`] suggests file names and the console command for a result

pub mod controls;
pub mod naming;
pub mod ongoing;
pub mod output;
pub mod persistent;
pub mod range;
pub mod restart;

use std::path::PathBuf;

use crate::container::ContainerError;
use crate::message::{Arg, Coded};
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
    #[error("the start is while the game is still loading a session, before its first full update")]
    StartDuringLoading,
    #[error("the start is at the first frame group, which an exact copy covers")]
    StartAtFirstGroup,
    #[error("delta base {0} before the start is not in the recording")]
    MissingDeltaBase(u32),
    #[error("the {0} message is not supported in clips yet")]
    UnknownPersistentMessage(String),
    #[error("{0} is under way at the start and cannot be carried over it yet")]
    ActiveEffect(String),
    #[error("a bootstrap time field cannot be sent faithfully: {0}")]
    UnrepresentableTimeField(String),
}

impl Coded for ExportError {
    fn code(&self) -> &'static str {
        match self {
            ExportError::Io { .. } => "export-io",
            ExportError::Container(e) => e.code(),
            ExportError::DestinationExists(_) => "export-destination-exists",
            ExportError::SameAsSource(_) => "export-same-as-source",
            ExportError::MissingParent(_) => "export-missing-parent",
            ExportError::SourceChanged(_) => "export-source-changed",
            ExportError::Unsupported(_) => "export-unsupported",
            ExportError::UnsupportedProtocol(e) => e.code(),
            ExportError::EmptyRange => "export-empty-range",
            ExportError::RangeBeyondEnd { .. } => "export-range-beyond-end",
            ExportError::Validation(_) => "export-validation-failed",
            ExportError::Decode(_) => "export-source-decode-failed",
            ExportError::SourceFailedCrossCheck => "export-source-failed-crosscheck",
            ExportError::UnprovenState(_) => "export-unproven-state",
            ExportError::ClientDroppedPackets(_) => "export-client-dropped-packets",
            ExportError::CrossSegmentRange(_) => "export-cross-segment",
            ExportError::StartDuringLoading => "export-start-during-loading",
            ExportError::StartAtFirstGroup => "export-start-first-group",
            ExportError::MissingDeltaBase(_) => "export-missing-delta-base",
            ExportError::UnknownPersistentMessage(_) => "export-unknown-persistent-message",
            ExportError::ActiveEffect(_) => "export-active-effect",
            ExportError::UnrepresentableTimeField(_) => "export-unrepresentable-time-field",
        }
    }

    fn args(&self) -> Vec<(&'static str, Arg)> {
        match self {
            ExportError::Io { path, source } => vec![
                ("path", path.as_path().into()),
                ("detail", source.to_string().into()),
            ],
            ExportError::Container(e) => e.args(),
            ExportError::UnsupportedProtocol(e) => e.args(),
            ExportError::DestinationExists(p)
            | ExportError::SameAsSource(p)
            | ExportError::MissingParent(p)
            | ExportError::SourceChanged(p) => vec![("path", p.as_path().into())],
            ExportError::Unsupported(d)
            | ExportError::Validation(d)
            | ExportError::Decode(d)
            | ExportError::UnrepresentableTimeField(d) => vec![("detail", d.clone().into())],
            ExportError::RangeBeyondEnd { end, segment_end } => vec![
                ("end", end.to_string().into()),
                ("segment_end", segment_end.to_string().into()),
            ],
            ExportError::UnprovenState(t)
            | ExportError::ClientDroppedPackets(t)
            | ExportError::CrossSegmentRange(t) => vec![("time", t.clone().into())],
            ExportError::MissingDeltaBase(sequence) => vec![("sequence", (*sequence).into())],
            ExportError::UnknownPersistentMessage(name) => vec![("name", name.clone().into())],
            ExportError::ActiveEffect(effect) => vec![("effect", effect.clone().into())],
            ExportError::EmptyRange
            | ExportError::SourceFailedCrossCheck
            | ExportError::StartDuringLoading
            | ExportError::StartAtFirstGroup => Vec::new(),
        }
    }
}
