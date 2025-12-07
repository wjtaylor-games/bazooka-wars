use godot::prelude::*;
use godot::classes::{RigidBody3D, IRigidBody3D, CharacterBody3D, ICharacterBody3D,
    Input, InputEvent};
// use std::f32::consts::{TAU, PI};
// use godot::global::{wrapf};
// use num::clamp;


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
#[class(base=CharacterBody3D, init)]
pub struct PlayerKinematicBody {
    base: Base<CharacterBody3D>
}

#[derive(GodotClass)]
#[class(base=RigidBody3D, init)]
pub struct PlayerDynamicBody {
    base: Base<RigidBody3D>
}
