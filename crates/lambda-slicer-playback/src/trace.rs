//! A point traced through the map's world, as the client traces its
//! temporary entities (implementation plan §13.6)
//!
//! The client moves a temporary entity that collides with the world along a
//! line each frame and stops it where the line first meets a solid leaf of
//! the world's point hull, the BSP's own node tree (`PM_PlayerTrace` with
//! the point hull and only the world). The trace is Quake's
//! `SV_RecursiveHullCheck`, which keeps a point a thirty-second of a unit
//! off the plane it meets. Brush entities are not traced, as the client's
//! world-only trace leaves them out

/// `CONTENTS_SOLID`
pub const SOLID: i32 = -2;

/// How far a trace keeps from the plane it meets (`DIST_EPSILON`)
const DIST_EPSILON: f32 = 0.03125;

/// The world's point hull
#[derive(Clone, Debug, PartialEq)]
pub struct Hull {
    /// Each plane's normal and distance from the origin
    pub planes: Vec<([f32; 3], f32)>,
    /// Each node's plane and its children, a node's index, or for a leaf
    /// its contents
    pub nodes: Vec<(usize, [Child; 2])>,
    pub root: Child,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Child {
    Node(usize),
    Contents(i32),
}

/// Where a trace stopped
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trace {
    /// How far along the line, 1 when it met nothing
    pub fraction: f32,
    pub end: [f32; 3],
    /// The normal of the plane it met, facing the side it came from
    pub normal: [f32; 3],
    /// It began in a solid leaf
    pub start_solid: bool,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn lerp(a: [f32; 3], b: [f32; 3], f: f32) -> [f32; 3] {
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * f)
}

impl Hull {
    /// The contents of the leaf holding `point`
    pub fn contents(&self, point: [f32; 3]) -> i32 {
        self.contents_from(self.root, point)
    }

    /// The contents of the leaf holding `point`, from `at` down
    fn contents_from(&self, mut at: Child, point: [f32; 3]) -> i32 {
        let mut steps = 0;
        while let Child::Node(i) = at {
            let Some((plane, children)) = self.nodes.get(i) else {
                return SOLID;
            };
            let (normal, dist) = self.planes.get(*plane).copied().unwrap_or(([0.; 3], 0.));
            at = children[usize::from(dot(normal, point) - dist < 0.)];
            // A malformed tree that loops counts as solid
            steps += 1;
            if steps > self.nodes.len() {
                return SOLID;
            }
        }
        match at {
            Child::Contents(c) => c,
            Child::Node(_) => SOLID,
        }
    }

    /// Trace the line from `start` to `end`
    pub fn trace(&self, start: [f32; 3], end: [f32; 3]) -> Trace {
        let mut trace = Trace {
            fraction: 1.,
            end,
            normal: [0.; 3],
            start_solid: false,
        };
        let mut all_solid = true;
        self.check(
            self.root,
            (0., 1.),
            (start, end),
            &mut trace,
            &mut all_solid,
            0,
        );
        if all_solid {
            trace.start_solid = true;
        }
        if trace.start_solid {
            trace.fraction = 0.;
            trace.end = start;
        }
        trace
    }

    /// `SV_RecursiveHullCheck`. False once the line has met something
    fn check(
        &self,
        at: Child,
        (f1, f2): (f32, f32),
        (p1, p2): ([f32; 3], [f32; 3]),
        trace: &mut Trace,
        all_solid: &mut bool,
        depth: usize,
    ) -> bool {
        let i = match at {
            Child::Contents(c) => {
                if c == SOLID {
                    trace.start_solid = true;
                } else {
                    *all_solid = false;
                }
                return true;
            }
            Child::Node(i) => i,
        };
        let Some((plane, children)) = self.nodes.get(i).copied() else {
            return true;
        };
        if depth > self.nodes.len() {
            return true;
        }
        let (normal, dist) = self.planes.get(plane).copied().unwrap_or(([0.; 3], 0.));
        let t1 = dot(normal, p1) - dist;
        let t2 = dot(normal, p2) - dist;
        if t1 >= 0. && t2 >= 0. {
            return self.check(children[0], (f1, f2), (p1, p2), trace, all_solid, depth + 1);
        }
        if t1 < 0. && t2 < 0. {
            return self.check(children[1], (f1, f2), (p1, p2), trace, all_solid, depth + 1);
        }

        // The line crosses the plane, so it is split a little short of it
        let mut frac = if t1 < 0. {
            (t1 + DIST_EPSILON) / (t1 - t2)
        } else {
            (t1 - DIST_EPSILON) / (t1 - t2)
        };
        frac = frac.clamp(0., 1.);
        let mut mid_f = f1 + (f2 - f1) * frac;
        let mut mid = lerp(p1, p2, frac);
        let side = usize::from(t1 < 0.);

        if !self.check(
            children[side],
            (f1, mid_f),
            (p1, mid),
            trace,
            all_solid,
            depth + 1,
        ) {
            return false;
        }
        if self.contents_from(children[side ^ 1], mid) != SOLID {
            return self.check(
                children[side ^ 1],
                (mid_f, f2),
                (mid, p2),
                trace,
                all_solid,
                depth + 1,
            );
        }
        if *all_solid {
            return false;
        }

        // The far side is solid, so the line stops here
        trace.normal = if side == 0 {
            normal
        } else {
            normal.map(|n| -n)
        };
        while self.contents(mid) == SOLID {
            // Backed up out of the solid, which rounding can leave it in
            frac -= 0.1;
            if frac < 0. {
                trace.fraction = mid_f;
                trace.end = mid;
                return false;
            }
            mid_f = f1 + (f2 - f1) * frac;
            mid = lerp(p1, p2, frac);
        }
        trace.fraction = mid_f;
        trace.end = mid;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A floor at height 0 with open space above and solid below, and a
    /// wall at x = 100 beyond which everything is solid
    fn room() -> Hull {
        Hull {
            planes: vec![([0., 0., 1.], 0.), ([-1., 0., 0.], -100.)],
            nodes: vec![
                (0, [Child::Node(1), Child::Contents(SOLID)]),
                (1, [Child::Contents(-1), Child::Contents(SOLID)]),
            ],
            root: Child::Node(0),
        }
    }

    #[test]
    fn a_line_stops_short_of_the_floor_and_the_wall() {
        let hull = room();
        assert_eq!(hull.contents([0., 0., 10.]), -1);
        assert_eq!(hull.contents([0., 0., -10.]), SOLID);
        assert_eq!(hull.contents([150., 0., 10.]), SOLID);

        let down = hull.trace([0., 0., 10.], [0., 0., -10.]);
        assert!((down.fraction - (10. - DIST_EPSILON) / 20.).abs() < 1e-6);
        assert_eq!(down.normal, [0., 0., 1.]);
        assert!(down.end[2] > 0.);

        let across = hull.trace([50., 0., 10.], [150., 0., 10.]);
        assert_eq!(across.normal, [-1., 0., 0.]);
        assert!(across.end[0] < 100. && across.end[0] > 99.9);

        let clear = hull.trace([0., 0., 10.], [50., 0., 20.]);
        assert_eq!(clear.fraction, 1.);
        assert!(!clear.start_solid);

        let inside = hull.trace([0., 0., -10.], [0., 0., 10.]);
        assert!(inside.start_solid);
    }
}
