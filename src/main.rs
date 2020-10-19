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

fn process_damage(target: &mut Combatant, damage_type: DamageType, damage_value: u32) {
    let defense_value = match damage_type {
        DamageType::Fire => target.get_stat(Stat::FireDefense),
        DamageType::Physical => target.get_stat(Stat::PhysicalDefense),
    };

    let absorbtion_value = match damage_type {
        DamageType::Fire => target.get_stat(Stat::FireAbsorbtion),
        DamageType::Physical => target.get_stat(Stat::PhysicalAbsorbtion),
    };

    if damage_value < defense_value { return; }
    let damage_value = damage_value - defense_value;

    if absorbtion_value > 100 { target.hp += std::cmp::min(target.hp_max - target.hp, damage_value * (absorbtion_value - 100) / 100); }
    else { target.hp -= std::cmp::min(target.hp, damage_value * (100 - absorbtion_value) / 100); }
}

fn apply_damage(target: &mut Combatant, source: EffectSource, damage_type: DamageType, multiplier: u32, divisor: u32, turns_to_live: Option<u32>) {
    let damage_value = match damage_type {
        DamageType::Fire => match source {
            EffectSource::None => 1,
            EffectSource::Origin => target.get_stat(Stat::FireAttack),
            EffectSource::Other(source) => source.get_stat(Stat::FireAttack),
        },
        DamageType::Physical => match source {
            EffectSource::None => 1,
            EffectSource::Origin => target.get_stat(Stat::PhysicalAttack),
            EffectSource::Other(source) => source.get_stat(Stat::PhysicalAttack),
        },
    } * multiplier / divisor;

    match turns_to_live {
        Some(ttl) => target.damage_per_turn.push(DamagePerTurn { damage_type: damage_type, value: damage_value, turns_to_live: ttl }),
        None => process_damage(target, damage_type, damage_value),
    };
}

fn apply_modifier(target: &mut Combatant, modifier: Modifier, stat: Stat) {
    target.stat_modifiers[stat as usize].push(modifier);
}

fn simulate_combat(combatants: &mut [Combatant]) {
    let turn_order = calculate_turn_order(combatants);
    let mut turn_order = turn_order.iter().cycle();
    let mut living_count = u32::MAX;

    while living_count > 1 {
        let source_index = match turn_order.next() { Some(i) => *i, None => panic!() };

        // decrement the lifetime of DOTS
        for damage_per_turn in &mut combatants[source_index].damage_per_turn {
            damage_per_turn.turns_to_live -= std::cmp::min(damage_per_turn.turns_to_live, 1)
        }

        // decrement the lifetime of modifiers
        for stat in 0..Stat::MaxValue as usize {
            for modifier in &mut combatants[source_index].stat_modifiers[stat] {
                if let Some(ref mut turns_left) = modifier.turns_to_live {
                    *turns_left -= std::cmp::min(*turns_left, 1)
                }
            }
        }

        // get action
        let action_index = if source_index == 0 { get_action_index(&combatants[source_index]) } else { 0 };
        let action = combatants[source_index].actions[action_index];

        // process action
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

        // apply DOT
        for i in 0..combatants[source_index].damage_per_turn.len() {
            let damage_per_turn = combatants[source_index].damage_per_turn[i];
            process_damage(&mut combatants[source_index], damage_per_turn.damage_type, damage_per_turn.value);
        }

        // remove dead DOTS
        combatants[source_index].damage_per_turn.retain(|damage_per_turn| damage_per_turn.turns_to_live > 0 );

        // remove dead modifiers
        for stat in 0..Stat::MaxValue as usize {
            combatants[source_index].stat_modifiers[stat].retain(|modifier| match modifier.turns_to_live { Some(x) => x > 0, _ => true } );
        }

        // determine winner or continue
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
                    Effect::Damage(DamageType::Fire, 2, 3, Some(3)),
                ],
                target_flags: &[&[TargetFlag::Any]],
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
        actions: &[&attack, &skip],

        hp: 70,
        hp_max: 70,
        damage_per_turn: vec![],

        stats: [12, 11, 0, 2, 67, 17, 7],
        stat_modifiers: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
    };

    let chay = Combatant {
        name: "Chay",
        gender: Gender::Male,
        actions: &[&scorch],

        hp: 46,
        hp_max: 46,
        damage_per_turn: vec![],

        stats: [26, 126, 16, 0, 8, 6, 3],
        stat_modifiers: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
    };

    let combatants = &mut vec![brayden, chay];
    simulate_combat(combatants);
    println!("{:#?}", combatants);

    Ok(())
}
