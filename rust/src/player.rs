use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{RigidBody3D, IRigidBody3D, CharacterBody3D, ICharacterBody3D,
    Input, InputEvent, Camera3D, InputEventMouseMotion, MeshInstance3D, Timer,
    InputEventAction, Area3D, IArea3D, CollisionShape3D,
    AnimationPlayer,
};
use godot::classes::input::MouseMode;
use godot::classes::ProjectSettings;
use std::f32::consts::{TAU, PI};
use godot::global::{wrapf};
use num::clamp;

use crate::explosion::Explosion;
use crate::rocket::Rocket;
use crate::game::Game;

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
    #[init(node="AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
    #[init(val=OnReady::from_loaded("res://rocket/rocket.tscn"))]
    rocket_scene: OnReady<Gd<PackedScene>>,
    #[init(node="../..")]
    game_root: OnReady<Gd<Game>>,
    #[export]
    #[init(val=15.0)]
    rocket_init_vel: f32,
    ragdoll: bool,
    #[init(val=true)]
    bazooka_loaded: bool,
    init_pos: Vector3,
    init_rot: Vector3,
    peer_id: i64,
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
        
        if self.base().is_multiplayer_authority() {
            // Make sure ragdoll states don't diverge
            let args = vslice![self.ragdoll];
            self.base_mut().rpc("sync_ragdoll", args);
            // Out of bounds condition
            if pos.y < -10.0 {
                let gd_ref = self.to_gd();
                self.signals().out_of_bounds().emit(&gd_ref);
                // self.base_mut().rpc("respawn", &[]);
            }
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if self.base().is_multiplayer_authority() 
                && Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            if event.is_action_pressed("ragdoll") {
                godot_print!("ragdoll activated");
                self.begin_ragdoll();
            }

            if !self.ragdoll && event.is_action_pressed("shoot") {
                let rocket_pos = self.player_kinematic_body.bind().get_aim_position();
                let rocket_rot = self.player_kinematic_body.bind().get_aim_rotation();
                let base_vel = self.player_kinematic_body.get_velocity();
                self.base_mut().rpc("shoot_rocket", vslice![rocket_pos, rocket_rot, base_vel]);
            }
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

        self.end_ragdoll();

        self.ragdoll_timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::end_ragdoll);

        self.animation_player
            .signals()
            .animation_finished()
            .connect_other(&self.to_gd(), |this, anim_name: StringName| {
                if anim_name == "reload".into() { 
                    this.bazooka_loaded = true;
                }
            });

        self.signals()
            .area_entered()
            .connect_self(Self::on_area_entered);

        if self.base().is_multiplayer_authority() {
            self.game_root
                .signals()
                .mouse_sensitivity_changed()
                .connect_other(&*self.player_kinematic_body,
                    PlayerKinematicBody::on_mouse_sensitivity_changed);
        }
    }    
}

#[godot_api]
impl Player {

    #[func]
    pub fn set_peer_id(&mut self, peer_id: i64) {
        self.peer_id = peer_id;
    }

    #[func]
    pub fn get_peer_id(&self) -> i64 {
        self.peer_id
    }

