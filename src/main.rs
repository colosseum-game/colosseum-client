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
use input_state::{
    InputState
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

    let mut input_state = Some(InputState::new());
    let mut combat_state = Some(CombatState::new(vec![brayden, chay]));

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("failed to create window_state");

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
