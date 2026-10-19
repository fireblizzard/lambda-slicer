//! Which sounds are playing, as logical channels a mixer can follow
//!
//! The rules are the engine's, as GoldSrc keeps them from Quake's sound
//! code, and are not yet compared with the game (docs/viewer-profile.md,
//! "Sound state"). A sound starting on an entity's channel replaces what
//! that channel played, except on the automatic channel, which never
//! replaces, and the static channel, which keeps ambient sounds of its
//! own. A stop or a volume or pitch change applies to the sound playing on
//! that entity and channel with the same sample, and a change that finds
//! nothing starts the sound instead. `svc_stopsound` stops the channel
//! whatever it plays. A new session silences everything.
//!
//! A sound that is not static takes one of the engine's eight dynamic
//! channels. When all are busy it takes the one with the least time left,
//! except that another entity's sound never takes one of the listener's
//! own, and when nothing can be taken the sound is not heard
//! (`SND_PickChannel`).
//!
//! How long a sample lasts, and whether it loops, is in the sample file, so
//! the length comes from the caller

use crate::Table;
use std::sync::Arc;

use lambda_slicer_core::container::{FrameBody, SoundRecord};
use lambda_slicer_core::protocol::messages::{Message, ParsedMessage, SoundMessage, resource_type};
use lambda_slicer_core::state::Decoder;

use crate::events::SoundCall;
use crate::time::SourceTime;

/// `svc_sound` and sound record flags
pub mod flags {
    pub const VOLUME: u16 = 1;
    pub const ATTENUATION: u16 = 2;
    pub const LARGE_INDEX: u16 = 4;
    pub const PITCH: u16 = 8;
    pub const SENTENCE: u16 = 16;
    pub const STOP: u16 = 32;
    pub const CHANGE_VOLUME: u16 = 64;
    pub const CHANGE_PITCH: u16 = 128;
}

/// The engine's dynamic channels
pub const DYNAMIC_CHANNELS: usize = 8;

pub const CHAN_AUTO: u8 = 0;
pub const CHAN_STATIC: u8 = 6;

/// The engine's defaults for fields a sound message leaves out
const DEFAULT_VOLUME: f32 = 1.;
const DEFAULT_ATTENUATION: f32 = 1.;
const DEFAULT_PITCH: u8 = 100;

/// A sound whose length is unknown counts as finished this long after it
/// started, until sample lengths are read (M10)
pub const UNKNOWN_LIMIT: f64 = 10.;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SoundName {
    /// A precached sample, as the resource list or the record names it,
    /// relative to `sound/`
    File(Arc<str>),
    /// A sentence by its number in `sentences.txt`, as network sounds give it
    Sentence(u16),
    /// A sentence by name, as sound records give it, `!` included
    SentenceName(Arc<str>),
    /// A precache index the session's resource list does not hold
    Unlisted(u16),
}

impl SoundName {
    fn is_sentence(&self) -> bool {
        matches!(self, SoundName::Sentence(_) | SoundName::SentenceName(_))
    }

