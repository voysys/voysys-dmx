use channel::ChannelWidget;
use eframe::egui::{self, DragValue, Slider, Widget};
use std::{
    io::Write,
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use zerocopy::AsBytes;

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

fn tcp_thread(rx: Receiver<DmxMessage>, run: Arc<AtomicBool>) {
    match TcpStream::connect("10.0.11.3:33333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 33333");
            while run.load(Ordering::SeqCst) {
                if let Ok(msg) = rx.recv_timeout(Duration::from_millis(10)) {
                    let data = msg.as_bytes();
                    stream.write_all(data).unwrap();
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Closing TCP Thread.");
}

#[derive(Debug, Default, Copy, Clone, AsBytes)]
#[repr(C)]
struct DmxMessage {
    channels: [DmxColor; 5],
}

#[derive(Debug, Default, Copy, Clone, AsBytes)]
#[repr(C)]
struct DmxColor {
    rgb: [u8; 3],
    white: u8,
    amber: u8,
    uv: u8,
}

struct Timeline {
    id: i8,
    red: ChannelWidget,
    green: ChannelWidget,
    blue: ChannelWidget,
    color: DmxColor,
    gain: f32,
    offset: f32,
}

impl Timeline {
    fn new(id: i8) -> Self {
        Self {
            id,
            red: ChannelWidget::new(),
            green: ChannelWidget::new(),
            blue: ChannelWidget::new(),
            color: DmxColor::default(),
            gain: 1.0,
            offset: 0.0,
        }
    }
}

struct MyApp {
    tcp_thread: Option<JoinHandle<()>>,
    run: Arc<AtomicBool>,
    tx: Sender<DmxMessage>,

    last_frame_time: Instant,
    time: f32,
    cycle_length: f32,
    timelines: Vec<Timeline>,
    lights: [i32; 5],
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
            tcp_thread,
            run,
            tx,
            last_frame_time: Instant::now(),
            time: 0.0,
            cycle_length: 5.0,
            timelines: vec![
                Timeline::new(0),
                Timeline::new(1),
                Timeline::new(2),
                Timeline::new(3),
                Timeline::new(4),
            ],
            lights: [0, 1, 2, 3, 4],
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

        let speed = 1000.0 / self.cycle_length;

        self.time += speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        self.timelines.retain(|timeline| timeline.id > -1);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Cycle");
                DragValue::new(&mut self.cycle_length).speed(0.01).ui(ui);
            });

            for i in &mut self.lights.iter_mut() {
                Slider::new(i, 0..=(self.timelines.len() as i32 - 1)).ui(ui);
            }

            if ui.button("Add track").clicked() {
                self.timelines
                    .push(Timeline::new(self.timelines.len() as i8))
            }

            for timeline in &mut self.timelines.iter_mut() {
                ui.horizontal(|ui| {
                    ui.label("Gain");
                    DragValue::new(&mut timeline.gain)
                        .clamp_range(0.0..=1.0)
                        .speed(0.01)
                        .ui(ui);
                    ui.label("Offset");
                    DragValue::new(&mut timeline.offset)
                        .clamp_range(0.0..=1000.0)
                        .speed(1.0)
                        .ui(ui);
                });

                timeline.color.rgb[0] = (timeline.red.ui(ui, self.time + timeline.offset)
                    * timeline.gain
                    * 255.0) as u8;
                timeline.color.rgb[1] = (timeline.green.ui(ui, self.time + timeline.offset)
                    * timeline.gain
                    * 255.0) as u8;
                timeline.color.rgb[2] = (timeline.blue.ui(ui, self.time + timeline.offset)
                    * timeline.gain
                    * 255.0) as u8;
                if ui.button("delete track").clicked() {
                    timeline.id = -1;
                }
                ui.add(egui::Separator::default());
            }

            let mut res = DmxMessage::default();

            for (i, light) in self.lights.iter().copied().enumerate() {
                if let Some(timeline) = self.timelines.get(light as usize) {
                    res.channels[i] = timeline.color;
                }
            }

            self.tx.send(res).ok();
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
