use colosseum::{
    actions::*,
    combatant::{
        Combatant,
        Gender,
        Stat,
    },
    damage::{
        Damage,
        DamageAspect,
        StatusEffect,
        StatusEffectEntry,
    },
    effects::{
        Effect,
        EffectSource,
    },
    lifetime::Lifetime,
    math::Fraction,
    modifiers::Modifier,
};

use gilrs::{
    Button,
    EventType,
    Gilrs,
};

use winit::{
    event::{
        Event,
        WindowEvent,
    },
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::Window,
};

fn calculate_turn_order(combatants: &[Combatant]) -> Vec<usize> {
    let mut turn_order = vec![0; combatants.len()];
    for i in 0..combatants.len() { turn_order[i] = i; }
    turn_order.sort_by(|a, b| combatants[*b].get_stat(Stat::Agility).cmp(&combatants[*a].get_stat(Stat::Agility)));
    turn_order
}

fn calculate_possible_targets(combatants: &[Combatant], source_index: usize, sub_action: &SubAction) -> Vec<usize> {
    combatants
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
        .collect()
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

fn apply_damage(target: &mut Combatant, source: EffectSource, damage: Damage) {
    let damage_value = calculate_damage_value(target, source, damage.aspect, damage.scaling);
    process_damage(target, damage.aspect, damage_value);
}

fn apply_modifier(target: &mut Combatant, modifier: Modifier, stat: Stat) {
    target.modifiers[stat as usize].push(modifier);
}

fn apply_status_effect(target: &mut Combatant, source: EffectSource, status_effect: StatusEffect) {
    let damage_value = calculate_damage_value(target, source, status_effect.aspect, status_effect.scaling);

    let status_effect = StatusEffectEntry {
        aspect: status_effect.aspect,
        value: damage_value,
        lifetime: status_effect.lifetime,
    };

    target.status_effects.push(status_effect);
}

#[derive(Clone, Copy, Debug)]
enum Input {
    Select,
    Cancel,
    Next,
    Previous,
}

fn get_input(input_instance: &mut Gilrs) -> Vec<Input> {
    let mut input = vec![];

    while let Some(gilrs::Event { event, .. }) = input_instance.next_event() {
        match event {
            EventType::ButtonPressed(button, ..) => {
                match button {
                    Button::DPadDown => input.push(Input::Next),
                    Button::DPadUp => input.push(Input::Previous),
                    Button::East => input.push(Input::Cancel),
                    Button::Select => input.push(Input::Cancel),
                    Button::South => input.push(Input::Select),
                    Button::Start => input.push(Input::Select),
                    _ => (),
                }
            },
            _ => (),
        }
    }

    input
}

#[derive(Clone, Debug)]
struct PreTurn;

#[derive(Clone, Debug)]
struct GetAction {
    source_index: usize,
    action_index: usize,
}

#[derive(Clone, Debug)]
struct GetSubAction {
    source_index: usize,
    action_index: usize,
    sub_action_index: usize,
    target_indices: Vec<Vec<usize>>,
}

#[derive(Clone, Debug)]
struct GetTargets {
    source_index: usize,
    action_index: usize,
    sub_action_index: usize,
    possible_targets: Vec<usize>,
    target_index: usize,
    target_count: usize,
    target_indices: Vec<Vec<usize>>,
}

#[derive(Clone, Debug)]
struct ApplyAction {
    source_index: usize,
    action_index: usize,
    target_indices: Vec<Vec<usize>>
}

#[derive(Clone, Debug)]
struct PostTurn {
    source_index: usize
}

// TODO: consider removing in favor of raw enum?
#[derive(Clone, Debug)]
struct CombatState<S> {
    combatants: Vec<Combatant>,
    turn_order: Vec<usize>,
    turn_order_iterator: usize,
    state: S,
}

enum GetSubActionResult {
    GetTargets(CombatState<GetTargets>),
    ApplyAction(CombatState<ApplyAction>),
}

enum GetTargetsResult {
    GetSubAction(CombatState<GetSubAction>),
    GetTargets(CombatState<GetTargets>),
}

#[derive(Clone, Debug)]
enum CombatInstance {
    PreTurn(CombatState<PreTurn>),
    GetAction(CombatState<GetAction>),
    GetSubAction(CombatState<GetSubAction>),
    GetTargets(CombatState<GetTargets>),
    ApplyAction(CombatState<ApplyAction>),
    PostTurn(CombatState<PostTurn>),
}

impl CombatInstance {
    pub fn new(combatants: Vec<Combatant>) -> Self {
        let turn_order = calculate_turn_order(&combatants);

        Self::PreTurn(
            CombatState {
                combatants,
                turn_order,
                turn_order_iterator: 0,
                state: PreTurn
            }
        )
    }
}

impl CombatState<PreTurn> {
    pub fn next(mut self) -> CombatState<GetAction> {
        let source_index = self.turn_order[self.turn_order_iterator];
        let turn_order_iterator = (self.turn_order_iterator + 1) % self.turn_order.len();

        // decrement status_effect lifetimes
        for status_effect in &mut self.combatants[source_index].status_effects {
            if let Lifetime::Active(ref mut lifetime) = status_effect.lifetime {
                *lifetime -= std::cmp::min(*lifetime, 1);
            }
        }

        // decrement modifier lifetimes
        for stat in 0..Stat::MaxValue as usize {
            for modifier in &mut self.combatants[source_index].modifiers[stat] {
                if let Lifetime::Active(ref mut lifetime) = modifier.lifetime {
                    *lifetime -= std::cmp::min(*lifetime, 1)
                }
            }
        }

        CombatState {
            combatants: self.combatants,
            turn_order: self.turn_order,
            turn_order_iterator,
            state: GetAction {
                source_index,
                action_index: 0,
            },
        }
    }
}

impl<'a> CombatState<GetAction> {
    pub fn next(self) -> CombatState<GetSubAction> {
        CombatState {
            combatants: self.combatants,
            turn_order: self.turn_order,
            turn_order_iterator: self.turn_order_iterator,
            state: GetSubAction {
                source_index: self.state.source_index,
                action_index: self.state.action_index,
                sub_action_index: 0,
                target_indices: vec![],
            }
        }
    }

    pub fn transform(mut self, input: Input) -> Self {
        self.state.action_index = match input {
            Input::Next => (self.state.action_index + 1) % self.combatants[self.state.source_index].actions.len(),
            Input::Previous => if self.state.action_index == 0 { self.combatants[self.state.source_index].actions.len() - 1 } else { self.state.action_index - 1 },
            _ => self.state.action_index,
        };

        self
    }
}

