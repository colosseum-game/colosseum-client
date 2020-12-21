mod game_state;
use game_state::GameState;

mod event;
use event::Event;

mod input;
use input::{
    InputState,
    RawInput,
};

use tokio::runtime;

use winit::{
    event::{
        Event as WinitEvent,
        WindowEvent,
    },
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::Window,
};

fn main() {
    // threaded runtime
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();

    // winit
    let event_loop = EventLoop::new();
    let _window = Window::new(&event_loop);

    // state
    let mut game_state = GameState::new();
    let mut input_state = InputState::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            WinitEvent::LoopDestroyed => {
                // exit code
            },
            WinitEvent::MainEventsCleared => {
                let mut terminate_application = false;
                // poll and apply input events
                input_state.poll();
                while let Some(input) = input_state.pop_input() {
                    game_state.transform(&mut terminate_application, &runtime, Event::InputEvent(input));
                }

                game_state.transform(&mut terminate_application, &runtime, Event::DeltaTimeEvent(0.0));

                if terminate_application { *control_flow = ControlFlow::Exit }
            },
            WinitEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            WinitEvent::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                if input.state == winit::event::ElementState::Pressed {
                    if let Some(keycode) = input.virtual_keycode {
                        input_state.push_input_raw(RawInput::Keyboard(keycode));
                    }
                }
            },
            _ => (),
        }
    });
}
