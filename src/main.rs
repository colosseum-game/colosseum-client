use coliseum::{
    actions::*,
    combatant::{
        Combatant,
        Gender,
        Stat,
    },
    damage::{
        ActiveDamage,
        DamageType,
    },
    effects::{
        Effect,
        EffectSource,
    },
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

fn apply_active_damage(target: &mut Combatant, source: EffectSource, damage_type: DamageType, multiplier: u32, divisor: u32, turns_to_live: u32) {
    let active_damage = ActiveDamage {
        damage_type: damage_type,
        value: damage_type.damage_from_source(target, source, multiplier, divisor),
    };

    target.active_damage.push((active_damage, turns_to_live));
}

fn apply_active_modifier(target: &mut Combatant, modifier: Modifier, stat: Stat, turns_to_live: u32) {
    target.active_stat_modifiers[stat as usize].push((modifier, turns_to_live))
}

fn apply_damage(target: &mut Combatant, source: EffectSource, damage_type: DamageType, multiplier: u32, divisor: u32) {
    let damage_value = damage_type.damage_from_source(target, source, multiplier, divisor);
    let damage_reduction = damage_type.damage_reduction_from_target(target);
    if damage_reduction > damage_value { return; }
    target.hp -= std::cmp::min(damage_value - damage_reduction, target.hp); // TODO: overflow stuff nananana
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
        let action_index = if source_index == 0 { get_action_index(&combatants[source_index]) } else { 0 };
        let action = combatants[source_index].actions[action_index];

        for sub_action in action.sub_actions {
            let target_index = (source_index + 1) % combatants.len();
            let (target, source) = if source_index == target_index {
                (&mut combatants[target_index], EffectSource::Target)
            } 
            else if source_index > target_index {
                let (target_container, source_container) = combatants.split_at_mut(source_index);
                (&mut target_container[target_index], EffectSource::Other(&source_container[0]))
            }
            else {
                let (source_container, target_container) = combatants.split_at_mut(target_index);
                (&mut target_container[0], EffectSource::Other(&source_container[source_index]))
            };

            for effect in sub_action.effects {
                match *effect {
                    Effect::ActiveDamage(damage_type, multiplier, divisor, turns_to_live) => apply_active_damage(target, source, damage_type, multiplier, divisor, turns_to_live),
                    Effect::ActiveModifier(modifier, stat, turns_to_live) => apply_active_modifier(target, modifier, stat, turns_to_live),
                    Effect::Damage(damage_type, multiplier, divisor) => apply_damage(target, source, damage_type, multiplier, divisor),
                    Effect::Modifier(modifier, stat) => apply_modifier(target, modifier, stat),
                };
            }
        }

        living_count = 0;
        combatants.iter().for_each(|combatant| if combatant.alive() { living_count += 1; });
    }
}

fn main() -> std::io::Result<()> {
    let brayden = Combatant {
        name: "Brayden".to_string(),
        gender: Gender::Male,

        actions: vec![&ATTACK, &SKIP],

        hp: 69,
        hp_max: 69,
        active_damage: vec![],

        stats: [12, 0, 800, 26, 9],
        stat_modifiers: [vec![], vec![], vec![], vec![], vec![]],
        active_stat_modifiers: [vec![], vec![], vec![], vec![], vec![]],
    };

    let chay = Combatant {
        name: "Chay".to_string(),
        gender: Gender::Male,

        actions: vec![&ATTACK, &SKIP],

        hp: 12,
        hp_max: 12,
        active_damage: vec![],

        stats: [26, 6, 3, 14, 15],
        stat_modifiers: [vec![], vec![], vec![], vec![], vec![]],
        active_stat_modifiers: [vec![], vec![], vec![], vec![], vec![]],
    };

    let combatants = &mut vec![brayden, chay];
    simulate_combat(combatants);
    println!("{:#?}", combatants);

    Ok(())
}
