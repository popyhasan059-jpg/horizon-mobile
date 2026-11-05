use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub aspect_ratio: f32,
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 5.0),
            yaw_degrees: 0.0,
            pitch_degrees: 0.0,
            aspect_ratio: width.max(1) as f32
                / height.max(1) as f32,
            fov_y_degrees: 70.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn direction(&self) -> Vec3 {
        let yaw = self.yaw_degrees.to_radians();
        let pitch = self.pitch_degrees.to_radians();

        Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        )
        .normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(
            self.position,
            self.direction(),
            Vec3::Y,
        )
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov_y_degrees.to_radians(),
            self.aspect_ratio.max(0.001),
            self.near,
            self.far,
        )
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn set_rotation(
        &mut self,
        yaw: f32,
        pitch: f32,
    ) {
        self.yaw_degrees = yaw;
        self.pitch_degrees = pitch.clamp(-89.9, 89.9);
    }

    pub fn set_viewport_size(
        &mut self,
        width: u32,
        height: u32,
    ) {
        self.aspect_ratio =
            width.max(1) as f32 / height.max(1) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_starts_facing_negative_z() {
        let camera = Camera::new(1280, 720);
        let direction = camera.direction();

        assert!(direction.z < 0.0);
        assert!(direction.x.abs() < 0.001);
    }

    #[test]
    fn camera_rotation_changes_direction() {
        let mut camera = Camera::new(1280, 720);
        let original = camera.direction();

        camera.set_rotation(90.0, 0.0);

        assert_ne!(original, camera.direction());
    }

    #[test]
    fn camera_projection_is_valid() {
        let camera = Camera::new(1280, 720);
        let matrix = camera.view_projection_matrix();

        assert!(matrix.is_finite());
    }
  }
