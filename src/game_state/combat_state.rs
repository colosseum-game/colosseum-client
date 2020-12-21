use colosseum::{
    combatant::Combatant,
    combat_event::CombatEvent,
    connection::Connection,
    message::Message,
    target::Target,
};

use crate::{
    event::Event,
    input::Input,
};

use super::GameState;

use tokio::runtime::Runtime;

type InternalCombatState = colosseum::combat_state::CombatState;

#[derive(Clone, Copy, Debug)]
pub enum GetTargetsType {
    Weapon,
    Skill(usize),
    Item(usize),
}

#[derive(Debug)]
pub enum CombatStateType {
    Waiting,
    GetAction { target: Target, action_index: usize },
    GetSkill { skill_index: usize },
    GetItem { item_index: usize },
    GetTargets { get_targets_type: GetTargetsType, targets: Vec<Target> },
    Confirm { combat_event: CombatEvent },
}

#[derive(Debug)]
pub struct CombatState {
    server: Connection,
    combat_state: InternalCombatState,
    combat_state_type: CombatStateType,
}

impl CombatState {
    pub fn new(server: Connection, mut combat_state: InternalCombatState) -> Self {
        CombatState {
            server,
            combat_state,
            combat_state_type: CombatStateType::Waiting,
        }
    }

    pub fn transform(mut self, runtime: &Runtime, event: Event) -> GameState {
        use CombatStateType::*;
        match self.combat_state_type {
            Waiting => {
                let message = runtime.block_on(self.server.read_message()).unwrap().unwrap();
                match message {
                    Message::CombatEvent(event) => self.combat_state.transform(, event),
                    Message::TakeTurn(target) => self.combat_state_type = CombatStateType::GetAction { target, action_index: 0 },
                };
            },
            GetAction { target, ref mut action_index } => match event {
                Event::InputEvent(input) => {
                    match input {
                        Input::Down => *action_index = (*action_index + 1) % 4,
                        Input::Select => match action_index {
                            0 => self.combat_state_type = CombatStateType::GetTargets { get_targets_type: GetTargetsType::Weapon, targets: vec![] },
                            1 => self.combat_state_type = CombatStateType::GetSkill { skill_index: 0 },
                            2 => self.combat_state_type = CombatStateType::GetItem { item_index: 0 },
                            3 => (), // skip turn
                            _ => panic!("unmatched action index"),
                        },
                        Input::Up => if *action_index == 0 { *action_index = 3 } else { *action_index -= 1},
                        _ => (),
                    }
                }
                _ => (),
            },
            GetSkill { skill_index } => {},
            GetItem { item_index } => {},
            GetTargets { get_targets_type, ref mut targets } => {},
            Confirm { ref combat_event } => match event {
                Event::InputEvent(input) => match input {
                    Input::Select => {
                        let message = Message::CombatEvent(combat_event.clone());
                        runtime.block_on(self.server.write_message(&message)).unwrap();
                        self.combat_state_type = CombatStateType::Waiting;
                    }
                    _ => (),
                }
                _ => (),
            },
        }

        GameState::CombatState(self)
    }
}
