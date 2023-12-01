use channel::ChannelWidget;
use eframe::{
    egui::{self, DragValue, Slider, Widget},
    Storage,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
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
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 800.0]),
        ..Default::default()
    };

    let state = fs::read_to_string("state.json")
        .ok()
        .and_then(|s| serde_json::from_str::<State>(&s).ok())
        .unwrap_or(State {
            cycle_length: 5.0,
            timelines: vec![
                Timeline::new(0),
                Timeline::new(1),
                Timeline::new(2),
                Timeline::new(3),
                Timeline::new(4),
            ],
            lights: [0, 1, 2, 3, 4],
        });

    eframe::run_native(
        "Voysys DMX controller",
        options,
        Box::new(|_cc| Box::new(App::new(state))),
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

#[derive(Debug, Default, Copy, Clone, AsBytes, Serialize, Deserialize)]
#[repr(C)]
struct DmxMessage {
    channels: [DmxColor; 5],
}

#[derive(Debug, Default, Copy, Clone, AsBytes, Serialize, Deserialize)]
#[repr(C)]
struct DmxColor {
    rgb: [u8; 3],
    white: u8,
    amber: u8,
    uv: u8,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct State {
    cycle_length: f32,
    timelines: Vec<Timeline>,
    lights: [i32; 5],
}

struct App {
    tcp_thread: Option<JoinHandle<()>>,
    run: Arc<AtomicBool>,
    tx: Sender<DmxMessage>,

    last_frame_time: Instant,
    time: f32,
    state: State,
}

impl App {
    fn new(state: State) -> Self {
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
            state,
        }
    }
}

impl eframe::App for App {
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

        let speed = 1000.0 / self.state.cycle_length;

        self.time += speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        self.state.timelines.retain(|timeline| timeline.id > -1);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Cycle");
                DragValue::new(&mut self.state.cycle_length)
                    .speed(0.01)
                    .ui(ui);
            });

            for i in &mut self.state.lights.iter_mut() {
                Slider::new(i, 0..=(self.state.timelines.len() as i32 - 1)).ui(ui);
            }

            if ui.button("Add track").clicked() {
                self.state
                    .timelines
                    .push(Timeline::new(self.state.timelines.len() as i8))
            }

            for timeline in &mut self.state.timelines.iter_mut() {
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

            for (i, light) in self.state.lights.iter().copied().enumerate() {
                if let Some(timeline) = self.state.timelines.get(light as usize) {
                    res.channels[i] = timeline.color;
                }
            }

            self.tx.send(res).ok();
        });
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        let s = serde_json::to_string(&self.state).unwrap();
        fs::write("state.json", s).ok();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.run.store(false, Ordering::SeqCst);
        if let Some(thread) = self.tcp_thread.take() {
            thread.join().ok();
        }
    }
}
