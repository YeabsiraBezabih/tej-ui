//! 3D Linear algebra math primitives: vectors, matrices, and transformations.

use std::ops::{Add, Mul, Sub};

/// 3D floating-point vector.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const UP: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.00001 {
            let inv = 1.0 / len;
            Self {
                x: self.x * inv,
                y: self.y * inv,
                z: self.z * inv,
            }
        } else {
            Self::ZERO
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// 4x4 Transformation Matrix (Column-major).
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Mat4 {
    pub cols: [[f32; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Translation matrix.
    #[must_use]
    pub fn translation(offset: Vec3) -> Self {
        let mut m = Self::IDENTITY;
        m.cols[3][0] = offset.x;
        m.cols[3][1] = offset.y;
        m.cols[3][2] = offset.z;
        m
    }

    /// Scaling matrix.
    #[must_use]
    pub const fn scale(factor: Vec3) -> Self {
        Self {
            cols: [
                [factor.x, 0.0, 0.0, 0.0],
                [0.0, factor.y, 0.0, 0.0],
                [0.0, 0.0, factor.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Rotation matrix around Y axis.
    #[must_use]
    pub fn rotation_y(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        let mut m = Self::IDENTITY;
        m.cols[0][0] = cos;
        m.cols[0][2] = -sin;
        m.cols[2][0] = sin;
        m.cols[2][2] = cos;
        m
    }

    /// Rotation matrix around X axis.
    #[must_use]
    pub fn rotation_x(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        let mut m = Self::IDENTITY;
        m.cols[1][1] = cos;
        m.cols[1][2] = sin;
        m.cols[2][1] = -sin;
        m.cols[2][2] = cos;
        m
    }

    /// Perspective projection matrix (OpenGL / WebGPU compatible).
    #[must_use]
    pub fn perspective(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_radians * 0.5).tan();
        let range_inv = 1.0 / (near - far);

        Self {
            cols: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (near + far) * range_inv, -1.0],
                [0.0, 0.0, 2.0 * near * far * range_inv, 0.0],
            ],
        }
    }

    /// Look-at view matrix.
    #[must_use]
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let z_axis = (eye - target).normalize();
        let x_axis = up.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis);

        Self {
            cols: [
                [x_axis.x, y_axis.x, z_axis.x, 0.0],
                [x_axis.y, y_axis.y, z_axis.y, 0.0],
                [x_axis.z, y_axis.z, z_axis.z, 0.0],
                [-x_axis.dot(eye), -y_axis.dot(eye), -z_axis.dot(eye), 1.0],
            ],
        }
    }

    /// Multiplies two 4x4 matrices.
    #[must_use]
    pub fn mul_mat(&self, rhs: &Self) -> Self {
        let mut out = [[0.0; 4]; 4];
        for (c, out_col) in out.iter_mut().enumerate() {
            for (r, out_val) in out_col.iter_mut().enumerate() {
                *out_val = self.cols[0][r] * rhs.cols[c][0]
                    + self.cols[1][r] * rhs.cols[c][1]
                    + self.cols[2][r] * rhs.cols[c][2]
                    + self.cols[3][r] * rhs.cols[c][3];
            }
        }
        Self { cols: out }
    }

    /// Transforms a 3D point (w = 1.0) and performs perspective divide.
    #[must_use]
    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let x =
            self.cols[0][0] * p.x + self.cols[1][0] * p.y + self.cols[2][0] * p.z + self.cols[3][0];
        let y =
            self.cols[0][1] * p.x + self.cols[1][1] * p.y + self.cols[2][1] * p.z + self.cols[3][1];
        let z =
            self.cols[0][2] * p.x + self.cols[1][2] * p.y + self.cols[2][2] * p.z + self.cols[3][2];
        let w =
            self.cols[0][3] * p.x + self.cols[1][3] * p.y + self.cols[2][3] * p.z + self.cols[3][3];

        if w.abs() > 0.00001 {
            let inv = 1.0 / w;
            Vec3::new(x * inv, y * inv, z * inv)
        } else {
            Vec3::new(x, y, z)
        }
    }
}
