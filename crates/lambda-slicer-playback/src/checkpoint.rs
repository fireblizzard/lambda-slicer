//! Saved playback states to seek from
//!
//! A checkpoint is everything sequential playback had after one group, the
//! decoder with its schemas, baselines and delta history, the scene, the
//! sound state and where the source goes on. Restoring one and playing on
//! gives exactly what playing from the first group would, which the
//! equivalence tests check. Playback takes one when a session begins and
//! then every [`SPACING`] seconds of source time, whichever playback passes
//! that way first, so the window's playback and a background indexer fill
//! the same store.
//!
//! Source times can step back between groups, so each checkpoint keeps the
//! latest time of any group up to it, its reach. Reaching time `t` from a
//! checkpoint is the same as from the start only if its reach is not after
//! `t`.
//!
//! A checkpoint holds the decoder's last 64 snapshots, any of which a later
//! packet may take as its delta base, so it weighs a megabyte or more in a
//! busy game. When the store passes its [`BUDGET`], it keeps every other
//! checkpoint and doubles the spacing from then on. Seeks then replay
//! further, at about 3 ms per second of a busy demo

use std::sync::{Arc, Mutex};

use lambda_slicer_core::message::Message;
use lambda_slicer_core::state::Decoder;

use crate::animation::Animations;
use crate::audio::Audio;
use crate::beams::Beams;
use crate::decals::Decals;
use crate::events::Events;
use crate::hud::Hud;
use crate::scene::Scene;
use crate::source::Cursor;
use crate::tempents::TempEntities;
use crate::time::{Position, SourceTime};

/// Seconds of source time between checkpoints, at first
pub const SPACING: f64 = 2.;

/// Bytes all of one demo's checkpoints may hold
pub const BUDGET: u64 = 256 * 1024 * 1024;

/// What sequential playback holds between groups
#[derive(Clone)]
pub(crate) struct State {
    pub decoder: Decoder,
    pub scene: Scene,
    pub audio: Audio,
    /// Events waiting for their time
    pub events: Events,
    pub animations: Animations,
    pub tempents: TempEntities,
    pub decals: Decals,
    pub beams: Beams,
    /// What the client's HUD shows, AG's countdown
    pub hud: Hud,
    pub position: Option<Position>,
    /// The latest source time of any group applied
    pub reach: SourceTime,
    /// The source has no group left
    pub exhausted: bool,
    pub stopped: Option<Message>,
}

pub struct Checkpoint {
    pub group: u64,
    pub segment: u32,
    pub time: SourceTime,
    pub reach: SourceTime,
    /// About how many bytes it holds that other checkpoints do not share
    pub bytes: u64,
    pub(crate) state: State,
    pub(crate) cursor: Cursor,
}

/// One demo's checkpoints, ordered by group
pub struct Checkpoints {
    list: Vec<Arc<Checkpoint>>,
    spacing: f64,
    budget: u64,
    bytes: u64,
    /// Some playback reached the last group, so the list covers the demo
    complete: bool,
}

pub type Shared = Arc<Mutex<Checkpoints>>;

impl Default for Checkpoints {
    fn default() -> Self {
        Self::new(SPACING, BUDGET)
    }
}

impl Checkpoints {
    pub fn new(spacing: f64, budget: u64) -> Self {
        Self {
            list: Vec::new(),
            spacing,
            budget,
            bytes: 0,
            complete: false,
        }
    }

    pub fn shared(spacing: f64, budget: u64) -> Shared {
        Arc::new(Mutex::new(Self::new(spacing, budget)))
    }

    /// The spacing new checkpoints get, which thinning doubles
    pub fn spacing(&self) -> f64 {
        self.spacing
    }

    /// About how many bytes the checkpoints hold
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// How far checkpoints reach, and whether they cover the whole demo
    pub fn indexed(&self) -> (SourceTime, bool) {
        let reach = self.list.last().map_or(SourceTime::ZERO, |c| c.reach);
        (reach, self.complete)
    }

    pub(crate) fn last(&self) -> Option<Arc<Checkpoint>> {
        self.list.last().cloned()
    }

    pub(crate) fn set_complete(&mut self) {
        self.complete = true;
    }

    /// Whether the state after `position` should be kept, given the
    /// checkpoint before it
    pub(crate) fn due(&self, position: Position) -> bool {
        let at = self.list.partition_point(|c| c.group <= position.group);
        let Some(before) = at.checked_sub(1).map(|i| &self.list[i]) else {
            return true;
        };
        if before.group == position.group {
            return false;
        }

        before.segment != position.segment || position.time.0 - before.time.0 >= self.spacing
    }

    pub(crate) fn insert(&mut self, checkpoint: Checkpoint) {
        let at = self.list.partition_point(|c| c.group < checkpoint.group);
        if self
            .list
            .get(at)
            .is_some_and(|c| c.group == checkpoint.group)
        {
            return;
        }
        self.bytes += checkpoint.bytes;
        self.list.insert(at, Arc::new(checkpoint));

        while self.bytes > self.budget && self.list.len() > 1 {
            let mut index = 0;
            self.list.retain(|_| {
                index += 1;
                index % 2 == 1
            });
            self.bytes = self.list.iter().map(|c| c.bytes).sum();
            self.spacing *= 2.;
        }
    }

    /// The latest checkpoint whose reach is not after `time`
    pub(crate) fn for_time(&self, time: SourceTime) -> Option<Arc<Checkpoint>> {
        let at = self.list.partition_point(|c| c.reach <= time);
        at.checked_sub(1).map(|i| self.list[i].clone())
    }

    /// The latest checkpoint at or before `group`
    pub(crate) fn for_group(&self, group: u64) -> Option<Arc<Checkpoint>> {
        let at = self.list.partition_point(|c| c.group <= group);
        at.checked_sub(1).map(|i| self.list[i].clone())
    }
}
