use crate::{
    client_event::{
        ClientEvent,
        ControlEvent,
        NetworkEvent,
    },
    input::Input,
};

mod menu_state;
use menu_state::MenuState;

use replace_with::replace_with_or_abort;

use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub struct WorldState;

#[derive(Debug)]
pub struct CombatState;

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
        replace_with_or_abort(self, |state| match state {
            ClientState::CombatState(state) => ClientState::CombatState(state),
            ClientState::MenuState(state) => menu_state::MenuState::transform(state, event_loop_proxy, event),
            ClientState::WorldState(state) => ClientState::WorldState(state),
        });

        println!("{:?}", self);
    }
}
