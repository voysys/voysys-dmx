use rust_dmx::{available_ports, DmxPort};
use std::{
    io::Read,
    net::{Shutdown, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use zerocopy::FromBytes;

#[derive(Debug, Default, Copy, Clone, FromBytes)]
#[repr(C)]
struct DmxMessage {
    channels: [DmxColor; 5],
}

#[derive(Debug, Default, Copy, Clone, FromBytes)]
#[repr(C)]
struct DmxColor {
    rgb: [u8; 3],
    white: u8,
    amber: u8,
    uv: u8,
}

impl DmxColor {
    fn dmx(self) -> [u8; 12] {
        [
            self.rgb[0],
            self.rgb[1],
            self.rgb[2],
            self.white,
            self.amber,
            self.uv,
            0xff,
            0xff,
            0x00,
            0x00,
            0x00,
            0x00,
        ]
    }
}

fn dmx(msg: &DmxMessage) -> [u8; 60] {
    let mut output = [0; 60];

    for i in 0..msg.channels.len() {
        let color = msg.channels[i];

        output[(12 * i)..(12 * (i + 1))].copy_from_slice(&color.dmx());
    }

    output
}

fn handle_client(mut stream: TcpStream, handle: Arc<Mutex<DmxHandle>>) {
    let mut data = [0_u8; 6 * 5];

    while match stream.read_exact(&mut data) {
        Ok(()) => {
            let msg = DmxMessage::read_from(data.as_slice()).unwrap();
            {
                let mut handle = handle.lock().unwrap();
                handle.port.write(&dmx(&msg)).unwrap();
            }

            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

struct DmxHandle {
    port: Box<dyn DmxPort>,
}

unsafe impl Send for DmxHandle {}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:33333").unwrap();
    println!("Server listening on port 33333");

    let port = Arc::new(Mutex::new({
        let mut ports = available_ports().unwrap();
        let mut port = ports.remove(1);
        port.open().unwrap();
        DmxHandle { port }
    }));

    for stream in listener.incoming() {
        let port = port.clone();
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream, port));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
