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

    pub fn initialize(&mut self, id: i64, name: &GString) {
        self.id = id;
        self.name_label.set_text(name);
    }
}
