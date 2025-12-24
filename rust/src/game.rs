use godot::prelude::*;
use crate::player_spawner::NPlayers;
use crate::pause_menu::PauseMenu;

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
    #[export]
    pause_menu: OnEditor<Gd<PauseMenu>>,
    #[init(val=0.7)]
    global_mouse_sensitivity: f64,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Game {
    fn ready(&mut self) {
        let gd_ref = self.to_gd();
        self.pause_menu
            .bind()
            .get_sensitivity_slider()
            .unwrap()
            .signals()
            .value_changed()
            .builder()
            .connect_other_mut(&gd_ref, |this, value: f64| {
                this.global_mouse_sensitivity = value;
                this.signals().mouse_sensitivity_changed().emit(value);
            });
        let ms = self.global_mouse_sensitivity;
        self.pause_menu.bind_mut().get_sensitivity_slider()
            .unwrap().set_value(ms);
        self.signals().mouse_sensitivity_changed().emit(ms);
    }
}

#[godot_api]
impl Game {
    // Called only by the host
    pub fn initialize_authority(&mut self, player_info: &VarDictionary) {
        self.player_spawner.bind_mut().initialize_authority(player_info);
    }

    #[signal]
    pub fn mouse_sensitivity_changed(value: f64);
}

