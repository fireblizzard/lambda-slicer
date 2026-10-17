//! The clocks playback keeps apart, and the one that moves the playhead
//!
//! - Source time is a record's time on the demo's playback clock, seconds
//!   from the start of playback. LOADING records count as 0, the state
//!   playback begins with
//! - Server time is what `svc_time` last said. It starts again with every
//!   session, so it repeats across map changes and never identifies a moment
//! - Presentation time is the source time a frame is shown for, which can
//!   fall between two record groups
//! - Wall time is the window's monotonic clock in seconds, given to
//!   [`Clock`] by its caller so tests control it
//!
//! Audio sample time joins these with the mixer (M10)

use std::cmp::Ordering;

/// Seconds on the demo's playback clock
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct SourceTime(pub f64);

impl SourceTime {
    pub const ZERO: Self = Self(0.);

    pub fn total_cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// Seconds on the server's clock of one session
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct ServerTime(pub f32);

/// Where playback stands. Group numbers only grow through the file, so a
/// source or server time that repeats across sessions never names two
/// moments
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// The last record group applied, counted from 0 at the first group of
    /// the LOADING entry
    pub group: u64,
    /// The decoder's session counter after that group
    pub segment: u32,
    /// That group's source time
    pub time: SourceTime,
}

/// The speeds playback offers. Backward playback comes with the timeline
/// controls (M11)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rate {
    Quarter,
    Half,
    #[default]
    Normal,
    Double,
    Quadruple,
}

impl Rate {
    pub fn factor(self) -> f64 {
        match self {
            Rate::Quarter => 0.25,
            Rate::Half => 0.5,
            Rate::Normal => 1.,
            Rate::Double => 2.,
            Rate::Quadruple => 4.,
        }
    }
}

/// Source time as a function of wall time. While playing, source time runs
/// from an anchor at the chosen rate. Paused, it stays where it stopped.
/// Every change re-anchors, so a new rate never moves the playhead
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    playing: bool,
    rate: Rate,
    anchor_wall: f64,
    anchor_source: SourceTime,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            playing: false,
            rate: Rate::Normal,
            anchor_wall: 0.,
            anchor_source: SourceTime::ZERO,
        }
    }
}

impl Clock {
    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn rate(&self) -> Rate {
        self.rate
    }

    /// The source time at `wall`
    pub fn at(&self, wall: f64) -> SourceTime {
        if !self.playing {
            return self.anchor_source;
        }

        let elapsed = (wall - self.anchor_wall).max(0.);
        SourceTime(self.anchor_source.0 + elapsed * self.rate.factor())
    }

    fn anchor(&mut self, wall: f64, source: SourceTime) {
        self.anchor_wall = wall;
        self.anchor_source = source;
    }

    pub fn play(&mut self, wall: f64) {
        let now = self.at(wall);
        self.anchor(wall, now);
        self.playing = true;
    }

    pub fn pause(&mut self, wall: f64) {
        let now = self.at(wall);
        self.anchor(wall, now);
        self.playing = false;
    }

    pub fn set_rate(&mut self, rate: Rate, wall: f64) {
        let now = self.at(wall);
        self.anchor(wall, now);
        self.rate = rate;
    }

    /// Move the playhead to `time`, playing on from there if it was playing
    pub fn set(&mut self, time: SourceTime, wall: f64) {
        self.anchor(wall, time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_runs_at_its_rate_and_stops_when_paused() {
        let mut clock = Clock::default();
        assert_eq!(clock.at(5.), SourceTime::ZERO);

        clock.play(10.);
        assert_eq!(clock.at(12.), SourceTime(2.));
        clock.set_rate(Rate::Quadruple, 12.);
        assert_eq!(clock.at(12.), SourceTime(2.));
        assert_eq!(clock.at(13.), SourceTime(6.));

        clock.pause(13.5);
        assert_eq!(clock.at(100.), SourceTime(8.));
        clock.set(SourceTime(1.), 101.);
        assert_eq!(clock.at(200.), SourceTime(1.));

        // A wall clock read earlier than the anchor never runs time backwards
        clock.play(300.);
        assert_eq!(clock.at(299.), SourceTime(1.));
    }
}