impl<'a> CombatState<GetSubAction> {
    pub fn next(self) -> GetSubActionResult {
        let action = Action::from_identifier(self.combatants[self.state.source_index].actions[self.state.action_index]);
        match action.sub_actions.get(self.state.sub_action_index) {
            Some(sub_action) => {
                let possible_targets = calculate_possible_targets(&self.combatants, self.state.source_index, sub_action);

                GetSubActionResult::GetTargets(
                    CombatState {
                        combatants: self.combatants,
                        turn_order: self.turn_order,
                        turn_order_iterator: self.turn_order_iterator,
                        state: GetTargets {
                            source_index: self.state.source_index,
                            action_index: self.state.action_index,
                            sub_action_index: self.state.sub_action_index,
                            possible_targets,
                            target_index: 0,
                            target_count: 0,
                            target_indices: self.state.target_indices,
                        }
                    }
                )
            },
            None => GetSubActionResult::ApplyAction(
                CombatState {
                    combatants: self.combatants,
                    turn_order: self.turn_order,
                    turn_order_iterator: self.turn_order_iterator,
                    state: ApplyAction {
                        source_index: self.state.source_index,
                        action_index: self.state.action_index,
                        target_indices: self.state.target_indices,
                    }
                }
            )
        }
    }
}

impl CombatState<GetTargets> {
    pub fn next(mut self) -> GetTargetsResult {
        let combatants = self.combatants;
        let turn_order = self.turn_order;
        let turn_order_iterator = self.turn_order_iterator;
        let source_index = self.state.source_index;
        let action_index = self.state.action_index;
        let action = Action::from_identifier(combatants[source_index].actions[action_index]);

        match action.sub_actions[self.state.sub_action_index].targeting_scheme {
            TargetingScheme::All => {
                // push all possible targets to target indices
                self.state.target_indices.push(self.state.possible_targets);

                GetTargetsResult::GetSubAction(CombatState {
                    combatants, turn_order, turn_order_iterator,
                    state: GetSubAction {
                        source_index, action_index,
                        sub_action_index: self.state.sub_action_index + 1,
                        target_indices: self.state.target_indices,
                    }
                })
            },
            TargetingScheme::MultiTarget(count) => {
                // add the entry if it doesn't exist and push the target to state after acquiring it from possible_targets
                if let None = self.state.target_indices.get(self.state.sub_action_index) { self.state.target_indices.push(vec![]); }
                self.state.target_indices[self.state.sub_action_index].push(self.state.possible_targets[self.state.target_index]);
                let target_count = self.state.target_count + 1;

                // determine if we've pushed enough targets
                if target_count == count {
                    GetTargetsResult::GetSubAction(CombatState {
                        combatants, turn_order, turn_order_iterator,
                        state: GetSubAction {
                            source_index, action_index,
                            sub_action_index: self.state.sub_action_index + 1,
                            target_indices: self.state.target_indices,
                        }
                    })
                } else {
                    GetTargetsResult::GetTargets(CombatState {
                        combatants, turn_order, turn_order_iterator,
                        state: GetTargets {
                            source_index, action_index,
                            sub_action_index: self.state.sub_action_index,
                            possible_targets: self.state.possible_targets,
                            target_index: self.state.target_index,
                            target_count,
                            target_indices: self.state.target_indices,
                        }
                    })
                }
            }
            TargetingScheme::SingleTarget => {
                // add the entry if it doesn't exist and push the target to state after acquiring it from possible_targets
                if let None = self.state.target_indices.get(self.state.sub_action_index) { self.state.target_indices.push(vec![]); }
                self.state.target_indices[self.state.sub_action_index].push(self.state.possible_targets[self.state.target_index]);

                GetTargetsResult::GetSubAction(CombatState {
                    combatants, turn_order, turn_order_iterator,
                    state: GetSubAction {
                        source_index, action_index,
                        sub_action_index: self.state.sub_action_index + 1,
                        target_indices: self.state.target_indices,
                    }
                })
            }
        }
    }

