//! Stereo samples from the channels playing, as the game's sound code mixes
//! them
//!
//! Where a channel is in its sample at a moment comes from playback, which
//! integrates its pitch over time, so the mix does not drift however it is
//! cut into blocks. A sound that loops wraps back to its loop's start, and
//! one that does not falls silent at its end. Samples are read between
//! stored ones linearly, at any rate.
//!
//! Each channel is placed as `S_Spatialize` places it. A sound from the
//! entity the listener views from plays at its full volume on both sides.
//! Any other fades with distance times its attenuation over the game's
//! nominal 1000 units, and moves between the ears by how far it lies to the
//! listener's right, so a sound straight to one side is twice as loud there
//! and silent on the other

use std::sync::Arc;

use lambda_slicer_assets::wav::Sound;
use lambda_slicer_playback::audio::{Channel, SoundName};
use lambda_slicer_playback::playback::Playback;
use lambda_slicer_playback::scene::Scene;
use lambda_slicer_playback::time::SourceTime;

/// The game's `sound_nominal_clip_dist`
const NOMINAL_DISTANCE: f32 = 1000.;

/// Where the sounds are heard from
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Listener {
    pub origin: [f32; 3],
    /// The listener's right, a unit vector
    pub right: [f32; 3],
    /// The entity it views from, whose sounds are not placed
    pub entity: u16,
}

/// What a listener away from every entity names as its own, so every sound
/// is placed
const NO_ENTITY: u16 = u16::MAX;

impl Listener {
    /// Where `playback`'s recorder hears from
    pub fn of(playback: &Playback) -> Self {
        let camera = &playback.scene().camera;
        Self {
            origin: camera.origin,
            right: right_of(camera.angles),
            entity: camera
                .view_entity
                .unwrap_or(u16::from(playback.decoder().player_num) + 1),
        }
    }

    /// Hearing from `origin` looking along `angles` instead, such as a free
    /// camera, where the recorder's own sounds are placed as everyone's are
    pub fn heard_from(origin: [f32; 3], angles: [f32; 3]) -> Self {
        Self {
            origin,
            right: right_of(angles),
            entity: NO_ENTITY,
        }
    }
}

/// The game's right for view angles in degrees, as `AngleVectors` turns
/// them
pub fn right_of(angles: [f32; 3]) -> [f32; 3] {
    let [p, y, r] = angles.map(f32::to_radians);
    let (sp, cp) = p.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sr, cr) = r.sin_cos();
    [-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp]
}

/// The channels with each sound moved to where its entity is now, as the
/// engine moves a dynamic sound with its entity while the latest packet
/// holds the entity (`SND_Spatialize`). Static sounds stay where they
/// started, and so do the world's and brush entities', which the engine
/// places at the brush's centre. The server leaves the recorder's own
/// entity, `own`, at nought, so its sounds go where the recorder's frames
/// say they are
pub fn follow(channels: &[Channel], scene: &Scene, own: u16) -> Vec<Channel> {
    channels
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if c.is_static() || c.entity == 0 {
                return c;
            }
            if c.entity == own {
                c.origin = scene.camera.player_origin;
                return c;
            }
            let found = scene
                .entities
                .binary_search_by_key(&c.entity, |e| e.id.number)
                .ok()
                .map(|i| &scene.entities[i]);
            if let Some(e) = found
                && !e.model.as_deref().is_some_and(|m| m.starts_with('*'))
            {
                c.origin = e.origin;
            }
            c
        })
        .collect()
}

/// A channel's left and right gains for this listener
pub fn spatialize(channel: &Channel, listener: &Listener) -> [f32; 2] {
    if channel.entity == listener.entity || channel.attenuation <= 0. {
        return [channel.volume; 2];
    }
    let to = [0, 1, 2].map(|k| channel.origin[k] - listener.origin[k]);
    let length = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
    let distance = length * channel.attenuation / NOMINAL_DISTANCE;
    let dot = if length > 0. {
        (to[0] * listener.right[0] + to[1] * listener.right[1] + to[2] * listener.right[2]) / length
    } else {
        0.
    };
    let near = 1. - distance;
    [
        (channel.volume * near * (1. - dot)).max(0.),
        (channel.volume * near * (1. + dot)).max(0.),
    ]
}

/// The sample frame `position` frames into `sound`, wrapped into its loop,
/// none past the end of one that does not loop
fn frame_at(sound: &Sound, position: f64) -> Option<f64> {
    let end = f64::from(sound.frames);
    if position < 0. || end <= 0. {
        return None;
    }
    if position < end {
        return Some(position);
    }
    let start = f64::from(sound.loop_start?);
    let span = end - start;
    if span <= 0. {
        return None;
    }
    Some(start + (position - start) % span)
}

