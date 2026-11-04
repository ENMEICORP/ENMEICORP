//! THE BLACK FOREST // NODE 03 // ANALYTICAL MATH ENGINE
//! Doneware Invariance  - ENMEICORP
//!
//! Analytical vector primitives, unit quaternions, SLERP geodesic interpolation,
//! and continuous Arcball projection with Bell's hyperbolic sheet continuation.

use std::ops::{Add, Mul, Neg, Sub};

pub const EPSILON: f32 = 1e-6;

// =========================================================================
// 1. ANALYTICAL 3D VECTOR
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline(always)]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline(always)]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    #[inline(always)]
    pub fn norm_sq(self) -> f32 {
        self.dot(self)
    }

    #[inline(always)]
    pub fn norm(self) -> f32 {
        self.norm_sq().sqrt()
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        let n = self.norm();
        if n > EPSILON {
            self * (1.0 / n)
        } else {
            Self::ZERO
        }
    }

    #[inline(always)]
    pub fn lerp(self, rhs: Self, t: f32) -> Self {
        self + (rhs - self) * t
    }

    /// Generates a deterministic orthogonal unit vector, eliminating axial singularities.
    pub fn orthogonal(self) -> Self {
        if self.x.abs() < 0.9 {
            Self::new(0.0, -self.z, self.y).normalize()
        } else {
            Self::new(-self.y, self.x, 0.0).normalize()
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

// =========================================================================
// 2. UNIT QUATERNION & HAMILTONIAN PRODUCT
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 };

    #[inline(always)]
    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    pub fn from_axis_angle(axis: Vec3, angle_rad: f32) -> Self {
        let half = angle_rad * 0.5;
        let s = half.sin();
        let norm_axis = axis.normalize();
        Self {
            w: half.cos(),
            x: norm_axis.x * s,
            y: norm_axis.y * s,
            z: norm_axis.z * s,
        }
    }

    /// Computes the minimal arc rotation mapping unit vector `from` onto unit vector `to`.
    pub fn from_rotation_arc(from: Vec3, to: Vec3) -> Self {
        let dot = from.dot(to);

        // Near-parallel alignment
        if dot >= 1.0 - EPSILON {
            return Self::IDENTITY;
        }

        // Antipodal singularity (180-degree reflection)
        if dot <= -1.0 + EPSILON {
            let ortho = from.orthogonal();
            return Self::new(0.0, ortho.x, ortho.y, ortho.z);
        }

        // Direct closed-form construction bypassing transcendental functions
        let cross = from.cross(to);
        let s = ((1.0 + dot) * 2.0).sqrt();
        let inv_s = 1.0 / s;

        Self {
            w: 0.5 * s,
            x: cross.x * inv_s,
            y: cross.y * inv_s,
            z: cross.z * inv_s,
        }.normalize()
    }

    #[inline(always)]
    pub fn norm_sq(self) -> f32 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline(always)]
    pub fn norm(self) -> f32 {
        self.norm_sq().sqrt()
    }

    #[inline(always)]
    pub fn normalize(self) -> Self {
        let n = self.norm();
        if n > EPSILON {
            let inv_n = 1.0 / n;
            Self {
                w: self.w * inv_n,
                x: self.x * inv_n,
                y: self.y * inv_n,
                z: self.z * inv_n,
            }
        } else {
            Self::IDENTITY
        }
    }

    #[inline(always)]
    pub fn conjugate(self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Rotates a vector via the optimized Rodrigues sandwich formulation:
    /// v' = v + 2 * q_v x (q_v x v + w * v)
    #[inline(always)]
    pub fn rotate_vec(self, v: Vec3) -> Vec3 {
        let q_vec = Vec3::new(self.x, self.y, self.z);
        let t = q_vec.cross(v) * 2.0;
        v + (t * self.w) + q_vec.cross(t)
    }

    /// Regularized spherical linear interpolation (SLERP) between unit quaternions.
    pub fn slerp(self, mut rhs: Self, mut t: f32) -> Self {
        t = t.clamp(0.0, 1.0);
        let mut cos_theta = self.w * rhs.w + self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;

        // Trace the geodesic along the shortest path on S^3
        if cos_theta < 0.0 {
            rhs = Self::new(-rhs.w, -rhs.x, -rhs.y, -rhs.z);
            cos_theta = -cos_theta;
        }

        if cos_theta > 1.0 - 1e-4 {
            // Fallback to normalized linear interpolation (NLERP) for near-parallel quaternions
            Self {
                w: self.w + (rhs.w - self.w) * t,
                x: self.x + (rhs.x - self.x) * t,
                y: self.y + (rhs.y - self.y) * t,
                z: self.z + (rhs.z - self.z) * t,
            }.normalize()
        } else {
            let theta = cos_theta.acos();
            let sin_theta = theta.sin();
            let s0 = ((1.0 - t) * theta).sin() / sin_theta;
            let s1 = (t * theta).sin() / sin_theta;

            Self {
                w: self.w * s0 + rhs.w * s1,
                x: self.x * s0 + rhs.x * s1,
                y: self.y * s0 + rhs.y * s1,
                z: self.z * s0 + rhs.z * s1,
            }
        }
    }
}

