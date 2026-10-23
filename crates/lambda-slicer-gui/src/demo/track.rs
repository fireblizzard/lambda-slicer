//! The timeline's arithmetic, kept apart from its drawing so it can be
//! tested
//!
//! Times are seconds of playback. A view is the stretch of the recording a
//! timeline shows, and the extent is the whole recording. Every surface
//! showing the same view shares one transform from times to points

use eframe::egui::{Pos2, Rangef};
use lambda_slicer_core::analysis::weapons::Weapon;

use super::Edge;
use crate::timeline::MarkKind;

/// The narrowest view in seconds, for a recording at least this long
pub const MIN_SPAN: f32 = 0.5;

/// How far a clip's edge, or the playhead, can be from the pointer and still
/// be taken, in points
pub const EDGE_SLOP: f32 = 6.;

/// The share of a view kept before the playhead when playing moves the
/// view on a page
const PAGE_LEAD: f32 = 0.1;

/// The narrowest view `extent` allows, none for a recording with no length,
/// which cannot be shown or sought in
pub fn narrowest(extent: (f32, f32)) -> Option<f32> {
    let length = extent.1 - extent.0;
    if !length.is_finite() || length <= 0. {
        return None;
    }

    Some(MIN_SPAN.min(length))
}

/// `view` kept within `extent`, no narrower than allowed and no wider than
/// the whole recording. A view that is not a range of numbers becomes the
/// whole recording
pub fn clamp_view(view: (f32, f32), extent: (f32, f32)) -> (f32, f32) {
    let Some(narrowest) = narrowest(extent) else {
        return extent;
    };
    if !view.0.is_finite() || !view.1.is_finite() {
        return extent;
    }

    let length = extent.1 - extent.0;
    let span = (view.1 - view.0).clamp(narrowest, length);
    let start = view.0.clamp(extent.0, extent.1 - span);
    (start, start + span)
}

/// `view` zoomed by `factor` around `anchor`, which stays where it was on
/// the timeline. A factor below one zooms in
pub fn zoom(view: (f32, f32), extent: (f32, f32), anchor: f32, factor: f32) -> (f32, f32) {
    let Some(narrowest) = narrowest(extent) else {
        return extent;
    };
    if !factor.is_finite() || factor <= 0. || !anchor.is_finite() {
        return clamp_view(view, extent);
    }

    // Where the anchor sits across the view, which the new view keeps
    let span = view.1 - view.0;
    let share = if span > 0. {
        ((anchor - view.0) / span).clamp(0., 1.)
    } else {
        0.5
    };

    let length = extent.1 - extent.0;
    let wanted = (span * factor).clamp(narrowest, length);
    let start = anchor - wanted * share;
    clamp_view((start, start + wanted), extent)
}

/// `view` moved later by `seconds`, or earlier for a negative amount
pub fn pan(view: (f32, f32), extent: (f32, f32), seconds: f32) -> (f32, f32) {
    if !seconds.is_finite() {
        return clamp_view(view, extent);
    }

    clamp_view((view.0 + seconds, view.1 + seconds), extent)
}

/// `view` at the same width with `time` in its middle, as far as the
/// recording allows
pub fn centre_on(view: (f32, f32), extent: (f32, f32), time: f32) -> (f32, f32) {
    let half = (view.1 - view.0) / 2.;
    clamp_view((time - half, time + half), extent)
}

/// `view` moved on by a page once the playhead at `time` has left it, so
/// the playhead shows again near the edge it comes in from
pub fn page_to(view: (f32, f32), extent: (f32, f32), time: f32, backward: bool) -> (f32, f32) {
    if time >= view.0 && time <= view.1 {
        return view;
    }

    let span = view.1 - view.0;
    let lead = span * PAGE_LEAD;
    let start = if backward {
        time - span + lead
    } else {
        time - lead
    };
    clamp_view((start, start + span), extent)
}

/// From times to points across a timeline and back
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub view: (f32, f32),
    /// Where the view starts on screen, and how wide it is drawn
    pub left: f32,
    pub width: f32,
}

impl Transform {
    /// Seconds a point across stands for
    pub fn seconds_per_point(&self) -> f32 {
        let span = (self.view.1 - self.view.0).max(f32::EPSILON);
        span / self.width.max(1.)
    }

