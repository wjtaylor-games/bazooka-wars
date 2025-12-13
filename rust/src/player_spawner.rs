use godot::prelude::*;
use godot::classes::{Node, INode};

use crate::player::Player;

// A good-enough implementation for a 2 player game
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct TwoPlayers {
    #[export]
    host_player: OnEditor<Gd<Player>>,
    #[export]
    client_player: OnEditor<Gd<Player>>,
    base: Base<Node>,
}

#[godot_api]
impl INode for TwoPlayers {
    fn ready(&mut self) {
        if self.base().get_multiplayer().unwrap().is_server() {
            // For the server, give control of player 2 to the other peer.
            let authority = self.base().get_multiplayer().unwrap().get_peers()[0];
            self.client_player.set_multiplayer_authority(authority);
            self.host_player.bind_mut().set_camera_current(true);
        } else {
            // For the client, give control of player 2 to itself.
            let authority = self.base().get_multiplayer().unwrap().get_unique_id();
            self.client_player.set_multiplayer_authority(authority);
            self.client_player.bind_mut().set_camera_current(true);
        }
    }
}
