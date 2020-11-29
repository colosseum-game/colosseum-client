use crate::client_event::{
    ClientEvent,
    NetworkEvent,
};

use tokio::{
    net::TcpStream,
    runtime::Runtime,
    sync::oneshot,
};

use winit::event_loop::EventLoopProxy;

pub enum ServerConnection {
    Connecting(oneshot::Receiver<Result<TcpStream, std::io::Error>>),
    Connected(TcpStream),
    ConnectionFailed,
}

impl ServerConnection {
    pub fn connect(runtime: &Runtime) -> ServerConnection {
        let (sender, receiver) = oneshot::channel();
        runtime.spawn(async move { sender.send(TcpStream::connect("localhost:40004").await).unwrap(); });
        ServerConnection::Connecting(receiver)
    }

    pub fn update(&mut self, event_loop_proxy: &EventLoopProxy<ClientEvent>) {
        match self {
            ServerConnection::Connecting(reciever) => {
                if let Ok(result) = reciever.try_recv() {
                    match result {
                        Ok(stream) => {
                            *self = ServerConnection::Connected(stream);
                            event_loop_proxy.send_event(ClientEvent::NetworkEvent(NetworkEvent::Connected)).expect("event loop was dropped");
                        },
                        Err(_) => {
                            *self = ServerConnection::ConnectionFailed;
                            event_loop_proxy.send_event(ClientEvent::NetworkEvent(NetworkEvent::ConnectFailed)).expect("event loop was dropped");
                        },
                    }
                }
            }
            ServerConnection::Connected(_) => (),
            ServerConnection::ConnectionFailed => (),
        }
    }
}