    pub fn x(&self, time: f32) -> f32 {
        self.left + (time - self.view.0) / self.seconds_per_point()
    }

    pub fn time(&self, x: f32) -> f32 {
        self.view.0 + (x - self.left) * self.seconds_per_point()
    }
}

/// What a point on a timeline is over, in the order a press there takes
/// them
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    ClipEdge {
        clip: usize,
        edge: Edge,
    },
    /// The ruler or the playhead, which drag the playhead
    Scrub,
    /// A shown event's marker
    Marker,
    /// A clip's band in the clip strip
    ClipBody(usize),
    Background,
}

/// The parts of a timeline a point can be over
pub struct Surface<'a> {
    pub transform: Transform,
    /// The heights of the ruler, the markers and the clip strip. The player
    /// lanes have neither a ruler nor a strip
    pub ruler: Option<Rangef>,
    pub markers: Rangef,
    pub strip: Option<Rangef>,
    /// Each clip's start and end in seconds
    pub clips: &'a [(f32, f32)],
    pub chosen_clip: Option<usize>,
    pub playhead: Option<f32>,
}

/// What `point` is over. `on_marker` says whether a shown event's marker is
/// within reach of it, which only the caller can tell
pub fn hit(surface: &Surface, point: Pos2, on_marker: bool) -> Target {
    if let Some((clip, edge)) = clip_edge(surface, point.x) {
        return Target::ClipEdge { clip, edge };
    }

    let x = |time: f32| surface.transform.x(time);
    let on_ruler = surface.ruler.is_some_and(|r| r.contains(point.y));
    let on_playhead = surface
        .playhead
        .is_some_and(|t| (x(t) - point.x).abs() <= EDGE_SLOP);
    if on_ruler || on_playhead {
        return Target::Scrub;
    }

    if on_marker && surface.markers.contains(point.y) {
        return Target::Marker;
    }

    let in_strip = surface.strip.is_some_and(|r| r.contains(point.y));
    if in_strip && let Some(clip) = clip_body(surface, point.x) {
        return Target::ClipBody(clip);
    }

    Target::Background
}

/// The clip edge nearest `x` within reach. Where edges of the chosen clip
/// and another are both in reach, the chosen clip's is taken
fn clip_edge(surface: &Surface, x: f32) -> Option<(usize, Edge)> {
    let mut best: Option<(usize, Edge, f32)> = None;
    for (clip, (start, end)) in surface.clips.iter().enumerate() {
        for (edge, time) in [(Edge::Start, *start), (Edge::End, *end)] {
            let distance = (surface.transform.x(time) - x).abs();
            if distance > EDGE_SLOP {
                continue;
            }

            let chosen = surface.chosen_clip == Some(clip);
            let better = match best {
                None => true,
                Some((other, _, nearest)) => {
                    let other_chosen = surface.chosen_clip == Some(other);
                    (chosen && !other_chosen) || (chosen == other_chosen && distance < nearest)
                }
            };
            if better {
                best = Some((clip, edge, distance));
            }
        }
    }

    best.map(|(clip, edge, _)| (clip, edge))
}

/// The clip whose band takes in `x`, the chosen clip first and otherwise
/// the one drawn last, on top
fn clip_body(surface: &Surface, x: f32) -> Option<usize> {
    let time = surface.transform.time(x);
    let under = |clip: usize| {
        let (start, end) = surface.clips[clip];
        time >= start && time <= end
    };

    if let Some(chosen) = surface.chosen_clip
        && chosen < surface.clips.len()
        && under(chosen)
    {
        return Some(chosen);
    }

    (0..surface.clips.len()).rev().find(|&clip| under(clip))
}

/// The side a kill's killer was on, for its colour. Only a team game has
/// sides, and only known ones are told apart
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Unknown,
    Red,
    Blue,
    /// Another team, by one of `OTHER_SIDES` colours that stays the same
    /// for the same team name
    Other(u8),
}

/// How many colours teams other than red and blue share
pub const OTHER_SIDES: u8 = 4;

