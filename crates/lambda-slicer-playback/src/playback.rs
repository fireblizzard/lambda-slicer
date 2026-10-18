//! Sequential playback, the reference for every way of reaching a moment
//!
//! Groups are applied in file order, each record to the core decoder and
//! then to the scene and audio state. Reaching a time applies every group
//! at or before it, so an event at exactly the chosen time has happened.
//! Going back, or far ahead, starts from the nearest
//! [`checkpoint`](crate::checkpoint) that can reach the destination, or
//! from the first group when there is none, and plays on from there. A
//! decode error stops playback where it happened, and the frames from then
//! on say why

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, MutexGuard};

use lambda_slicer_core::container::{FrameBody, Limits};
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::state::Decoder;

use crate::audio::{Audio, Channel, SoundLength, SoundName, UNKNOWN_LIMIT};
use crate::checkpoint::{self, Checkpoint, Checkpoints, State};
use crate::controller::{CameraMode, Stamp};
use crate::scene::{self, Camera, Entity, PlayerView, Scene, Viewmodel};
use crate::source::{Cursor, Group, Source};
use crate::time::{Position, ServerTime, SourceTime};

/// Everything a renderer and a mixer need for one moment, and which load
/// and seek it answers
#[derive(Clone, Debug)]
pub struct PresentedFrame {
    pub demo: Arc<Path>,
    pub stamp: Stamp,
    /// The last group applied, none before the first
    pub position: Option<Position>,
    /// The source time shown, at or after the position's
    pub at: SourceTime,
    pub server_time: ServerTime,
    pub map: Option<String>,
    /// Grows whenever the scene changes
    pub scene_revision: u64,
    pub entities: Arc<Vec<Entity>>,
    /// The recorder's view, whatever camera the viewer uses
    pub recorder: Camera,
    pub camera: CameraMode,
    pub viewmodel: Option<Viewmodel>,
    pub light_styles: BTreeMap<u8, Vec<u8>>,
    /// Sounds playing at `at`
    pub sounds: Vec<Channel>,
    pub audio_revision: u64,
    /// False while seeking or stepping, when sounds are kept but not played
    pub audible: bool,
    /// No group is left to apply
    pub end: bool,
    /// Why playback stopped before the end of the demo
    pub stopped: Option<Message>,
    /// How far checkpoints reach, and whether they cover the whole demo
    pub indexed: (SourceTime, bool),
    pub players: Vec<PlayerView>,
    /// The recorder's own player entity
    pub local_entity: u16,
}

type Lengths = Box<dyn Fn(&SoundName) -> SoundLength + Send>;

pub struct Playback {
    path: Arc<Path>,
    source: Source,
    state: State,
    /// The next group, read but not applied
    next: Option<Group>,
    /// The state before the first group, to start again from
    initial: (State, Cursor),
    checkpoints: checkpoint::Shared,
    lengths: Lengths,
}

impl Playback {
    /// Open `path` with a checkpoint store of its own
    pub fn open(path: &Path, limits: Limits) -> Result<Self, Message> {
        Self::open_with(
            path,
            limits,
            Checkpoints::shared(checkpoint::SPACING, checkpoint::BUDGET),
        )
    }

