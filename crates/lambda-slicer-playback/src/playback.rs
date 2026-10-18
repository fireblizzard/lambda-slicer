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

use lambda_slicer_core::analysis::events::recorded_event;
use lambda_slicer_core::analysis::motion;
use lambda_slicer_core::container::{FrameBody, Limits};
use lambda_slicer_core::message::{Coded, Message};
use lambda_slicer_core::protocol::messages::{self as protocol, resource_type};
use lambda_slicer_core::state::Decoder;

use crate::GameFiles;
use crate::animation::{Animations, Fired, Flash, IN_EYE, Sequence};
use crate::audio::{Audio, CHAN_AUTO, Channel, SoundLength, SoundName, UNKNOWN_LIMIT, view_entity};
use crate::checkpoint::{self, Checkpoint, Checkpoints, State};
use crate::controller::{CameraMode, Stamp};
use crate::events::{ATTN_NORM, Action, ClientEvent, Events, PITCH_NORM, SoundCall, handle};
use crate::scene::{self, Camera, Entity, PlayerView, Scene, Viewmodel};
use crate::source::{Cursor, Group, Source};
use crate::tempents::{Lookup, TempEntities, TempEntity};
use crate::time::{Position, ServerTime, SourceTime};
use crate::trace::Hull;

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
    /// The client's clock at `at`, the latest snapshot's server time plus
    /// the source time since it arrived
    pub client_time: f64,
    pub map: Option<String>,
    /// Grows whenever the scene changes
    pub scene_revision: u64,
    /// As the client draws them at `client_time`, moved between snapshots
    /// and with players' legs
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
    /// Muzzle flashes fired lately, at or before `at`
    pub flashes: Vec<Flash>,
    /// The client's own sprites and models, shells among them
    pub temp_entities: Vec<TempEntity>,
}

