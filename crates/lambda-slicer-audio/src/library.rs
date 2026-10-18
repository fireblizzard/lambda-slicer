//! The game's sounds, read once each through its search path
//!
//! A sample named `weapons/gauss2.wav` is `sound/weapons/gauss2.wav`. A
//! leading `*`, which marks a sound the game streams, is dropped. Sentences
//! (`!` names and sentence numbers) are not resolved to their words yet, so
//! they have no samples. A sound that cannot be read is remembered as
//! missing, so it is not read again

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::search::SearchPath;
use lambda_slicer_assets::wav::{self, Sound};
use lambda_slicer_playback::audio::{SoundLength, SoundLengths, SoundName};

pub struct Library {
    search: SearchPath,
    limits: Limits,
    sounds: Mutex<HashMap<Arc<str>, Option<Arc<Sound>>>>,
}

impl Library {
    pub fn new(search: SearchPath, limits: Limits) -> Self {
        Self {
            search,
            limits,
            sounds: Mutex::new(HashMap::new()),
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
}