impl Mul for Quat {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}

// =========================================================================
// 3. S² GEODESIC & VECTOR SLERP
// =========================================================================

/// SLERP across the unit sphere S² between two directional unit vectors.
/// Resolves the antipodal singularity via rotation about an orthogonal axis.
pub fn slerp_sphere(p0: Vec3, p1: Vec3, t: f32) -> Vec3 {
    let dot = p0.dot(p1).clamp(-1.0, 1.0);

    // Identical points
    if dot >= 1.0 - EPSILON {
        return p0;
    }

    // Exact antipodal singularity
    if dot <= -1.0 + EPSILON {
        let axis = p0.orthogonal();
        let rot = Quat::from_axis_angle(axis, std::f32::consts::PI * t);
        return rot.rotate_vec(p0);
    }

    let theta = dot.acos();
    let sin_theta = theta.sin();
    let s0 = ((1.0 - t) * theta).sin() / sin_theta;
    let s1 = (t * theta).sin() / sin_theta;

    (p0 * s0 + p1 * s1).normalize()
}

// =========================================================================
// 4. ARCBALL KINEMATICS & HYPERBOLIC SHEET CONTINUATION (BELL)
// =========================================================================

#[derive(Debug, Clone)]
pub struct Arcball {
    pub radius: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub orientation: Quat,
    pub angular_velocity: Quat,
    pub is_dragging: bool,
    pub last_pos: Vec3,
    pub friction: f32,
}

impl Arcball {
    pub fn new(center_x: f32, center_y: f32, radius: f32) -> Self {
        Self {
            radius,
            center_x,
            center_y,
            orientation: Quat::IDENTITY,
            angular_velocity: Quat::IDENTITY,
            is_dragging: false,
            last_pos: Vec3::ZERO,
            friction: 0.94, // Fluid momentum damping at 60 FPS
        }
    }

    /// Continuous projection from screen coordinates (2D) onto the virtual sphere S² (3D).
    /// Employs hemisphere mapping for r² <= 0.5, smoothly transitioning to Bell's hyperbola beyond.
    pub fn project_screen_point(&self, screen_x: f32, screen_y: f32) -> Vec3 {
        // Centered normalization with inverted Y-axis (standard Cartesian coordinate space)
        let x = (screen_x - self.center_x) / self.radius;
        let y = -(screen_y - self.center_y) / self.radius;
        let r_sq = x * x + y * y;

        let z = if r_sq <= 0.5 {
            // Interior hemispherical surface
            (1.0 - r_sq).sqrt()
        } else {
            // Smooth Bell hyperbolic continuation: 1 / (2 * sqrt(r_sq))
            0.5 / r_sq.sqrt()
        };

        Vec3::new(x, y, z).normalize()
    }

    pub fn on_mouse_down(&mut self, screen_x: f32, screen_y: f32) {
        self.is_dragging = true;
        self.last_pos = self.project_screen_point(screen_x, screen_y);
        self.angular_velocity = Quat::IDENTITY;
    }

    pub fn on_mouse_move(&mut self, screen_x: f32, screen_y: f32) {
        if !self.is_dragging {
            return;
        }

        let current_pos = self.project_screen_point(screen_x, screen_y);
        let delta_rot = Quat::from_rotation_arc(self.last_pos, current_pos);

        // Apply incremental rotation
        self.orientation = (delta_rot * self.orientation).normalize();
        self.angular_velocity = delta_rot;
        self.last_pos = current_pos;
    }

    pub fn on_mouse_up(&mut self) {
        self.is_dragging = false;
    }

    /// Gyroscopic momentum integration (invoked per frame dt).
    pub fn update(&mut self, dt: f32) {
        if !self.is_dragging {
            // Exponential decay towards identity
            let decay = self.friction.powf(dt * 60.0);
            self.angular_velocity = Quat::IDENTITY.slerp(self.angular_velocity, decay);
            self.orientation = (self.angular_velocity * self.orientation).normalize();
        }
    }
}
