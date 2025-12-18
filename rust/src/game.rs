use godot::prelude::*;
use crate::player_spawner::NPlayers;

#[allow(unused_imports)]
use godot::classes::{
    Node, INode,
    Node3D, INode3D,
};


#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct Game {
    #[export]
    player_spawner: OnEditor<Gd<NPlayers>>,
    base: Base<Node3D>,
}

impl Game {
    // Called only by the host
    pub fn initialize_authority(&mut self, player_info: &VarDictionary) {
        self.player_spawner.bind_mut().initialize_authority(player_info);
    }
}