    pub fn transform(mut self, input: Input) -> Self {
        self.state.target_index = match input {
            Input::Next => (self.state.target_index + 1) % self.state.possible_targets.len(),
            Input::Previous => if self.state.target_index == 0 { self.state.possible_targets.len() - 1 } else { self.state.target_index - 1 },
            _ => self.state.target_index,
        };

        self
    }
}

impl<'a> CombatState<ApplyAction> {
    pub fn next(mut self) -> CombatState<PostTurn> {
        let source_index = self.state.source_index;
        let action = Action::from_identifier(self.combatants[source_index].actions[self.state.action_index]);
        
        for i in 0..action.sub_actions.len() {
            for target_index in &self.state.target_indices[i] {
                // split combatants array into two arrays then pull an element from each: blame the borrow checker
                let (target, source) = match source_index {
                    source_index if source_index > *target_index => {
                        let (target_container, source_container) = self.combatants.split_at_mut(source_index);
                        (&mut target_container[*target_index], EffectSource::Other(&source_container[0]))
                    },
                    source_index if source_index < *target_index => {
                        let (source_container, target_container) = self.combatants.split_at_mut(*target_index);
                        (&mut target_container[0], EffectSource::Other(&source_container[source_index]))
                    },
                    _ => (&mut self.combatants[*target_index], EffectSource::Origin),
                };
    
                // apply every effect in the current sub_action
                for effect in action.sub_actions[i].effects {
                    match *effect {
                        Effect::Damage(damage) => apply_damage(target, source, damage),
                        Effect::Modifier(modifier, stat) => apply_modifier(target, modifier, stat),
                        Effect::StatusEffect(status_effect) => apply_status_effect(target, source, status_effect),
                    };
                }
            }
        }

        CombatState {
            combatants: self.combatants,
            turn_order: self.turn_order,
            turn_order_iterator: self.turn_order_iterator,
            state: PostTurn { source_index },
        }
    }
}

