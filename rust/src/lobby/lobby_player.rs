use godot::prelude::*;

#[allow(unused_imports)]
use godot::classes::{
    Node, INode,
    Label,
    Control, IControl,
    Button, Os, ColorPickerButton
};

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct LobbyPlayer {
    id: i64,
    #[export]
    name_label: OnEditor<Gd<Label>>,
    #[export]
    color_picker: OnEditor<Gd<ColorPickerButton>>,

    base: Base<Control>,
}

#[godot_api]
impl IControl for LobbyPlayer {

}

#[godot_api]
impl LobbyPlayer {

    pub fn initialize(&mut self, id: i64, name: &GString, is_local: bool) {
        self.id = id;
        self.name_label.set_text(name);
        // Don't want other players editing our color
        if !is_local {
            self.color_picker.set_disabled(true);
        }
    }

    #[func]
    pub fn on_peer_disconnected(&mut self, id: i64) {
        // Check if *this* is the disconnected player
        if id == self.id {
            self.base_mut().queue_free();
        }
    }
}
