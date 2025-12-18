use godot::classes::enet_connection::CompressionMode;
#[allow(unused_imports)]
use godot::classes::object::ConnectFlags;
use godot::classes::{
    Button, ENetMultiplayerPeer, Label, LineEdit, LinkButton, ProjectSettings,
    Control, IControl,
    Input,
};
use godot::classes::input::{MouseMode};
use godot::global::Error;
use godot::prelude::*;

mod lobby_player;
use lobby_player::LobbyPlayer;

const DEFAULT_PORT: i32 = 8910;

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct Lobby {
    #[init(val=OnReady::from_loaded("res://lobby/lobby_player.tscn"))]
    lobby_player_scene: OnReady<Gd<PackedScene>>,
    #[export]
    join_panel: OnEditor<Gd<Control>>,
    #[export]
    created_lobby: OnEditor<Gd<Control>>,
    #[export]
    players_joined_container: OnEditor<Gd<Control>>,
    #[export]
    name_input: OnEditor<Gd<LineEdit>>,
    #[export]
    address: OnEditor<Gd<LineEdit>>,
    #[export]
    host_button: OnEditor<Gd<Button>>,
    #[export]
    start_game_button: OnEditor<Gd<Button>>,
    #[export]
    join_button: OnEditor<Gd<Button>>,
    #[export]
    status_ok: OnEditor<Gd<Label>>,
    #[export]
    status_fail: OnEditor<Gd<Label>>,
    #[export]
    port_forward_label: OnEditor<Gd<Label>>,
    #[export]
    find_public_ip_button: OnEditor<Gd<LinkButton>>,
    peer: Option<Gd<ENetMultiplayerPeer>>,
    base: Base<Control>,
}

#[godot_api]
impl IControl for Lobby {
    fn ready(&mut self) {
        /*
        # Connect all the callbacks related to networking.
        multiplayer.peer_connected.connect(_player_connected)
        multiplayer.peer_disconnected.connect(_player_disconnected)
        multiplayer.connected_to_server.connect(_connected_ok)
        multiplayer.connection_failed.connect(_connected_fail)
        multiplayer.server_disconnected.connect(_server_disconnected)
        */
        let multiplayer = self.base().get_multiplayer().unwrap();
        let gd_ref = self.to_gd();

        // Shows the main menu
        self.hide_lobby();

        // multiplayer
        //     .signals()
        //     .connected_to_server()
        //     .builder()
        //     .connect_other_gd(&gd_ref, |mut this: Gd<Self>| {
        //         if let Some(peer) = &this.bind().peer {
        //         } else {
        //             godot_error!("peer is None type!")
        //         }
        //     });

        multiplayer
            .signals()
            .peer_connected()
            .builder()
            .connect_other_gd(&gd_ref, |mut this: Gd<Self>, id: i64| {
                
                let mut multiplayer = this.bind().base().get_multiplayer().unwrap();
                if multiplayer.is_server() {
                } else {
                    this.bind_mut().show_lobby();
                }

                if let Some(peer) = &this.bind().peer {
                    godot_print!("Player {} met! I am {}", id, peer.get_unique_id());
                } else {
                    godot_print!("Player {} met! I have no peer", id);
                }
                let name: GString = this.bind().name_input.get_text().clone();

                // Tell the other player about yourself
                this.rpc_id(id, "register_player", vslice![name]);

            });

        // multiplayer
        //     .signals()
        //     .peer_disconnected()
        //     .builder()
        //     .connect_other_mut(&gd_ref, |this, _id: i64| {
        //         if this.base().get_multiplayer().unwrap().is_server() {
        //             this.end_game("Client disconnected.");
        //         } else {
        //             this.end_game("Server disconnected.");
        //         }
        //     });

        multiplayer
            .signals()
            .connection_failed()
            .builder()
            .connect_other_mut(&gd_ref, |this| {
                this.set_status("Couldn't connect.", false);
                let mut multiplayer = this.base().get_multiplayer().unwrap();
                multiplayer.set_multiplayer_peer(Gd::null_arg()); // Remove peer.
                this.host_button.set_disabled(false);
                this.join_button.set_disabled(false);
            });
        multiplayer
            .signals()
            .server_disconnected()
            .builder()
            .connect_other_mut(&gd_ref, |this| {
                this.end_game("Server disconnected.");
            });

        self.host_button
            .signals()
            .pressed()
            .builder()
            .connect_other_mut(&gd_ref, |this| {
                this.on_host_btn_pressed();
            });

        self.join_button
            .signals()
            .pressed()
            .builder()
            .connect_other_mut(&gd_ref, |this| {
                this.on_join_btn_pressed();
            });

        self.start_game_button
            .signals()
            .pressed()
            .connect_other(&gd_ref, Self::on_start_game_pressed);

    }
}

