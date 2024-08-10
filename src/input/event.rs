use winit::{
    event::{KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Hash, Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum InputType {
    Key(KeyCode),
    MouseButton(MouseButton),
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Input {
    pub(super) ty: InputType,
    pub(super) value: f32,
}

impl From<&WindowEvent> for Input {
    fn from(value: &WindowEvent) -> Self {
        match value {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if !repeat {
                    Input {
                        ty: InputType::Key(*code),
                        value: if state.is_pressed() { 1.0 } else { 0.0 },
                    }
                } else {
                    Input::default()
                }
            }
            WindowEvent::MouseInput { state, button, .. } => Input {
                ty: InputType::MouseButton(*button),
                value: if state.is_pressed() { 1.0 } else { 0.0 },
            },
            _ => Input::default(),
        }
    }
}
