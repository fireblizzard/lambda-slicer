//! A camera of the demo player's own, flown like the game's noclip while
//! the demo plays on or stands paused
//!
//! W, A, S and D move along the view and to its sides, Space up and Ctrl
//! down, and Shift faster. Holding the right mouse button and dragging
//! turns the view. It starts where the recording player's view is. Angles
//! are the game's, pitch, yaw and roll in degrees, a positive pitch looking
//! down

/// Units a second the camera flies, the game's usual top running speed,
/// and how many times faster with Shift
const SPEED: f32 = 320.;
const FAST: f32 = 3.;

/// Degrees the view turns for each point the pointer is dragged, and the
/// most it looks up or down
const TURN: f32 = 0.2;
const PITCH_LIMIT: f32 = 89.;

/// Where the free camera is and where it looks
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeCamera {
    pub origin: [f32; 3],
    pub angles: [f32; 3],
}

/// The movement keys held
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Held {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub fast: bool,
}

impl Held {
    /// Whether any key moves the camera
    pub fn moving(&self) -> bool {
        self.forward || self.back || self.left || self.right || self.up || self.down
    }
}

impl FreeCamera {
    /// A camera at `origin` looking along `angles`, level from side to side
    pub fn new(origin: [f32; 3], angles: [f32; 3]) -> Self {
        let pitch = angles[0].clamp(-PITCH_LIMIT, PITCH_LIMIT);
        Self {
            origin,
            angles: [pitch, angles[1], 0.],
        }
    }

    /// Flown for `seconds` with the keys `held`. Forward and back follow
    /// the view up and down too, as noclip does, and up and down are the
    /// world's
    pub fn fly(&mut self, held: Held, seconds: f32) {
        let (forward, right) = directions(self.angles);
        let axis = |plus: bool, minus: bool| f32::from(u8::from(plus)) - f32::from(u8::from(minus));
        let ahead = axis(held.forward, held.back);
        let aside = axis(held.right, held.left);
        let rise = axis(held.up, held.down);

        let mut speed = SPEED * seconds;
        if held.fast {
            speed *= FAST;
        }
        for k in 0..3 {
            let up = if k == 2 { rise } else { 0. };
            self.origin[k] += (forward[k] * ahead + right[k] * aside + up) * speed;
        }
    }

    /// Moved `units` straight back from where it looks
    pub fn back_off(&mut self, units: f32) {
        let (forward, _) = directions(self.angles);
        for (k, along) in forward.iter().enumerate() {
            self.origin[k] -= along * units;
        }
    }

    /// Turned by a drag of `delta` points, right turning right and down
    /// looking down
    pub fn turn(&mut self, delta: [f32; 2]) {
        let pitch = self.angles[0] + delta[1] * TURN;
        self.angles[0] = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        self.angles[1] = (self.angles[1] - delta[0] * TURN).rem_euclid(360.);
    }
}

/// The directions straight ahead and to the right of `angles`, the right
/// one level, as the game's `AngleVectors` gives them with no roll
fn directions(angles: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let (pitch, yaw) = (angles[0].to_radians(), angles[1].to_radians());
    let forward = [
        pitch.cos() * yaw.cos(),
        pitch.cos() * yaw.sin(),
        -pitch.sin(),
    ];
    let right = [yaw.sin(), -yaw.cos(), 0.];
    (forward, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 0.01)
    }

    #[test]
    fn keys_fly_along_the_view_and_the_world_up() {
        // Looking along x, a second forward goes 320 units along it
        let mut camera = FreeCamera::new([0.; 3], [0.; 3]);
        let forward = Held {
            forward: true,
            ..Held::default()
        };
        camera.fly(forward, 1.);
        assert!(close(camera.origin, [320., 0., 0.]));

        // Right of x is minus y, and Space rises whatever the view
        let mut camera = FreeCamera::new([0.; 3], [45., 0., 0.]);
        let right_and_up = Held {
            right: true,
            up: true,
            ..Held::default()
        };
        camera.fly(right_and_up, 0.5);
        assert!(close(camera.origin, [0., -160., 160.]));

        // Looking down, forward goes down too, and Shift is faster
        let mut camera = FreeCamera::new([0.; 3], [89., 0., 0.]);
        let fast = Held {
            forward: true,
            fast: true,
            ..Held::default()
        };
        camera.fly(fast, 1.);
        assert!(camera.origin[2] < -950.);
    }

    #[test]
    fn dragging_turns_and_never_looks_past_straight_up_or_down() {
        let mut camera = FreeCamera::new([0.; 3], [0., 10., 30.]);
        assert_eq!(camera.angles, [0., 10., 0.]);

        camera.turn([100., 0.]);
        assert_eq!(camera.angles[1], 350.);

        camera.turn([0., 10_000.]);
        assert_eq!(camera.angles[0], PITCH_LIMIT);
    }
}
