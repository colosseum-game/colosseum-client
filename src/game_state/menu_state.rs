use colosseum::{
    connection::Connection,
    message::Message,
};

use crate::{
    event::Event,
    input::Input,
};

use std::net::SocketAddr;

use super::{
    CombatState,
    GameState,
};

use tokio::{
    net::TcpStream,
    runtime::Runtime,
};

#[derive(Debug)]
pub enum MenuState {
    AwaitingInput { cursor_position: usize },
    AwaitingCombatState { server: Connection },
}

impl MenuState {
    pub fn transform(mut self, terminate_application: &mut bool, runtime: &Runtime, event: Event) -> GameState {
        let state = match self {
            MenuState::AwaitingInput { ref mut cursor_position } => match event {
                Event::InputEvent(input) => {
                    match input {
                        Input::Down => *cursor_position = (*cursor_position + 1) % 3,
                        Input::Select => match *cursor_position {
                            0 => {
                                let addr: SocketAddr = "127.0.0.1:40004".parse().unwrap();
                                let stream = runtime.block_on(TcpStream::connect(addr)).unwrap();
                                let server = Connection::new(addr, stream);
                                return GameState::MenuState(MenuState::AwaitingCombatState { server });
                            },
                            1 => unimplemented!("options menu not added"),
                            2 => *terminate_application = true,
                            _ => panic!("unmatched cursor position")
                        },
                        Input::Up => *cursor_position = match *cursor_position {
                            0 => 2,
                            _ => *cursor_position - 1,
                        },
                        _ => (),
                    }

                    self
                }
                _ => self,
            },
            MenuState::AwaitingCombatState { mut server } => match event {
                Event::DeltaTimeEvent(delta_time) => {
                    if let Some(message) = runtime.block_on(server.read_message()).unwrap() {
                        if let Message::CombatState(combat_state) = message {
                            return GameState::CombatState(CombatState::new(server, combat_state));
                        }
                    }

                    MenuState::AwaitingCombatState { server }
                },
                _ => MenuState::AwaitingCombatState { server },
            }
        };

        GameState::MenuState(state)
    }
}