    /// Open `path` with a store other playbacks of the same demo share
    pub fn open_with(
        path: &Path,
        limits: Limits,
        checkpoints: checkpoint::Shared,
    ) -> Result<Self, Message> {
        let mut decoder = Decoder::new();
        decoder.collect_events = false;
        let source = Source::open(path, limits)?;
        let state = State {
            decoder,
            scene: Scene::default(),
            audio: Audio::default(),
            position: None,
            reach: SourceTime::ZERO,
            exhausted: false,
            stopped: None,
        };

        Ok(Self {
            path: Arc::from(path),
            initial: (state.clone(), source.cursor()),
            source,
            state,
            next: None,
            checkpoints,
            lengths: Box::new(|_| SoundLength::Unknown),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn checkpoints(&self) -> &checkpoint::Shared {
        &self.checkpoints
    }

    /// How long each sample plays, from the sample files. Without it every
    /// sound lasts [`UNKNOWN_LIMIT`] seconds
    pub fn set_sound_lengths(
        &mut self,
        lengths: impl Fn(&SoundName) -> SoundLength + Send + 'static,
    ) {
        self.lengths = Box::new(lengths);
    }

    pub fn position(&self) -> Option<Position> {
        self.state.position
    }

    pub fn scene(&self) -> &Scene {
        &self.state.scene
    }

    pub fn audio(&self) -> &Audio {
        &self.state.audio
    }

    pub fn decoder(&self) -> &Decoder {
        &self.state.decoder
    }

    fn store(&self) -> MutexGuard<'_, Checkpoints> {
        // A panic while holding the lock leaves the list itself whole
        self.checkpoints.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Go on from `checkpoint`, or from the first group when none
    fn restore(&mut self, checkpoint: Option<&Checkpoint>) {
        let (state, cursor) = match checkpoint {
            Some(c) => (&c.state, &c.cursor),
            None => (&self.initial.0, &self.initial.1),
        };
        self.state = state.clone();
        self.source.restore(cursor);
        self.next = None;
    }

    /// The next group, read now if it was not yet
    fn peek(&mut self) -> Option<&Group> {
        if self.next.is_none() && !self.state.exhausted && self.state.stopped.is_none() {
            match self.source.next_group() {
                Ok(Some(group)) => self.next = Some(group),
                Ok(None) => {
                    self.state.exhausted = true;
                    self.store().set_complete();
                }
                Err(e) => self.state.stopped = Some(e),
            }
        }
        self.next.as_ref()
    }

    /// Apply the next group, and keep the state after it when a checkpoint
    /// is due. False when there is no group to apply
    fn apply_next(&mut self) -> bool {
        if self.peek().is_none() {
            return false;
        }
        let Some(group) = self.next.take() else {
            return false;
        };

        // Sounds finish at the same group times however playback got here,
        // so a stop or change always finds the same sounds
        self.state.audio.retire(group.time, &*self.lengths);

        let time = group.time;
        let State {
            decoder,
            scene,
            audio,
            stopped,
            ..
        } = &mut self.state;
        for record in &group.records {
            let result = match &record.body {
                FrameBody::Network(frame) => {
                    let applied = decoder.network_frame(record.location, frame, |d, m| {
                        scene.on_message(d, m, time);
                        audio.on_message(d, m, time);
                    });
                    scene.after_network(decoder, frame);
                    applied
                }
                FrameBody::Event(event) => {
                    decoder.demo_event(record.location, event);
                    Ok(())
                }
                other => {
                    scene.on_record(other, time);
                    audio.on_record(decoder, other, time);
                    Ok(())
                }
            };
            if let Err(e) = result {
                *stopped = Some(e.message());
                break;
            }
        }

        let position = Position {
            group: group.number,
            segment: self.state.decoder.segment,
            time,
        };
        self.state.position = Some(position);
        self.state.reach = SourceTime(self.state.reach.0.max(time.0));

        if self.store().due(position) {
            let entities = self.state.scene.entities.len() * size_of::<Entity>();
            let sounds = size_of_val(self.state.audio.channels());
            let bytes = self.state.decoder.retained_bytes() + 2 * entities + sounds;
            let checkpoint = Checkpoint {
                group: position.group,
                segment: position.segment,
                time,
                reach: self.state.reach,
                bytes: bytes as u64,
                state: self.state.clone(),
                cursor: self.source.cursor(),
            };
            self.store().insert(checkpoint);
        }
        true
    }

    /// Apply every group at or before `time`. Playback goes on from where it
    /// is when that can reach `time` and no checkpoint is nearer, and starts
    /// from the nearest checkpoint that can reach it otherwise
    pub fn advance_to(&mut self, time: SourceTime) -> Result<(), Message> {
        let time = SourceTime(time.0.max(0.));
        let live_can = self.state.reach <= time;
        let live = self.state.position.map(|p| p.group);
        let nearest = self.store().for_time(time);
        match &nearest {
            Some(c) if !live_can || live.is_none_or(|g| c.group > g) => self.restore(Some(c)),
            None if !live_can => self.restore(None),
            _ => {}
        }

        while self.peek().is_some_and(|g| g.time <= time) {
            self.apply_next();
        }
        Ok(())
    }

    /// Apply groups until `group` is the last applied, or the demo ends
    pub fn seek_group(&mut self, group: u64) -> Result<(), Message> {
        let live = self.state.position.map(|p| p.group);
        let live_can = live.is_none_or(|g| g <= group);
        let nearest = self.store().for_group(group);
        match &nearest {
            Some(c) if !live_can || live.is_none_or(|g| c.group > g) => self.restore(Some(c)),
            None if !live_can => self.restore(None),
            _ => {}
        }

        while self.state.position.is_none_or(|p| p.group < group) {
            if !self.apply_next() {
                break;
            }
        }
        Ok(())
    }

    /// Play on from the furthest checkpoint for up to `groups` groups,
    /// adding checkpoints as they fall due. True once the demo is covered
    pub fn index(&mut self, groups: usize) -> bool {
        let furthest = self.store().last();
        if let Some(c) = furthest
            && self.state.position.is_none_or(|p| c.group > p.group)
        {
            self.restore(Some(&c));
        }

        for _ in 0..groups {
            if !self.apply_next() {
                break;
            }
        }
        self.at_end()
    }

    /// No group is left, because the demo ended or decoding stopped
    pub fn at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    /// The state at `at`, which should be at or after the position
    pub fn present(
        &mut self,
        stamp: Stamp,
        at: SourceTime,
        camera: CameraMode,
        audible: bool,
    ) -> PresentedFrame {
        let end = self.at_end();
        let indexed = self.store().indexed();
        let state = &self.state;
        let at = match state.position {
            Some(p) if at < p.time => p.time,
            _ => at,
        };
        let sounds = state
            .audio
            .channels()
            .iter()
            .filter(|c| match (self.lengths)(&c.sound) {
                SoundLength::Once(seconds) => c.cursor(at) < seconds,
                SoundLength::Looping => true,
                SoundLength::Unknown => at.0 - c.started.0 < UNKNOWN_LIMIT,
            })
            .cloned()
            .collect();

        PresentedFrame {
            demo: self.path.clone(),
            stamp,
            position: state.position,
            at,
            server_time: ServerTime(f32::from_bits(state.decoder.time_bits)),
            map: state.scene.map.clone(),
            scene_revision: state.scene.revision,
            entities: state.scene.entities.clone(),
            recorder: state.scene.camera,
            camera,
            viewmodel: state.scene.viewmodel,
            light_styles: state.scene.light_styles.clone(),
            sounds,
            audio_revision: state.audio.revision,
            audible,
            end,
            stopped: state.stopped.clone(),
            indexed,
            players: scene::players(&state.decoder),
            local_entity: u16::from(state.decoder.player_num) + 1,
        }
    }
}
