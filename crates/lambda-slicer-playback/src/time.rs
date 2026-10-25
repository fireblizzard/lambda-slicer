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

/// The speeds playback offers, forward or backward, slowest first
pub const RATES: [f64; 15] = [
    0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 1., 1.25, 1.5, 2., 3., 5., 10., 50., 100.,
];

/// One of the speeds in [`RATES`], by its place there
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rate(usize);

impl Default for Rate {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl Rate {
    /// The demo's own speed
    pub const NORMAL: Self = Self(6);

    /// The speed offered that is `factor` times the demo's own, if any
    pub fn of(factor: f64) -> Option<Self> {
        let place = RATES.iter().position(|&r| r == factor)?;
        Some(Self(place))
    }

    pub fn factor(self) -> f64 {
        RATES[self.0]
    }

    /// The next speed down, none below the slowest
    pub fn slower(self) -> Option<Self> {
        let place = self.0.checked_sub(1)?;
        Some(Self(place))
    }

    /// The next speed up, none above the fastest
    pub fn faster(self) -> Option<Self> {
        if self.0 + 1 < RATES.len() {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

/// How much faster than its rate a held boost plays
pub const BOOST: f64 = 2.;

/// Source time as a function of wall time. While playing, source time runs
/// from an anchor at the chosen rate, [`BOOST`] times that while boosted,
/// forward or backward, never below 0. Paused, it stays where it stopped.
/// Every change re-anchors, so a new rate or direction never moves the
/// playhead
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    playing: bool,
    rate: Rate,
    boosted: bool,
    backward: bool,
    anchor_wall: f64,
    anchor_source: SourceTime,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            playing: false,
            rate: Rate::NORMAL,
            boosted: false,
            backward: false,
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

    /// Playing faster than its rate for as long as it is held
    pub fn boosted(&self) -> bool {
        self.boosted
    }

    /// Source seconds a wall second plays while playing
    pub fn speed(&self) -> f64 {
        if self.boosted {
            self.rate.factor() * BOOST
        } else {
            self.rate.factor()
        }
    }

    /// Running toward the start
    pub fn backward(&self) -> bool {
        self.backward
    }

    /// The source time at `wall`
    pub fn at(&self, wall: f64) -> SourceTime {
        if !self.playing {
            return self.anchor_source;
        }

        let elapsed = (wall - self.anchor_wall).max(0.);
        let sign = if self.backward { -1. } else { 1. };
        SourceTime((self.anchor_source.0 + sign * elapsed * self.speed()).max(0.))
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

    /// Play [`BOOST`] times faster than the rate, or at the rate again
    pub fn set_boosted(&mut self, boosted: bool, wall: f64) {
        let now = self.at(wall);
        self.anchor(wall, now);
        self.boosted = boosted;
    }

    /// Run toward the start, or forward again
    pub fn set_backward(&mut self, backward: bool, wall: f64) {
        let now = self.at(wall);
        self.anchor(wall, now);
        self.backward = backward;
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
        clock.set_rate(Rate::of(5.).unwrap(), 12.);
        assert_eq!(clock.at(12.), SourceTime(2.));
        assert_eq!(clock.at(13.), SourceTime(7.));

        clock.pause(13.5);
        assert_eq!(clock.at(100.), SourceTime(9.5));
        clock.set(SourceTime(1.), 101.);
        assert_eq!(clock.at(200.), SourceTime(1.));

        // A wall clock read earlier than the anchor never runs time backwards
        clock.play(300.);
        assert_eq!(clock.at(299.), SourceTime(1.));
    }

    #[test]
    fn backward_the_clock_runs_toward_the_start_and_stops_there() {
        let mut clock = Clock::default();
        clock.set(SourceTime(3.), 0.);
        clock.play(0.);
        clock.set_backward(true, 1.);
        assert_eq!(clock.at(1.), SourceTime(4.));
        clock.set_rate(Rate::of(2.).unwrap(), 1.);
        assert_eq!(clock.at(2.), SourceTime(2.));
        assert_eq!(clock.at(10.), SourceTime::ZERO);
        clock.set_backward(false, 2.);
        assert_eq!(clock.at(2.5), SourceTime(3.));
    }

    #[test]
    fn a_boost_doubles_the_rate_while_it_lasts() {
        let mut clock = Clock::default();
        clock.set_rate(Rate::of(5.).unwrap(), 0.);
        clock.play(0.);
        clock.set_boosted(true, 1.);
        assert_eq!(clock.at(1.), SourceTime(5.));
        assert_eq!(clock.at(2.), SourceTime(15.));
        clock.set_boosted(false, 2.);
        assert_eq!(clock.at(3.), SourceTime(20.));
        assert_eq!(clock.rate(), Rate::of(5.).unwrap());
    }

    #[test]
    fn the_speeds_step_one_at_a_time_and_stop_at_either_end() {
        assert_eq!(Rate::default().factor(), 1.);
        assert_eq!(Rate::NORMAL.faster().map(Rate::factor), Some(1.25));
        assert_eq!(Rate::NORMAL.slower().map(Rate::factor), Some(0.75));
        assert_eq!(Rate::of(0.01).unwrap().slower(), None);
        assert_eq!(Rate::of(100.).unwrap().faster(), None);
        assert_eq!(Rate::of(4.), None);
    }
}
