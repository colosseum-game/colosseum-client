use gilrs::{
    Button,
    Event,
    EventType,
    Gilrs,
};

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Unmapped,
    Select,
    Cancel,
    Next,
    Previous,
}

#[derive(Debug)]
pub struct InputState {
    gilrs: gilrs::Gilrs,
    pub input_buffer: Vec<Input>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            gilrs: Gilrs::new().unwrap(),
            input_buffer: vec![],
        }
    }

    pub fn update(mut self) -> Self {
        self.input_buffer = vec![];

        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            self.input_buffer.push(match event {
                EventType::ButtonPressed(button, ..) => {
                    match button {
                        Button::DPadDown => Input::Next,
                        Button::DPadUp => Input::Previous,
                        Button::East => Input::Cancel,
                        Button::Select => Input::Cancel,
                        Button::South => Input::Select,
                        Button::Start => Input::Select,
                        _ => Input::Unmapped,
                    }
                },
                _ => Input::Unmapped,
            });
        }

        self
    }
}
