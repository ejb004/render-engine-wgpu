use cgmath::*;
use wgpu::Queue;

use crate::camera::camera;

use super::camera::{convert_matrix4_to_array, Camera, CameraUniform};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

/// An [OrbitCamera] only permits rotation of the eye on a spherical shell around a target.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct OrbitCamera {
    /// The distance of the eye from the target.
    pub distance: f32,

    /// The pitch angle in radians.
    pub pitch: f32,

    /// The yaw angle in radians.
    pub yaw: f32,

    /// The eye of the camera in cartesian coordinates.
    pub(crate) eye: Vector3<f32>,

    /// The target of the orbit camera.
    pub target: Vector3<f32>,

    /// The cameras' up vector.
    pub up: Vector3<f32>,

    /// The bounds within which the camera can be moved.
    pub bounds: OrbitCameraBounds,

    /// The aspect ratio of the camera.
    pub aspect: f32,

    /// The field of view of the camera.
    pub fovy: Rad<f32>,

    /// The near clipping plane of the camera.
    pub znear: f32,

    /// The far clipping plane of the camera.
    pub zfar: f32,

    pub uniform: CameraUniform,
}

impl Camera for OrbitCamera {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let eye = Point3::from_vec(self.eye);
        let target = Point3::from_vec(self.target);
        let view = Matrix4::look_at_rh(eye, target, self.up);
        let proj =
            OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

impl OrbitCamera {
    /// Creates a new [OrbitCamera].
    ///
    /// Arguments:
    ///
    /// * `distance`: The distance of the eye to the `target`.
    /// * `pitch`: The pitch angle in radians.
    /// * `yaw`: The yaw angle in radians.
    /// * `target`: The point around which the camera rotates.
    /// * `aspect`: The aspect ratio of the camera.
    pub fn new(distance: f32, pitch: f32, yaw: f32, target: Vector3<f32>, aspect: f32) -> Self {
        let mut camera = Self {
            distance,
            pitch,
            yaw,
            eye: Vector3::zero(), // Will be auto-calculted in `update()` nevertheless.
            target,
            up: Vector3::unit_y(),
            bounds: OrbitCameraBounds::default(),
            aspect,
            fovy: cgmath::Rad(std::f32::consts::PI / 4.0),
            znear: 0.1,
            zfar: 1000.0,
            uniform: CameraUniform::default(),
        };
        camera.update();
        camera
    }

    /// Sets the distance of the [OrbitCamera] from the target.
    ///
    /// Arguments:
    ///
    /// * `distance`: The euclidean distance between the cameras' eye and the target.
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.clamp(
            self.bounds.min_distance.unwrap_or(f32::EPSILON),
            self.bounds.max_distance.unwrap_or(f32::MAX),
        );
        self.update();
    }

    /// Incrementally changes the distance of the [OrbitCamera] from the target.
    ///
    /// Arguments:
    ///
    /// `delta`: The amount by which the distance will be changed.
    pub fn add_distance(&mut self, delta: f32) {
        let corrected_zoom = f32::log10(self.distance) * delta;
        self.set_distance(self.distance + corrected_zoom);
        println!("{:}", self.distance)
    }

    /// Sets the pitch of the [OrbitCamera].
    ///
    /// Arguments:
    ///
    /// * `pitch`: The new pitch angle in radians.
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(self.bounds.min_pitch, self.bounds.max_pitch);
        self.update();
    }

    /// Incrementally changes the pitch of the [OrbitCamera].
    ///
    /// Arguments:
    ///
    /// `delta`: The amount by which the pitch will be changed.
    pub fn add_pitch(&mut self, delta: f32) {
        self.set_pitch(self.pitch + delta);
    }

    /// Sets the yaw of the [OrbitCamera].
    ///
    /// Arguments:
    ///
    /// * `yaw`: The new yaw angle in radians.
    pub fn set_yaw(&mut self, yaw: f32) {
        let mut bounded_yaw = yaw;
        if let Some(min_yaw) = self.bounds.min_yaw {
            bounded_yaw = bounded_yaw.clamp(min_yaw, f32::MAX);
        }
        if let Some(max_yaw) = self.bounds.max_yaw {
            bounded_yaw = bounded_yaw.clamp(f32::MIN, max_yaw);
        }
        self.yaw = bounded_yaw;
        self.update();
    }

    /// Incrementally changes the yaw of the [OrbitCamera].
    ///
    /// Arguments:
    ///
    /// `delta`: The amount by which the yaw will be changed.
    pub fn add_yaw(&mut self, delta: f32) {
        self.set_yaw(self.yaw + delta);
    }

    pub fn pan(&mut self, delta: (f32, f32)) {
        self.eye.y += delta.1 * self.distance;
        self.target.y += delta.1 * self.distance;
        // println!("{:?}", self.distance);

        let forward = (self.target - self.eye).normalize();

        let cross = forward.cross(self.up).normalize();

        self.eye -= cross * delta.0 * self.distance;
        self.target -= cross * delta.0 * self.distance;
    }

    /// Updates the camera after changing `distance`, `pitch` or `yaw`.
    fn update(&mut self) {
        self.eye =
            calculate_cartesian_eye_position(self.pitch, self.yaw, self.distance, self.target);
    }

    pub fn resize_projection(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn update_view_proj(&mut self) {
        self.uniform.view_position = [self.eye.x, self.eye.y, self.eye.z, 1.0];
        self.uniform.view_proj = convert_matrix4_to_array(self.build_view_projection_matrix());
    }
}

/// The boundaries for how an [OrbitCamera] can be rotated.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Copy)]
pub struct OrbitCameraBounds {
    /// The minimum distance between the eye and the target.
    /// This should not be negative. In order to ensure this the minimum distance
    /// should never be smaller than [f32::EPSILON].
    pub min_distance: Option<f32>,

    /// If set it is not possible to move the camera further from the target
    /// than the specified amount.
    pub max_distance: Option<f32>,

    /// The `min_pitch` can only be between `]-PI / 2, 0]` due to mathematical reasons.
    pub min_pitch: f32,

    /// The `min_pitch` can only be between `]0, PI / 2]` due to mathematical reasons.
    pub max_pitch: f32,

    /// If set the yaw angle will be constrained. The constrain should be in the
    /// interval `[-PI, 0]`.
    pub min_yaw: Option<f32>,

    /// If set the yaw angle will be constrained. The constrain should be in the
    /// interval `[0, PI]`.
    pub max_yaw: Option<f32>,
}

impl Default for OrbitCameraBounds {
    fn default() -> Self {
        Self {
            min_distance: None,
            max_distance: Some(16.0),
            min_pitch: -std::f32::consts::PI / 2.0 + f32::EPSILON,
            max_pitch: std::f32::consts::PI / 2.0 - f32::EPSILON,
            min_yaw: None,
            max_yaw: None,
        }
    }
}

/// Calulcates the eye position in cartesian coordinates from spherical coordinates.
///
/// Arguments:
///
/// * `pitch`: The pitch angle in radians.
/// * `yaw`: The yaw angle in radians.
/// * `distance`: The euclidean distance to the target.
fn calculate_cartesian_eye_position(
    pitch: f32,
    yaw: f32,
    distance: f32,
    target: Vector3<f32>,
) -> Vector3<f32> {
    return Vector3::new(
        distance * yaw.sin() * pitch.cos(),
        distance * pitch.sin(),
        distance * yaw.cos() * pitch.cos(),
    ) + target;
}