type Lengths = Box<dyn Fn(&SoundName) -> SoundLength + Send>;
type Sequences = Box<dyn Fn(&str) -> Option<Arc<[Sequence]>> + Send>;
type Hulls = Box<dyn Fn(&str) -> Option<Arc<Hull>> + Send>;
type SpriteFrames = Box<dyn Fn(&str) -> Option<u32> + Send>;

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
    sequences: Sequences,
    hulls: Hulls,
    sprite_frames: SpriteFrames,
    /// The map traced through and its world, once read
    hull: Option<(String, Option<Arc<Hull>>)>,
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
        // Events are taken from the decoder as each record brings them
        let mut decoder = Decoder::new();
        decoder.collect_events = true;
        let source = Source::open(path, limits)?;
        let state = State {
            decoder,
            scene: Scene::default(),
            audio: Audio::default(),
            events: Events::default(),
            animations: Animations::default(),
            tempents: TempEntities::default(),
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
            sequences: Box::new(|_| None),
            hulls: Box::new(|_| None),
            sprite_frames: Box::new(|_| None),
            hull: None,
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

    /// Each model's sequences, from the model files. Without them no
    /// animation fires its client events
    pub fn set_model_sequences(
        &mut self,
        sequences: impl Fn(&str) -> Option<Arc<[Sequence]>> + Send + 'static,
    ) {
        self.sequences = Box::new(sequences);
    }

    /// Both from the game's files
    pub fn set_game_files(&mut self, files: &GameFiles) {
        let lengths = files.lengths.clone();
        self.set_sound_lengths(move |name| lengths(name));
        let sequences = files.sequences.clone();
        self.set_model_sequences(move |model| sequences(model));
        let hulls = files.hulls.clone();
        self.hulls = Box::new(move |map| hulls(map));
        let frames = files.sprite_frames.clone();
        self.sprite_frames = Box::new(move |sprite| frames(sprite));
        self.hull = None;
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

    pub fn animations(&self) -> &Animations {
        &self.state.animations
    }

    pub fn tempents(&self) -> &TempEntities {
        &self.state.tempents
    }

    pub fn decoder(&self) -> &Decoder {
        &self.state.decoder
    }

    /// The world of the session's map to trace temporary entities through
    fn world(&mut self) -> Option<Arc<Hull>> {
        let map = self.state.scene.map.as_deref()?;
        if self.hull.as_ref().is_none_or(|(name, _)| name != map) {
            self.hull = Some((map.to_owned(), (self.hulls)(map)));
        }
        self.hull.as_ref()?.1.clone()
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
        let lengths: &dyn Fn(&SoundName) -> SoundLength = &*self.lengths;
        let frames: &dyn Fn(&str) -> Option<u32> = &*self.sprite_frames;
        let State {
            decoder,
            scene,
            audio,
            events,
            tempents,
            stopped,
            ..
        } = &mut self.state;
        let mut effects = Effects {
            audio,
            events,
            tempents,
            lengths,
        };
        for event in effects.events.due(time) {
            dispatch(&event, decoder, scene, &mut effects);
        }
        for record in &group.records {
            let result = match &record.body {
                FrameBody::Network(frame) => {
                    let audio = &mut *effects.audio;
                    let tempents = &mut *effects.tempents;
                    let applied = decoder.network_frame(record.location, frame, |d, m| {
                        scene.on_message(d, m, time);
                        audio.on_message(d, m, time, lengths);
                        match &m.message {
                            protocol::Message::ServerInfo(_) => tempents.clear(),
                            protocol::Message::TempEntity { kind, body } => {
                                let model = |index: i16| precached_model(d, index);
                                let lookup = Lookup {
                                    model: &model,
                                    frames,
                                };
                                let view = view_entity(d);
                                for call in tempents.message(*kind, body, time, &lookup) {
                                    audio.play(&call, time, view, lengths);
                                }
                            }
                            _ => {}
                        }
                    });
                    scene.after_network(decoder, frame, time);
                    applied
                }
                FrameBody::Event(event) => {
                    decoder.demo_event(record.location, event);
                    Ok(())
                }
                other => {
                    scene.on_record(other, time);
                    effects.audio.on_record(decoder, other, time, lengths);
                    Ok(())
                }
            };
            for decoded in std::mem::take(&mut decoder.events) {
                let recorded = recorded_event(decoder, 0, &decoded);
                let max_clients = decoder.max_clients;
                if let Some(event) = ClientEvent::new(&recorded, time, max_clients)
                    .and_then(|event| effects.events.arrive(event, time))
                {
                    dispatch(&event, decoder, scene, &mut effects);
                }
            }
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
        self.state.scene.step_gaits(time);
        let fired = self
            .state
            .animations
            .step(&self.state.scene, &*self.sequences, time);
        let view = view_entity(&self.state.decoder);
        for f in fired {
            if let Fired::Sound { sample, origin } = f {
                let call = SoundCall {
                    entity: view,
                    channel: CHAN_AUTO,
                    sample,
                    volume: 1.,
                    attenuation: ATTN_NORM,
                    flags: 0,
                    pitch: PITCH_NORM,
                    origin,
                };
                self.state.audio.play(&call, time, view, &*self.lengths);
            }
        }
        let world = self.world();
        let gravity = motion::gravity(&self.state.decoder);
        let bounces = self.state.tempents.step(time, world.as_deref(), gravity);
        for call in bounces {
            self.state.audio.play(&call, time, view, &*self.lengths);
        }
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
        let client_time = state.scene.client_time(at);
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
            entities: Arc::new(state.scene.presented(client_time)),
            client_time,
            recorder: state.scene.camera.clone(),
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
            flashes: state
                .animations
                .flashes()
                .iter()
                .filter(|f| f.time <= at)
                .copied()
                .collect(),
            temp_entities: state
                .tempents
                .entities()
                .iter()
                .filter(|e| e.visible())
                .cloned()
                .collect(),
        }
    }
}

/// The model precached at `index`, a sprite or studio model's path
fn precached_model(decoder: &Decoder, index: i16) -> Option<Arc<str>> {
    let index = u16::try_from(index).ok()?;
    decoder
        .resources()
        .iter()
        .find(|r| r.kind == resource_type::MODEL && r.index == index)
        .map(|r| Arc::from(String::from_utf8_lossy(&r.name).as_ref()))
}

/// What an event's handler changes besides the scene
struct Effects<'a> {
    audio: &'a mut Audio,
    events: &'a mut Events,
    tempents: &'a mut TempEntities,
    lengths: &'a dyn Fn(&SoundName) -> SoundLength,
}

/// Run the client's handler for `event`. An event that gives no origin
/// sounds where its entity is
fn dispatch(event: &ClientEvent, decoder: &Decoder, scene: &mut Scene, effects: &mut Effects) {
    let Effects {
        audio,
        events,
        tempents,
        lengths,
    } = effects;
    let mut event = event.clone();
    if event.args.origin == [0.; 3]
        && let Some(e) = scene.entities.iter().find(|e| e.id.number == event.entity)
    {
        event.args.origin = e.origin;
    }
    let view = view_entity(decoder);
    let camera = &scene.camera;
    let local = if camera.observer_mode == IN_EYE {
        i64::from(event.entity) == camera.observer_target
    } else {
        event.entity == u16::from(decoder.player_num) + 1
    };
    for action in handle(&event, local, events.swings()) {
        match action {
            Action::Sound(call) => audio.play(&call, event.time, view, lengths),
            Action::Kill { entity, script } => events.kill(entity, script),
            Action::WeaponAnimation { sequence, body } => {
                scene.animate_viewmodel(sequence, body, event.time)
            }
            Action::Shell(throw) => tempents.shell(&throw, event.time),
        }
    }
}