/// The side of a team by its name
pub fn side_of(team: Option<&str>) -> Side {
    let Some(team) = team.map(str::trim).filter(|t| !t.is_empty()) else {
        return Side::Unknown;
    };

    let team = team.to_lowercase();
    match team.as_str() {
        "red" => Side::Red,
        "blue" => Side::Blue,
        _ => {
            // A small hash of the name, the same on every run
            let mut hash: u32 = 2_166_136_261;
            for byte in team.bytes() {
                hash = (hash ^ u32::from(byte)).wrapping_mul(16_777_619);
            }
            Side::Other((hash % u32::from(OTHER_SIDES)) as u8)
        }
    }
}

/// What the compact timeline draws of one shown event, worked out with the
/// shown rows so drawing never reads the events again
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowMark {
    /// Index into the model's events
    pub event: usize,
    pub segment: u32,
    pub time: f32,
    pub kind: MarkKind,
    pub weapon: Option<Weapon>,
    pub side: Side,
    /// A kill known to be by a hit to the head
    pub headshot: bool,
}

/// The marks within `tolerance` seconds of `time`, in time order
pub fn marks_near(marks: &[RowMark], time: f32, tolerance: f32) -> &[RowMark] {
    let from = marks.partition_point(|m| m.time < time - tolerance);
    let to = marks.partition_point(|m| m.time <= time + tolerance);
    &marks[from..to.max(from)]
}

/// Shown events of one kind sharing a few points of the timeline, drawn as
/// one marker
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkerBin {
    /// The bin's first moment, its marker drawn at the bin's middle
    pub start: f32,
    pub kind: MarkKind,
    pub count: usize,
    /// The weapon and side every event in the bin shares, if any
    pub weapon: Option<Weapon>,
    pub side: Side,
    /// Any of its kills is known to be a headshot
    pub headshot: bool,
}

/// The marks within `view` grouped into bins `width` seconds wide, one per
/// kind and bin, in time order. Bins are counted from the recording's
/// start rather than the view's, so moving the view leaves them in place
pub fn marker_bins(marks: &[RowMark], view: (f32, f32), width: f32) -> Vec<MarkerBin> {
    let width = width.max(f32::EPSILON);
    let from = marks.partition_point(|m| m.time < view.0);
    let to = marks.partition_point(|m| m.time <= view.1).max(from);

    let mut bins: Vec<MarkerBin> = Vec::new();
    for mark in &marks[from..to] {
        let start = (mark.time / width).floor() * width;

        // Bins of the same start are the last ones made, one per kind
        let mut found = None;
        for (index, bin) in bins.iter().enumerate().rev() {
            if bin.start != start {
                break;
            }
            if bin.kind == mark.kind {
                found = Some(index);
                break;
            }
        }

        let Some(index) = found else {
            bins.push(MarkerBin {
                start,
                kind: mark.kind,
                count: 1,
                weapon: mark.weapon,
                side: mark.side,
                headshot: mark.headshot,
            });
            continue;
        };

        let bin = &mut bins[index];
        bin.count += 1;
        bin.headshot |= mark.headshot;
        if bin.weapon != mark.weapon {
            bin.weapon = None;
        }
        if bin.side != mark.side {
            bin.side = Side::Unknown;
        }
    }

    bins
}

/// Bins across the whole recording at the finest level of density, and how
/// many coarser levels, each with half the bins of the one before
const DENSITY_BINS: usize = 4096;
const DENSITY_LEVELS: usize = 7;

/// How many shown events fall in each stretch of the recording, counted
/// once each however many lanes mark them. Coarser levels sum finer ones,
/// so a wide view reads a few bins rather than every event
#[derive(Clone, Debug, PartialEq)]
pub struct Density {
    extent: (f32, f32),
    /// Counts per bin, the finest level first
    levels: Vec<Vec<u32>>,
    /// The busiest bin of each level, which heights are measured against
    peaks: Vec<u32>,
    /// Counts at the finest level for each session, as sessions can reuse
    /// the same times
    sessions: Vec<(u32, Vec<u32>)>,
}

/// One bar of density, its start and its height against the busiest bin
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub start: f32,
    pub end: f32,
    pub share: f32,
}

