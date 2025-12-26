use godot::prelude::*;
use godot::classes::{Node, INode};

use crate::player::Player;


// A spawner for any number of players
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct NPlayers {
    #[init(val=OnReady::from_loaded("res://player/player.tscn"))]
    player_scene: OnReady<Gd<PackedScene>>,
    #[init(node="../Arena1/SpawnPoints")]
    spawn_points_container: OnReady<Gd<Node>>,
    player_info: VarDictionary,
    base: Base<Node>,
}

#[godot_api]
impl INode for NPlayers {
    fn ready(&mut self) {
        // self.base().get_node_as::<Node>("../Arena1");

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

        player.set_position(self.sample_spawn_point());

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

        player.signals()
            .out_of_bounds()
            .connect_other(&self.to_gd(), Self::respawn_player);
    }

    pub fn respawn_player(&mut self, mut player: Gd<Player>) {
        let pos = self.sample_spawn_point();
        player.rpc("respawn", vslice![pos]);
    }

    fn sample_spawn_point(&self) -> Vector3 {
        // Pick a random spawn point
        let spawn_points = self.spawn_points_container.get_children();
        let point = spawn_points.pick_random().unwrap().cast::<Node3D>();
        point.get_position()
    }

    // fn sample_spawn_point_filtered(&mut self) -> Vector3 {
        // Pick a random spawn point, filtering out ones with players nearby
        // let spawn_points = self.spawn_points_container.get_children();
        // PackedVector3Array::
        // let filtered = spawn_points
        //     .functional_ops()
        //     .filter(&Callable::from_fn("is_even", |args| {
        //         let arg = args[0];
        //         *arg.try_to::<Node3D>();
        //         true
// }));
        // let point = filtered.pick_random().unwrap().cast::<Node3D>();
        // point.get_position()
    // }

    #[func]
    fn filter_point(&mut self, args: Array<Variant>) -> bool {
        godot_print!("Filtering: {:?}", args);
        // point: Gd<Node3D> = args;
        true
    }
}
