use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{Area3D, IArea3D, Node3D, INode3D,
    GpuParticles3D, Timer, ITimer
};

#[allow(unused_imports)]
use crate::player::{Player, PlayerKinematicBody, PlayerDynamicBody};

#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct Explosion {
    #[init(node="ExplosionParticles")]
    explosion_particles: OnReady<Gd<GpuParticles3D>>,
    physics_time: f32,
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for Explosion {
    fn ready(&mut self) {
        // Explosion should explode one time
        godot_print!("Time to explode");
        self.explosion_particles.set_one_shot(true);
        self.explosion_particles.set_emitting(true);

        self.explosion_particles
            .signals()
            .finished()
            .connect_other(&self.to_gd(), Self::on_visibility_screen_exited);
    }

    fn physics_process(&mut self, delta: f32) {
        // Keep track of how long the explosion has existed.
        self.physics_time += delta;
    }
}

#[godot_api]
impl Explosion {
    #[func]
    fn on_visibility_screen_exited(&mut self) {
        // Explosion is done exploding
        self.base_mut().queue_free();
    }

    #[func]
    pub fn get_time(&self) -> f32 {
        self.physics_time
    }
}

// A repeating explosion emitter
#[derive(GodotClass)]
#[class(base=Node3D, init)]
pub struct RepeatExploder {
    #[init(val=OnReady::from_loaded("res://explosion/rocket_explosion.tscn"))]
    explosion_scene: OnReady<Gd<PackedScene>>,
    #[init(node="Timer")]
    timer: OnReady<Gd<Timer>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for RepeatExploder {
    fn ready(&mut self) {
        self.timer.start();
        self.timer
            .signals()
            .timeout()
            .connect_other(&self.to_gd(), Self::on_timer_timeout);
    }
}

#[godot_api]
impl RepeatExploder {
    #[func]
    fn on_timer_timeout(&mut self) {
        godot_print!("timeout");
        self.spawn_new_explosion();
        self.timer.start();
    }

    #[func]
    pub fn spawn_new_explosion(&mut self) {
        let explosion = self.explosion_scene
            .instantiate_as::<Explosion>();
        self.base_mut().add_child(&explosion);
    }
}

// A mine. Don't mine it
#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct Mine {
    #[init(val=OnReady::from_loaded("res://explosion/rocket_explosion.tscn"))]
    explosion_scene: OnReady<Gd<PackedScene>>,
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for Mine {
    fn ready(&mut self) {
        self.signals()
            .area_entered()
            .connect_self(Self::on_stomped);
    }
}

#[godot_api]
impl Mine {
    #[func]
    fn on_stomped(&mut self, area: Gd<Area3D>) {
        match area.try_cast::<Player>() {
            Ok(_player) => {
                let mut explosion = self.explosion_scene
                    .instantiate_as::<Explosion>();
                explosion.set_position(self.base().get_position());
                self.base_mut().add_sibling(&explosion);
                self.base_mut().queue_free();
            }
            Err(_) => {}
        }
    }
}