impl Density {
    /// The density of `marks` across `extent`, none for a recording with no
    /// length
    pub fn new(marks: &[RowMark], extent: (f32, f32)) -> Option<Self> {
        narrowest(extent)?;

        let mut finest = vec![0u32; DENSITY_BINS];
        let mut sessions: Vec<(u32, Vec<u32>)> = Vec::new();
        for mark in marks {
            let bin = bin_of(extent, DENSITY_BINS, mark.time);
            finest[bin] += 1;

            let at = match sessions.iter().position(|(s, _)| *s == mark.segment) {
                Some(at) => at,
                None => {
                    sessions.push((mark.segment, vec![0; DENSITY_BINS]));
                    sessions.len() - 1
                }
            };
            sessions[at].1[bin] += 1;
        }

        let mut levels = vec![finest];
        while levels.len() < DENSITY_LEVELS {
            let finer = &levels[levels.len() - 1];
            let mut coarser = Vec::with_capacity(finer.len() / 2);
            for pair in finer.chunks(2) {
                coarser.push(pair.iter().sum());
            }
            levels.push(coarser);
        }

        let peaks = levels
            .iter()
            .map(|level| level.iter().copied().max().unwrap_or(0))
            .collect();

        Some(Self {
            extent,
            levels,
            peaks,
            sessions,
        })
    }

    /// The finest level whose bins are at least `least_points` wide when a
    /// point stands for `seconds_per_point`, the coarsest when none is
    pub fn level_for(&self, seconds_per_point: f32, least_points: f32) -> usize {
        for level in 0..self.levels.len() {
            let points = self.bin_seconds(level) / seconds_per_point.max(f32::EPSILON);
            if points >= least_points {
                return level;
            }
        }

        self.levels.len() - 1
    }

    fn bin_seconds(&self, level: usize) -> f32 {
        (self.extent.1 - self.extent.0) / self.levels[level].len() as f32
    }

    /// The bars of `level` within `view` that hold any events
    pub fn bars(&self, level: usize, view: (f32, f32)) -> Vec<Bar> {
        let counts = &self.levels[level];
        let peak = self.peaks[level].max(1) as f32;
        let seconds = self.bin_seconds(level);
        let first = bin_of(self.extent, counts.len(), view.0);
        let last = bin_of(self.extent, counts.len(), view.1);

        let mut bars = Vec::new();
        for (bin, &count) in counts.iter().enumerate().take(last + 1).skip(first) {
            if count == 0 {
                continue;
            }

            let start = self.extent.0 + bin as f32 * seconds;
            bars.push(Bar {
                start,
                end: start + seconds,
                share: count as f32 / peak,
            });
        }
        bars
    }

    /// How many events the bin of `level` at `time` holds, and how many of
    /// them each session holds
    pub fn at(&self, level: usize, time: f32) -> (u32, Vec<(u32, u32)>) {
        let bins = self.levels[level].len();
        let bin = bin_of(self.extent, bins, time);
        let total = self.levels[level][bin];

        // A bin of this level covers this many of the finest
        let width = DENSITY_BINS / bins;
        let finest = bin * width..(bin + 1) * width;
        let mut sessions = Vec::new();
        for (segment, counts) in &self.sessions {
            let count: u32 = counts[finest.clone()].iter().sum();
            if count > 0 {
                sessions.push((*segment, count));
            }
        }

        (total, sessions)
    }
}

/// The bin of `bins` across `extent` holding `time`, times outside it in
/// the first or last bin
fn bin_of(extent: (f32, f32), bins: usize, time: f32) -> usize {
    let length = extent.1 - extent.0;
    let share = ((time - extent.0) / length).clamp(0., 1.);
    let bin = (share * bins as f32) as usize;
    bin.min(bins - 1)
}

#[cfg(test)]
mod tests {
    use eframe::egui::pos2;

    use super::*;

    const EXTENT: (f32, f32) = (0., 100.);

    #[test]
    fn views_stay_within_the_recording() {
        assert_eq!(clamp_view((-10., 20.), EXTENT), (0., 30.));
        assert_eq!(clamp_view((90., 120.), EXTENT), (70., 100.));
        assert_eq!(clamp_view((-50., 500.), EXTENT), EXTENT);

        // No narrower than half a second, and a view of no numbers is the
        // whole recording
        assert_eq!(clamp_view((10., 10.1), EXTENT), (10., 10.5));
        assert_eq!(clamp_view((f32::NAN, 3.), EXTENT), EXTENT);
        assert_eq!(clamp_view((0., f32::INFINITY), EXTENT), EXTENT);
    }

