use cgmath::num_traits::Pow;
use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, Vector3, Zero};
use log::info;
use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use crate::input::Input;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ViewMode {
    FPS,
    Orbit,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Camera {
    pub position: Point3<f32>,
    pub forward_direction: Vector3<f32>,
    pub up_direction: Vector3<f32>,
    pub target: Point3<f32>,
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub view_projection: Matrix4<f32>,
    pub view_mode: ViewMode,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new_fps(
        position: Point3<f32>,
        forward_direction: Vector3<f32>,
        aspect_ratio: f32,
    ) -> Self {
        let projection = Self::create_perspective_matrix(aspect_ratio);
        let view = Self::create_view_matrix(position, forward_direction);

        let yaw = forward_direction.z.atan2(forward_direction.x);
        let pitch = forward_direction.y.asin();

        let right_direction = forward_direction.cross(Vector3::unit_y()).normalize();
        let up_direction = forward_direction.cross(right_direction);

        Self {
            position,
            target: position + forward_direction,
            forward_direction,
            up_direction,
            projection,
            view,
            view_projection: projection * view,
            view_mode: ViewMode::FPS,
            yaw,
            pitch,
        }
    }

    pub fn new_orbital(position: Point3<f32>, target: Point3<f32>) -> Self {
        unimplemented!()
    }

    pub fn update(&mut self, input: &Input) {
        match self.view_mode {
            ViewMode::Orbit => unimplemented!(),
            ViewMode::FPS => self.update_fps(input),
        }

        self.view = Self::create_view_matrix(self.position, self.forward_direction);
        self.view_projection = self.projection * self.view;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.projection = Self::create_perspective_matrix(aspect_ratio);
    }

    fn create_view_matrix(position: Point3<f32>, forward_direction: Vector3<f32>) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            position,
            position + forward_direction,
            Vector3::new(0.0, 1.0, 0.0),
        )
    }

    fn create_perspective_matrix(aspect_ratio: f32) -> Matrix4<f32> {
        cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0)
    }

    fn update_fps(&mut self, input: &Input) {
        let speed = 0.1;

        let offset = input.device_offset();

        self.yaw += offset.x % (2.0 * std::f32::consts::PI);
        self.pitch -= offset.y;

        let epsilon = 0.00001;

        self.pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + epsilon,
            std::f32::consts::FRAC_PI_2 - epsilon,
        );

        self.forward_direction = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        let right_direction = self.forward_direction.cross(Vector3::unit_y()).normalize();
        self.up_direction = self.forward_direction.cross(right_direction);

        if input.key_down(KeyCode::KeyW) {
            self.position += speed * self.forward_direction;
        }

        if input.key_down(KeyCode::KeyS) {
            self.position -= speed * self.forward_direction;
        }

        if input.key_down(KeyCode::KeyA) {
            self.position -= speed * right_direction;
        }

        if input.key_down(KeyCode::KeyD) {
            self.position += speed * right_direction;
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        let position = Point3::new(5.0, 2.0, 5.0);

        Self::new_fps(position, -position.to_vec().normalize(), 1920.0 / 1009.0)
    }
}
