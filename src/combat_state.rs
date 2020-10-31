use colosseum::{
    actions::*,
    combatant::{
        Combatant,
        Stat,
    },
    damage::{
        DamageAspect,
        StatusEffectEntry,
    },
    effects::{
        Effect,
        EffectSource,
    },
    lifetime::Lifetime,
    math::Fraction,
};

use crate::input_state::{
    Input,
    InputState
};

use either::Either;

#[derive(Clone, Debug)]
pub struct PreTurn {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
}

#[derive(Clone, Debug)]
pub struct GetAction {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
    source_index: usize,
    action_index: usize,
}

#[derive(Clone, Debug)]
pub struct GetSubAction {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
    source_index: usize,
    action_index: usize,
    sub_action_index: usize,
    target_indices: Vec<Vec<usize>>,
}

#[derive(Clone, Debug)]
pub struct GetTargets {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
    source_index: usize,
    action_index: usize,
    sub_action_index: usize,
    target_indices: Vec<Vec<usize>>,
    possible_targets: Vec<usize>,
    target_index: usize,
    target_count: usize,
}

#[derive(Clone, Debug)]
pub struct ApplyAction {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
    source_index: usize,
    action_index: usize,
    target_indices: Vec<Vec<usize>>
}

#[derive(Clone, Debug)]
pub struct PostTurn {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
    source_index: usize
}

#[derive(Clone, Debug)]
pub enum CombatStateType {
    PreTurn(PreTurn),
    GetAction(GetAction),
    GetSubAction(GetSubAction),
    GetTargets(GetTargets),
    ApplyAction(ApplyAction),
    PostTurn(PostTurn),
}

#[derive(Debug)]
pub struct CombatState {
    state_type: CombatStateType,
    state_buffer: Vec<CombatStateType>,
}

impl CombatState {
    pub fn new(combatants: Vec<Combatant>) -> Self {
        let mut turn_order = vec![0; combatants.len()];
        for i in 0..combatants.len() { turn_order[i] = i; }
        turn_order.sort_by(|a, b| combatants[*b].get_stat(Stat::Agility).cmp(&combatants[*a].get_stat(Stat::Agility)));

        Self {
            state_type: CombatStateType::PreTurn(PreTurn {
                combatants,
                turn_order,
                turn_order_iterator: 0,
            }),
            state_buffer: vec![],
        }
    }

    pub fn update(mut self, input_state: &InputState) -> Self {
        for input in &input_state.input_buffer {
            match input {
                Input::Unmapped => (),
                Input::Select => match self.state_type {
                    CombatStateType::GetAction(state) => {
                        self.state_buffer = vec![CombatStateType::GetAction(state.clone())];
                        self.state_type = CombatStateType::GetSubAction(GetSubAction::from(state));
                    },
                    CombatStateType::GetTargets(state) => {
                        self.state_buffer.push(CombatStateType::GetTargets(state.clone()));
                        self.state_type = match Either::from(state) {
                            Either::Left(state) => CombatStateType::GetSubAction(GetSubAction::from(state)),
                            Either::Right(state) => CombatStateType::GetTargets(GetTargets::from(state)),
                        }
                    },
                    _ => (),
                },
                Input::Cancel => if let Some(state) = self.state_buffer.pop() { self.state_type = state; },
                Input::Next | Input::Previous => match self.state_type {
                    CombatStateType::GetAction(state) => self.state_type = CombatStateType::GetAction(state.transform(*input)),
                    CombatStateType::GetTargets(state) => self.state_type = CombatStateType::GetTargets(state.transform(*input)),
                    _ => (),
                },
            }

            if !matches!(input, Input::Unmapped) {
                println!("{:?}", self.state_type);
            }
        }

        loop {
            match self.state_type {
                CombatStateType::PreTurn(state) => self.state_type = CombatStateType::GetAction(GetAction::from(state)),
                CombatStateType::GetSubAction(state) => self.state_type = match Either::from(state) {
                    Either::Left(state) => CombatStateType::GetTargets(state),
                    Either::Right(state) => CombatStateType::ApplyAction(state),
                },
                CombatStateType::ApplyAction(state) => self.state_type = CombatStateType::PostTurn(PostTurn::from(state)),
                CombatStateType::PostTurn(state) => self.state_type = CombatStateType::PreTurn(PreTurn::from(state)),
                _ => break,
            }
        }

        self
    }
}

impl From<PreTurn> for GetAction {
    fn from(from: PreTurn) -> Self {
        let mut combatants = from.combatants;
        let turn_order = from.turn_order;
        let turn_order_iterator = (from.turn_order_iterator + 1) % turn_order.len();
        let source_index = turn_order[from.turn_order_iterator]; 
        let action_index = 0;

        // decrement status_effect lifetimes
        for status_effect in &mut combatants[source_index].status_effects {
            if let Lifetime::Active(ref mut lifetime) = status_effect.lifetime {
                *lifetime -= std::cmp::min(*lifetime, 1);
            }
        }

        // decrement modifier lifetimes
        for stat in 0..Stat::MaxValue as usize {
            for modifier in &mut combatants[source_index].modifiers[stat] {
                if let Lifetime::Active(ref mut lifetime) = modifier.lifetime {
                    *lifetime -= std::cmp::min(*lifetime, 1)
                }
            }
        }

        GetAction { combatants, turn_order, turn_order_iterator, source_index, action_index }
    }
}

