//! Sequential playback, the reference for every way of reaching a moment
//!
//! Groups are applied in file order, each record to the core decoder and
//! then to the scene and audio state. Reaching a time applies every group
//! at or before it, so an event at exactly the chosen time has happened.
//! Going back starts again from the first group, until checkpoints (8e)
//! give a nearer place to start from. A decode error stops playback where
//! it happened, and the frames from then on say why

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lambda_slicer_core::container::{FrameBody, Limits};
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::state::Decoder;

use crate::audio::{Audio, Channel, SoundLength, SoundName, UNKNOWN_LIMIT};
use crate::controller::{CameraMode, Stamp};
use crate::scene::{Camera, Entity, Scene, Viewmodel};
use crate::source::{Group, Source};
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
}

type Lengths = Box<dyn Fn(&SoundName) -> SoundLength + Send>;

pub struct Playback {
    path: Arc<Path>,
    limits: Limits,
    source: Source,
    decoder: Decoder,
    scene: Scene,
    audio: Audio,
    position: Option<Position>,
    /// The next group, read but not applied
    next: Option<Group>,
    /// The source has no group left
    exhausted: bool,
    stopped: Option<Message>,
    lengths: Lengths,
}

impl Playback {
    pub fn open(path: &Path, limits: Limits) -> Result<Self, Message> {
        let mut decoder = Decoder::new();
        decoder.collect_events = false;

        Ok(Self {
            path: Arc::from(path),
            limits,
            source: Source::open(path, limits)?,
            decoder,
            scene: Scene::default(),
            audio: Audio::default(),
            position: None,
            next: None,
            exhausted: false,
            stopped: None,
            lengths: Box::new(|_| SoundLength::Unknown),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
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
        self.position
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    pub fn decoder(&self) -> &Decoder {
        &self.decoder
    }

    /// Back to before the first group
    fn restart(&mut self) -> Result<(), Message> {
        let path = PathBuf::from(&*self.path);
        let lengths = std::mem::replace(&mut self.lengths, Box::new(|_| SoundLength::Unknown));
        *self = Self::open(&path, self.limits)?;
        self.lengths = lengths;
        Ok(())
    }

    /// The next group, read now if it was not yet
    fn peek(&mut self) -> Option<&Group> {
        if self.next.is_none() && !self.exhausted && self.stopped.is_none() {
            match self.source.next_group() {
                Ok(Some(group)) => self.next = Some(group),
                Ok(None) => self.exhausted = true,
                Err(e) => self.stopped = Some(e),
            }
        }
        self.next.as_ref()
    }

    /// Apply the next group. False when there is none to apply
    fn apply_next(&mut self) -> bool {
        if self.peek().is_none() {
            return false;
        }
        let Some(group) = self.next.take() else {
            return false;
        };

        // Sounds finish at the same group times however playback got here,
        // so a stop or change always finds the same sounds
        self.audio.retire(group.time, &*self.lengths);

        let time = group.time;
        let Self {
            decoder,
            scene,
            audio,
            ..
        } = self;
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
                self.stopped = Some(e.message());
                break;
            }
        }

        self.position = Some(Position {
            group: group.number,
            segment: self.decoder.segment,
            time,
        });
        true
    }

    /// Apply every group at or before `time`, starting again when `time` is
    /// before the current position
    pub fn advance_to(&mut self, time: SourceTime) -> Result<(), Message> {
        let time = SourceTime(time.0.max(0.));
        if self.position.is_some_and(|p| time < p.time) {
            self.restart()?;
        }

        while self.peek().is_some_and(|g| g.time <= time) {
            self.apply_next();
        }
        Ok(())
    }

    /// Apply groups until `group` is the last applied, or the demo ends
    pub fn seek_group(&mut self, group: u64) -> Result<(), Message> {
        if self.position.is_some_and(|p| group < p.group) {
            self.restart()?;
        }

        while self.position.is_none_or(|p| p.group < group) {
            if !self.apply_next() {
                break;
            }
        }
        Ok(())
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
        let at = match self.position {
            Some(p) if at < p.time => p.time,
            _ => at,
        };
        let sounds = self
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
            position: self.position,
            at,
            server_time: ServerTime(f32::from_bits(self.decoder.time_bits)),
            map: self.scene.map.clone(),
            scene_revision: self.scene.revision,
            entities: self.scene.entities.clone(),
            recorder: self.scene.camera,
            camera,
            viewmodel: self.scene.viewmodel,
            light_styles: self.scene.light_styles.clone(),
            sounds,
            audio_revision: self.audio.revision,
            audible,
            end,
            stopped: self.stopped.clone(),
        }
    }
}
