use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use channel::ChannelWidget;
use eframe::egui::color_picker::color_edit_button_srgb;
use eframe::egui::{DragValue, Sense, Widget};
use eframe::epaint::{self, Color32, PathShape, Pos2, Rect, Shape, Stroke, Vec2};
use eframe::{egui, emath};

mod channel;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Voysys DMX controller",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

fn tcp_thread(rx: Receiver<DmxColor>, run: Arc<AtomicBool>) {
    match TcpStream::connect("10.0.11.3:33333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 33333");
            while run.load(Ordering::SeqCst) {
                if let Ok(msg) = rx.recv_timeout(Duration::from_millis(10)) {
                    let data = [
                        msg.rgb[0], msg.rgb[1], msg.rgb[2], msg.white, msg.amber, msg.uv,
                    ];

                    stream.write_all(&data).unwrap();
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

    red: ChannelWidget,
    green: ChannelWidget,
    blue: ChannelWidget,

    last_frame_time: Instant,
    time: f32,
    speed: f32,
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
            red: ChannelWidget::new(),
            green: ChannelWidget::new(),
            blue: ChannelWidget::new(),
            last_frame_time: Instant::now(),
            time: 0.0,
            speed: 10.0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let dt = {
            let new_time = Instant::now();
            let dt = new_time
                .saturating_duration_since(self.last_frame_time)
                .as_secs_f32();
            self.last_frame_time = new_time;
            dt
        };

        self.time += self.speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            DragValue::new(&mut self.speed).speed(0.01).ui(ui);

            color_edit_button_srgb(ui, &mut self.colors.rgb).changed();
            ui.add(egui::Slider::new(&mut self.colors.white, 0..=255).text("White"));
            ui.add(egui::Slider::new(&mut self.colors.amber, 0..=255).text("Amber"));
            ui.add(egui::Slider::new(&mut self.colors.uv, 0..=255).text("UV"));

            self.colors.rgb[0] = (self.red.ui(ui, self.time) * 255.0) as u8;
            self.colors.rgb[1] = (self.green.ui(ui, self.time) * 255.0) as u8;
            self.colors.rgb[2] = (self.blue.ui(ui, self.time) * 255.0) as u8;

            self.tx.send(self.colors.clone()).ok();
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
