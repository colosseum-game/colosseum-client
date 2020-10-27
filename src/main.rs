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
    items::Item,
    lifetime::Lifetime,
    math::Fraction,
    modifiers::Modifier,
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
    println!();
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
    println!("Select an action:");
    combatant.actions.iter().enumerate().for_each(|(i, action)| {
        println!("{}: {}", i, action);
    });

    let input = get_input();
    if input >= combatant.actions.len() { get_action_index(combatant) }
    else { input }
}

fn get_target_index(combatants: &[Combatant], possible_targets: &[usize]) -> usize {
    println!("Select a target:");
    for index in possible_targets {
        println!("{}. {}", index, combatants[*index]);
    }

    let input = get_input();
    let mut valid_input = false;
    for index in possible_targets {
        if input == *index { valid_input = true }
    }

    if valid_input { input }
    else { get_target_index(combatants, possible_targets) }
}

fn calculate_damage_value(target: &Combatant, source: EffectSource, aspect: DamageAspect, scaling: Fraction) -> u32 {
    let mut damage_value = match aspect {
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

    damage_value * scaling.numerator / scaling.denominator
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

fn simulate_combat(combatants: &mut [Combatant]) {
    let turn_order = calculate_turn_order(combatants);
    let mut turn_order = turn_order.iter().cycle();
    let mut living_count = u32::MAX;

    while living_count > 1 {
        let source_index = match turn_order.next() { Some(i) => *i, None => panic!() };

        // decrement the lifetime of status_effects
        for status_effect in &mut combatants[source_index].status_effects {
            if let Lifetime::Active(ref mut lifetime) = status_effect.lifetime {
                *lifetime -= std::cmp::min(*lifetime, 1);
            }
        }

        // decrement the lifetime of modifiers
        for stat in 0..Stat::MaxValue as usize {
            for modifier in &mut combatants[source_index].modifiers[stat] {
                if let Lifetime::Active(ref mut lifetime) = modifier.lifetime {
                    *lifetime -= std::cmp::min(*lifetime, 1)
                }
            }
        }

        // get action
        let action_index = if source_index == 0 { get_action_index(&combatants[source_index]) } else { 0 };
        let action = combatants[source_index].actions[action_index];

        // process action
        for subaction in action.sub_actions {
            let possible_targets: Vec<usize> = combatants
                .iter()
                .enumerate()
                .filter(|&(index, combatant)| {
                    subaction.target_flags.iter().fold(true, |is_valid_target, or_conditions| {
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

            let target_indices = match subaction.targeting_scheme {
                TargetingScheme::All => possible_targets,
                TargetingScheme::MultiTarget(target_count) => {
                    let mut target_indices = vec![];
                    for _ in 0..target_count {
                        target_indices.push(
                            if source_index == 0 { get_target_index(combatants, &possible_targets) }
                            else { (source_index + 1) % combatants.len() }
                        );
                    }

                    target_indices
                },
                TargetingScheme::SingleTarget => {
                    let target_index = {
                        if source_index == 0 { get_target_index(combatants, &possible_targets) }
                        else { (source_index + 1) % combatants.len() }
                    };

                    vec![target_index]
                },
            };

            for target_index in target_indices {
                let (target, source) = match source_index {
                    source_index if source_index > target_index => {
                        let (target_container, source_container) = combatants.split_at_mut(source_index);
                        (&mut target_container[target_index], EffectSource::Other(&source_container[0]))
                    },
                    source_index if source_index < target_index => {
                        let (source_container, target_container) = combatants.split_at_mut(target_index);
                        (&mut target_container[0], EffectSource::Other(&source_container[source_index]))
                    },
                    _ => (&mut combatants[target_index], EffectSource::Origin),
                };

                for effect in subaction.effects {
                    match *effect {
                        Effect::Damage(damage) => apply_damage(target, source, damage),
                        Effect::Modifier(modifier, stat) => apply_modifier(target, modifier, stat),
                        Effect::StatusEffect(status_effect) => apply_status_effect(target, source, status_effect),
                    };
                }
            }
        }

        // apply status_effects
        for i in 0..combatants[source_index].status_effects.len() {
            let status_effect = combatants[source_index].status_effects[i];
            process_damage(&mut combatants[source_index], status_effect.aspect, status_effect.value);
        }

        // remove dead status_effects
        combatants[source_index].status_effects.retain(|status_effect|
            if let Lifetime::Active(lifetime) = status_effect.lifetime { lifetime > 0 } else { true }
        );

        // remove dead modifiers
        for stat in 0..Stat::MaxValue as usize {
            combatants[source_index].modifiers[stat].retain(|modifier|
                if let Lifetime::Active(lifetime) = modifier.lifetime { lifetime > 0 } else { true }
            );
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
                    Effect::Damage(Damage {
                        aspect: DamageAspect::Physical, 
                        scaling: Fraction::new(1, 1),
                    }),
                ],
                target_flags: &[&[TargetFlag::Any]],
                targeting_scheme: TargetingScheme::SingleTarget,
            },
        ],
    };

    let sweep = Action {
        display_name: "Sweep",
        sub_actions: &[
            SubAction {
                effects: &[
                    Effect::Damage(Damage {
                        aspect: DamageAspect::Physical, 
                        scaling: Fraction::new(2, 3),
                    }),
                ],
                target_flags: &[&[TargetFlag::Any]],
                targeting_scheme: TargetingScheme::MultiTarget(3),
            },
        ],
    };

    let scorch = Action {
        display_name: "Scorch",
        sub_actions: &[
            SubAction {
                effects: &[
                    Effect::StatusEffect(StatusEffect {
                        aspect: DamageAspect::Fire,
                        scaling: Fraction::new(2, 3),
                        lifetime: Lifetime::Active(3),
                    }),
                ],
                target_flags: &[&[TargetFlag::Any]],
                targeting_scheme: TargetingScheme::SingleTarget,
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
            Effect::Damage(Damage {
                aspect: DamageAspect::Physical,
                scaling: Fraction::new(12, 1),
            }),
            Effect::StatusEffect(StatusEffect {
                aspect: DamageAspect::Fire,
                scaling: Fraction::new(5, 2),
                lifetime: Lifetime::Active(3),
            }),
        ],
        target_flags: &[&[TargetFlag::Any]],
        target_count: 1,
    };

    let cracked_bellroot_seed = Item {
        display_name: "Cracked Bellroot Seed",
        effects: &[
            Effect::Damage(Damage {
                aspect: DamageAspect::Physical,
                scaling: Fraction::new(3, 1)
            }),
        ],
        target_flags: &[&[TargetFlag::Any]],
        target_count: 3,
    };

    let brayden = Combatant {
        name: "Brayden",
        gender: Gender::Male,
        actions: &[&attack, &sweep, &skip],

        hp: 70,
        hp_max: 70,
        stats: [12, 11, 0, 2, 67, 17, 7],

        status_effects: vec![],
        modifiers: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
    };

    let chay = Combatant {
        name: "Chay",
        gender: Gender::Male,
        actions: &[&scorch],

        hp: 46,
        hp_max: 46,
        stats: [26, 126, 16, 0, 8, 6, 3],

        status_effects: vec![],
        modifiers: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
    };

    let combatants = &mut vec![brayden, chay];
    simulate_combat(combatants);
    println!("{:#?}", combatants);

    Ok(())
}
