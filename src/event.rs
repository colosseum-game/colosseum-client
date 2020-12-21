use crate::input::Input;

// Events which modify the control flow of the application in some way.
#[derive(Clone, Copy, Debug)]
pub enum ControlEvent {
    Terminate,
}

/// Encompasses all event variants.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    ControlEvent(ControlEvent),
    DeltaTimeEvent(f64),
    InputEvent(Input),
}
