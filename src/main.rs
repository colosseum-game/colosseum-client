use colosseum::{
    actions::*,
    combatant::{
        Combatant,
        Gender,
    },
};

mod combat_state;
use combat_state::CombatState;

mod input_state;
use input_state::InputState;

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
        agility: 12,
        physical_attack: 17,
        physical_defense: 7,
        physical_absorbtion: 0,
        fire_attack: 11,
        fire_defense: 0,
        fire_absorbtion: 12,

        agility_modifiers: vec![],
        physical_attack_modifiers: vec![],
        physical_defense_modifiers: vec![],
        physical_absorbtion_modifiers: vec![],
        fire_attack_modifiers: vec![],
        fire_defense_modifiers: vec![],
        fire_absorbtion_modifiers: vec![],

        status_effects: vec![],
    };

    let chay = Combatant {
        name: "Chay".to_string(),
        gender: Gender::Male,
        actions: vec![ActionIdentifier::Scorch],

        hp: 46,
        hp_max: 46,
        agility: 26,
        physical_attack: 6,
        physical_defense: 3,
        physical_absorbtion: 8,
        fire_attack: 16,
        fire_defense: 0,
        fire_absorbtion: 126,

        agility_modifiers: vec![],
        physical_attack_modifiers: vec![],
        physical_defense_modifiers: vec![],
        physical_absorbtion_modifiers: vec![],
        fire_attack_modifiers: vec![],
        fire_defense_modifiers: vec![],
        fire_absorbtion_modifiers: vec![],

        status_effects: vec![],
    };

    std::fs::write("combatants/brayden.json", serde_json::to_string_pretty(&brayden).expect("Unable to write to file"))?;

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("failed to create window_state");
    let mut input_state = Some(InputState::new());
    let mut combat_state = Some(CombatState::new(vec![brayden, chay]));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                input_state = Some(input_state.take().unwrap().update());
                combat_state = Some(combat_state.take().unwrap().update(input_state.as_ref().unwrap()));
            },
            _ => (),
        }
    });
}