impl CombatState<PostTurn> {
    pub fn next(mut self) -> CombatState<PreTurn> {
        let source_index = self.state.source_index;
        let source = &mut self.combatants[source_index];

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

        CombatState {
            combatants: self.combatants,
            turn_order: self.turn_order,
            turn_order_iterator: self.turn_order_iterator,
            state: PreTurn,
        }
    }
}

fn main() -> std::io::Result<()> {
    let brayden = Combatant {
        name: "Brayden".to_string(),
        gender: Gender::Male,
        actions: vec![
            ActionIdentifier::Attack,
            ActionIdentifier::Sweep,
            ActionIdentifier::Skip,
        ],

        hp: 70,
        hp_max: 70,
        stats: [12, 11, 0, 2, 67, 17, 7],

        status_effects: vec![],
        modifiers: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
    };

    let chay = Combatant {
        name: "Chay".to_string(),
        gender: Gender::Male,
        actions: vec![ActionIdentifier::Scorch],

        hp: 46,
        hp_max: 46,
        stats: [26, 126, 16, 0, 8, 6, 3],

        status_effects: vec![],
        modifiers: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
    };

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("error creating window");

    let mut input_instance = Gilrs::new().unwrap();
    let mut combat_instance = Some(CombatInstance::new(vec![brayden, chay]));
    let mut state_buffer = vec![];

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                // application update code
                let input = get_input(&mut input_instance);
                for input in input {
                    loop {
                        combat_instance = Some(match combat_instance.take().unwrap() { // TODO: better way than take/unwrap?
                            CombatInstance::PreTurn(state) => CombatInstance::GetAction(state.next()),
                            CombatInstance::GetAction(state) => {
                                match input {
                                    Input::Select => {
                                        state_buffer = vec![CombatInstance::GetAction(state.clone())];
                                        CombatInstance::GetSubAction(state.next())
                                    },
                                    _ => CombatInstance::GetAction(state.transform(input)),
                                }
                            },
                            CombatInstance::GetSubAction(state) => match state.next() {
                                GetSubActionResult::GetTargets(state) => CombatInstance::GetTargets(state),
                                GetSubActionResult::ApplyAction(state) => CombatInstance::ApplyAction(state),
                            },
                            CombatInstance::GetTargets(state) => match input {
                                Input::Select => {
                                    match state.next() {
                                        GetTargetsResult::GetSubAction(state) => CombatInstance::GetSubAction(state),
                                        GetTargetsResult::GetTargets(state) => {
                                            state_buffer.push(CombatInstance::GetTargets(state.clone()));
                                            CombatInstance::GetTargets(state)
                                        },
                                    }
                                },
                                Input::Cancel => state_buffer.pop().unwrap(),
                                _ => CombatInstance::GetTargets(state.transform(input)),
                            },
                            CombatInstance::ApplyAction(state) => CombatInstance::PostTurn(state.next()),
                            CombatInstance::PostTurn(state) => CombatInstance::PreTurn(state.next()),
                        });

                        match combat_instance.as_ref().unwrap() {
                            CombatInstance::GetAction(_) => break,
                            CombatInstance::GetTargets(_) => break,
                            _ => ()
                        }
                    }
                    println!("{:?}", combat_instance);
                }
            },
            _ => (),
        }
    });
}
