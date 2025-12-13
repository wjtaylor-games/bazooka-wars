use godot::prelude::*;

struct BazookaExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BazookaExtension {}


mod lobby;
mod player;
mod player_spawner;
mod explosion;
mod rocket;
mod pause_menu;