/// The sound's value at a fractional frame, both channels averaged, read
/// between stored frames linearly
fn value_at(sound: &Sound, frame: f64) -> f32 {
    let channels = usize::from(sound.channels.max(1));
    let stored = sound.samples.len() / channels;
    let read = |i: usize| -> f32 {
        let i = i.min(stored.saturating_sub(1));
        let sum: i32 = (0..channels)
            .map(|c| i32::from(sound.samples[i * channels + c]))
            .sum();
        sum as f32 / channels as f32 / 32768.
    };
    if stored == 0 {
        return 0.;
    }
    let i = frame.floor() as usize;
    let f = (frame - frame.floor()) as f32;
    let next = if i + 1 < sound.frames as usize {
        i + 1
    } else {
        sound.loop_start.map_or(i, |s| s as usize)
    };
    read(i) * (1. - f) + read(next) * f
}

/// How samples are made
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Output {
    /// Samples a second
    pub rate: u32,
    /// Source seconds passing for each second of samples
    pub speed: f64,
    /// The master volume every sample is scaled by, as the game's `volume`
    pub volume: f32,
}

/// `frames` stereo samples from source time `from`. `sound` gives each
/// channel's samples, none for one that has none
pub fn mix(
    channels: &[Channel],
    sound: &dyn Fn(&SoundName) -> Option<Arc<Sound>>,
    listener: &Listener,
    from: SourceTime,
    frames: usize,
    output: &Output,
) -> Vec<[f32; 2]> {
    let mut out = vec![[0.; 2]; frames];
    let seconds_per_frame = output.speed / f64::from(output.rate.max(1));
    for channel in channels {
        let Some(samples) = sound(&channel.sound) else {
            continue;
        };
        let gains = spatialize(channel, listener);
        if gains == [0.; 2] {
            continue;
        }

        for (k, out) in out.iter_mut().enumerate() {
            let at = SourceTime(from.0 + k as f64 * seconds_per_frame);
            if at < channel.started {
                continue;
            }
            let Some(frame) = frame_at(&samples, channel.cursor(at) * f64::from(samples.rate))
            else {
                break;
            };
            let value = value_at(&samples, frame);
            out[0] += value * gains[0] * output.volume;
            out[1] += value * gains[1] * output.volume;
        }
    }
    for sample in &mut out {
        *sample = sample.map(|v| v.clamp(-1., 1.));
    }
    out
}

/// Stereo samples as a 16-bit WAV file, for listening to a mix
pub fn write_wav(path: &std::path::Path, rate: u32, samples: &[[f32; 2]]) -> std::io::Result<()> {
    let data: Vec<u8> = samples
        .iter()
        .flat_map(|s| s.map(|v| (v.clamp(-1., 1.) * 32767.) as i16))
        .flat_map(i16::to_le_bytes)
        .collect();
    let mut out = b"RIFF".to_vec();
    out.extend((36 + data.len() as u32).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * 4).to_le_bytes());
    out.extend(4u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    std::fs::write(path, out)
}

#[cfg(test)]
mod tests {
    use lambda_slicer_playback::audio::SoundSource;

    use super::*;

    fn channel(entity: u16, origin: [f32; 3], started: f64) -> Channel {
        Channel {
            id: 0,
            source: SoundSource::Network,
            entity,
            channel: 1,
            sound: SoundName::File(Arc::from("test.wav")),
            volume: 1.,
            attenuation: 1.,
            origin,
            started: SourceTime(started),
            pitches: vec![(SourceTime(started), 100)],
        }
    }

    #[test]
    fn a_sound_moves_with_its_entity_unless_static_or_on_a_brush() {
        use lambda_slicer_playback::scene::{Entity, EntityId};
        let entity = |number: u16, model: &str| Entity {
            id: EntityId {
                number,
                ..EntityId::default()
            },
            model: Some(Arc::from(model)),
            origin: [100., 0., 0.],
            ..Entity::default()
        };
        let mut scene = Scene::default();
        scene.entities = Arc::new(vec![entity(3, "models/rpgrocket.mdl"), entity(4, "*2")]);
        let mut fixed = channel(3, [0.; 3], 0.);
        fixed.channel = lambda_slicer_playback::audio::CHAN_STATIC;
        let moved = follow(
            &[
                channel(3, [0.; 3], 0.),
                channel(4, [5.; 3], 0.),
                fixed,
                channel(9, [7.; 3], 0.),
            ],
            &scene,
            1,
        );
        let origins: Vec<[f32; 3]> = moved.iter().map(|c| c.origin).collect();
        assert_eq!(origins, [[100., 0., 0.], [5.; 3], [0.; 3], [7.; 3]]);
    }

