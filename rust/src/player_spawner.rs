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


// A better implementation for 3+ players
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct NPlayers {
    #[init(val=OnReady::from_loaded("res://player/player.tscn"))]
    player_scene: OnReady<Gd<PackedScene>>,
    #[export]
    spawn_points_container: OnEditor<Gd<Node>>,
    player_info: VarDictionary,
    base: Base<Node>,
}

#[godot_api]
impl INode for NPlayers {
    fn ready(&mut self) {
        if self.base().is_multiplayer_authority() {
            let player_info = self.player_info.clone();
            for (id, _name) in player_info.iter_shared().typed::<i64, GString>() {
                self.base_mut().rpc("spawn_player", vslice![id]);
            }
        }
    }
}

#[godot_api]
impl NPlayers {
    // Called only by the host
    pub fn initialize_authority(&mut self, player_info: &VarDictionary) {
        self.player_info = player_info.clone()
    }

    #[rpc(authority, call_local, reliable)]
    pub fn spawn_player(&mut self, peer_id: i64) {
        // Crate player instance
        let mut player: Gd<Player> = self.player_scene.instantiate_as();

        // Set player authority and camera state
        player.set_multiplayer_authority(peer_id as i32);
        // Must set multiplayer authority before adding
        // to scene tree so that it inherits correctly
        self.base_mut().add_child(&player);

        let this_id = self.base().get_multiplayer().unwrap().get_unique_id();
        let do_connect_camera: bool = this_id as i64 == peer_id;
        if do_connect_camera {
            player.bind_mut().set_camera_current(true);
        }
    }
}