    #[func]
    pub fn on_area_entered(&mut self, area: Gd<Area3D>) {
        if let Ok(explosion) = area.try_cast::<Explosion>() {
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
    }

    #[signal]
    pub fn out_of_bounds(&mut player: Gd<Player>);

    #[rpc(authority, call_local)]
    pub fn respawn(&mut self, pos: Vector3) {
        // Reset to a given spawn point
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

    fn begin_ragdoll(&mut self) {
        self.ragdoll_timer.start();
        self.ragdoll = true;
        self.player_kinematic_body.set_visible(false);
        self.player_kinematic_body.set_physics_process(false);
        self.player_dynamic_body.set_visible(true);
        self.player_dynamic_body.set_physics_process(true);
    }

    fn end_ragdoll(&mut self) {
        self.ragdoll = false;
        self.player_kinematic_body.set_visible(true);
        self.player_kinematic_body.set_physics_process(true);
        self.player_dynamic_body.set_visible(false);
        self.player_dynamic_body.set_physics_process(false);
    }

    #[rpc(authority, call_remote, unreliable_ordered)]
    fn sync_ragdoll(&mut self, ragdoll: bool) {
        if self.ragdoll && !ragdoll {
            self.end_ragdoll();
        } else if !self.ragdoll && ragdoll {
            self.begin_ragdoll();
        }
    }

    #[rpc(authority, call_local, reliable)]
    pub fn shoot_rocket(&mut self, position: Vector3, rotation: Vector3,
                        base_velocity: Vector3) {
        if self.bazooka_loaded {
            let mut rocket: Gd<Rocket> = self.rocket_scene.instantiate_as();
            rocket.set_multiplayer_authority(
                self.base().get_multiplayer_authority()
            );
            rocket.set_position(position);
            rocket.set_rotation(rotation);
            let rocket_basis = rocket.get_basis();
            rocket.set_linear_velocity(
                rocket_basis * Vector3::FORWARD * self.rocket_init_vel
                + base_velocity
            );
            self.base_mut()
                .get_tree()
                .unwrap()
                .get_root()
                .unwrap()
                .add_child(&rocket);

            self.bazooka_loaded = false;
            self.animation_player.play_ex().name("reload").done();
        }
    }

    #[func]
    pub fn set_camera_current(&mut self, enabled: bool) {
        self.player_kinematic_body.bind_mut().set_camera_current(enabled);
    }
}


#[derive(GodotClass)]
#[class(base=CharacterBody3D)]
pub struct PlayerKinematicBody {
    camera: OnReady<Gd<Camera3D>>,
    bazooka: OnReady<Gd<Node3D>>,
    mesh: OnReady<Gd<MeshInstance3D>>,
    rocket_mesh: OnReady<Gd<MeshInstance3D>>,
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
    base_mouse_sensitivity: f64,
    mouse_sensitivity_mult: f64,
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
            rocket_mesh: OnReady::from_node("Bazooka/RocketMesh"),
            parent_player: OnReady::from_node(".."),
            jumping: true,
            gravity: project_settings.get_setting("physics/3d/default_gravity").to::<f32>() *
                project_settings.get_setting("physics/3d/default_gravity_vector").to::<Vector3>(),
            jump_velocity: 4.0,
            accel: 12.0,
            deaccel: 12.0,
            max_speed: 4.0,
            base_mouse_sensitivity: 0.007,
            mouse_sensitivity_mult: 0.5,
            base,
        }
    }

    fn ready(&mut self) {
    }

    fn physics_process(&mut self, delta: f32) {
        if self.base().is_multiplayer_authority() {
            let mut velocity = self.base().get_velocity();
            velocity += self.gravity * delta;
            let mut vertical_velocity = velocity.y;
            let mut horizontal_velocity = Vector3::new(velocity.x, 0.0, velocity.z);

            // Declare input commands
            let movement_vec2: Vector2;
            let jump_attempt: bool;

            // Get inputs, if not paused
            let input = Input::singleton();
            if input.get_mouse_mode() == MouseMode::CAPTURED {
                // Not paused
                movement_vec2 = input.get_vector("left", "right", "forward", "back");
                jump_attempt = input.is_action_pressed("jump");
            } else {
                // Paused
                movement_vec2 = Vector2::ZERO;
                jump_attempt = false;
            }

            let basis = self.base().get_basis();
            let mut movement_direction =
                basis * Vector3::new(movement_vec2.x, 0.0, movement_vec2.y);
            movement_direction.y = 0.0;
            let movement_direction = movement_direction.normalized_or_zero();


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

            self.base_mut().set_velocity(horizontal_velocity + Vector3::UP * vertical_velocity);
            let args = vslice![
                self.jumping,
                self.base().get_position(),
                self.base().get_velocity(),
                self.base().get_rotation(),
                self.camera.get_rotation(),
            ];
            self.base_mut().rpc("sync_state", args);
        }

        self.base_mut().move_and_slide();
    }
    
