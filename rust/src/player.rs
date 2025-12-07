use godot::prelude::*;
use godot::classes::{RigidBody3D, IRigidBody3D, CharacterBody3D, ICharacterBody3D,
    Input, InputEvent, Camera3D};
use godot::global::{deg_to_rad, acos};
use std::f32::consts::{TAU, PI};
use godot::classes::ProjectSettings;
// use godot::global::{wrapf};
// use num::clamp;

const SHOOT_TIME: f32 = 1.5;
const SHOOT_SCALE: f32 = 2.0;
const CHAR_SCALE: Vector3 = Vector3::new(0.3, 0.3, 0.3);
const TURN_SPEED: f32 = 40.0;
const BULLET_SPEED: f32 = 20.0;
const AIR_IDLE_DEACCEL: bool = false;
const AIR_ACCEL_FACTOR: f32 = 0.8;
const SHARP_TURN_THRESHOLD: f32 = 140.0 * PI / 180.0;


#[derive(GodotClass)]
#[class(base=Node3D, init)]
pub struct Player {
    #[init(node="PlayerKinematicBody")]
    player_kinematic_body: OnReady<Gd<PlayerKinematicBody>>,
    #[init(node="PlayerDynamicBody")]
    player_dynamic_body: OnReady<Gd<PlayerDynamicBody>>,
    base: Base<Node3D>
}


#[derive(GodotClass)]
#[class(base=CharacterBody3D)]
pub struct PlayerKinematicBody {
    player_camera: OnReady<Gd<Camera3D>>,
    jumping: bool,
    #[export]
    gravity: Vector3,
    #[export]
    jump_velocity: f32,
    #[export]
    accel: f32,
    #[export]
    deaccel: f32,
    #[export]
    max_speed: f32,
    base: Base<CharacterBody3D>
}

#[godot_api]
impl ICharacterBody3D for PlayerKinematicBody {

    fn init(base: Base<CharacterBody3D>) -> Self {
        let project_settings = ProjectSettings::singleton();
        Self {
            player_camera: OnReady::from_node("PlayerCamera"),
            jumping: true,
            gravity: project_settings.get_setting("physics/3d/default_gravity").to::<f32>() *
                project_settings.get_setting("physics/3d/default_gravity_vector").to::<Vector3>(),
            jump_velocity: 4.0,
            accel: 8.0,
            deaccel: 8.0,
            max_speed: 4.0,
            base,
        }
    }

    fn physics_process(&mut self, delta: f32) {
        
        let mut velocity = self.base().get_velocity();
        velocity += self.gravity * delta;
        let mut vertical_velocity = velocity.y;
        let mut horizontal_velocity = Vector3::new(velocity.x, 0.0, velocity.z);


        let basis = self.base().get_basis();
        let input = Input::singleton();
        let movement_vec2 = input.get_vector("left", "right", "forward", "back");
        let mut movement_direction =
            basis * Vector3::new(movement_vec2.x, 0.0, movement_vec2.y);
        movement_direction.y = 0.0;
        let movement_direction = movement_direction.normalized_or_zero();

        let jump_attempt: bool = input.is_action_pressed("jump");

        if self.base().is_on_floor() {
            self.jumping = false;
        } else {
            self.jumping = true;
        }

        if movement_direction.length() > 0.1 {
            // We are actually walking
            horizontal_velocity += movement_direction * self.accel * delta;
            horizontal_velocity = horizontal_velocity.limit_length(Some(self.max_speed));
        } else {
            // Not walking, slow to a stop
            let mut horizontal_speed = horizontal_velocity.length();
            horizontal_speed -= self.deaccel * delta;
            if horizontal_speed < 0.0 {
                horizontal_speed = 0.0;
            }
            horizontal_velocity = horizontal_velocity.normalized_or_zero() * horizontal_speed;
        }

        if !self.jumping && jump_attempt {
            vertical_velocity = self.jump_velocity;
            self.jumping = true;
        }

        self.base_mut().set_velocity(
            horizontal_velocity + Vector3::UP * vertical_velocity
        );

        self.base_mut().move_and_slide();


    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D, init)]
pub struct PlayerDynamicBody {
    base: Base<RigidBody3D>
}
