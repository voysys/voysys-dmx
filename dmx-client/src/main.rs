use channel::ChannelWidget;
use dmx_device::DmxDevice;
use dmx_shared::{DmxColor, DmxMessage};
use eframe::{
    egui::{self, DragValue, Slider, Widget},
    Storage,
};
use ewebsock::{WsMessage, WsReceiver, WsSender};
use serde::{Deserialize, Serialize};
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

mod channel;
mod dmx_device;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 800.0]),
        ..Default::default()
    };

    let state = fs::read_to_string("state.json")
        .ok()
        .and_then(|s| serde_json::from_str::<State>(&s).ok())
        .unwrap_or(State {
            lights: [0, 1, 2, 3, 4],
            devices: Vec::new(),
        });

    eframe::run_native(
        "Voysys DMX controller",
        options,
        Box::new(|_cc| Box::new(App::new(state))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    let state = State {
        cycle_length: 5.0,
        timelines: vec![
            Timeline::new(0),
            Timeline::new(1),
            Timeline::new(2),
            Timeline::new(3),
            Timeline::new(4),
        ],
        lights: [0, 1, 2, 3, 4],
    };

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "voysys-dmx", // hardcode it
                web_options,
                Box::new(|cc| Box::new(App::new(state))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[derive(Serialize, Deserialize)]
struct State {
    lights: [i32; 5],
    devices: Vec<DmxDevice>,
}

struct App {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,

    last_frame_time: Instant,

    state: State,
    smoke: Option<u8>,
}

impl App {
    fn new(state: State) -> Self {
        let (ws_sender, ws_receiver) = ewebsock::connect("ws://10.0.11.3:33333").unwrap();

        Self {
            ws_sender,
            ws_receiver,
            last_frame_time: Instant::now(),
            state,
            smoke: None,
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

        // self.state.timelines.retain(|timeline| timeline.id > -1);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Add DMX Device").clicked() {
                        self.state.devices.push(DmxDevice::default());
                    }

                    if ui.button("Enable All").clicked() {
                        for device in &mut self.state.devices {
                            device.enabled = true;
                        }
                    }

                    if ui.button("Disable All").clicked() {
                        for device in &mut self.state.devices {
                            device.enabled = false;
                        }
                    }
                });

                let mut res = DmxMessage {
                    buffer: vec![0u8; 512],
                };

                for (index, device) in self.state.devices.iter_mut().enumerate() {
                    device.update(ui, index, &mut res, dt);
                }

                /* for i in &mut self.state.lights.iter_mut() {
                    Slider::new(i, 0..=(self.state.timelines.len() as i32 - 1)).ui(ui);
                }

                if ui.button("Add track").clicked() {
                    self.state
                        .timelines
                        .push(Timeline::new(self.state.timelines.len() as i8))
                }

                if ui.button("Add smoke").is_pointer_button_down_on() {
                    self.smoke = Some(128);
                } else {
                    self.smoke = None;
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

                let mut res = DmxMessage {
                    buffer: vec![0u8; 512],
                };

                for (i, light) in self.state.lights.iter().copied().enumerate() {
                    if let Some(timeline) = self.state.timelines.get(light as usize) {
                        let start = i * 12;
                        let end = start + 12;

                        res.buffer[start..end].copy_from_slice(&timeline.color.dmx());
                    }
                }

                res.buffer[63] = self.smoke.unwrap_or_default();
                */

                while let Some(_event) = self.ws_receiver.try_recv() {}

                self.ws_sender
                    .send(WsMessage::Text(serde_json::to_string(&res).unwrap()));
            });
        });
    }

    fn save(&mut self, _storage: &mut dyn Storage) {
        let s = serde_json::to_string(&self.state).unwrap();
        fs::write("state.json", s).ok();
    }
}

impl Drop for App {
    fn drop(&mut self) {}
}
