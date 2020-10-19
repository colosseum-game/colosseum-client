use coliseum::{
    actions::*,
    combatant::{
        Combatant,
        Gender,
        Stat,
    },
    damage::{
        DamagePerTurn,
        DamageType,
    },
    effects::{
        Effect,
        EffectSource,
    },
    items::Item,
    modifiers::{
        Modifier,
        ModifierType,
    },
};

fn calculate_turn_order(combatants: &[Combatant]) -> Vec<usize> {
    let mut turn_order = vec![0; combatants.len()];
    for i in 0..combatants.len() { turn_order[i] = i; }
    turn_order.sort_by(|a, b| combatants[*b].get_stat(Stat::Agility).cmp(&combatants[*a].get_stat(Stat::Agility)));
    turn_order
}

pub fn get_input() -> usize {
    let mut buf = String::new();
    let result = std::io::stdin().read_line(&mut buf);
    match result {
        Result::Ok(_) => {
            let result = buf.trim_end().parse::<i32>();
            match result {
                Result::Ok(index) => index as usize,
                Result::Err(_) => { get_input() },
            }
        },
        Result::Err(_) => { get_input() },
    }
}

fn get_action_index(combatant: &Combatant) -> usize {
    combatant.actions.iter().enumerate().for_each(|(i, action)| {
        println!("{}: {}", i, action);
    });

    let input = get_input();
    if input >= combatant.actions.len() { get_action_index(combatant) }
    else { input }
}

fn apply_damage(target: &mut Combatant, source: EffectSource, damage_type: DamageType, multiplier: u32, divisor: u32, turns_to_live: Option<u32>) {
    let damage_value = damage_type.damage_from_source(target, source, multiplier, divisor);
    let damage_reduction = damage_type.damage_reduction_from_target(target);
    if damage_reduction > damage_value { return; }

    match turns_to_live {
        Some(ttl) => target.damage_per_turn.push(DamagePerTurn { damage_type: damage_type, value: damage_value, turns_to_live: ttl }),
        None => target.hp -= std::cmp::min(damage_value - damage_reduction, target.hp),
    };
}

fn apply_modifier(target: &mut Combatant, modifier: Modifier, stat: Stat) {
    match stat {
        Stat::Agility => target.agility_modifiers.push(modifier),
        Stat::FireAttack => target.fire_attack_modifiers.push(modifier),
        Stat::FireResistance => target.fire_resistance_modifiers.push(modifier),
        Stat::PhysicalAttack => target.physical_attack_modifiers.push(modifier),
        Stat::PhysicalResistance => target.physical_resistance_modifiers.push(modifier),
    }
}

fn simulate_combat(combatants: &mut [Combatant]) {
    let turn_order = calculate_turn_order(combatants);
    let mut turn_order = turn_order.iter().cycle();
    let mut living_count = u32::MAX;

    while living_count > 1 {
        let source_index = match turn_order.next() { Some(i) => *i, None => panic!() };

        for damage_per_turn in &mut combatants[source_index].damage_per_turn { damage_per_turn.turns_to_live -= std::cmp::min(damage_per_turn.turns_to_live, 1) }
        for modifier in &mut combatants[source_index].agility_modifiers { if let Some(ref mut turns_left) = modifier.turns_to_live { *turns_left -= std::cmp::min(*turns_left, 1) } }
        for modifier in &mut combatants[source_index].fire_attack_modifiers { if let Some(ref mut turns_left) = modifier.turns_to_live { *turns_left -= std::cmp::min(*turns_left, 1) } }
        for modifier in &mut combatants[source_index].fire_resistance_modifiers { if let Some(ref mut turns_left) = modifier.turns_to_live { *turns_left -= std::cmp::min(*turns_left, 1) } }
        for modifier in &mut combatants[source_index].physical_attack_modifiers { if let Some(ref mut turns_left) = modifier.turns_to_live { *turns_left -= std::cmp::min(*turns_left, 1) } }
        for modifier in &mut combatants[source_index].physical_resistance_modifiers { if let Some(ref mut turns_left) = modifier.turns_to_live { *turns_left -= std::cmp::min(*turns_left, 1) } }

        let action_index = if source_index == 0 { get_action_index(&combatants[source_index]) } else { 0 };
        let action = combatants[source_index].actions[action_index];

        for sub_action in action.sub_actions {
            let target_index = (source_index + 1) % combatants.len();

            let (target, source) = match source_index {
                source_index if source_index > target_index => {
                    let (target_container, source_container) = combatants.split_at_mut(source_index);
                    (&mut target_container[target_index], EffectSource::Other(&source_container[0]))
                },
                source_index if source_index < target_index => {
                    let (source_container, target_container) = combatants.split_at_mut(target_index);
                    (&mut target_container[0], EffectSource::Other(&source_container[source_index]))
                }
                _ => (&mut combatants[target_index], EffectSource::Origin),
            };

            for effect in sub_action.effects {
                match *effect {
                    Effect::Damage(damage_type, multiplier, divisor, turns_to_live) => apply_damage(target, source, damage_type, multiplier, divisor, turns_to_live),
                    Effect::Modifier(modifier, stat) => apply_modifier(target, modifier, stat),
                };
            }
        }

        // Apply damage per turn post turn
        for i in 0..combatants[source_index].damage_per_turn.len() {
            let damage_per_turn = combatants[source_index].damage_per_turn[i];
            apply_damage(&mut combatants[source_index], EffectSource::None, damage_per_turn.damage_type, damage_per_turn.value, 1, None)
        }

        // Remove modifiers and damage per turn if the turns_to_live count is 0
        combatants[source_index].damage_per_turn.retain(|damage_per_turn| damage_per_turn.turns_to_live > 0 );
        combatants[source_index].agility_modifiers.retain(|modifier| match modifier.turns_to_live { Some(x) => x > 0, _ => true } );
        combatants[source_index].fire_attack_modifiers.retain(|modifier| match modifier.turns_to_live { Some(x) => x > 0, _ => true } );
        combatants[source_index].fire_resistance_modifiers.retain(|modifier| match modifier.turns_to_live { Some(x) => x > 0, _ => true } );
        combatants[source_index].physical_attack_modifiers.retain(|modifier| match modifier.turns_to_live { Some(x) => x > 0, _ => true } );
        combatants[source_index].physical_resistance_modifiers.retain(|modifier| match modifier.turns_to_live { Some(x) => x > 0, _ => true } );

        // Calculate living to determine winner
        living_count = 0;
        combatants.iter().for_each(|combatant| if combatant.alive() { living_count += 1; });
    }
}