impl From<GetAction> for GetSubAction {
    fn from(from: GetAction) -> Self {
        GetSubAction {
            combatants: from.combatants,
            turn_order: from.turn_order,
            turn_order_iterator: from.turn_order_iterator,
            source_index: from.source_index,
            action_index: from.action_index,
            sub_action_index: 0,
            target_indices: vec![],
        }
    }
}

impl GetAction {
    pub fn transform(mut self, input: Input) -> Self {
        self.action_index = match input {
            Input::Next => (self.action_index + 1) % self.combatants[self.source_index].actions.len(),
            Input::Previous => if self.action_index == 0 { self.combatants[self.source_index].actions.len() - 1 } else { self.action_index - 1 },
            _ => self.action_index,
        };

        self
    }
}

impl From<GetSubAction> for Either<GetTargets, ApplyAction> {
    fn from(from: GetSubAction) -> Self {
        let combatants = from.combatants;
        let turn_order = from.turn_order;
        let turn_order_iterator = from.turn_order_iterator;
        let source_index = from.source_index;
        let action_index = from.action_index;
        let sub_action_index = from.sub_action_index;
        let target_indices = from.target_indices;

        let action = Action::from_identifier(combatants[source_index].actions[action_index]);
        match action.sub_actions.get(sub_action_index) {
            Some(sub_action) => {
                let possible_targets = combatants
                    .iter()
                    .enumerate()
                    .filter(|&(index, combatant)| {
                        sub_action.target_flags.iter().fold(true, |is_valid_target, or_conditions| {
                            is_valid_target && or_conditions.iter().fold(false, |is_valid_target, target_flag| {
                                is_valid_target || match *target_flag {
                                    TargetFlag::Any => true,
                                    TargetFlag::Gender(gender) => combatant.gender == gender,
                                    TargetFlag::Origin => source_index == index,
                                }
                            })
                        }
                    )})
                    .map(|(index, _)| index)
                    .collect();
    
                Either::Left(
                    GetTargets {
                        combatants, turn_order, turn_order_iterator, source_index,
                        action_index, sub_action_index, target_indices,
                        possible_targets, target_index: 0, target_count: 0,
                    }
                )
            },
            None => Either::Right(
                ApplyAction {
                    combatants, turn_order, turn_order_iterator,
                    source_index, action_index, target_indices,
                }
            )
        }
    }
}

impl From<GetTargets> for Either<GetSubAction, GetTargets> {
    fn from(from: GetTargets) -> Self {
        let combatants = from.combatants;
        let turn_order = from.turn_order;
        let turn_order_iterator = from.turn_order_iterator;
        let source_index = from.source_index;
        let action_index = from.action_index;
        let mut sub_action_index = from.sub_action_index;
        let mut target_indices = from.target_indices;
        let possible_targets = from.possible_targets;
        let target_index = from.target_index;
        let mut target_count = from.target_count;

        let action = Action::from_identifier(combatants[source_index].actions[action_index]);

        match action.sub_actions[sub_action_index].targeting_scheme {
            TargetingScheme::All => {
                // push all possible targets to target indices
                sub_action_index += 1;
                target_indices.push(possible_targets);

                Either::Left(GetSubAction {
                    combatants, turn_order, turn_order_iterator,
                    source_index, action_index, sub_action_index,
                    target_indices,
                })
            },
            TargetingScheme::MultiTarget(count) => {
                // add the entry if it doesn't exist and push the target to state after acquiring it from possible_targets
                if let None = target_indices.get(sub_action_index) { target_indices.push(vec![]); }
                target_indices[sub_action_index].push(possible_targets[target_index]);
                target_count += 1;

                // determine if we've pushed enough targets
                if target_count == count {
                    sub_action_index += 1;

                    Either::Left(GetSubAction {
                        combatants, turn_order, turn_order_iterator,
                        source_index, action_index, sub_action_index,
                        target_indices,
                    })
                } else {
                    Either::Right(GetTargets {
                        combatants, turn_order, turn_order_iterator,
                        source_index, action_index, sub_action_index,
                        target_indices, possible_targets, target_index,
                        target_count,
                    })
                }
            }
            TargetingScheme::SingleTarget => {
                // add the entry if it doesn't exist and push the target to state after acquiring it from possible_targets
                if let None = target_indices.get(sub_action_index) { target_indices.push(vec![]); }
                target_indices[sub_action_index].push(possible_targets[target_index]);
                sub_action_index += 1;

                Either::Left(GetSubAction {
                    combatants, turn_order, turn_order_iterator,
                    source_index, action_index,
                    sub_action_index,
                    target_indices,
                })
            }
        }
    }
}

