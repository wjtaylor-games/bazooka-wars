use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{RigidBody3D, IRigidBody3D, CharacterBody3D, ICharacterBody3D,
    Input, InputEvent, Camera3D, InputEventMouseMotion, MeshInstance3D, Timer,
    InputEventAction, Area3D, IArea3D, CollisionShape3D
};
use godot::classes::input::MouseMode;
use godot::classes::ProjectSettings;
use std::f32::consts::{TAU, PI};
use godot::global::{wrapf};
use num::clamp;

use crate::explosion::Explosion;

// const SHOOT_SCALE: f32 = 2.0;
// const CHAR_SCALE: Vector3 = Vector3::new(0.3, 0.3, 0.3);


#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct Player {
    #[init(node="PlayerKinematicBody")]
    player_kinematic_body: OnReady<Gd<PlayerKinematicBody>>,
    #[init(node="PlayerDynamicBody")]
    player_dynamic_body: OnReady<Gd<PlayerDynamicBody>>,
    #[init(node="SphereCollider")]
    sphere_collider: OnReady<Gd<CollisionShape3D>>,
    #[init(node="RagdollTimer")]
    ragdoll_timer: OnReady<Gd<Timer>>,
    #[export]
    #[init(val=false)]
    ragdoll: bool,
    init_pos: Vector3,
    init_rot: Vector3,
    base: Base<Area3D>
}

#[godot_api]
impl IArea3D for Player {
    fn physics_process(&mut self, _delta: f32) {
        if self.ragdoll {
            self.player_kinematic_body.set_position(
                self.player_dynamic_body.get_position()
            );
            self.player_kinematic_body.set_velocity(
                self.player_dynamic_body.get_linear_velocity()
            );
        } else {
            self.player_dynamic_body.set_position(
                self.player_kinematic_body.get_position()
            );
            self.player_dynamic_body.set_linear_velocity(
                self.player_kinematic_body.get_velocity()
            );
        }
        let pos = self.player_dynamic_body.get_position();
        self.base_mut().set_position(pos);
        
        // Out of bounds condition
        if pos.y < -10.0 {
            self.reset_pos();
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("ragdoll") {
            godot_print!("ragdoll activated");
            self.begin_ragdoll();
        }
    }

    fn ready(&mut self) {
        let pos = self.base().get_position();
        let rot = self.base().get_rotation();
        self.init_pos = pos;
        self.init_rot = rot;
        self.player_dynamic_body.set_position(pos);
        self.player_kinematic_body.set_position(pos);
        self.player_dynamic_body.set_rotation(rot);
        self.player_kinematic_body.set_rotation(rot);
        if self.ragdoll {
            self.begin_ragdoll();
        } else {
            self.end_ragdoll();
        }

        self.ragdoll_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::end_ragdoll);

        self.signals()
            .area_entered()
            .connect_self(Self::on_area_entered);
    }    
}

#[godot_api]
impl Player {

    #[func]
    pub fn on_area_entered(&mut self, area: Gd<Area3D>) {
        match area.try_cast::<Explosion>() {
            Ok(explosion) => {
                if explosion.bind().get_time() < 0.2 {
                    self.begin_ragdoll();
                    let radius_vec = self.player_dynamic_body.get_position()
                        - explosion.get_position();
                    let new_velocity =
                        radius_vec.normalized_or_zero() * 20.0;
                        // + Vector3::UP * 15.0;
                    self.player_dynamic_body.set_linear_velocity(new_velocity);
                }
            }
            Err(_) => {}
        }
    }

    #[func]
    pub fn reset_pos(&mut self) {
        // Reset to initial position
        let pos = self.init_pos;
        self.base_mut().set_position(pos);
        self.player_dynamic_body.set_position(pos);
        self.player_kinematic_body.set_position(pos);
        self.player_dynamic_body.set_rotation(self.init_rot);
        self.player_kinematic_body.set_rotation(self.init_rot);
        self.player_dynamic_body.set_linear_velocity(Vector3::ZERO);
        self.player_kinematic_body.set_velocity(Vector3::ZERO);
        self.player_dynamic_body.set_angular_velocity(Vector3::ZERO);
        self.end_ragdoll();
    }


    #[func]
    pub fn begin_ragdoll(&mut self) {
        self.ragdoll_timer.start();
        self.ragdoll = true;
        self.player_kinematic_body.set_visible(false);
        self.player_kinematic_body.set_physics_process(false);
        self.player_dynamic_body.set_visible(true);
        self.player_dynamic_body.set_physics_process(true);
    }

    #[func]
    pub fn end_ragdoll(&mut self) {
        self.ragdoll = false;
        self.player_kinematic_body.set_visible(true);
        self.player_kinematic_body.set_physics_process(true);
        self.player_dynamic_body.set_visible(false);
        self.player_dynamic_body.set_physics_process(false);
    }
}


#[derive(GodotClass)]
#[class(base=CharacterBody3D)]
pub struct PlayerKinematicBody {
    camera: OnReady<Gd<Camera3D>>,
    bazooka: OnReady<Gd<Node3D>>,
    mesh: OnReady<Gd<MeshInstance3D>>,
    parent_player: OnReady<Gd<Player>>,
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
    #[export]
    mouse_sensitivity: f32,
    base: Base<CharacterBody3D>
}

#[godot_api]
impl ICharacterBody3D for PlayerKinematicBody {

    fn init(base: Base<CharacterBody3D>) -> Self {
        let project_settings = ProjectSettings::singleton();
        Self {
            camera: OnReady::from_node("PlayerCamera"),
            bazooka: OnReady::from_node("Bazooka"),
            mesh: OnReady::from_node("PlayerMesh"),
            parent_player: OnReady::from_node(".."),
            jumping: true,
            gravity: project_settings.get_setting("physics/3d/default_gravity").to::<f32>() *
                project_settings.get_setting("physics/3d/default_gravity_vector").to::<Vector3>(),
            jump_velocity: 4.0,
            accel: 12.0,
            deaccel: 12.0,
            max_speed: 4.0,
            mouse_sensitivity: 0.005,
            base,
        }
    }

    fn ready(&mut self) {
        let mut input = Input::singleton();
        input.set_mouse_mode(MouseMode::CAPTURED);
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
    
    fn input(&mut self, event: Gd<InputEvent>) {
        if Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            match event.try_cast::<InputEventMouseMotion>() {
                Ok(e) => {
                    // Set the Kinematic Player yaw rotation
                    let motion_vec = e.get_relative() * self.mouse_sensitivity;
                    let mut rotation = self.base().get_rotation();
                    rotation.y = wrapf(
                        (rotation.y - motion_vec.x) as f64,
                        0.0, TAU as f64) as f32;
                    self.base_mut().set_rotation(rotation);

                    // Set the Camera pitch rotation
                    let mut cam_rotation = self.camera.get_rotation();
                    cam_rotation.x = clamp::<f32>(cam_rotation.x - motion_vec.y,
                        -PI/2.0, PI/2.0);
                    self.camera.set_rotation(cam_rotation);
                    self.bazooka.set_rotation(cam_rotation);
                    self.mesh.set_rotation(cam_rotation);
                }
                Err(_) => {}
            }
        }
    }
}

#[godot_api]
impl PlayerKinematicBody {
    #[func]
    pub fn on_explosion(&mut self) {

    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D, init)]
pub struct PlayerDynamicBody {
    base: Base<RigidBody3D>
}
