use crate::{
    client_event::{
        ClientEvent,
        ControlEvent,
        NetworkEvent,
    },
    input::Input,
};

use replace_with::replace_with_or_abort;

use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub struct CombatState;

#[derive(Debug)]
pub enum MenuState {
    AwaitingInput { cursor_position: usize },
    ConnectingToServer,
}

#[derive(Debug)]
pub struct WorldState;

#[derive(Debug)]
pub enum ClientState {
    CombatState(CombatState),
    MenuState(MenuState),
    WorldState(WorldState),
}

impl ClientState {
    pub fn new() -> ClientState {
        ClientState::MenuState(MenuState::AwaitingInput { cursor_position: 0 })
    }

    pub fn transform(&mut self, event_loop_proxy: &EventLoopProxy<ClientEvent>, event: ClientEvent) {
        replace_with_or_abort(self, |mut state| match state {
            ClientState::CombatState(ref mut _combat_state) => state,
            ClientState::MenuState(ref mut menu_state) => match menu_state {
                MenuState::AwaitingInput { ref mut cursor_position } => match event {
                    ClientEvent::Input(input) => {
                        match input {
                            Input::Down => *cursor_position = (*cursor_position + 1) % 3,
                            Input::Select => 
                            match *cursor_position {
                                0 => {
                                    event_loop_proxy.send_event(ClientEvent::NetworkEvent(NetworkEvent::Connect)).expect("event loop destroyed");
                                    return ClientState::MenuState(MenuState::ConnectingToServer);
                                },
                                1 => unimplemented!("options menu not added"),
                                2 => event_loop_proxy.send_event(ClientEvent::ControlEvent(ControlEvent::Terminate)).expect("event loop destroyed"),
                                _ => panic!("unmatched cursor position")
                            },
                            Input::Up => *cursor_position = match *cursor_position{
                                0 => 2,
                                _ => *cursor_position - 1,
                            },
                            _ => (),
                        }

                        state
                    }
                    _ => state,
                },
                MenuState::ConnectingToServer => match event {
                    ClientEvent::NetworkEvent(event) => match event {
                        NetworkEvent::Connected => ClientState::WorldState(WorldState),
                        NetworkEvent::ConnectFailed => ClientState::MenuState(MenuState::AwaitingInput { cursor_position: 0 }),
                        _ => state,
                    }
                    _ => state,
                }
            }
            ClientState::WorldState(state) => ClientState::WorldState(state),
        });

        println!("{:?}", self);
    }
}
