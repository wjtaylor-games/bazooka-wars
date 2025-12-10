use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{Area3D, IArea3D, Node3D, INode3D
};

#[allow(unused_imports)]
use crate::player::{Player, PlayerKinematicBody, PlayerDynamicBody};

#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct Explosion {
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for Explosion {
    fn ready(&mut self) {
    }
}

#[godot_api]
impl Explosion {

}
