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

    // Horizontal movement
    pub fn move_forward(&mut self, distance: f32) {
        let yaw = self.yaw_degrees.to_radians();

        let forward = Vec3::new(
            yaw.sin(),
            0.0,
            -yaw.cos(),
        );

        self.position += forward * distance;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    pub fn move_right(&mut self, distance: f32) {
        let yaw = self.yaw_degrees.to_radians();

        let right = Vec3::new(
            yaw.cos(),
            0.0,
            yaw.sin(),
        );

        self.position += right * distance;
    }

    pub fn move_left(&mut self, distance: f32) {
        self.move_right(-distance);
    }

    // Vertical movement
    pub fn move_up(&mut self, distance: f32) {
        self.position.y += distance;
    }

    pub fn move_down(&mut self, distance: f32) {
        self.position.y -= distance;
    }

    // Mouse-look style rotation, ready for touch input
    pub fn rotate_by(
        &mut self,
        yaw_delta: f32,
        pitch_delta: f32,
    ) {
        self.set_rotation(
            self.yaw_degrees + yaw_delta,
            self.pitch_degrees + pitch_delta,
        );
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
        assert!(camera.view_projection_matrix().is_finite());
    }

    #[test]
    fn camera_moves_forward() {
        let mut camera = Camera::new(1280, 720);
        let original = camera.position;

        camera.move_forward(1.0);

        assert!(camera.position.z < original.z);
    }

    #[test]
    fn camera_moves_backward() {
        let mut camera = Camera::new(1280, 720);
        let original = camera.position;

        camera.move_backward(1.0);

        assert!(camera.position.z > original.z);
    }

    #[test]
    fn camera_moves_left_and_right() {
        let mut camera = Camera::new(1280, 720);

        camera.move_right(1.0);
        assert!(camera.position.x > 0.0);

        camera.move_left(2.0);
        assert!(camera.position.x < 0.0);
    }

    #[test]
    fn camera_can_move_vertically() {
        let mut camera = Camera::new(1280, 720);

        camera.move_up(2.0);
        assert_eq!(camera.position.y, 4.0);

        camera.move_down(1.0);
        assert_eq!(camera.position.y, 3.0);
    }

    #[test]
    fn camera_rotation_is_clamped() {
        let mut camera = Camera::new(1280, 720);

        camera.rotate_by(10.0, 200.0);

        assert_eq!(camera.yaw_degrees, 10.0);
        assert_eq!(camera.pitch_degrees, 89.9);
    }
    }
