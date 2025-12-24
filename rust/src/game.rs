use godot::prelude::*;
use crate::player_spawner::NPlayers;
use crate::pause_menu::PauseMenu;

#[allow(unused_imports)]
use godot::classes::{
    Node, INode,
    Node3D, INode3D,
    ConfigFile,
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

        let mut config_file = ConfigFile::new_gd();
        config_file.load("user://settings.cfg");

        let mut ms: f64 = self.global_mouse_sensitivity;
        let value: Variant = config_file.get_value_ex(
            "Controls",
            "mouse_sensitivity",
        ).default(&Variant::from(ms)).done();
        ms = if let Ok(value) = value.try_to::<f64>() {
            // self.global_mouse_sensitivity = value;
            value
        } else {
            self.global_mouse_sensitivity
        };

        self.pause_menu.bind_mut().get_sensitivity_slider()
            .unwrap().set_value(ms);

        self.pause_menu
            .signals()
            .unpause()
            .builder()
            .connect_other_mut(&gd_ref, |this| {
                let mut config_file = ConfigFile::new_gd();
                config_file.set_value(
                    "Controls",
                    "mouse_sensitivity",
                    &Variant::from(this.global_mouse_sensitivity),
                );
                config_file.save("user://settings.cfg");
                godot_print!("Configs saved! {}", this.global_mouse_sensitivity);
            });

        self.pause_menu
            .bind_mut()
            .get_sensitivity_slider()
            .unwrap()
            .signals()
            .value_changed()
            .builder()
            .connect_other_mut(&gd_ref, |this, value: f64| {
                this.global_mouse_sensitivity = value;
                this.signals().mouse_sensitivity_changed().emit(value);
            });

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