fn main() -> std::io::Result<()> {
    let attack = Action {
        display_name: "Attack",
        sub_actions: &[
            SubAction {
                effects: &[
                    Effect::Damage(DamageType::Physical, 1, 1, None),
                ],
                target_flags: &[&[TargetFlag::Any]],
                target_count: 1,
            },
        ],
    };

    let scorch = Action {
        display_name: "Scorch",
        sub_actions: &[
            SubAction {
                effects: &[
                    Effect::Damage(DamageType::Fire, 5, 1, Some(3)),
                ],
                target_flags: &[&[TargetFlag::Any]],
                target_count: 1,
            },
        ],
    };
    
    let beat_female = Action {
        display_name: "Beat female",
        sub_actions: &[
            SubAction {
                effects: &[
                    Effect::Damage(DamageType::Physical, 2, 1, None),
                ],
                target_flags: &[
                    &[TargetFlag::Gender(Gender::Female)]
                ],
                target_count: 1,
            },
            SubAction {
                effects: &[
                    Effect::Modifier(
                        Modifier {
                            modifier_type: ModifierType::Subtract, 
                            value: 5,
                            turns_to_live: Some(1),
                        },
                        Stat::PhysicalAttack,
                    ),
                ],
                target_flags: &[
                    &[TargetFlag::Origin]
                ],
                target_count: 1,
            },
        ],
    };

    let skip = Action {
        display_name: "Skip",
        sub_actions: &[],
    };

    let grenade = Item {
        display_name: "Grenade",
        effects: &[
            Effect::Damage(DamageType::Physical, 12, 1, None),
            Effect::Damage(DamageType::Fire, 5, 2, Some(3)),
        ],
        target_flags: &[&[TargetFlag::Any]],
        target_count: 1,
    };

    let cracked_bellroot_seed = Item {
        display_name: "Cracked Bellroot Seed",
        effects: &[
            Effect::Damage(DamageType::Physical, 4, 1, None),
        ],
        target_flags: &[&[TargetFlag::Any]],
        target_count: 3,
    };

    let brayden = Combatant {
        name: "Brayden",
        gender: Gender::Male,
        actions: &[&attack, &beat_female, &skip],

        hp: 70,
        hp_max: 70,
        damage_per_turn: vec![],

        agility: 12,
        fire_attack: 0,
        fire_resistance: 0,
        physical_attack: 26,
        physical_resistance: 9,

        agility_modifiers: vec![],
        fire_attack_modifiers: vec![],
        fire_resistance_modifiers: vec![],
        physical_attack_modifiers: vec![],
        physical_resistance_modifiers: vec![],
    };

    let chay = Combatant {
        name: "Chay",
        gender: Gender::Male,
        actions: &[&scorch],

        hp: 46,
        hp_max: 46,
        damage_per_turn: vec![],

        agility: 26,
        fire_attack: 2,
        fire_resistance: 1000,
        physical_attack: 17,
        physical_resistance: 12,

        agility_modifiers: vec![],
        fire_attack_modifiers: vec![],
        fire_resistance_modifiers: vec![],
        physical_attack_modifiers: vec![],
        physical_resistance_modifiers: vec![],
    };

    let combatants = &mut vec![brayden, chay];
    simulate_combat(combatants);
    println!("{:#?}", combatants);

    Ok(())
}
