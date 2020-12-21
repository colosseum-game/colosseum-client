mod combat_state;
use combat_state::CombatState;

use crate::event::Event;

mod menu_state;
use menu_state::MenuState;

use replace_with::replace_with_or_abort;

use tokio::runtime::Runtime;

#[derive(Debug)]
pub enum GameState {
    CombatState(CombatState),
    MenuState(MenuState),
}

impl GameState {
    pub fn new() -> GameState {
        GameState::MenuState(MenuState::AwaitingInput { cursor_position: 0 })
    }

    pub fn transform(&mut self, terminate_application: &mut bool, runtime: &Runtime, event: Event) {
        replace_with_or_abort(self, |state| match state {
            GameState::CombatState(combat_state) => combat_state.transform(runtime, event),
            GameState::MenuState(menu_state) => menu_state.transform(terminate_application, runtime, event),
        });
    }
}
