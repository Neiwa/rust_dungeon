use crossterm::event::{KeyCode, MouseButton, MouseEventKind};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Input {
    Key(KeyCode),
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseScrollDown,
    MouseScrollUp,
    MouseScrollLeft,
    MouseScrollRight,
}

pub trait AsInput {
    fn as_input(&self) -> Option<Input>;
}

impl AsInput for MouseEventKind {
    fn as_input(&self) -> Option<Input> {
        match self {
            MouseEventKind::Down(key) | MouseEventKind::Up(key) | MouseEventKind::Drag(key) => {
                key.as_input()
            }
            MouseEventKind::Moved => None,
            MouseEventKind::ScrollDown => Some(Input::MouseScrollDown),
            MouseEventKind::ScrollUp => Some(Input::MouseScrollUp),
            MouseEventKind::ScrollLeft => Some(Input::MouseScrollLeft),
            MouseEventKind::ScrollRight => Some(Input::MouseScrollRight),
        }
    }
}

impl AsInput for MouseButton {
    fn as_input(&self) -> Option<Input> {
        match self {
            MouseButton::Left => Some(Input::MouseLeft),
            MouseButton::Right => Some(Input::MouseRight),
            MouseButton::Middle => Some(Input::MouseMiddle),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub enum InputState {
    Press(Input),
    Release(Input),
    Active(Input),
}