    #[test]
    fn a_short_recording_is_shown_whole_from_its_start() {
        let short = (0., 0.2);
        assert_eq!(narrowest(short), Some(0.2));
        assert_eq!(clamp_view((-1., 5.), short), short);
        assert_eq!(zoom(short, short, 0.1, 0.5), short);

        // A recording with no length shows nothing to seek in
        assert_eq!(narrowest((0., 0.)), None);
        assert_eq!(narrowest((0., f32::NAN)), None);
    }

    #[test]
    fn zooming_keeps_the_anchor_in_place() {
        let view = zoom((20., 60.), EXTENT, 30., 0.5);
        assert_eq!(view, (25., 45.));

        // Out past the whole recording stops at the whole recording
        assert_eq!(zoom((20., 60.), EXTENT, 30., 10.), EXTENT);

        // In past the narrowest view stops there
        let narrow = zoom((20., 21.), EXTENT, 20.5, 0.01);
        assert!((narrow.1 - narrow.0 - MIN_SPAN).abs() < 1e-5);

        // A factor that is not a number leaves the view
        assert_eq!(zoom((20., 60.), EXTENT, 30., f32::NAN), (20., 60.));
    }

    #[test]
    fn panning_and_centring_stop_at_the_ends() {
        assert_eq!(pan((10., 30.), EXTENT, -50.), (0., 20.));
        assert_eq!(pan((10., 30.), EXTENT, 5.), (15., 35.));
        assert_eq!(pan((10., 30.), EXTENT, f32::NAN), (10., 30.));
        assert_eq!(centre_on((10., 30.), EXTENT, 95.), (80., 100.));
    }

    #[test]
    fn playing_out_of_view_turns_a_page() {
        assert_eq!(page_to((10., 30.), EXTENT, 20., false), (10., 30.));
        assert_eq!(page_to((10., 30.), EXTENT, 31., false), (29., 49.));
        assert_eq!(page_to((50., 70.), EXTENT, 45., true), (27., 47.));

        // A page never reaches past the start
        assert_eq!(page_to((10., 30.), EXTENT, 9., true), (0., 20.));
    }

    #[test]
    fn transforms_go_back_and_forth() {
        let t = Transform {
            view: (10., 20.),
            left: 100.,
            width: 500.,
        };
        assert_eq!(t.x(10.), 100.);
        assert_eq!(t.x(20.), 600.);
        assert!((t.time(t.x(13.7)) - 13.7).abs() < 1e-4);

        // A view or width of nothing never divides by zero
        let empty = Transform {
            view: (5., 5.),
            left: 0.,
            width: 0.,
        };
        assert!(empty.x(6.).is_finite());
    }

