//! 3D Linear algebra math primitives: vectors, matrices, and transformations.

use std::ops::{Add, Mul, Neg, Sub};

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

impl Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
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

    /// Rotation matrix around Z axis.
    #[must_use]
    pub fn rotation_z(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        let mut m = Self::IDENTITY;
        m.cols[0][0] = cos;
        m.cols[0][1] = sin;
        m.cols[1][0] = -sin;
        m.cols[1][1] = cos;
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

    /// Perspective projection matrix with Reversed-Z depth mapping (near -> 1.0, far -> 0.0).
    ///
    /// Distributes floating-point precision where it is needed most at long view distances,
    /// eliminating Z-fighting on fine CAD assemblies and architectural scenes.
    #[must_use]
    pub fn perspective_reversed_z(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_radians * 0.5).tan();
        let d = far - near;
        let a = near / d;
        let b = (far * near) / d;

        Self {
            cols: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, a, -1.0],
                [0.0, 0.0, b, 0.0],
            ],
        }
    }

    /// Right-handed orthographic projection matrix (`-Z` forward matching [`Mat4::look_at`]),
    /// mapping view-space depth `[-near, -far]` into `[0.0, 1.0]`.
    #[must_use]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let r_l = right - left;
        let t_b = top - bottom;
        let f_n = far - near;

        Self {
            cols: [
                [2.0 / r_l, 0.0, 0.0, 0.0],
                [0.0, 2.0 / t_b, 0.0, 0.0],
                [0.0, 0.0, -1.0 / f_n, 0.0],
                [
                    -(right + left) / r_l,
                    -(top + bottom) / t_b,
                    -near / f_n,
                    1.0,
                ],
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

    /// Flattens the column-major matrix into a 16-element float array.
    #[must_use]
    pub const fn to_cols_array(&self) -> [f32; 16] {
        [
            self.cols[0][0],
            self.cols[0][1],
            self.cols[0][2],
            self.cols[0][3],
            self.cols[1][0],
            self.cols[1][1],
            self.cols[1][2],
            self.cols[1][3],
            self.cols[2][0],
            self.cols[2][1],
            self.cols[2][2],
            self.cols[2][3],
            self.cols[3][0],
            self.cols[3][1],
            self.cols[3][2],
            self.cols[3][3],
        ]
    }

    /// Transforms a 3D point (w = 1.0) into homogeneous clip coordinates [x, y, z, w].
    #[must_use]
    pub fn transform_point4(&self, pt: Vec3) -> [f32; 4] {
        let cx = self.cols[0][0] * pt.x
            + self.cols[1][0] * pt.y
            + self.cols[2][0] * pt.z
            + self.cols[3][0];
        let cy = self.cols[0][1] * pt.x
            + self.cols[1][1] * pt.y
            + self.cols[2][1] * pt.z
            + self.cols[3][1];
        let cz = self.cols[0][2] * pt.x
            + self.cols[1][2] * pt.y
            + self.cols[2][2] * pt.z
            + self.cols[3][2];
        let cw = self.cols[0][3] * pt.x
            + self.cols[1][3] * pt.y
            + self.cols[2][3] * pt.z
            + self.cols[3][3];
        [cx, cy, cz, cw]
    }

    /// Transforms a 3D point (w = 1.0) and performs perspective divide.
    #[must_use]
    pub fn transform_point(&self, pt: Vec3) -> Vec3 {
        let [cx, cy, cz, cw] = self.transform_point4(pt);
        if cw.abs() > 0.00001 {
            let inv = 1.0 / cw;
            Vec3::new(cx * inv, cy * inv, cz * inv)
        } else {
            Vec3::new(cx, cy, cz)
        }
    }

    /// Transforms a 3D direction vector (w = 0.0, ignoring translation).
    #[must_use]
    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        let x = self.cols[0][0] * v.x + self.cols[1][0] * v.y + self.cols[2][0] * v.z;
        let y = self.cols[0][1] * v.x + self.cols[1][1] * v.y + self.cols[2][1] * v.z;
        let z = self.cols[0][2] * v.x + self.cols[1][2] * v.y + self.cols[2][2] * v.z;
        Vec3::new(x, y, z)
    }
}

/// Lossless `usize` to `f32` conversion for bounded viewport dimensions and mesh counts.
#[inline]
pub(crate) fn count_to_f32(count: usize) -> f32 {
    f32::from(u16::try_from(count).unwrap_or(u16::MAX))
}
