//! A demo's sounds, mixed from playback's logical channels (implementation
//! plan §13.7)
//!
//! [`library::Library`] reads the game's sound files through its search
//! path and tells playback how long each one plays. [`mixer`] turns the
//! channels playing at a moment into stereo samples, deterministically, so
//! the same moment always sounds the same however it was reached. Sending
//! the samples to a sound device is separate (10b), so everything here can
//! be tested and rendered offline

pub mod library;
pub mod mixer;
