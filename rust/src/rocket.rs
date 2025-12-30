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
    #[export]
    #[init(val=-5.0)]
    cm_alpha: f32,
    #[export]
    #[init(val=0.5)]
    cd: f32,
    // lift coefficient
    #[export]
    #[init(val=20.0)]
    cl_alpha: f32,
    // induced drag coefficient
    #[export]
    #[init(val=0.3)]
    cd_i: f32,
    #[export]
    #[init(val=1.0)]
    lifetime: f32,
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
        let vel_norm = curr_vel.normalized_or_zero();
        let body_forward = self.base().get_basis() * Vector3::FORWARD;
        // Yeah yeah I know it's "1/2 rho v squared" its all proportional
        let dyn_pressure = curr_vel.length_squared();

        let thrust = body_forward * self.thrust;
        let drag = -vel_norm * dyn_pressure * self.cd;
        let lift_norm = -body_forward.cross(curr_vel).cross(curr_vel).normalized_or_zero();
        let lift_magnitude = -(lift_norm.dot(lift_norm) * dyn_pressure * self.cl_alpha);
        let lift = lift_norm * lift_magnitude;
        let drag_induced = -vel_norm * lift_magnitude * self.cd_i;

        // Apply propulsion forces
        self.base_mut().apply_force(thrust + drag + lift + drag_induced);

        // Cm_alpha is always negative.
        // No need to memorize this--just don't forget it!
        let aero_moment = curr_vel.cross(body_forward) * dyn_pressure * self.cm_alpha;
        self.base_mut().apply_torque(aero_moment);

        // Remove when lifetime exceeded
        self.lifetime -= delta;
        if self.lifetime < 0.0 {
            self.base_mut().queue_free();
        }

        

    }
}

#[godot_api]
impl Rocket {
    #[func]
    fn on_body_entered(&mut self, body: Gd<Node>) {
        let pos = self.base().get_position();
        if let Ok(_) = body.try_cast::<RigidBody3D>() {
            // TODO: This occasionally causes errors because
            // the other player's missile already hit a wall and exploded
            self.base_mut().rpc("explode", vslice![pos]);
        } else {
            self.explode(pos);
        }
    }

    #[rpc(any_peer, call_local, unreliable)]
    pub fn explode(&mut self, position: Vector3) {
        // Explode with an explosion
        let mut explosion = self.explosion_scene
            .instantiate_as::<Explosion>();
        explosion.set_position(position);
        self.base_mut().add_sibling(&explosion);
        self.base_mut().queue_free();
    }
}

