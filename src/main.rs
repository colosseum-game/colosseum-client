use coliseum::{
    actions::Action,
    combatant::Combatant,
};

fn calculate_turn_order(combatants: &[&mut Combatant]) -> Vec<usize> {
    let combatant_count = combatants.len();
    let mut turn_order = vec![0; combatant_count];

    for i in 0..combatant_count { turn_order[i] = i; }
    turn_order.sort_by(|a, b| combatants[*a].speed.cmp(&combatants[*b].speed).reverse());

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

fn simulate_combat(combatants: &mut [&mut Combatant]) {
    let turn_order = calculate_turn_order(combatants);

    let mut living_count = u32::MAX;
    while living_count > 1 {
        turn_order.iter().for_each(|combatant_index| {
            let action_index = if *combatant_index == 0 { get_action_index(combatants[*combatant_index]) }
            else { 0 };

            let action = combatants[*combatant_index].actions[action_index].function();
            action(combatants, &[(combatant_index + 1) % combatants.len()], *combatant_index);
        });

        living_count = 0;
        combatants.iter().for_each(|combatant| if combatant.alive() { living_count += 1; });
    }
}

fn main() {
    let mut brayden = Combatant {
        name: "Brayden".to_string(),
        hp: 45,
        hp_max: 45,
        physical_attack: 12,
        physical_resistance: 6,
        intelligence: 69,
        speed: 8,
        actions: vec![
            Action::Attack,
            Action::Cry,
            Action::Skip,
            Action::UseItem,
        ],
    };

    let mut chay = Combatant {
        name: "Chay".to_string(),
        hp: 30,
        hp_max: 30,
        physical_attack: 7,
        physical_resistance: 8,
        intelligence: 420,
        speed: 12,
        actions: vec![
            Action::Attack,
            Action::Cry,
            Action::Skip,
            Action::UseItem,
        ],
    };

    let mut tree = Combatant {
        name: "Birch Boy".to_string(),
        hp: 700,
        hp_max: 700,
        physical_attack: 0,
        physical_resistance: 8,
        intelligence: 0,
        speed: 1,
        actions: vec![
            Action::Skip,
            Action::Cry,
        ],
    };

    let mut combatants = vec![&mut brayden, &mut chay, &mut tree];
    simulate_combat(&mut combatants);
    println!("{:?}", combatants);
}
