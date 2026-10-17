//! Where an export of `[start, end)` would really start and end, worked out
//! from the playback entry's frame groups without reading the demo again
//!
//! A restart starts at the first frame group at or after the requested
//! start and stops before the first group at or after the requested end, so
//! the effective range is the requested one snapped outwards to the group
//! times. The rules and refusals here follow [`super::restart`]'s plan, so
//! a clip can show its effective range and the cheap refusals before any
//! export runs. Refusals that need the decoded state, such as unproven
//! state or dropped packets, come only from the export itself

use std::collections::BTreeMap;

use serde::Serialize;

use super::ExportError;
use crate::timeline::time::DemoTime;

/// The start of one frame group in the playback entry
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupStart {
    pub time: f32,
    /// The decoder's session before the group's first record
    pub segment: u32,
    /// The group opens with the playback entry's `DemoStart` record, where
    /// a restart never starts
    pub demo_start: bool,
}

/// The playback entry's frame groups, gathered while decoding
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroupIndex {
    /// The directory's entry types, a restart needing exactly `[0, 1]`
    pub directory_types: Vec<i32>,
    /// The session the LOADING entry ends in
    pub loading_segment: Option<u32>,
    /// Groups in file order
    pub groups: Vec<GroupStart>,
    /// Groups during which a new session began
    pub session_changes: Vec<usize>,
    /// For each later session, the first group a restart can start at,
    /// the one after its setup is complete and a full update arrived
    pub ready: BTreeMap<u32, usize>,
    /// The time of the playback entry's terminal record
    pub end: Option<f32>,
}

/// A range snapped to frame groups
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Snapped {
    pub start: DemoTime,
    pub end: DemoTime,
}

fn time(t: f32) -> DemoTime {
    DemoTime::from_f32(t).unwrap_or(DemoTime::ZERO)
}

impl GroupIndex {
    /// The effective range of a restart of `[start, end)`, or the reason a
    /// restart refuses it. No `end` means the end of the recording
    pub fn snap(&self, start: DemoTime, end: Option<DemoTime>) -> Result<Snapped, ExportError> {
        if self.directory_types != [0, 1] {
            return Err(ExportError::Unsupported(format!(
                "a restart needs exactly LOADING(0) and Playback(1) entries, found types {:?}",
                self.directory_types
            )));
        }
        let terminal = self.end.map(time);

        let Some(first) = self
            .groups
            .iter()
            .position(|g| time(g.time) >= start && !g.demo_start)
        else {
            return Err(ExportError::RangeBeyondEnd {
                end: start,
                segment_end: terminal.unwrap_or(DemoTime::ZERO),
            });
        };
        let g = &self.groups[first];
        if Some(g.segment) != self.loading_segment
            && self.ready.get(&g.segment).is_none_or(|&r| first < r)
        {
            return Err(ExportError::StartDuringLoading);
        }
        if first == 0 {
            return Err(ExportError::StartAtFirstGroup);
        }

        let stop =
            end.and_then(|e| (first..self.groups.len()).find(|&i| time(self.groups[i].time) >= e));
        let effective_end = match (stop, end, terminal) {
            (Some(i), _, _) => time(self.groups[i].time),
            (None, Some(e), Some(t)) if e > t => {
                return Err(ExportError::RangeBeyondEnd {
                    end: e,
                    segment_end: t,
                });
            }
            (None, _, Some(t)) => t,
            (None, _, None) => {
                return Err(ExportError::Unsupported(
                    "the playback entry has no terminal record".into(),
                ));
            }
        };
        if stop == Some(first) {
            return Err(ExportError::EmptyRange);
        }

        let last = stop.unwrap_or(self.groups.len());
        if let Some(&c) = self
            .session_changes
            .iter()
            .find(|&&c| c >= first && c < last)
        {
            return Err(ExportError::CrossSegmentRange(
                time(self.groups[c].time).to_string(),
            ));
        }

        Ok(Snapped {
            start: time(g.time),
            end: effective_end,
        })
    }

