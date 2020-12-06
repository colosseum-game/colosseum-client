mod client_state;
use client_state::ClientState;

mod client_event;
use client_event::{
    ClientEvent,
    ControlEvent,
    NetworkEvent,
};

use colosseum::connection::Connection;

use gilrs::Button;

mod input;
use input::{
    Input,
    RawInput,
};

use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    net::SocketAddr,
};

use tokio::{
    net::TcpStream,
    runtime,
};

use winit::{
    event::{
        Event as WinitEvent,
        VirtualKeyCode,
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
    let event_loop = EventLoop::with_user_event();
    let event_loop_proxy = event_loop.create_proxy();
    let _window = Window::new(&event_loop);

    // state
    let mut client_state = ClientState::new();

    //input
    let mut gilrs = gilrs::Gilrs::new().unwrap();
    let mut input_queue = VecDeque::new();
    let input_map: HashMap<RawInput, Input> = [
        (RawInput::Gamepad(Button::DPadDown), Input::Down),
        (RawInput::Gamepad(Button::DPadUp), Input::Up),
        (RawInput::Gamepad(Button::DPadRight), Input::Right),
        (RawInput::Gamepad(Button::DPadLeft), Input::Left),
        (RawInput::Gamepad(Button::East), Input::Cancel),
        (RawInput::Gamepad(Button::South), Input::Select),
        (RawInput::Keyboard(VirtualKeyCode::A), Input::Left),
        (RawInput::Keyboard(VirtualKeyCode::D), Input::Right),
        (RawInput::Keyboard(VirtualKeyCode::E), Input::Select),
        (RawInput::Keyboard(VirtualKeyCode::Q), Input::Cancel),
        (RawInput::Keyboard(VirtualKeyCode::S), Input::Down),
        (RawInput::Keyboard(VirtualKeyCode::W), Input::Up),
    ].iter().cloned().collect();

    // server connection
    let mut server_connection: Option<Connection> = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            WinitEvent::UserEvent(event) => match event {
                ClientEvent::ControlEvent(event) => match event {
                    ControlEvent::Terminate => *control_flow = ControlFlow::Exit
                },
                ClientEvent::Input(input) => input_queue.push_back(input),
                ClientEvent::NetworkEvent(event) => match event {
                    NetworkEvent::Connect => {
                        let addr: SocketAddr = "127.0.0.1:40004".parse().unwrap();
                        let stream = runtime.block_on(async { TcpStream::connect(addr).await.unwrap() });
                        server_connection = Some(Connection::new(addr, stream));
                        client_state.transform(&event_loop_proxy, ClientEvent::NetworkEvent(NetworkEvent::Connected));
                    },
                    NetworkEvent::Disconnect => server_connection = None,
                    _ => (),
                }
            },
            WinitEvent::LoopDestroyed => {
                // Exit code
            },
            WinitEvent::MainEventsCleared => {
                while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                    match event {
                        gilrs::EventType::ButtonPressed(button, ..) => {
                            if let Some(input) = input_map.get(&RawInput::Gamepad(button)) { input_queue.push_back(*input) }
                        },
                        _ => (),
                    };
                }

                while let Some(input) = input_queue.pop_front() {
                    client_state.transform(&event_loop_proxy, ClientEvent::Input(input));
                }
            },
            WinitEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            WinitEvent::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                if input.state == winit::event::ElementState::Pressed {
                    if let Some(keycode) = input.virtual_keycode {
                        if let Some(input) = input_map.get(&RawInput::Keyboard(keycode)) { input_queue.push_back(*input) }
                    }
                }
            },
            _ => (),
        }
    });
}
