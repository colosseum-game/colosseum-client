use crate::{
    client_event::{
        ClientEvent,
        ControlEvent,
        NetworkEvent,
    },
    input::Input,
};

use super::{
    ClientState,
    WorldState,
};

use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub enum MenuState {
    AwaitingInput { cursor_position: usize },
    ConnectingToServer,
}

impl MenuState {
    pub fn transform(mut self, event_loop_proxy: &EventLoopProxy<ClientEvent>, event: ClientEvent) -> ClientState {
        match self {
            MenuState::AwaitingInput { ref mut cursor_position } => match event {
                ClientEvent::Input(input) => {
                    match input {
                        Input::Down => *cursor_position = (*cursor_position + 1) % 3,
                        Input::Select => {
                            if *cursor_position == 0 {
                                event_loop_proxy.send_event(ClientEvent::NetworkEvent(NetworkEvent::Connect)).expect("event loop destroyed");
                                self = MenuState::ConnectingToServer;
                            } else if *cursor_position == 1 {
                                unimplemented!("options menu not added");
                            } else if *cursor_position == 2 {
                                event_loop_proxy.send_event(ClientEvent::ControlEvent(ControlEvent::Terminate)).expect("event loop destroyed");
                            } else {
                                panic!("unmatched cursor position");
                            }
                        },
                        Input::Up => if *cursor_position == 0 { *cursor_position = 2; } else { *cursor_position -= 1; }
                        _ => (),
                    }
                }
                _ => (),
            },
            MenuState::ConnectingToServer => match event {
                ClientEvent::NetworkEvent(event) => match event {
                    NetworkEvent::Connected => return ClientState::WorldState(WorldState),
                    NetworkEvent::ConnectFailed => self = MenuState::AwaitingInput { cursor_position: 0 },
                    _ => (),
                }
                _ => (),
            }
        }

        ClientState::MenuState(self)
    }
}