    /// The session of the group a start at `start` would snap to
    pub fn segment_at(&self, start: DemoTime) -> Option<u32> {
        self.groups
            .iter()
            .find(|g| time(g.time) >= start && !g.demo_start)
            .map(|g| g.segment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ten groups a tenth of a second apart after the `DemoStart` group,
    /// ending at 1.1
    fn index() -> GroupIndex {
        let mut groups = vec![GroupStart {
            time: 0.,
            segment: 1,
            demo_start: true,
        }];
        groups.extend((0..10).map(|k| GroupStart {
            time: k as f32 * 0.1,
            segment: 1,
            demo_start: false,
        }));
        GroupIndex {
            directory_types: vec![0, 1],
            loading_segment: Some(1),
            groups,
            session_changes: Vec::new(),
            ready: BTreeMap::new(),
            end: Some(1.1),
        }
    }

    fn at(s: &str) -> DemoTime {
        s.parse().unwrap()
    }

    #[test]
    fn snaps_outwards_to_group_starts() {
        let s = index().snap(at("0.25"), Some(at("0.75"))).unwrap();
        assert_eq!(s.start, at("0.3"));
        assert_eq!(s.end, at("0.8"));
        // Exact group times stay
        let s = index().snap(at("0.3"), Some(at("0.8"))).unwrap();
        assert_eq!((s.start, s.end), (at("0.3"), at("0.8")));
    }

    #[test]
    fn an_end_past_the_last_group_stops_at_the_terminal_record() {
        let s = index().snap(at("0.5"), Some(at("1.05"))).unwrap();
        assert_eq!(s.end, at("1.1"));
        let s = index().snap(at("0.5"), None).unwrap();
        assert_eq!(s.end, at("1.1"));
        let err = index().snap(at("0.5"), Some(at("2"))).unwrap_err();
        assert!(matches!(err, ExportError::RangeBeyondEnd { .. }), "{err}");
    }

    #[test]
    fn refuses_what_a_restart_refuses() {
        // The first playable group is the first group after DemoStart
        let mut i = index();
        i.groups.remove(0);
        let err = i.snap(at("0"), None).unwrap_err();
        assert!(matches!(err, ExportError::StartAtFirstGroup), "{err}");

        let err = index().snap(at("0.33"), Some(at("0.36"))).unwrap_err();
        assert!(matches!(err, ExportError::EmptyRange), "{err}");

        let err = index().snap(at("1.5"), None).unwrap_err();
        assert!(matches!(err, ExportError::RangeBeyondEnd { .. }), "{err}");

        // A later session from group 6, ready to start at group 8
        let mut i = index();
        for g in &mut i.groups[6..] {
            g.segment = 2;
        }
        let err = i.snap(at("0.55"), None).unwrap_err();
        assert!(matches!(err, ExportError::StartDuringLoading), "{err}");
        i.ready.insert(2, 8);
        let err = i.snap(at("0.55"), None).unwrap_err();
        assert!(matches!(err, ExportError::StartDuringLoading), "{err}");
        assert_eq!(i.snap(at("0.65"), None).unwrap().start, at("0.7"));
        assert_eq!(i.segment_at(at("0.65")), Some(2));
        assert_eq!(i.segment_at(at("0.25")), Some(1));

        let mut i = index();
        i.session_changes.push(7);
        let err = i.snap(at("0.3"), Some(at("0.9"))).unwrap_err();
        assert!(matches!(err, ExportError::CrossSegmentRange(_)), "{err}");
        assert!(i.snap(at("0.3"), Some(at("0.6"))).is_ok());

        let mut i = index();
        i.directory_types = vec![0, 1, 1];
        assert!(matches!(
            i.snap(at("0.3"), None),
            Err(ExportError::Unsupported(_))
        ));
    }
}
