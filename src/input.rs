use gilrs::Button;

use winit::event::VirtualKeyCode;

/// TODO
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RawInput {
    Gamepad(Button),
    Keyboard(VirtualKeyCode),
}

/// A discriminated union which describes all of the possible application level inputs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Input {
    Cancel,
    Down,
    Left,
    Next,
    Previous,
    Right,
    Select,
    Up,
}
