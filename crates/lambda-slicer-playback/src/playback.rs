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

use std::collections::{BTreeMap, VecDeque};
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
use crate::announcements::Announced;
use crate::audio::{Audio, CHAN_AUTO, Channel, SoundLength, SoundName, UNKNOWN_LIMIT, view_entity};
use crate::beams::{Beams, Trail};
use crate::bullets;
use crate::checkpoint::{self, Checkpoint, Checkpoints, State};
use crate::controller::{CameraMode, Stamp};
use crate::damage::DamageShown;
use crate::decals::{Decal, Decals};
use crate::events::angle_vectors;
use crate::events::{ATTN_NORM, Action, ClientEvent, Events, PITCH_NORM, SoundCall, handle};
use crate::gauss;
use crate::hud::{ChatLine, Countdown, DeathNotice, Hud};
use crate::hud_messages::{HudMessage, ShownMessage};
use crate::scene::{self, Camera, Entity, Movement, PlayerView, Scene, Viewmodel};
use crate::scores::{Board, ScoreRow};
use crate::source::{Cursor, Group, Source};
use crate::status;
use crate::tempents::{DynamicLight, Lookup, TempEntities, TempEntity};
use crate::time::{Position, ServerTime, SourceTime};
use crate::trace::Hull;
use crate::weapon_menu::MenuShown;

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
    /// Each player's movement worked out from the snapshots, by entity
    /// number, for players other than the recorder
    pub movement: BTreeMap<u16, Movement>,
    pub camera: CameraMode,
    pub viewmodel: Option<Viewmodel>,
    pub light_styles: BTreeMap<u8, Vec<u8>>,
    /// The sky the server names, the map's own when none
    pub sky: Option<Arc<str>>,
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
    /// The dynamic lights messages made that still shine, such as
    /// explosions' flashes. Those of entities' effects are left to whoever
    /// draws them
    pub lights: Vec<DynamicLight>,
    /// The decals on the map, oldest first
    pub decals: Arc<VecDeque<Decal>>,
    /// Grows whenever a decal is made or they are cleared
    pub decal_revision: u64,
    /// Beams, trails and tracers
    pub beams: Beams,
    /// AG's countdown before a match, while it shows
    pub countdown: Option<Countdown>,
    /// The kill feed's lines, oldest first
    pub death_notices: Vec<DeathNotice>,
    /// The chat's lines, oldest first
    pub chat: Vec<ChatLine>,
    /// AG's messages on how the match goes
    pub announced: Announced,
    /// The recording player's health, armour and ammo
    pub status: status::Shown,
    /// What the recording player picked up, while it shows
    pub pickups: Vec<status::Pickup>,
    /// Where the recording player was hit from and what hurts them
    pub damage: DamageShown,
    /// The text the server placed on the HUD
    pub hud_messages: Vec<ShownMessage>,
    /// The game's logo, while it shows
    pub game_title: Option<ShownMessage>,
    /// The weapon menu, while the recording player has it open
    pub weapon_menu: Option<MenuShown>,
    /// The small scoreboard's rows, highest first
    pub scores: Vec<ScoreRow>,
    /// The full scoreboard, shown while its key is held
    pub board: Board,
}

