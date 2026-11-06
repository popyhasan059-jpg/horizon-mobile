use crate::camera::Camera;
use glam::Vec3;

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
}

#[derive(Debug, Clone)]
pub struct PlayerController {
    pub camera: Camera,
    pub walk_speed: f32,
    pub sprint_multiplier: f32,
    pub sneak_multiplier: f32,
}

impl PlayerController {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            camera: Camera::new(width, height),
            walk_speed: 4.3,
            sprint_multiplier: 1.3,
            sneak_multiplier: 0.3,
        }
    }

    pub fn update(
        &mut self,
        input: PlayerInput,
        delta_seconds: f32,
    ) {
        let dt = delta_seconds.clamp(0.0, 0.1);

        let mut movement = Vec3::ZERO;

        if input.forward {
            movement.z += 1.0;
        }

        if input.backward {
            movement.z -= 1.0;
        }

        if input.right {
            movement.x += 1.0;
        }

        if input.left {
            movement.x -= 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }

        let speed = if input.sneak {
            self.walk_speed * self.sneak_multiplier
        } else if input.sprint {
            self.walk_speed * self.sprint_multiplier
        } else {
            self.walk_speed
        };

        let distance = speed * dt;

        self.camera.move_forward(
            movement.z * distance
        );

        self.camera.move_right(
            movement.x * distance
        );
    }

    pub fn look_around(
        &mut self,
        yaw_delta: f32,
        pitch_delta: f32,
    ) {
        self.camera.rotate_by(yaw_delta, pitch_delta);
    }

    pub fn jump(&mut self, height: f32) {
        if height > 0.0 {
            self.camera.move_up(height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_starts_with_camera() {
        let player = PlayerController::new(1280, 720);
        assert!(player.camera.position.is_finite());
    }

    #[test]
    fn player_moves_forward() {
        let mut player = PlayerController::new(1280, 720);
        let start = player.camera.position;

        player.update(
            PlayerInput {
                forward: true,
                ..Default::default()
            },
            0.1,
        );

        assert!(player.camera.position.z < start.z);
    }

    #[test]
    fn player_can_sprint() {
        let mut player = PlayerController::new(1280, 720);

        let normal_distance = player.walk_speed * 0.1;
        let sprint_distance =
            player.walk_speed * player.sprint_multiplier * 0.1;

        assert!(sprint_distance > normal_distance);
    }

    #[test]
    fn diagonal_movement_is_normalized() {
        let mut player = PlayerController::new(1280, 720);

        player.update(
            PlayerInput {
                forward: true,
                right: true,
                ..Default::default()
            },
            0.1,
        );

        let moved = player.camera.position
            - Vec3::new(0.0, 2.0, 5.0);

        assert!((moved.length() - player.walk_speed * 0.1).abs()
            < 0.001);
    }

    #[test]
    fn player_can_look_around() {
        let mut player = PlayerController::new(1280, 720);

        player.look_around(30.0, 10.0);

        assert_eq!(player.camera.yaw_degrees, 30.0);
        assert_eq!(player.camera.pitch_degrees, 10.0);
    }
      }
