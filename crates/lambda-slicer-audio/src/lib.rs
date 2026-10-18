//! A demo's sounds, mixed from playback's logical channels (implementation
//! plan §13.7)
//!
//! [`library::Library`] reads the game's sound files through its search
//! path and tells playback how long each one plays. [`mixer`] turns the
//! channels playing at a moment into stereo samples, deterministically, so
//! the same moment always sounds the same however it was reached.
//! [`device`], behind the `device` feature, sends the mix to the system's
//! sound output, and everything else can be tested and rendered offline

#[cfg(feature = "device")]
pub mod device;
pub mod library;
pub mod mixer;
