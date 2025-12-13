use godot::prelude::*;
use godot::classes::input::MouseMode;

#[allow(unused_imports)]
use godot::classes::{Control, IControl, Button, Input, InputEvent};



#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct PauseMenu {
    #[export]
    unpause_button: OnEditor<Gd<Button>>,
    #[export]
    paused: bool,
    base: Base<Control>
}

#[godot_api]
impl IControl for PauseMenu {
    fn ready(&mut self) {
        if self.paused {
            self.pause();
        } else {
            self.unpause();
        }
        self.unpause_button
            .signals()
            .pressed()
            .connect_other(&self.to_gd(), Self::unpause);
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("pause") {
            if !self.paused {
                self.pause();
            }
        }
    }
}

#[godot_api]
impl PauseMenu {
    #[func]
    fn pause(&mut self) {
        self.paused = true;
        self.base_mut().set_visible(true);
        Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
    }

    #[func]
    fn unpause(&mut self) {
        self.paused = false;
        self.base_mut().set_visible(false);
        Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
    }
}