impl GetTargets {
    pub fn transform(mut self, input: Input) -> Self {
        self.target_index = match input {
            Input::Next => (self.target_index + 1) % self.possible_targets.len(),
            Input::Previous => if self.target_index == 0 { self.possible_targets.len() - 1 } else { self.target_index - 1 },
            _ => self.target_index,
        };

        self
    }
}

impl From<ApplyAction> for PostTurn {
    fn from(from: ApplyAction) -> Self {
        let mut combatants = from.combatants;
        let turn_order = from.turn_order;
        let turn_order_iterator = from.turn_order_iterator;
        let source_index = from.source_index;
        let action_index = from.action_index;
        let target_indices = from.target_indices;

        let action = Action::from_identifier(combatants[source_index].actions[action_index]);

        for i in 0..action.sub_actions.len() {
            for target_index in &target_indices[i] {
                // split combatants array into two arrays then pull an element from each: blame the borrow checker
                let (target, source) = match source_index {
                    source_index if source_index > *target_index => {
                        let (target_container, source_container) = combatants.split_at_mut(source_index);
                        (&mut target_container[*target_index], EffectSource::Other(&source_container[0]))
                    },
                    source_index if source_index < *target_index => {
                        let (source_container, target_container) = combatants.split_at_mut(*target_index);
                        (&mut target_container[0], EffectSource::Other(&source_container[source_index]))
                    },
                    _ => (&mut combatants[*target_index], EffectSource::Origin),
                };
    
                // apply every effect in the current sub_action
                for effect in action.sub_actions[i].effects {
                    match *effect {
                        Effect::Damage(damage) => {
                            let damage_value = calculate_damage_value(target, source, damage.aspect, damage.scaling);
                            process_damage(target, damage.aspect, damage_value);
                        },
                        Effect::Modifier(modifier, stat) => {
                            target.modifiers[stat as usize].push(modifier)
                        },
                        Effect::StatusEffect(status_effect) => {
                            let damage_value = calculate_damage_value(target, source, status_effect.aspect, status_effect.scaling);

                            let status_effect = StatusEffectEntry {
                                aspect: status_effect.aspect,
                                value: damage_value,
                                lifetime: status_effect.lifetime,
                            };
                        
                            target.status_effects.push(status_effect);
                        },
                    };
                }
            }
        }

        PostTurn { combatants, turn_order, turn_order_iterator, source_index }
    }
}

impl From<PostTurn> for PreTurn {
    fn from(from: PostTurn) -> Self {
        let mut combatants = from.combatants;
        let turn_order = from.turn_order;
        let turn_order_iterator = from.turn_order_iterator;
        let source_index = from.source_index;

        let source = &mut combatants[source_index];

        for i in 0..source.status_effects.len() {
            let status_effect = source.status_effects[i];
            process_damage(source, status_effect.aspect, status_effect.value);
        }

        // remove all dead status_effects
        source.status_effects.retain(|status_effect|
            if let Lifetime::Active(lifetime) = status_effect.lifetime { lifetime > 0 } else { true }
        );

        // remove all dead modifiers
        for stat in 0..Stat::MaxValue as usize {
            source.modifiers[stat].retain(|modifier|
                if let Lifetime::Active(lifetime) = modifier.lifetime { lifetime > 0 } else { true }
            );
        }

        PreTurn { combatants, turn_order, turn_order_iterator }
    }
}

fn calculate_damage_value(target: &Combatant, source: EffectSource, aspect: DamageAspect, scaling: Fraction) -> u32 {
    let damage_value = match aspect {
        DamageAspect::Fire => match source {
            EffectSource::None => 1,
            EffectSource::Origin => target.get_stat(Stat::FireAttack),
            EffectSource::Other(source) => source.get_stat(Stat::FireAttack),
        },
        DamageAspect::Physical => match source {
            EffectSource::None => 1,
            EffectSource::Origin => target.get_stat(Stat::PhysicalAttack),
            EffectSource::Other(source) => source.get_stat(Stat::PhysicalAttack),
        },
    };

    damage_value * scaling.0 / scaling.1
}

fn calculate_defense_value(target: &Combatant, aspect: DamageAspect) -> u32 {
    match aspect {
        DamageAspect::Fire => target.get_stat(Stat::FireDefense),
        DamageAspect::Physical => target.get_stat(Stat::PhysicalDefense),
    }
}

fn calculate_absorbtion_value(target: &Combatant, aspect: DamageAspect) -> u32 {
    match aspect {
        DamageAspect::Fire => target.get_stat(Stat::FireAbsorbtion),
        DamageAspect::Physical => target.get_stat(Stat::PhysicalAbsorbtion),
    }
}

fn process_damage(target: &mut Combatant, aspect: DamageAspect, damage_value: u32) {
    let defense_value = calculate_defense_value(target, aspect);
    let absorbtion_value = calculate_absorbtion_value(target, aspect);

    if damage_value < defense_value { return; }
    let damage_value = damage_value - defense_value;

    if absorbtion_value > 100 { target.hp += std::cmp::min(target.hp_max - target.hp, damage_value * (absorbtion_value - 100) / 100); }
    else { target.hp -= std::cmp::min(target.hp, damage_value * (100 - absorbtion_value) / 100); }
}