    fn input(&mut self, event: Gd<InputEvent>) {
        if self.base().is_multiplayer_authority()
                && Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            if let Ok(e) = event.try_cast::<InputEventMouseMotion>() {
                // Set the Kinematic Player yaw rotation
                let motion_vec = e.get_relative()
                    * (self.base_mouse_sensitivity * self.mouse_sensitivity_mult) as f32;
                let mut rotation = self.base().get_rotation();
                rotation.y = wrapf(
                    (rotation.y - motion_vec.x) as f64,
                    0.0, TAU as f64) as f32;

                // Set the Camera pitch rotation
                let mut cam_rotation = self.camera.get_rotation();
                cam_rotation.x = clamp::<f32>(cam_rotation.x - motion_vec.y,
                    -PI/2.0, PI/2.0);
                self.base_mut().set_rotation(rotation);
                self.camera.set_rotation(cam_rotation);
                self.mesh.set_rotation(cam_rotation);
                let bazooka_rotation = cam_rotation + Vector3::UP * self.bazooka.get_rotation().y;
                self.bazooka.set_rotation(bazooka_rotation);
            }
        }
    }
}

#[godot_api]
impl PlayerKinematicBody {

    #[rpc(authority, unreliable_ordered, call_remote)]
    pub fn sync_state(
        &mut self,
        jumping: bool,
        position: Vector3,
        velocity: Vector3,
        yaw_rotation: Vector3,
        cam_rotation: Vector3,
    ) {
        self.jumping = jumping;
        self.base_mut().set_position(position);
        self.base_mut().set_velocity(velocity);
        self.base_mut().set_rotation(yaw_rotation);
        self.camera.set_rotation(cam_rotation);
        self.mesh.set_rotation(cam_rotation);
        let bazooka_rotation = cam_rotation + Vector3::UP * self.bazooka.get_rotation().y;
        self.bazooka.set_rotation(bazooka_rotation);
    }

    #[func]
    pub fn get_aim_position(&self) -> Vector3 {
        self.rocket_mesh.get_global_position()
    }

    #[func]
    pub fn get_aim_rotation(&self) -> Vector3 {
        self.bazooka.get_global_rotation()
    }

    #[func]
    pub fn set_camera_current(&mut self, enabled: bool) {
        self.camera.set_current(enabled);
    }

    pub fn on_mouse_sensitivity_changed(&mut self, value: f64) {
        self.mouse_sensitivity_mult = value;
    }
}

#[derive(GodotClass)]
#[class(base=RigidBody3D, init)]
pub struct PlayerDynamicBody {
    base: Base<RigidBody3D>
}

#[godot_api]
impl IRigidBody3D for PlayerDynamicBody {
    fn ready(&mut self) {
        // set collision layer 8 on if and only if
        // this is the multiplayer authority
        let layer: u32 = self.base().get_collision_layer();
        if self.base().is_multiplayer_authority() {
            self.base_mut().set_collision_layer(layer | 0b10000000);
        } else {
            self.base_mut().set_collision_layer(layer & 0b01111111);
        }
    }

    fn physics_process(&mut self, _delta: f32) {
        if self.base().is_multiplayer_authority() {
            let args = vslice![
                self.base().get_position(),
                self.base().get_rotation(),
                self.base().get_linear_velocity(),
                self.base().get_angular_velocity(),
            ];
            self.base_mut().rpc("update_states", args);
        }
    }
}

#[godot_api]
impl PlayerDynamicBody {

    // This is called for remote, not local
    #[rpc(authority, unreliable_ordered, call_remote)]
    pub fn update_states(&mut self,
                         position: Vector3,
                         rotation: Vector3,
                         velocity: Vector3,
                         ang_velocity: Vector3,
                         ) {
        self.base_mut().set_position(position);
        self.base_mut().set_rotation(rotation);
        self.base_mut().set_linear_velocity(velocity);
        self.base_mut().set_angular_velocity(ang_velocity);
    }
}
