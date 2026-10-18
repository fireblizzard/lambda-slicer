//! The one place playback time is chosen (implementation plan §13.4)
//!
//! Every control that moves the playhead becomes a [`Command`] here. The
//! controller keeps the clock and answers with a [`Request`] for the
//! worker, stamped with the load generation and the seek revision. Loading
//! a demo starts a new generation, and every seek, step or loop restart a
//! new revision. A reply is accepted only when it answers the current
//! generation and a revision no older than the current one, so a slow
//! answer to an earlier seek never moves the playhead or shows a frame

use std::path::PathBuf;

use lambda_slicer_core::message::Message;

use crate::PresentedFrame;
use crate::time::{Clock, Position, Rate, SourceTime};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stamp {
    pub generation: u64,
    pub revision: u64,
}

/// A source interval played over and over
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRange {
    pub start: SourceTime,
    pub end: SourceTime,
}

/// Where the viewer looks from. Only the recorder's view is certain to show
/// what the recording client received
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CameraMode {
    #[default]
    Recorder,
    Free {
        origin: [f32; 3],
        angles: [f32; 3],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Load(PathBuf),
    Play,
    Pause,
    SeekTo(SourceTime),
    /// Move this many record groups, forward when positive. Pauses
    StepGroup(i64),
    SetRate(Rate),
    SetLoop(Option<LoopRange>),
    SetCamera(CameraMode),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    Time(SourceTime),
    /// The group to be the last applied
    Group(u64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub stamp: Stamp,
    /// The demo to open first, on a new generation
    pub load: Option<PathBuf>,
    pub target: Target,
    pub camera: CameraMode,
    pub audible: bool,
}

/// What the worker answers
#[derive(Clone, Debug)]
pub enum Reply {
    Frame(Box<PresentedFrame>),
    Failed { stamp: Stamp, message: Message },
}

impl Reply {
    pub fn stamp(&self) -> Stamp {
        match self {
            Reply::Frame(frame) => frame.stamp,
            Reply::Failed { stamp, .. } => *stamp,
        }
    }
}

#[derive(Debug, Default)]
pub struct Controller {
    stamp: Stamp,
    clock: Clock,
    demo: Option<PathBuf>,
    loop_range: Option<LoopRange>,
    camera: CameraMode,
    /// The position of the newest accepted frame
    shown: Option<Position>,
    /// The group the newest step asked for, which later steps go on from
    stepping_to: Option<u64>,
    /// Why the current demo could not be played
    failed: Option<Message>,
}

impl Controller {
    pub fn stamp(&self) -> Stamp {
        self.stamp
    }

    pub fn playing(&self) -> bool {
        self.clock.playing()
    }

    /// The clock, for whatever follows the playhead on its own, such as
    /// sound
    pub fn clock(&self) -> Clock {
        self.clock
    }

    pub fn rate(&self) -> Rate {
        self.clock.rate()
    }

    pub fn loop_range(&self) -> Option<LoopRange> {
        self.loop_range
    }

    pub fn failed(&self) -> Option<&Message> {
        self.failed.as_ref()
    }

    /// The playhead at `wall`
    pub fn time(&self, wall: f64) -> SourceTime {
        self.clock.at(wall)
    }

    fn request(&self, target: Target, audible: bool) -> Option<Request> {
        self.demo.as_ref()?;
        Some(Request {
            stamp: self.stamp,
            load: None,
            target,
            camera: self.camera,
            audible,
        })
    }

    fn new_revision(&mut self) {
        self.stamp.revision += 1;
        self.stepping_to = None;
    }

    /// Apply a command at wall time `wall`, and what the worker must do for
    /// it, if anything
    pub fn command(&mut self, command: Command, wall: f64) -> Option<Request> {
        match command {
            Command::Load(path) => {
                self.stamp = Stamp {
                    generation: self.stamp.generation + 1,
                    revision: 0,
                };
                let rate = self.clock.rate();
                self.clock = Clock::default();
                self.clock.set_rate(rate, wall);
                self.loop_range = None;
                self.shown = None;
                self.stepping_to = None;
                self.failed = None;
                self.demo = Some(path.clone());
                Some(Request {
                    stamp: self.stamp,
                    load: Some(path),
                    target: Target::Time(SourceTime::ZERO),
                    camera: self.camera,
                    audible: false,
                })
            }
            Command::Play => {
                self.clock.play(wall);
                self.tick(wall)
            }
            Command::Pause => {
                self.clock.pause(wall);
                None
            }
            Command::SeekTo(time) => {
                self.new_revision();
                self.clock.set(time, wall);
                self.request(Target::Time(time), false)
            }
            Command::StepGroup(by) => {
                let from = self.stepping_to.or(self.shown.map(|p| p.group))?;
                let to = from.saturating_add_signed(by);
                self.clock.pause(wall);
                self.new_revision();
                self.stepping_to = Some(to);
                self.request(Target::Group(to), false)
            }
            Command::SetRate(rate) => {
                self.clock.set_rate(rate, wall);
                None
            }
            Command::SetLoop(range) => {
                self.loop_range = range.filter(|r| r.end > r.start);
                None
            }
            Command::SetCamera(camera) => {
                self.camera = camera;
                let at = self.clock.at(wall);
                self.request(Target::Time(at), false)
            }
        }
    }

    /// While playing, the request for the playhead at `wall`. At the end of
    /// a loop the playhead goes back to its start as a new seek
    pub fn tick(&mut self, wall: f64) -> Option<Request> {
        if !self.clock.playing() {
            return None;
        }

        let at = self.clock.at(wall);
        if let Some(range) = self.loop_range
            && at >= range.end
        {
            self.new_revision();
            self.clock.set(range.start, wall);
            return self.request(Target::Time(range.start), true);
        }
        self.request(Target::Time(at), true)
    }

    /// Whether to show `reply`. A step's frame moves the paused playhead to
    /// the group it reached, and the end of the demo pauses playback
    pub fn accept(&mut self, reply: &Reply, wall: f64) -> bool {
        let stamp = reply.stamp();
        if stamp.generation != self.stamp.generation || stamp.revision < self.stamp.revision {
            return false;
        }

        let frame = match reply {
            Reply::Frame(frame) => frame,
            Reply::Failed { message, .. } => {
                self.failed = Some(message.clone());
                self.clock.pause(wall);
                return true;
            }
        };
        if self.stepping_to.is_some() {
            self.clock.set(frame.at, wall);
        }
        if frame.end && self.clock.playing() && frame.at <= self.clock.at(wall) {
            self.clock.pause(wall);
            self.clock.set(frame.at, wall);
        }
        self.shown = frame.position;
        true
    }
}
