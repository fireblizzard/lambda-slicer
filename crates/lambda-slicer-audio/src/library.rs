//! The game's sounds and its models' sequences, read once each through its
//! search path
//!
//! A sample named `weapons/gauss2.wav` is `sound/weapons/gauss2.wav`. A
//! leading `*`, which marks a sound the game streams, is dropped. Sentences
//! (`!` names and sentence numbers) are not resolved to their words yet, so
//! they have no samples. A sound or model that cannot be read is remembered
//! as missing, so it is not read again. Of a model only its main file is
//! read, which describes every sequence and its events

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::search::SearchPath;
use lambda_slicer_assets::wav::{self, Sound};
use lambda_slicer_assets::xenoxide::studio::StudioModel;
use lambda_slicer_playback::GameFiles;
use lambda_slicer_playback::animation::{Sequence, StudioEvent};
use lambda_slicer_playback::audio::{SoundLength, SoundLengths, SoundName};

/// `STUDIO_LOOPING`
const STUDIO_LOOPING: i32 = 1;

pub struct Library {
    search: SearchPath,
    limits: Limits,
    sounds: Mutex<HashMap<Arc<str>, Option<Arc<Sound>>>>,
    sequences: Mutex<HashMap<String, Option<Arc<[Sequence]>>>>,
}

impl Library {
    pub fn new(search: SearchPath, limits: Limits) -> Self {
        Self {
            search,
            limits,
            sounds: Mutex::new(HashMap::new()),
            sequences: Mutex::new(HashMap::new()),
        }
    }

    /// The samples of `name`, none when it is a sentence or cannot be read
    pub fn sound(&self, name: &SoundName) -> Option<Arc<Sound>> {
        let SoundName::File(file) = name else {
            return None;
        };
        let mut sounds = self.sounds.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(known) = sounds.get(file) {
            return known.clone();
        }

        let path = format!("sound/{}", file.trim_start_matches('*'));
        let sound = self
            .search
            .find(&path)
            .and_then(|found| self.search.read(&found))
            .and_then(|bytes| wav::parse(&bytes, self.limits))
            .ok()
            .map(Arc::new);
        sounds.insert(file.clone(), sound.clone());
        sound
    }

    /// How long `name` plays, as its file says
    pub fn length(&self, name: &SoundName) -> SoundLength {
        match self.sound(name) {
            Some(s) if s.loop_start.is_some() => SoundLength::Looping,
            Some(s) => SoundLength::Once(s.seconds()),
            None => SoundLength::Unknown,
        }
    }

    /// [`Library::length`] for playback
    pub fn lengths(self: &Arc<Self>) -> SoundLengths {
        let library = self.clone();
        Arc::new(move |name| library.length(name))
    }

    /// The sequences of the studio model at `path`, such as
    /// `models/v_9mmar.mdl`, none when it is not a studio model or cannot
    /// be read
    pub fn sequences(&self, path: &str) -> Option<Arc<[Sequence]>> {
        let mut known = self.sequences.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(sequences) = known.get(path) {
            return sequences.clone();
        }

        let read = self
            .search
            .find(path)
            .and_then(|found| self.search.read(&found))
            .ok();
        let sequences = read
            .filter(|bytes| bytes.starts_with(b"IDST"))
            .and_then(|bytes| StudioModel::parse(&bytes).ok())
            .map(|model| {
                model
                    .sequences()
                    .iter()
                    .map(|s| Sequence {
                        fps: s.fps,
                        frames: u32::try_from(s.frames).unwrap_or(u32::MAX),
                        looping: s.flags & STUDIO_LOOPING != 0,
                        events: s
                            .events
                            .iter()
                            .map(|e| StudioEvent {
                                frame: e.frame,
                                event: e.event,
                                options: Arc::from(e.options.as_str()),
                            })
                            .collect(),
                    })
                    .collect()
            });
        known.insert(path.to_owned(), sequences.clone());
        sequences
    }

    /// The sound lengths and model sequences playback reads
    pub fn game_files(self: &Arc<Self>) -> GameFiles {
        let library = self.clone();
        GameFiles {
            lengths: self.lengths(),
            sequences: Arc::new(move |path| library.sequences(path)),
        }
    }
}
