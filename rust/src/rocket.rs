use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{Area3D, IArea3D, Node3D, INode3D,
    GpuParticles3D, Timer, ITimer, RigidBody3D, IRigidBody3D
};

#[allow(unused_imports)]
use crate::explosion::{Explosion};

#[derive(GodotClass)]
#[class(base=RigidBody3D, init)]
pub struct Rocket {
    #[init(val=OnReady::from_loaded("res://explosion/rocket_explosion.tscn"))]
    explosion_scene: OnReady<Gd<PackedScene>>,
    #[export]
    #[init(val=15.0)]
    thrust: f32,
    base: Base<RigidBody3D>
}

#[godot_api]
impl IRigidBody3D for Rocket {
    fn ready(&mut self) {
        self.signals()
            .body_entered()
            .connect_self(Self::on_body_entered);
    }

    fn physics_process(&mut self, delta: f32) {
        // The entire 6-DOF calculation for this vehicle
        let curr_vel = self.base().get_linear_velocity();
        let body_forward = self.base().get_basis() * Vector3::FORWARD;
        let thrust = self.thrust;
        self.base_mut().set_linear_velocity(
            curr_vel + body_forward * thrust * delta
        );
    }
}

#[godot_api]
impl Rocket {
    #[func]
    fn on_body_entered(&mut self, _body: Gd<Node>) {
        self.explode();
    }

    #[func]
    pub fn explode(&mut self) {
        // Explode with an explosion
        let mut explosion = self.explosion_scene
            .instantiate_as::<Explosion>();
        explosion.set_position(self.base().get_position());
        self.base_mut().add_sibling(&explosion);
        self.base_mut().queue_free();
    }
}

