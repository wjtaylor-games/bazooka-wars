use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{Area3D, IArea3D, Node3D, INode3D,
    GpuParticles3D,
};

#[allow(unused_imports)]
use crate::player::{Player, PlayerKinematicBody, PlayerDynamicBody};

#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct Explosion {
    #[init(node="ExplosionParticles")]
    explosion_particles: OnReady<Gd<GpuParticles3D>>,
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for Explosion {
    fn ready(&mut self) {
        // Explosion should explode one time
        self.explosion_particles.set_one_shot(false);
        self.explosion_particles.set_emitting(true);
    }
}

#[godot_api]
impl Explosion {
}

// // Make multiple explosions
// #[derive(GodotClass)]
// #[class(base=Node3D, init)]
// pub struct RepeatExploder {