    #[test]
    fn the_recorders_own_sounds_are_placed_for_a_free_camera() {
        // The server leaves the recorder's entity at nought
        let mut scene = Scene::default();
        scene.camera.player_origin = [0., -100., 0.];
        let own = follow(&[channel(1, [0.; 3], 0.)], &scene, 1);
        assert_eq!(own[0].origin, [0., -100., 0.]);

        // Heard by the recorder they stay in the middle, and by a camera
        // looking along x they come from its right, minus y
        let recorder = Listener {
            origin: [0.; 3],
            right: right_of([0.; 3]),
            entity: 1,
        };
        assert_eq!(spatialize(&own[0], &recorder), [1.; 2]);
        let [left, right] = spatialize(&own[0], &Listener::heard_from([0.; 3], [0.; 3]));
        assert!(right > left, "{left} {right}");
    }

    /// A ramp from 0 up by 1/16 of full scale a frame, at 10 frames a second
    fn ramp(loop_start: Option<u32>) -> Arc<Sound> {
        Arc::new(Sound {
            rate: 10,
            channels: 1,
            samples: (0..8).map(|i| i * 2048).collect(),
            loop_start,
            frames: 8,
        })
    }

    fn listener() -> Listener {
        Listener {
            origin: [0.; 3],
            right: [0., -1., 0.],
            entity: 1,
        }
    }

    #[test]
    fn channels_play_from_their_start_and_loop_or_end() {
        let once = ramp(None);
        let looped = ramp(Some(4));
        let ch = channel(1, [0.; 3], 0.);

        // Ten output frames a second at speed 1 read the ramp frame by frame
        let out = mix(
            std::slice::from_ref(&ch),
            &|_| Some(once.clone()),
            &listener(),
            SourceTime(0.),
            10,
            &Output {
                rate: 10,
                speed: 1.,
                volume: 1.,
            },
        );
        let left: Vec<f32> = out.iter().map(|s| s[0]).collect();
        assert_eq!(
            left[..8],
            (0..8).map(|i| i as f32 / 16.).collect::<Vec<_>>()[..]
        );
        assert_eq!(left[8..], [0., 0.]);

        // A loop goes back to frame 4
        let out = mix(
            std::slice::from_ref(&ch),
            &|_| Some(looped.clone()),
            &listener(),
            SourceTime(0.),
            10,
            &Output {
                rate: 10,
                speed: 1.,
                volume: 1.,
            },
        );
        assert_eq!(out[8][0], 4. / 16.);
        assert_eq!(out[9][0], 5. / 16.);

        // Twice the speed or twice the pitch reads every other frame, and a
        // sound started later is silent before it starts
        let out = mix(
            std::slice::from_ref(&ch),
            &|_| Some(once.clone()),
            &listener(),
            SourceTime(0.),
            4,
            &Output {
                rate: 10,
                speed: 2.,
                volume: 1.,
            },
        );
        assert_eq!(out[1][0], 2. / 16.);
        let mut high = ch.clone();
        high.pitches = vec![(SourceTime(0.), 200)];
        let out = mix(
            &[high],
            &|_| Some(once.clone()),
            &listener(),
            SourceTime(0.),
            4,
            &Output {
                rate: 10,
                speed: 1.,
                volume: 1.,
            },
        );
        assert_eq!(out[1][0], 2. / 16.);
        let late = channel(1, [0.; 3], 0.5);
        let out = mix(
            &[late],
            &|_| Some(once.clone()),
            &listener(),
            SourceTime(0.),
            7,
            &Output {
                rate: 10,
                speed: 1.,
                volume: 1.,
            },
        );
        assert_eq!(out[4][0], 0.);
        assert_eq!(out[6][0], 1. / 16.);
    }

    #[test]
    fn sounds_are_placed_by_distance_and_side() {
        // The listener's own sounds are not placed
        assert_eq!(
            spatialize(&channel(1, [500., 0., 0.], 0.), &listener()),
            [1., 1.]
        );

        // 500 units to the right, attenuation 1, is at half volume, and all
        // of it on the right
        let right = spatialize(&channel(2, [0., -500., 0.], 0.), &listener());
        assert_eq!(right, [0., 1.]);
        let ahead = spatialize(&channel(2, [500., 0., 0.], 0.), &listener());
        assert_eq!(ahead, [0.5, 0.5]);
        // Beyond the nominal distance nothing is heard
        assert_eq!(
            spatialize(&channel(2, [1500., 0., 0.], 0.), &listener()),
            [0., 0.]
        );
    }
}
