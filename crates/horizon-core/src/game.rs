use crate::camera::Camera;
use crate::player::{PlayerController, PlayerInput};
use crate::renderer::{CameraGpuResources, GpuContext};

pub struct GameRuntime {
    pub player: PlayerController,
}

impl GameRuntime {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            player: PlayerController::new(width, height),
        }
    }

    // Called once for every game update.
    pub fn update(
        &mut self,
        input: PlayerInput,
        delta_seconds: f32,
    ) {
        self.player.update(input, delta_seconds);
    }

    // Update the camera from mouse or touchscreen input.
    pub fn look_around(
        &mut self,
        yaw_delta: f32,
        pitch_delta: f32,
    ) {
        self.player.look_around(yaw_delta, pitch_delta);
    }

    // Access the current camera.
    pub fn camera(&self) -> &Camera {
        &self.player.camera
    }

    // Send the latest camera position and rotation to the GPU.
    pub fn sync_camera_to_gpu(
        &self,
        gpu: &GpuContext,
        resources: &CameraGpuResources,
    ) {
        gpu.update_camera(resources, self.camera());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_creates_player() {
        let game = GameRuntime::new(1280, 720);
        assert!(game.camera().position.is_finite());
    }

    #[test]
    fn runtime_updates_player_movement() {
        let mut game = GameRuntime::new(1280, 720);
        let start = game.camera().position;

        game.update(
            PlayerInput {
                forward: true,
                ..Default::default()
            },
            0.1,
        );

        assert!(game.camera().position.z < start.z);
    }

    #[test]
    fn runtime_updates_camera_rotation() {
        let mut game = GameRuntime::new(1280, 720);

        game.look_around(45.0, 15.0);

        assert_eq!(game.camera().yaw_degrees, 45.0);
        assert_eq!(game.camera().pitch_degrees, 15.0);
    }
}