    /// Whether a stop or change for `other` applies to this sound. Any
    /// sentence matches any other, as the engine compares only that much
    fn matches(&self, other: &SoundName) -> bool {
        self == other || (self.is_sentence() && other.is_sentence())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundSource {
    /// `svc_sound`
    Network,
    /// `svc_spawnstaticsound`
    Static,
    /// A demo sound record, a sound the recording client started itself
    Recorded,
    /// A sound the game's client starts for an event
    Client,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    /// Unique within one playback, in starting order
    pub id: u64,
    pub source: SoundSource,
    pub entity: u16,
    pub channel: u8,
    pub sound: SoundName,
    pub volume: f32,
    pub attenuation: f32,
    pub origin: [f32; 3],
    pub started: SourceTime,
    /// Each pitch and the time it took effect, the first at `started`
    pub pitches: Vec<(SourceTime, u8)>,
}

impl Channel {
    /// Whether it plays on one of the engine's static channels, which
    /// dynamic sounds never replace or stop
    pub fn is_static(&self) -> bool {
        self.source == SoundSource::Static || self.channel == CHAN_STATIC
    }

    pub fn pitch_at(&self, at: SourceTime) -> u8 {
        self.pitches
            .iter()
            .rev()
            .find(|(from, _)| *from <= at)
            .map_or(DEFAULT_PITCH, |(_, pitch)| *pitch)
    }

    /// Seconds of the sample played by `at`, at the sample's own rate. Each
    /// stretch counts at its own pitch, so a change does not rescale what
    /// was already played
    pub fn cursor(&self, at: SourceTime) -> f64 {
        let mut played = 0.;
        for (i, (from, pitch)) in self.pitches.iter().enumerate() {
            let until = self
                .pitches
                .get(i + 1)
                .map_or(at.0, |(next, _)| next.0.min(at.0));
            if until <= from.0 {
                break;
            }
            played += (until - from.0) * f64::from(*pitch) / 100.;
        }
        played
    }
}

/// Where playback learns each sample's length, shared by every playback of
/// a demo so their checkpoints agree
pub type SoundLengths = Arc<dyn Fn(&SoundName) -> SoundLength + Send + Sync>;

/// How long a sample plays, which the sample file says
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundLength {
    /// Seconds at pitch 100
    Once(f64),
    Looping,
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct Audio {
    channels: Vec<Channel>,
    next_id: u64,
    /// Grows whenever a channel starts, stops or changes
    pub revision: u64,
    /// Sample names by precache index, and the resource count they were
    /// read from
    sounds: Arc<Table<u16, Arc<str>>>,
    sounds_from: usize,
}

/// One sound to start, stop or change, from any of its sources
struct Request {
    source: SoundSource,
    flags: u16,
    entity: u16,
    channel: u8,
    sound: SoundName,
    volume: f32,
    attenuation: f32,
    origin: [f32; 3],
    pitch: u8,
}

fn coord(bytes: &[u8], at: usize) -> f32 {
    f32::from(i16::from_le_bytes([bytes[at], bytes[at + 1]])) / 8.
}

fn short(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

impl Audio {
    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    fn sample(&mut self, decoder: &Decoder, index: u16) -> SoundName {
        let resources = decoder.resources();
        if resources.len() != self.sounds_from {
            self.sounds = resources
                .iter()
                .filter(|r| r.kind == resource_type::SOUND)
                .map(|r| {
                    (
                        r.index,
                        Arc::from(String::from_utf8_lossy(&r.name).as_ref()),
                    )
                })
                .collect::<Table<_, _>>()
                .into();
            self.sounds_from = resources.len();
        }
        match self.sounds.get(&index) {
            Some(name) => SoundName::File(name.clone()),
            None => SoundName::Unlisted(index),
        }
    }

    /// A message the decoder has just applied, at `time`
    pub(crate) fn on_message(
        &mut self,
        decoder: &Decoder,
        m: &ParsedMessage,
        time: SourceTime,
        lengths: &dyn Fn(&SoundName) -> SoundLength,
    ) {
        let view = view_entity(decoder);
        match &m.message {
            Message::ServerInfo(_) => {
                self.channels.clear();
                self.revision += 1;
                // The new session's sounds are read again, even when it
                // has as many resources as the last
                self.sounds = Arc::default();
                self.sounds_from = usize::MAX;
            }
            Message::Sound(sound) => {
                let request = self.network(decoder, sound);
                self.apply(request, time, view, lengths);
            }
            Message::StopSound(value) => {
                let (entity, channel) = (value >> 3, (value & 7) as u8);
                let before = self.channels.len();
                self.channels
                    .retain(|c| c.is_static() || (c.entity, c.channel) != (entity, channel));
                if self.channels.len() != before {
                    self.revision += 1;
                }
            }
            Message::SpawnStaticSound(body) if body.len() == 14 => {
                let request = Request {
                    source: SoundSource::Static,
                    flags: u16::from(body[13]),
                    entity: short(body, 10),
                    channel: CHAN_STATIC,
                    sound: self.sample(decoder, short(body, 6)),
                    volume: f32::from(body[8]) / 255.,
                    attenuation: f32::from(body[9]) / 64.,
                    origin: [coord(body, 0), coord(body, 2), coord(body, 4)],
                    pitch: body[12],
                };
                self.apply(request, time, view, lengths);
            }
            _ => {}
        }
    }

    fn network(&mut self, decoder: &Decoder, m: &SoundMessage) -> Request {
        let sound = if m.flags & flags::SENTENCE != 0 {
            SoundName::Sentence(m.sound_index)
        } else {
            self.sample(decoder, m.sound_index)
        };
        Request {
            source: SoundSource::Network,
            flags: m.flags,
            entity: m.entity,
            channel: m.channel,
            sound,
            volume: m.volume.map_or(DEFAULT_VOLUME, |v| f32::from(v) / 255.),
            attenuation: m
                .attenuation
                .map_or(DEFAULT_ATTENUATION, |a| f32::from(a) / 64.),
            origin: m.origin.map(|c| c.map_or(0., |c| c.value())),
            pitch: m.pitch.unwrap_or(DEFAULT_PITCH),
        }
    }

    /// A record other than a network one. A sound record carries no entity,
    /// so it plays on the entity the client views from
    pub(crate) fn on_record(
        &mut self,
        decoder: &Decoder,
        body: &FrameBody,
        time: SourceTime,
        lengths: &dyn Fn(&SoundName) -> SoundLength,
    ) {
        let view = view_entity(decoder);
        let FrameBody::Sound(record) = body else {
            return;
        };
        let request = recorded(decoder, record);
        self.apply(request, time, view, lengths);
    }

    /// A sound an event's handler starts, changes or stops at `time`
    pub(crate) fn play(
        &mut self,
        call: &SoundCall,
        time: SourceTime,
        view: u16,
        lengths: &dyn Fn(&SoundName) -> SoundLength,
    ) {
        let request = Request {
            source: SoundSource::Client,
            flags: call.flags,
            entity: call.entity,
            channel: call.channel,
            sound: SoundName::File(call.sample.clone()),
            volume: call.volume,
            attenuation: call.attenuation,
            origin: call.origin,
            pitch: call.pitch,
        };
        self.apply(request, time, view, lengths);
    }

    fn apply(
        &mut self,
        r: Request,
        time: SourceTime,
        view: u16,
        lengths: &dyn Fn(&SoundName) -> SoundLength,
    ) {
        let found = self.channels.iter().position(|c| {
            c.entity == r.entity && c.channel == r.channel && c.sound.matches(&r.sound)
        });

        if r.flags & (flags::STOP | flags::CHANGE_VOLUME | flags::CHANGE_PITCH) != 0
            && let Some(i) = found
        {
            if r.flags & flags::STOP != 0 {
                self.channels.remove(i);
            } else {
                let c = &mut self.channels[i];
                if r.flags & flags::CHANGE_VOLUME != 0 {
                    c.volume = r.volume;
                }
                if r.flags & flags::CHANGE_PITCH != 0 && c.pitch_at(time) != r.pitch {
                    c.pitches.push((time, r.pitch));
                }
            }
            self.revision += 1;
            return;
        }
        if r.flags & flags::STOP != 0 {
            return;
        }

        if r.channel != CHAN_AUTO && r.channel != CHAN_STATIC {
            self.channels
                .retain(|c| c.is_static() || (c.entity, c.channel) != (r.entity, r.channel));
        }

        let is_static = r.source == SoundSource::Static || r.channel == CHAN_STATIC;
        let dynamic = self.channels.iter().filter(|c| !c.is_static()).count();
        if !is_static && dynamic >= DYNAMIC_CHANNELS {
            let left = |c: &Channel| match lengths(&c.sound) {
                SoundLength::Once(seconds) => seconds - c.cursor(time),
                SoundLength::Looping => f64::INFINITY,
                SoundLength::Unknown => UNKNOWN_LIMIT - (time.0 - c.started.0),
            };
            let taken = self
                .channels
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.is_static() && !(c.entity == view && r.entity != view))
                .min_by(|(_, a), (_, b)| left(a).total_cmp(&left(b)))
                .map(|(i, _)| i);
            match taken {
                Some(i) => {
                    self.channels.remove(i);
                }
                None => return,
            }
        }
        self.channels.push(Channel {
            id: self.next_id,
            source: r.source,
            entity: r.entity,
            channel: r.channel,
            sound: r.sound,
            volume: r.volume,
            attenuation: r.attenuation,
            origin: r.origin,
            started: time,
            pitches: vec![(time, r.pitch)],
        });
        self.next_id += 1;
        self.revision += 1;
    }

    /// Drop the sounds that have finished by `at`. A sound of unknown length
    /// lasts [`UNKNOWN_LIMIT`] seconds
    pub fn retire(&mut self, at: SourceTime, length: &dyn Fn(&SoundName) -> SoundLength) {
        let before = self.channels.len();
        self.channels.retain(|c| match length(&c.sound) {
            SoundLength::Once(seconds) => c.cursor(at) < seconds,
            SoundLength::Looping => true,
            SoundLength::Unknown => at.0 - c.started.0 < UNKNOWN_LIMIT,
        });
        if self.channels.len() != before {
            self.revision += 1;
        }
    }
}

/// The entity the recording client views from, whose sounds are its own
pub(crate) fn view_entity(decoder: &Decoder) -> u16 {
    decoder
        .view_entity
        .unwrap_or(u16::from(decoder.player_num) + 1)
}

fn recorded(decoder: &Decoder, record: &SoundRecord) -> Request {
    let name = String::from_utf8_lossy(&record.sample).into_owned();
    let sound = if name.starts_with('!') {
        SoundName::SentenceName(Arc::from(name.as_str()))
    } else {
        SoundName::File(Arc::from(name.as_str()))
    };
    let entity = decoder
        .view_entity
        .unwrap_or(u16::from(decoder.player_num) + 1);
    Request {
        source: SoundSource::Recorded,
        flags: record.flags as u16,
        entity,
        channel: record.channel.clamp(0, 7) as u8,
        sound,
        volume: f32::from_bits(record.volume_bits),
        attenuation: f32::from_bits(record.attenuation_bits),
        origin: [0.; 3],
        pitch: record.pitch.clamp(0, 255) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(flags: u16, entity: u16, channel: u8, name: &str, pitch: u8) -> Request {
        Request {
            source: SoundSource::Network,
            flags,
            entity,
            channel,
            sound: SoundName::File(Arc::from(name)),
            volume: 1.,
            attenuation: 1.,
            origin: [0.; 3],
            pitch,
        }
    }

    fn names(audio: &Audio) -> Vec<(u16, u8, String)> {
        audio
            .channels()
            .iter()
            .map(|c| match &c.sound {
                SoundName::File(n) => (c.entity, c.channel, n.to_string()),
                other => (c.entity, c.channel, format!("{other:?}")),
            })
            .collect()
    }

    #[test]
    fn channels_replace_stop_and_change_as_the_engine_does() {
        let mut audio = Audio::default();
        let t = |s: f64| SourceTime(s);

        audio.apply(
            request(0, 1, 1, "weapons/gauss2.wav", 100),
            t(0.),
            1,
            &|_| SoundLength::Unknown,
        );
        // The same channel replaces it, the automatic channel adds
        audio.apply(
            request(0, 1, 1, "weapons/electro4.wav", 100),
            t(0.1),
            1,
            &|_| SoundLength::Unknown,
        );
        audio.apply(
            request(0, 1, CHAN_AUTO, "player/pl_step1.wav", 100),
            t(0.2),
            1,
            &|_| SoundLength::Unknown,
        );
        audio.apply(
            request(0, 1, CHAN_AUTO, "player/pl_step2.wav", 100),
            t(0.3),
            1,
            &|_| SoundLength::Unknown,
        );
        assert_eq!(
            names(&audio),
            [
                (1, 1, "weapons/electro4.wav".into()),
                (1, 0, "player/pl_step1.wav".into()),
                (1, 0, "player/pl_step2.wav".into()),
            ]
        );

        // A stop names the sample, so another sample on the channel plays on
        audio.apply(
            request(flags::STOP, 1, 1, "weapons/gauss2.wav", 100),
            t(0.4),
            1,
            &|_| SoundLength::Unknown,
        );
        assert_eq!(audio.channels().len(), 3);
        audio.apply(
            request(flags::STOP, 1, 1, "weapons/electro4.wav", 100),
            t(0.4),
            1,
            &|_| SoundLength::Unknown,
        );
        assert_eq!(audio.channels().len(), 2);

        // A change that finds nothing starts the sound, one that finds it
        // changes the pitch from then on
        audio.apply(
            request(flags::CHANGE_PITCH, 2, 2, "ambience/hum.wav", 100),
            t(1.),
            1,
            &|_| SoundLength::Unknown,
        );
        assert_eq!(audio.channels().len(), 3);
        audio.apply(
            request(flags::CHANGE_PITCH, 2, 2, "ambience/hum.wav", 200),
            t(2.),
            1,
            &|_| SoundLength::Unknown,
        );
        let hum = audio.channels().last().unwrap();
        assert_eq!(hum.pitch_at(t(1.5)), 100);
        assert_eq!(hum.pitch_at(t(2.5)), 200);
        assert_eq!(hum.cursor(t(3.)), 1. + 2.);
        assert_eq!(hum.cursor(t(0.5)), 0.);
    }

    #[test]
    fn finished_sounds_are_retired_by_their_length() {
        let mut audio = Audio::default();
        audio.apply(
            request(0, 1, 1, "once.wav", 200),
            SourceTime(0.),
            1,
            &|_| SoundLength::Unknown,
        );
        audio.apply(
            request(0, 1, 2, "loop.wav", 100),
            SourceTime(0.),
            1,
            &|_| SoundLength::Unknown,
        );
        audio.apply(
            request(0, 1, 3, "unknown.wav", 100),
            SourceTime(0.),
            1,
            &|_| SoundLength::Unknown,
        );
        let length = |s: &SoundName| match s {
            SoundName::File(n) if &**n == "once.wav" => SoundLength::Once(1.),
            SoundName::File(n) if &**n == "loop.wav" => SoundLength::Looping,
            _ => SoundLength::Unknown,
        };

        // A one-second sample at pitch 200 is done after half a second
        audio.retire(SourceTime(0.4), &length);
        assert_eq!(audio.channels().len(), 3);
        audio.retire(SourceTime(0.5), &length);
        assert_eq!(audio.channels().len(), 2);
        audio.retire(SourceTime(UNKNOWN_LIMIT), &length);
        assert_eq!(names(&audio), [(1, 2, "loop.wav".into())]);
    }

    #[test]
    fn a_ninth_dynamic_sound_takes_the_channel_closest_to_its_end() {
        let mut audio = Audio::default();
        let length = |s: &SoundName| match s {
            SoundName::File(n) if n.starts_with("long") => SoundLength::Once(10.),
            SoundName::File(n) if n.starts_with("loop") => SoundLength::Looping,
            _ => SoundLength::Once(1.),
        };
        // Eight automatic sounds on entity 2, the shortest started first
        audio.apply(
            request(0, 2, CHAN_AUTO, "short.wav", 100),
            SourceTime(0.),
            1,
            &length,
        );
        for i in 0..6 {
            audio.apply(
                request(0, 2, CHAN_AUTO, &format!("long{i}.wav"), 100),
                SourceTime(0.),
                1,
                &length,
            );
        }
        audio.apply(
            request(0, 1, CHAN_AUTO, "loop.wav", 100),
            SourceTime(0.),
            1,
            &length,
        );
        // A static sound takes no dynamic channel
        audio.apply(
            request(0, 3, CHAN_STATIC, "ambient.wav", 100),
            SourceTime(0.),
            1,
            &length,
        );
        assert_eq!(audio.channels().len(), 9);

        audio.apply(
            request(0, 2, CHAN_AUTO, "new.wav", 100),
            SourceTime(0.5),
            1,
            &length,
        );
        let playing = names(&audio);
        assert!(!playing.iter().any(|(_, _, n)| n == "short.wav"));
        assert!(playing.iter().any(|(_, _, n)| n == "new.wav"));

        // With every other channel the listener's own, another entity's sound
        // is not heard
        let mut own = Audio::default();
        for i in 0..8 {
            own.apply(
                request(0, 1, CHAN_AUTO, &format!("s{i}.wav"), 100),
                SourceTime(0.),
                1,
                &length,
            );
        }
        own.apply(
            request(0, 2, CHAN_AUTO, "other.wav", 100),
            SourceTime(0.),
            1,
            &length,
        );
        assert!(!names(&own).iter().any(|(_, _, n)| n == "other.wav"));
        own.apply(
            request(0, 1, CHAN_AUTO, "mine.wav", 100),
            SourceTime(0.),
            1,
            &length,
        );
        assert!(names(&own).iter().any(|(_, _, n)| n == "mine.wav"));
    }
}
