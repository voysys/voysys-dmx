use rust_dmx::available_ports;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

#[derive(Copy, Clone, Default)]
struct Color(u8, u8, u8, u8, u8, u8);

impl Color {
    fn dmx(self) -> [u8; 12] {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    /*
    fn mix(&self, next: Color, t: f32) -> Color {
        Color(
            self.0 * t + next.0 * (1.0 - t),
            self.1 * t + next.1 * (1.0 - t),
            self.2 * t + next.2 * (1.0 - t),
            self.3 * t + next.3 * (1.0 - t),
            self.4 * t + next.4 * (1.0 - t),
            self.5 * t + next.5 * (1.0 - t),
        )
    }*/
}

fn dmx(leds: &[Color; 5]) -> [u8; 60] {
    let mut output = [0; 60];

    for i in 0..leds.len() {
        let next = (i + 1) % leds.len();

        let a = leds[i];
        let b = leds[next];

        let mix = a; //.mix(b, t);

        output[(12 * i)..(12 * (i + 1))].copy_from_slice(&mix.dmx());
    }

    output
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0_u8; 6]; // using 50 byte buffer
    let mut ports = available_ports().unwrap();
    let port = &mut ports[1];
    port.open().unwrap();

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

                port.write(&dmx(&leds)).unwrap();
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

const TIME: f32 = 5000.0;
fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);

    /*let mut ports = available_ports().unwrap();
    let port = &mut ports[1];
    port.open().unwrap();

    let colors: [Color; 5] = [
        Color(0.1, 0.2, 0.9, 0.0, 0.0, 0.0),
        Color(0.4, 0.2, 0.7, 0.0, 0.0, 0.0),
        Color(0.9, 0.1, 0.2, 0.0, 0.0, 0.0),
        Color(0.5, 0.2, 0.2, 0.0, 0.0, 0.0),
        Color(0.9, 0.1, 0.3, 0.0, 0.0, 0.0),
    ];
    let mut leds = [Color::default(); 5];

    leds = colors;

    let mut swap = Instant::now();

    loop {
        let t = swap.elapsed().as_millis() as f32 / TIME;

        if swap.elapsed() > Duration::from_millis(TIME as _) {
            swap = Instant::now();
            let temp0 = leds[0];
            for index in 0..(leds.len() - 1) {
                let next = index + 1;

                leds[index] = leds[next];
            }

            leds[leds.len() - 1] = temp0;
        }

        port.write(&dmx(&leds, t)).unwrap();
        //thread::sleep(Duration::from_millis(5));
    }*/
}