    fn surface(clips: &[(f32, f32)], chosen: Option<usize>) -> Surface<'_> {
        Surface {
            transform: Transform {
                view: EXTENT,
                left: 0.,
                width: 1000.,
            },
            ruler: Some(Rangef::new(0., 18.)),
            markers: Rangef::new(30., 60.),
            strip: Some(Rangef::new(60., 76.)),
            clips,
            chosen_clip: chosen,
            playhead: Some(50.),
        }
    }

    #[test]
    fn presses_take_the_first_target_in_order() {
        let clips = [(20., 30.)];
        let s = surface(&clips, None);

        // A clip edge wins over the ruler it crosses
        let edge = Target::ClipEdge {
            clip: 0,
            edge: Edge::Start,
        };
        assert_eq!(hit(&s, pos2(203., 5.), true), edge);

        // The ruler and the playhead scrub, ahead of a marker
        assert_eq!(hit(&s, pos2(100., 5.), true), Target::Scrub);
        assert_eq!(hit(&s, pos2(504., 40.), true), Target::Scrub);

        assert_eq!(hit(&s, pos2(100., 40.), true), Target::Marker);
        assert_eq!(hit(&s, pos2(100., 40.), false), Target::Background);

        // A clip's band takes a press only in the strip
        assert_eq!(hit(&s, pos2(250., 70.), false), Target::ClipBody(0));
        assert_eq!(hit(&s, pos2(250., 40.), false), Target::Background);
    }

    #[test]
    fn the_chosen_clip_wins_a_shared_edge() {
        let clips = [(20., 30.), (30., 40.)];
        let first = Target::ClipEdge {
            clip: 0,
            edge: Edge::End,
        };
        let second = Target::ClipEdge {
            clip: 1,
            edge: Edge::Start,
        };
        assert_eq!(
            hit(&surface(&clips, Some(1)), pos2(299., 70.), false),
            second
        );
        assert_eq!(
            hit(&surface(&clips, Some(0)), pos2(301., 70.), false),
            first
        );

        // With neither chosen, the nearer edge
        let apart = [(20., 30.), (30.8, 40.)];
        assert_eq!(hit(&surface(&apart, None), pos2(305., 70.), false), second);
    }

    fn mark(event: usize, segment: u32, time: f32, kind: MarkKind) -> RowMark {
        RowMark {
            event,
            segment,
            time,
            kind,
            weapon: Some(Weapon::Gauss),
            side: Side::Unknown,
            headshot: false,
        }
    }

    #[test]
    fn crowded_marks_share_a_bin_per_kind() {
        let mut marks = vec![
            mark(0, 0, 1.1, MarkKind::Shot),
            mark(1, 0, 1.2, MarkKind::Kill),
            mark(2, 0, 1.3, MarkKind::Shot),
            mark(3, 0, 5.5, MarkKind::Kill),
        ];
        marks[1].side = Side::Red;
        marks[2].weapon = Some(Weapon::Mp5);

        let bins = marker_bins(&marks, (0., 10.), 1.);
        let summary: Vec<_> = bins.iter().map(|b| (b.start, b.kind, b.count)).collect();
        assert_eq!(
            summary,
            [
                (1., MarkKind::Shot, 2),
                (1., MarkKind::Kill, 1),
                (5., MarkKind::Kill, 1)
            ]
        );

        // Mixed weapons leave no single weapon, a lone kill keeps its side
        assert_eq!(bins[0].weapon, None);
        assert_eq!(bins[1].side, Side::Red);

        // Marks outside the view are left out
        assert_eq!(marker_bins(&marks, (2., 4.), 1.), []);
    }

    #[test]
    fn density_counts_each_shown_event_once_per_session() {
        let marks = vec![
            mark(0, 0, 10., MarkKind::Kill),
            mark(1, 0, 10.01, MarkKind::Shot),
            mark(2, 1, 10.02, MarkKind::Kill),
            mark(3, 0, 90., MarkKind::Kill),
        ];
        let density = Density::new(&marks, EXTENT).unwrap();

        // The coarsest level puts the first three in one bin
        let coarsest = DENSITY_LEVELS - 1;
        let (total, sessions) = density.at(coarsest, 10.);
        assert_eq!(total, 3);
        assert_eq!(sessions, [(0, 2), (1, 1)]);

        // Heights are against the busiest bin of the whole demo
        let bars = density.bars(coarsest, EXTENT);
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].share, 1.);
        assert!((bars[1].share - 1. / 3.).abs() < 1e-6);

        // No length, no density
        assert!(Density::new(&marks, (0., 0.)).is_none());
    }

    #[test]
    fn density_reads_coarser_levels_for_wider_views() {
        let density = Density::new(&[], EXTENT).unwrap();

        // The finest bins are 100 / 4096 seconds wide
        assert_eq!(density.level_for(0.001, 2.), 0);
        assert_eq!(density.level_for(1000., 2.), DENSITY_LEVELS - 1);
        let level = density.level_for(0.05, 2.);
        assert!(density.bin_seconds(level) / 0.05 >= 2.);
        assert!(density.bin_seconds(level - 1) / 0.05 < 2.);
    }

    #[test]
    fn team_names_give_steady_sides() {
        assert_eq!(side_of(Some("Red")), Side::Red);
        assert_eq!(side_of(Some(" blue ")), Side::Blue);
        assert_eq!(side_of(Some("")), Side::Unknown);
        assert_eq!(side_of(None), Side::Unknown);
        assert_eq!(side_of(Some("Robo")), side_of(Some("robo")));
    }
}
