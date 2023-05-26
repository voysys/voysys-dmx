use rust_dmx::{available_ports, DmxPort};
use std::{
    io::Read,
    net::{Shutdown, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

#[derive(Copy, Clone, Default)]
struct Color(u8, u8, u8, u8, u8, u8);

impl Color {
    fn dmx(self) -> [u8; 12] {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
        ]
    }
}

fn dmx(leds: &[Color; 5]) -> [u8; 60] {
    let mut output = [0; 60];

    for i in 0..leds.len() {
        let color = leds[i];

        output[(12 * i)..(12 * (i + 1))].copy_from_slice(&color.dmx());
    }

    output
}

fn handle_client(mut stream: TcpStream, handle: Arc<Mutex<DmxHandle>>) {
    let mut data = [0_u8; 6]; // using 50 byte buffer

    while match stream.read(&mut data) {
        Ok(size) => {
            if size > 0 {
                println!("{size} {data:?}");
                let mut leds = [Color::default(); 5];
                leds[0] = Color(data[0], data[1], data[2], data[3], data[4], data[5]);
                leds[1] = leds[0];
                leds[2] = leds[0];
                leds[3] = leds[0];
                leds[4] = leds[0];

                {
                    let mut handle = handle.lock().unwrap();
                    handle.port.write(&dmx(&leds)).unwrap();
                }
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
        let mut port = ports.remove(0);
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
