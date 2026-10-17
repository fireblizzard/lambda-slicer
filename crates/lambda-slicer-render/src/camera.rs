//! A view in the game's coordinates, x forward, y left and z up, with
//! angles in degrees as pitch, yaw and roll, turned the way the game's
//! `AngleVectors` turns them

/// A column-major 4 by 4 matrix, as WGSL reads one
pub type Matrix = [[f32; 4]; 4];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    /// The vertical field of view in degrees. The game's 90 degrees across
    /// a 4 by 3 view is about 73.74 degrees high
    pub fov_y: f32,
}

/// The game's forward, right and up for these angles
pub fn angle_vectors(angles: [f32; 3]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let [p, y, r] = angles.map(f32::to_radians);
    let (sp, cp) = p.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sr, cr) = r.sin_cos();
    let forward = [cp * cy, cp * sy, -sp];
    let right = [-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp];
    let up = [cr * sp * cy + sr * sy, cr * sp * sy - sr * cy, cr * cp];
    (forward, right, up)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = [[0.; 4]; 4];
    for (column, out) in out.iter_mut().enumerate() {
        for (row, value) in out.iter_mut().enumerate() {
            *value = (0..4).map(|k| a[k][row] * b[column][k]).sum();
        }
    }
    out
}

impl Camera {
    /// A camera at `eye` turned towards `target`, without roll
    pub fn looking_at(eye: [f32; 3], target: [f32; 3], fov_y: f32) -> Self {
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
        Self {
            origin: eye,
            angles: [
                -d[2].atan2(flat).to_degrees(),
                d[1].atan2(d[0]).to_degrees(),
                0.,
            ],
            fov_y,
        }
    }

    /// World to clip space for a view `aspect` wide per unit high, depth
    /// from 0 at `near` to 1 at `far` as wgpu takes it
    pub fn view_projection(&self, aspect: f32, near: f32, far: f32) -> Matrix {
        let (forward, right, up) = angle_vectors(self.angles);
        let o = self.origin;
        // Camera space with x right, y up and z backwards
        let view: Matrix = [
            [right[0], up[0], -forward[0], 0.],
            [right[1], up[1], -forward[1], 0.],
            [right[2], up[2], -forward[2], 0.],
            [-dot(right, o), -dot(up, o), dot(forward, o), 1.],
        ];
        let f = 1. / (self.fov_y.to_radians() / 2.).tan();
        let projection: Matrix = [
            [f / aspect, 0., 0., 0.],
            [0., f, 0., 0.],
            [0., 0., far / (near - far), -1.],
            [0., 0., near * far / (near - far), 0.],
        ];
        multiply(&projection, &view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(m: &Matrix, p: [f32; 3]) -> [f32; 3] {
        let v = [p[0], p[1], p[2], 1.];
        let mut out = [0.; 4];
        for (row, value) in out.iter_mut().enumerate() {
            *value = (0..4).map(|k| m[k][row] * v[k]).sum();
        }
        [out[0] / out[3], out[1] / out[3], out[2] / out[3]]
    }

    #[test]
    fn yaw_turns_forward_around_up() {
        let (f, r, u) = angle_vectors([0., 90., 0.]);
        let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        assert!(near(f, [0., 1., 0.]));
        assert!(near(r, [1., 0., 0.]));
        assert!(near(u, [0., 0., 1.]));
        // Looking down
        let (f, _, _) = angle_vectors([90., 0., 0.]);
        assert!(near(f, [0., 0., -1.]));
    }

    #[test]
    fn looking_at_a_point_puts_it_in_the_middle() {
        let camera = Camera::looking_at([100., -50., 80.], [0., 0., 0.], 70.);
        let m = camera.view_projection(1., 4., 4096.);
        let p = apply(&m, [0., 0., 0.]);
        assert!(p[0].abs() < 1e-5 && p[1].abs() < 1e-5);
    }

    #[test]
    fn a_point_ahead_lands_in_the_middle_and_in_depth() {
        let camera = Camera {
            origin: [10., 20., 30.],
            angles: [0., 0., 0.],
            fov_y: 73.74,
        };
        let m = camera.view_projection(16. / 9., 4., 4096.);
        let ahead = apply(&m, [110., 20., 30.]);
        assert!(ahead[0].abs() < 1e-5 && ahead[1].abs() < 1e-5);
        assert!(ahead[2] > 0. && ahead[2] < 1.);
        // Left of the view is to the left of the screen, up is up
        assert!(apply(&m, [110., 30., 30.])[0] < 0.);
        assert!(apply(&m, [110., 20., 40.])[1] > 0.);
    }
}
