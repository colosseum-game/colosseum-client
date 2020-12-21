use gilrs::{
    Button,
    Gilrs,
};

use std::collections::{
    HashMap,
    VecDeque,
};

use winit::event::VirtualKeyCode;

/// TODO: description
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
    Right,
    Select,
    Up,
}

pub struct InputState {
    gilrs: Gilrs,
    input_map: HashMap<RawInput, Input>,
    input_queue: VecDeque<Input>,
}

impl InputState {
    pub fn new() -> InputState {
        let input_map: HashMap<RawInput, Input> = [
            (RawInput::Gamepad(Button::DPadDown), Input::Down),
            (RawInput::Gamepad(Button::DPadUp), Input::Up),
            (RawInput::Gamepad(Button::DPadRight), Input::Right),
            (RawInput::Gamepad(Button::DPadLeft), Input::Left),
            (RawInput::Gamepad(Button::East), Input::Cancel),
            (RawInput::Gamepad(Button::South), Input::Select),
            (RawInput::Keyboard(VirtualKeyCode::A), Input::Left),
            (RawInput::Keyboard(VirtualKeyCode::D), Input::Right),
            (RawInput::Keyboard(VirtualKeyCode::E), Input::Select),
            (RawInput::Keyboard(VirtualKeyCode::Q), Input::Cancel),
            (RawInput::Keyboard(VirtualKeyCode::S), Input::Down),
            (RawInput::Keyboard(VirtualKeyCode::W), Input::Up),
        ].iter().cloned().collect();

        InputState {
            gilrs: Gilrs::new().unwrap(),
            input_map,
            input_queue: VecDeque::new(),
        }
    }

    pub fn poll(&mut self) {
        while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(button, ..) => {
                    if let Some(input) = self.input_map.get(&RawInput::Gamepad(button)) {
                        self.input_queue.push_back(*input)
                    }
                },
                _ => (),
            };
        }
    }

    pub fn push_input(&mut self, input: Input) {
        self.input_queue.push_back(input);
    }

    pub fn push_input_raw(&mut self, input: RawInput) {
        if let Some(input) = self.input_map.get(&input) {
            self.push_input(*input);
        }
    }

    pub fn pop_input(&mut self) -> Option<Input> {
        self.input_queue.pop_front()
    }
}
