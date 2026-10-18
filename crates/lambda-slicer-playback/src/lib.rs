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
pub mod audio;
pub mod checkpoint;
pub mod controller;
pub mod events;
pub mod playback;
pub mod scene;
pub mod source;
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
}

/// Where playback learns how many frames a sprite has, by its path
pub type SpriteFrames = std::sync::Arc<dyn Fn(&str) -> Option<u32> + Send + Sync>;

/// Where playback learns a map's world, by its path such as
/// `maps/crossfire.bsp`
pub type MapHulls =
    std::sync::Arc<dyn Fn(&str) -> Option<std::sync::Arc<trace::Hull>> + Send + Sync>;
