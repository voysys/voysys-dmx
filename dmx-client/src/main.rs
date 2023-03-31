use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use eframe::egui;
use eframe::egui::color_picker::color_edit_button_srgb;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Voysys DMX controller",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

fn tcp_thread(rx: Receiver<DmxColor>, run: Arc<AtomicBool>) {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");
            while run.load(Ordering::SeqCst) {
                if let Ok(msg) = rx.recv_timeout(Duration::from_millis(10)) {
                    let data = [
                        msg.rgb[0], msg.rgb[1], msg.rgb[2], msg.white, msg.amber, msg.uv,
                    ];

                    stream.write(&data).unwrap();
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Closing TCP Thread.");
}

#[derive(Debug, Default, Clone)]
struct DmxColor {
    rgb: [u8; 3],
    white: u8,
    amber: u8,
    uv: u8,
}

struct MyApp {
    colors: DmxColor,
    tcp_thread: Option<JoinHandle<()>>,
    run: Arc<AtomicBool>,
    tx: Sender<DmxColor>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let run = Arc::new(AtomicBool::new(true));

        let tcp_thread = {
            let run = run.clone();
            Some(thread::spawn(move || tcp_thread(rx, run)))
        };
        Self {
            colors: DmxColor::default(),
            tcp_thread,
            tx,
            run,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut send = false;
            send |= color_edit_button_srgb(ui, &mut self.colors.rgb).changed();
            send |= ui
                .add(egui::Slider::new(&mut self.colors.white, 0..=255).text("White"))
                .changed();
            send |= ui
                .add(egui::Slider::new(&mut self.colors.amber, 0..=255).text("Amber"))
                .changed();
            send |= ui
                .add(egui::Slider::new(&mut self.colors.uv, 0..=255).text("UV"))
                .changed();

            if send {
                self.tx.send(self.colors.clone()).ok();
            }
        });
    }
}

impl Drop for MyApp {
    fn drop(&mut self) {
        self.run.store(false, Ordering::SeqCst);
        if let Some(thread) = self.tcp_thread.take() {
            thread.join().ok();
        }
    }
}
