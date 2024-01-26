use dmx_device::DmxDevice;
use dmx_shared::DmxMessage;
use eframe::{
    egui::{self},
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
mod dmx_gui;
mod timeline;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 800.0]),
        ..Default::default()
    };

    let state = fs::read_to_string("state.json").ok().unwrap();

    let state = serde_json::from_str::<State>(&state);

    let state = match state {
        Ok(s) => s,
        Err(e) => {
            log::info!("{e}");

            State {
                devices: Vec::new(),
            }
        }
    };

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
    devices: Vec<DmxDevice>,
}

struct App {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,

    last_frame_time: Instant,

    state: State,

    selected_device: i32,
}

impl App {
    fn new(state: State) -> Self {
        let (ws_sender, ws_receiver) = ewebsock::connect("ws://10.0.11.4:33333").unwrap();

        Self {
            ws_sender,
            ws_receiver,
            last_frame_time: Instant::now(),
            state,
            selected_device: -1,
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

        // Update
        {
            let mut res = DmxMessage {
                buffer: vec![0u8; 512],
            };

            for device in &mut self.state.devices {
                device.update(&mut res, dt);
            }

            while let Some(_event) = self.ws_receiver.try_recv() {}

            self.ws_sender
                .send(WsMessage::Text(serde_json::to_string(&res).unwrap()));
        }

        egui::SidePanel::left("item_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(80.0..=250.0)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("left_panel")
                    .resizable(false)
                    .min_height(64.0)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Enable All").clicked() {
                                for device in &mut self.state.devices {
                                    if device.dmx_type != dmx_device::DmxDeviceType::Af250 {
                                        device.enabled = true;
                                    }
                                }
                            }

                            if ui.button("Disable All").clicked() {
                                for device in &mut self.state.devices {
                                    device.enabled = false;
                                }
                            }
                        });

                        ui.menu_button("Add Device", |ui| {
                            let device = if ui.button("Generic").clicked() {
                                Some(dmx_device::DmxDeviceType::Generic)
                            } else if ui.button("Hero S").clicked() {
                                Some(dmx_device::DmxDeviceType::HeroSpot90)
                            } else if ui.button("Show Bar Tri").clicked() {
                                Some(dmx_device::DmxDeviceType::ShowBarTri)
                            } else if ui.button("5 Px Hex").clicked() {
                                Some(dmx_device::DmxDeviceType::Generic)
                            } else if ui.button("Af 250 Smoke").clicked() {
                                Some(dmx_device::DmxDeviceType::Af250)
                            } else {
                                None
                            };

                            if let Some(device_type) = device {
                                ui.close_menu();
                                self.state.devices.push(DmxDevice::new(device_type));
                            }
                        });
                    });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, device) in self.state.devices.iter_mut().enumerate() {
                        ui.selectable_value(&mut self.selected_device, index as i32, &device.name);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.selected_device >= 0 {
                let device = &mut self.state.devices[self.selected_device as usize];
                if device.gui(ui) {
                    self.state.devices.remove(self.selected_device as usize);
                    self.selected_device = -1;
                }
            }
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