type Lengths = Box<dyn Fn(&SoundName) -> SoundLength + Send>;
type Sequences = Box<dyn Fn(&str) -> Option<Arc<[Sequence]>> + Send>;
type Hulls = Box<dyn Fn(&str) -> Option<Arc<Hull>> + Send>;
type SpriteFrames = Box<dyn Fn(&str) -> Option<u32> + Send>;
type DecalIndex = Box<dyn Fn(&str) -> Option<u16> + Send>;
type Titles = Box<dyn Fn(&str) -> Option<HudMessage> + Send>;

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
    decal_index: DecalIndex,
    titles: Titles,
    /// The map traced through and its world, once read
    hull: Option<(String, Option<Arc<Hull>>)>,
    /// True when the caller no longer wants the seek under way
    interrupt: Box<dyn Fn() -> bool + Send>,
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
            decals: Decals::default(),
            beams: Beams::default(),
            hud: Hud::default(),
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
            decal_index: Box::new(|_| None),
            titles: Box::new(|_| None),
            hull: None,
            interrupt: Box::new(|| false),
        })
    }

    /// Stop [`Playback::advance_to`] and [`Playback::seek_group`] between
    /// groups once `interrupt` says so, leaving playback at the group it
    /// reached, for a seek a newer one replaced
    pub fn set_interrupt(&mut self, interrupt: impl Fn() -> bool + Send + 'static) {
        self.interrupt = Box::new(interrupt);
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
        let decal_index = files.decal_index.clone();
        self.decal_index = Box::new(move |name| decal_index(name));
        let titles = files.titles.clone();
        self.titles = Box::new(move |name| titles(name));
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

    pub fn decals(&self) -> &Decals {
        &self.state.decals
    }

    pub fn events(&self) -> &Events {
        &self.state.events
    }

    pub fn decoder(&self) -> &Decoder {
        &self.state.decoder
    }

    /// Why playback stopped before the end of the demo, none while it can
    /// go on or once it reached the end
    pub fn stopped(&self) -> Option<&Message> {
        self.state.stopped.as_ref()
    }

    /// Everything playback holds between groups as text, for checking that
    /// two ways of reaching a moment hold the same. Of the decoder only
    /// its public state and latest snapshot, as checkpoints keep it whole
    pub fn fingerprint(&self) -> String {
        let s = &self.state;
        let d = &s.decoder;
        let decoder = (
            (d.max_clients, d.player_num, d.hltv, d.time_bits),
            (d.view_entity, &d.movevars, d.segment, &d.stats),
            (&d.players, &d.name_history, d.resources().len()),
            d.latest_snapshot(),
        );
        format!(
            "{:?}",
            (
                decoder,
                &s.scene,
                &s.audio,
                &s.events,
                &s.animations,
                &s.tempents,
                &s.decals,
                &s.beams,
                &s.hud,
                (s.position, s.reach, s.exhausted, &s.stopped),
            )
        )
    }

    /// Each map's world, without the rest of the game's files, for tracing
    /// through the maps alone
    pub fn set_map_hulls(&mut self, hulls: crate::MapHulls) {
        self.hulls = Box::new(move |map| hulls(map));
        self.hull = None;
    }

    /// Apply the next group. False once none is left
    pub fn step(&mut self) -> bool {
        self.apply_next()
    }

    /// The world of the map being played, none before a map is known or
    /// when its world cannot be read
    pub fn map_world(&mut self) -> Option<Arc<Hull>> {
        self.world()
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

        // The map before this group's records, which bullets are traced
        // through
        let world = self.world();
        let time = group.time;
        let lengths: &dyn Fn(&SoundName) -> SoundLength = &*self.lengths;
        let frames: &dyn Fn(&str) -> Option<u32> = &*self.sprite_frames;
        let decal_index: &dyn Fn(&str) -> Option<u16> = &*self.decal_index;
        let titles: &dyn Fn(&str) -> Option<HudMessage> = &*self.titles;
        let State {
            decoder,
            scene,
            audio,
            events,
            tempents,
            decals,
            beams,
            hud,
            stopped,
            ..
        } = &mut self.state;
        let mut effects = Effects {
            audio,
            events,
            tempents,
            decals,
            beams,
            frames,
            lengths,
            world: world.as_deref(),
            decal_index,
        };
        for event in effects.events.due(time) {
            dispatch(&event, decoder, scene, &mut effects);
        }
        for record in &group.records {
            let result = match &record.body {
                FrameBody::Network(frame) => {
                    let audio = &mut *effects.audio;
                    let tempents = &mut *effects.tempents;
                    let decals = &mut *effects.decals;
                    let beams = &mut *effects.beams;
                    let events = &mut *effects.events;
                    let applied = decoder.network_frame(record.location, frame, |d, m| {
                        scene.on_message(d, m, time);
                        audio.on_message(d, m, time, lengths);
                        if let Some(sample) = hud.on_message(d, m, time, &scene.camera, titles) {
                            let call = hud_sound(d, sample, scene.camera.origin);
                            audio.play(&call, time, view_entity(d), lengths);
                        }
                        match &m.message {
                            protocol::Message::ServerInfo(_) => {
                                tempents.clear();
                                decals.clear();
                                beams.clear();
                                events.clear();
                            }
                            protocol::Message::TempEntity { kind, body } => {
                                let model = |index: i16| precached_model(d, index);
                                let texture = |number: u16| server_decal(d, decal_index, number);
                                decals.message(*kind, body, scene, &model, &texture);
                                beams.message(*kind, body, time, scene, &model);
                                let lookup = Lookup {
                                    model: &model,
                                    frames,
                                    world: world.as_deref(),
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
                FrameBody::ConsoleCommand(command) => {
                    let end = command
                        .iter()
                        .position(|&b| b == 0)
                        .unwrap_or(command.len());
                    let command = String::from_utf8_lossy(&command[..end]);

                    if let Some(sample) = hud.on_command(command.trim(), &scene.camera) {
                        let call = hud_sound(decoder, sample, scene.camera.origin);
                        effects
                            .audio
                            .play(&call, time, view_entity(decoder), lengths);
                    }
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
        self.state.beams.step(time, &self.state.scene);
        let world = self.world();
        if self.state.beams.egon() {
            let end = egon_end(&self.state.scene, &self.state.decoder, world.as_deref());
            let eye = self.state.scene.camera.origin;
            self.state.beams.egon_aim(eye, end, time);
        }
        let gravity = motion::gravity(&self.state.decoder);
        let bounces = self.state.tempents.step(time, world.as_deref(), gravity);
        for call in bounces {
            self.state.audio.play(&call, time, view, &*self.lengths);
        }
        self.state.position = Some(position);
        self.state.reach = SourceTime(self.state.reach.0.max(time.0));

        if self.store().due(position) {
            let checkpoint = self.checkpoint(position);
            self.store().insert(checkpoint);
        }
        true
    }

    /// The state after `position`, the group just applied, to go on from
    fn checkpoint(&self, position: Position) -> Checkpoint {
        let entities = self.state.scene.entities.len() * size_of::<Entity>();
        let sounds = size_of_val(self.state.audio.channels());
        // The effects alive, the decals shared with the checkpoint before
        // until one is made
        let beams = &self.state.beams;
        let effects = size_of_val(self.state.tempents.entities())
            + size_of_val(beams.beams.as_slice())
            + size_of_val(beams.tracers.as_slice())
            + beams
                .trails
                .iter()
                .map(|t| size_of::<Trail>() + t.points.len() * size_of::<([f32; 3], SourceTime)>())
                .sum::<usize>();
        let bytes = self.state.decoder.retained_bytes() + 2 * entities + sounds + effects;
        Checkpoint {
            group: position.group,
            segment: position.segment,
            time: position.time,
            reach: self.state.reach,
            bytes: bytes as u64,
            state: self.state.clone(),
            cursor: self.source.cursor(),
        }
    }

    /// Seek to `group` and keep the state after it as a checkpoint,
    /// whatever the spacing, for a place to come back to. False when the
    /// demo ends first
    pub fn keep_checkpoint_at(&mut self, group: u64) -> Result<bool, Message> {
        self.seek_group(group)?;
        // A seek stops once `group` is applied, before reading the next
        match self.state.position {
            Some(position) if position.group == group && self.next.is_none() => {
                let checkpoint = self.checkpoint(position);
                self.store().insert(checkpoint);
                Ok(true)
            }
            _ => Ok(false),
        }
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
            if (self.interrupt)() {
                break;
            }
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
            if (self.interrupt)() || !self.apply_next() {
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
        let world = self.world();
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

        let own = u16::from(state.decoder.player_num) + 1;
        let entities = match &world {
            Some(world) => scene::in_the_open(state.scene.presented(client_time), world, own),
            None => state.scene.presented(client_time),
        };
        let mut lit = false;
        for e in &entities {
            if e.id.number == own && e.effects & status::FLASHLIGHT_EFFECT != 0 {
                lit = true;
            }
        }

        PresentedFrame {
            demo: self.path.clone(),
            stamp,
            position: state.position,
            at,
            server_time: ServerTime(f32::from_bits(state.decoder.time_bits)),
            map: state.scene.map.clone(),
            scene_revision: state.scene.revision,
            entities: Arc::new(entities),
            client_time,
            recorder: state.scene.camera.clone(),
            movement: state.scene.movement.clone(),
            camera,
            viewmodel: state.scene.viewmodel,
            light_styles: state.scene.light_styles.clone(),
            sky: state.scene.sky.clone(),
            sounds,
            audio_revision: state.audio.revision,
            audible,
            end,
            stopped: state.stopped.clone(),
            indexed,
            players: scene::players(&state.decoder),
            local_entity: own,
            flashes: state
                .animations
                .flashes()
                .iter()
                .filter(|f| f.time <= at)
                .copied()
                .collect(),
            decals: state.decals.list().clone(),
            decal_revision: state.decals.revision,
            beams: state.beams.clone(),
            countdown: state.hud.countdown(at).cloned(),
            death_notices: state.hud.death_notices(at),
            chat: state.hud.chat(at),
            announced: state.hud.announced(at),
            status: state.hud.status(&state.scene.camera, lit),
            pickups: state.hud.pickups(at),
            damage: state.hud.damage(at),
            hud_messages: state.hud.messages(at),
            game_title: state.hud.game_title_at(at),
            weapon_menu: state.hud.weapon_menu(&state.scene.camera),
            scores: state.hud.scores(&state.decoder),
            board: state.hud.board(&state.decoder),
            temp_entities: state
                .tempents
                .entities()
                .iter()
                .filter(|e| e.visible())
                .cloned()
                .collect(),
            lights: state.tempents.lights(at),
        }
    }
}

/// A sound the HUD plays, such as the countdown's numbers or the weapon
/// menu's clicks, which the client plays where the listener at `origin`
/// is, at full volume (`pfnPlaySoundByName`)
fn hud_sound(decoder: &Decoder, sample: &'static str, origin: [f32; 3]) -> SoundCall {
    SoundCall {
        entity: view_entity(decoder),
        channel: CHAN_AUTO,
        sample: Arc::from(sample),
        volume: 1.,
        attenuation: 0.,
        flags: 0,
        pitch: PITCH_NORM,
        origin,
    }
}

/// Where the Egon's beams end, 2048 units along the recorder's view or at
/// what they meet first
fn egon_end(scene: &Scene, decoder: &Decoder, world: Option<&Hull>) -> [f32; 3] {
    let eye = scene.camera.origin;
    let [forward, ..] = angle_vectors(scene.camera.angles);
    let end = [0, 1, 2].map(|k| eye[k] + forward[k] * 2048.);
    let shooter = view_entity(decoder);
    bullets::trace(eye, end, &[shooter], decoder.max_clients, scene, world)
        .map_or(end, |(_, at, _)| at)
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

/// The client's place in `decals.wad` of the decal the server numbers
/// `number`. The session's resource list names the server's numbers, and the
/// client finds each name in its own files (`Draw_DecalSetName`). A session
/// naming none keeps the server's numbering
fn server_decal(
    decoder: &Decoder,
    decal_index: &dyn Fn(&str) -> Option<u16>,
    number: u16,
) -> Option<u16> {
    let named = decoder
        .resources()
        .iter()
        .find(|r| r.kind == resource_type::DECAL && r.index == number);
    match named {
        Some(resource) => decal_index(&String::from_utf8_lossy(&resource.name)),
        None => Some(number),
    }
}

/// What an event's handler changes besides the scene
struct Effects<'a> {
    audio: &'a mut Audio,
    events: &'a mut Events,
    tempents: &'a mut TempEntities,
    decals: &'a mut Decals,
    beams: &'a mut Beams,
    /// A sprite's frame count by its path
    frames: &'a dyn Fn(&str) -> Option<u32>,
    lengths: &'a dyn Fn(&SoundName) -> SoundLength,
    /// The map to trace bullets through
    world: Option<&'a Hull>,
    /// A decal's place in `decals.wad` by its name
    decal_index: &'a dyn Fn(&str) -> Option<u16>,
}

/// Where an event that gives no origin or angles happens, as the client
/// takes them from the event's entity, a player's pitch turned back from
/// the model's third of it to the view's. The recorder's own come from its
/// view, as a singleplayer recording can leave the player's entity behind
/// where it spawned
fn fill_place(event: &mut ClientEvent, decoder: &Decoder, scene: &Scene) {
    let args = &mut event.args;
    let (origin, angles) = if event.entity == view_entity(decoder) {
        let eye = scene.camera.origin;
        // The eye, less the view height the handlers add back
        ([eye[0], eye[1], eye[2] - 28.], scene.camera.angles)
    } else if let Some(e) = scene.entities.iter().find(|e| e.id.number == event.entity) {
        let mut angles = e.angles;
        if (1..=u16::from(decoder.max_clients)).contains(&e.id.number) {
            angles[0] *= -3.;
        }
        (e.origin, angles)
    } else {
        return;
    };
    if args.origin == [0.; 3] {
        args.origin = origin;
    }
    if args.angles == [0.; 3] {
        args.angles = angles;
    }
}

/// Run the client's handler for `event`. An event that gives no origin
/// sounds where its entity is
fn dispatch(event: &ClientEvent, decoder: &Decoder, scene: &mut Scene, effects: &mut Effects) {
    let Effects {
        audio,
        events,
        tempents,
        decals,
        beams,
        frames,
        lengths,
        world,
        decal_index,
    } = effects;
    let mut event = event.clone();
    fill_place(&mut event, decoder, scene);
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
            Action::Bullets(volley) => {
                let tracers = events.tracers(event.entity);
                let max = decoder.max_clients;
                let impacts = bullets::fire(&volley, event.entity, max, tracers, scene, *world);
                for call in impacts.sounds {
                    audio.play(&call, event.time, view, lengths);
                }
                for (name, entity, at) in impacts.decals {
                    if let Some(texture) = decal_index(name) {
                        decals.shoot(texture, entity, at, None, false, scene);
                    }
                }
                for (start, end, seed) in impacts.tracers {
                    beams.tracer(start, end, event.time, seed);
                }
            }
            Action::Gauss(shot) => {
                let max = decoder.max_clients;
                let out = gauss::fire(&shot, event.entity, max, scene, *world);
                for call in out.impacts.sounds {
                    audio.play(&call, event.time, view, lengths);
                }
                for (name, entity, at) in out.impacts.decals {
                    if let Some(texture) = decal_index(name) {
                        decals.shoot(texture, entity, at, None, false, scene);
                    }
                }
                for b in out.beams {
                    let gun = b.from_gun.then_some(event.entity);
                    let (sprite, life) = (gauss::BEAM_SPRITE, 0.1);
                    beams.line(
                        sprite, b.start, gun, b.end, b.width, b.color, life, event.time,
                    );
                }
                let glow = gauss::GLOW_SPRITE;
                let glow_frames = frames(glow).unwrap_or(1);
                for g in out.glows {
                    let (at, scale) = (g.at, g.scale);
                    tempents.glow(
                        glow,
                        glow_frames,
                        at,
                        scale,
                        g.brightness,
                        g.life,
                        event.time,
                    );
                }
                for b in out.balls {
                    tempents.sprite_trail(
                        glow,
                        glow_frames,
                        b.start,
                        b.towards,
                        b.count,
                        b.life,
                        b.size,
                        b.spread,
                        b.brightness,
                        b.speed,
                        event.time,
                        b.seed,
                    );
                }
            }
            Action::EgonStart => {
                let end = egon_end(scene, decoder, *world);
                beams.egon_start(event.entity, scene.camera.origin, end, event.time);
            }
            Action::EgonStop => beams.egon_stop(),
        }
    }
}
