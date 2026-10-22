//! Playing a demo inside Lambda Slicer (implementation plan §13.2 and §13.4)
//!
//! [`source`] reads a demo's records in groups, the frames the recording
//! client wrote together, and [`playback::Playback`] applies them in order
//! to the core decoder, the [`scene`] and the [`audio`] state. That
//! sequential playback is the reference every faster way of seeking is
//! checked against later.
//!
//! [`controller::Controller`] is the one clock and command handler a window
//! drives. It turns commands into requests stamped with a load generation
//! and a seek revision, and [`worker`] answers them off the window's thread
//! with [`PresentedFrame`]s. A frame from an older load or seek is never
//! shown. Nothing here draws, plays sound or depends on a GPU, a sound device
//! or the installed game

pub mod animation;
pub mod announcements;
pub mod audio;
pub mod beams;
pub mod bullets;
pub mod checkpoint;
pub mod controller;
pub mod damage;
pub mod decals;
pub mod events;
pub mod gauss;
pub mod hud;
pub mod kill_features;
pub mod playback;
pub mod scene;
pub mod scores;
pub mod source;
pub mod status;
pub mod tempents;
pub mod time;
pub mod trace;
pub mod worker;

pub use playback::PresentedFrame;

/// What playback reads from the game's files, the length of each sound,
/// each model's sequences and each map's world, shared by every playback of
/// a demo so their checkpoints agree
#[derive(Clone)]
pub struct GameFiles {
    pub lengths: audio::SoundLengths,
    pub sequences: animation::ModelSequences,
    pub hulls: MapHulls,
    /// Each sprite's frame count
    pub sprite_frames: SpriteFrames,
    /// A decal's place in `decals.wad` by its name
    pub decal_index: DecalIndex,
}

/// Where playback learns a decal's place in `decals.wad`, by its name
pub type DecalIndex = std::sync::Arc<dyn Fn(&str) -> Option<u16> + Send + Sync>;

/// Where playback learns how many frames a sprite has, by its path
pub type SpriteFrames = std::sync::Arc<dyn Fn(&str) -> Option<u32> + Send + Sync>;

/// Where playback learns a map's world, by its path such as
/// `maps/crossfire.bsp`
pub type MapHulls =
    std::sync::Arc<dyn Fn(&str) -> Option<std::sync::Arc<trace::Hull>> + Send + Sync>;

/// A hash map whose `Debug` lists its entries by key, so two playbacks
/// holding the same print the same, as the equivalence checks compare
/// them as text
#[derive(Clone)]
pub(crate) struct Table<K, V>(std::collections::HashMap<K, V>);

impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self(std::collections::HashMap::new())
    }
}

impl<K, V> Table<K, V> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(std::collections::HashMap::with_capacity(capacity))
    }
}

impl<K, V> std::ops::Deref for Table<K, V> {
    type Target = std::collections::HashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> std::ops::DerefMut for Table<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, K, V> IntoIterator for &'a Table<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<K: Eq + std::hash::Hash, V> FromIterator<(K, V)> for Table<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(entries: I) -> Self {
        Self(entries.into_iter().collect())
    }
}

impl<K: Ord + std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Table<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        f.debug_map().entries(entries).finish()
    }
}
