use dmx_shared::DmxMessage;
use eframe::{
    egui::Ui,
    egui::{self, DragValue, Widget},
};
use serde::{Deserialize, Serialize};

use crate::{dmx_gui, timeline::Timeline};

#[derive(Serialize, Deserialize, Default)]

struct DmxAddress {
    adress: u16,
    size: u16,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum DmxDeviceType {
    Generic,
    HeroSpot90,
    ShowBarTri,
    PxHex5,
    Af250,
}

impl DmxDeviceType {
    pub fn name(&self) -> &'static str {
        match self {
            DmxDeviceType::Generic => "Generic",
            DmxDeviceType::HeroSpot90 => "Hero S",
            DmxDeviceType::ShowBarTri => "Show Bar Tri",
            DmxDeviceType::PxHex5 => "5 Px Hex",
            DmxDeviceType::Af250 => "Af 250 Smoke",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DmxDevice {
    pub enabled: bool,
    dmx_addres: DmxAddress,
    pub name: String,
    cycle_length: f32,
    timelines: Vec<Timeline>,
    values: Vec<u8>,
    #[serde(skip)]
    time: f32,
    pub dmx_type: DmxDeviceType,
}

impl Default for DmxDevice {
    fn default() -> Self {
        DmxDevice {
            enabled: false,
            dmx_addres: DmxAddress::default(),
            name: "Device".to_string(),
            cycle_length: 10.0,
            timelines: Vec::new(),
            values: Vec::new(),
            time: 0.0,
            dmx_type: DmxDeviceType::Generic,
        }
    }
}

impl DmxDevice {
    pub fn new(dmx_type: DmxDeviceType) -> DmxDevice {
        let (dmx_addres, values, timelines) = match dmx_type {
            DmxDeviceType::Generic => (DmxAddress::default(), Vec::new(), Vec::new()),
            DmxDeviceType::HeroSpot90 => (DmxAddress::default(), Vec::new(), Vec::new()),
            DmxDeviceType::ShowBarTri => (DmxAddress::default(), Vec::new(), Vec::new()),
            DmxDeviceType::PxHex5 => (DmxAddress::default(), Vec::new(), Vec::new()),
            DmxDeviceType::Af250 => (
                DmxAddress { adress: 0, size: 1 },
                vec![0; 1],
                vec![Timeline::new(); 1],
            ),
        };

        DmxDevice {
            enabled: false,
            dmx_addres,
            name: dmx_type.name().to_string(),
            cycle_length: 10.0,
            timelines,
            values,
            time: 0.0,
            dmx_type,
        }
    }

    pub fn update(&mut self, dmx_message: &mut DmxMessage, dt: f32) {
        let speed = 1000.0 / self.cycle_length;

        self.time += speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        for (index, value) in &mut self.values.iter_mut().enumerate() {
            let timeline = &mut self.timelines[index];

            if timeline.channel.enabled {
                *value = (timeline.channel.update(self.time + timeline.offset)
                    * timeline.gain
                    * 255.0) as u8;
            }
        }

        if self.enabled && self.dmx_addres.size as usize == self.values.len() {
            for i in 0..self.dmx_addres.size {
                dmx_message.buffer[(self.dmx_addres.adress + i) as usize] = self.values[i as usize];
            }
        }
    }

    pub fn gui(&mut self, ui: &mut Ui) -> bool {
        let mut delete = false;
        egui::TopBottomPanel::top("top_dmx_panel")
            .resizable(false)
            .min_height(64.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.name);

                    delete = ui.button("Delete").clicked();
                });

                ui.horizontal(|ui| {
                    ui.label("Address:");
                    let mut address = self.dmx_addres.adress + 1;

                    DragValue::new(&mut address)
                        .clamp_range(1..=512)
                        .speed(1.0)
                        .ui(ui);
                    self.dmx_addres.adress = address - 1;
                    if self.dmx_type == DmxDeviceType::Generic {
                        ui.label("Size:");
                        if DragValue::new(&mut self.dmx_addres.size)
                            .clamp_range(0..=512)
                            .speed(1.0)
                            .ui(ui)
                            .changed()
                        {
                            self.values.resize(self.dmx_addres.size as usize, 0);
                            self.timelines
                                .resize(self.dmx_addres.size as usize, Timeline::new());
                        }
                    }
                });

                if self.dmx_type != DmxDeviceType::Af250
                    && ui
                        .button(if self.enabled { "Enabled" } else { "Disabled" })
                        .clicked()
                {
                    self.enabled = !self.enabled;
                }
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            match &mut self.dmx_type {
                DmxDeviceType::Generic => dmx_gui::generic_gui(
                    ui,
                    &mut self.cycle_length,
                    &mut self.values,
                    &mut self.timelines,
                ),
                DmxDeviceType::HeroSpot90 => (),
                DmxDeviceType::ShowBarTri => (),
                DmxDeviceType::PxHex5 => (),
                DmxDeviceType::Af250 => dmx_gui::smoke(ui, &mut self.values, &mut self.enabled),
            };
        });

        delete
    }
}
