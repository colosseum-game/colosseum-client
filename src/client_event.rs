use crate::input::Input;

// Events which modify the control flow of the application in some way.
#[derive(Clone, Copy, Debug)]
pub enum ControlEvent {
    Terminate,
}

/// Events which modify or provide information on the server/client connection.
#[derive(Clone, Copy, Debug)]
pub enum NetworkEvent {
    Connect,
    Connected,
    ConnectFailed,
    Disconnect,
    Disconnected,
}

/// Encompasses all event variants.
#[derive(Clone, Copy, Debug)]
pub enum ClientEvent {
    ControlEvent(ControlEvent),
    Input(Input),
    NetworkEvent(NetworkEvent),
}