#[godot_api]
impl Lobby {
    fn set_status(&mut self, text: &str, is_ok: bool) {
        // Simple way to show status.
        if is_ok {
            self.status_ok.set_text(text);
            self.status_fail.set_text("");
        } else {
            self.status_ok.set_text("");
            self.status_fail.set_text(text);
        }
    }

    fn on_start_game_pressed(&mut self) {
        if self.base().get_multiplayer().unwrap().is_server() {
            self.base_mut().rpc("start_game", &[]);
        }
    }

    #[rpc(authority, call_local)]
    fn start_game(&mut self) {
        // Instantiate the game scene
        let game = load::<PackedScene>("res://game.tscn").instantiate_as::<Node3D>();

        self.base_mut()
            .get_tree()
            .unwrap()
            .get_root()
            .unwrap()
            .add_child(&game);
        self.base_mut().hide();
    }

    fn end_game(&mut self, with_error: &str) {
        if self.base().has_node("/root/Game") {
            // Erase immediately, otherwise network might show
            // errors (this is why we connected deferred above).
            self.base().get_node_as::<Node>("/root/Game").free();
            self.base_mut().show();
            // Free the mouse cursor
            let mut input = Input::singleton();
            input.set_mouse_mode(MouseMode::VISIBLE);
        }

        let mut multiplayer = self.base().get_multiplayer().unwrap();
        multiplayer.set_multiplayer_peer(Gd::null_arg()); // Remove peer.

        self.hide_lobby();

        self.set_status(with_error, false);
    }

    fn on_host_btn_pressed(&mut self) {
        let mut peer = ENetMultiplayerPeer::new_gd();
        self.peer = Some(peer.clone());
        // Set a maximum of ... let's say 10 players.
        let err = peer.create_server_ex(DEFAULT_PORT).max_clients(9).done();
        if err != Error::OK {
            // Is another server running?
            self.set_status("Can't host, address in use.", false);
            return;
        }
        peer.get_host()
            .unwrap()
            .compress(CompressionMode::RANGE_CODER);

        let mut multiplayer = self.base().get_multiplayer().unwrap();
        multiplayer.set_multiplayer_peer(&peer);
        self.show_lobby();
        let application_name = ProjectSettings::singleton()
            .get_setting("application/config/name")
            .to_string();
        self.base_mut()
            .get_window()
            .unwrap()
            .set_title(&format!("{application_name}: Server"));
        // Only show hosting instructions when relevant.
        self.port_forward_label.set_visible(true);
        self.find_public_ip_button.set_visible(true);
        self.add_new_lobby_player(
            peer.get_unique_id() as i64,
            &self.name_input.get_text(),
            true
        );
    }

    fn on_join_btn_pressed(&mut self) {
        let ip = self.address.get_text();
        if !ip.is_valid_ip_address() {
            self.set_status("IP address is invalid.", false);
            return;
        }

        let mut peer = ENetMultiplayerPeer::new_gd();
        self.peer = Some(peer.clone());
        peer.create_client(&ip, DEFAULT_PORT);
        peer.get_host()
            .unwrap()
            .compress(CompressionMode::RANGE_CODER);
        let mut multiplayer = self.base().get_multiplayer().unwrap();
        multiplayer.set_multiplayer_peer(&peer);

        self.set_status("Connecting...", true);
        let application_name = ProjectSettings::singleton()
            .get_setting("application/config/name")
            .to_string();
        self.base_mut()
            .get_window()
            .unwrap()
            .set_title(&format!("{application_name}: Client"));
        self.add_new_lobby_player(
            peer.get_unique_id() as i64,
            &self.name_input.get_text(),
            true
        );
    }

    #[rpc(any_peer, call_remote, reliable)]
    fn register_player(&mut self, name: GString) {
        let mut multiplayer = self.base().get_multiplayer().unwrap();
        let id = multiplayer.get_remote_sender_id() as i64;
        self.add_new_lobby_player(id, &name, false);
    }

    fn add_new_lobby_player(&mut self, id: i64, name: &GString, is_self: bool) {
        let mut lobby_player: Gd<LobbyPlayer> = self.lobby_player_scene.instantiate_as();
        lobby_player.bind_mut().initialize(id, &name, is_self);

        // Connect signals to the LobbyPlayer
        let multiplayer = self.base().get_multiplayer().unwrap();
        multiplayer
            .signals()
            .peer_disconnected()
            .connect_other(&lobby_player, LobbyPlayer::on_peer_disconnected);

        self.players_joined_container.add_child(&lobby_player);
    }

    fn show_lobby(&mut self) {
        self.join_panel.hide();
        self.created_lobby.show();
    }

    fn hide_lobby(&mut self) {
        self.join_panel.show();
        self.created_lobby.hide();
    }
}
