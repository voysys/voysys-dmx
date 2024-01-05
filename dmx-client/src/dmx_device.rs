use dmx_shared::{DmxColor, DmxMessage};
use eframe::{
    egui::Ui,
    egui::{self, CollapsingHeader, DragValue, Widget},
};
use serde::{Deserialize, Serialize};

use crate::channel::ChannelWidget;

#[derive(Serialize, Deserialize, Clone)]
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
pub struct DmxDevice {
    pub enabled: bool,
    adress: u16,
    size: u16,
    pub name: String,
    cycle_length: f32,
    timelines: Vec<Timeline>,

    values: Vec<u8>,
    #[serde(skip)]
    time: f32,
}

impl Default for DmxDevice {
    fn default() -> Self {
        DmxDevice {
            enabled: false,
            adress: 0,
            size: 1,
            name: "Device".to_string(),
            cycle_length: 10.0,
            timelines: Vec::new(),
            values: Vec::new(),
            time: 0.0,
        }
    }
}

impl DmxDevice {
    pub fn update(&mut self, dmx_message: &mut DmxMessage, dt: f32) {
        let speed = 1000.0 / self.cycle_length;

        self.time += speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        for (index, value) in &mut self.values.iter_mut().enumerate() {
            let timeline = &mut self.timelines[index];

            if timeline.red.enabled {
                *value = (timeline.red.update(self.time + timeline.offset) * timeline.gain * 255.0)
                    as u8;
            }
        }

        if self.enabled && self.size as usize == self.values.len() {
            for i in 0..self.size {
                dmx_message.buffer[(self.adress + i) as usize] = self.values[i as usize];
            }
        }
    }

    pub fn gui(&mut self, ui: &mut Ui, dt: f32) -> bool {
        let mut delete = false;
        let speed = 1000.0 / self.cycle_length;

        self.time += speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.name);

                if ui
                    .button(if self.enabled { "Enabled" } else { "Disabled" })
                    .clicked()
                {
                    self.enabled = !self.enabled;
                }

                delete = ui.button("Delete").clicked();
            });

            ui.horizontal(|ui| {
                ui.label("Address:");
                let mut address = self.adress + 1;

                DragValue::new(&mut address)
                    .clamp_range(1..=512)
                    .speed(1.0)
                    .ui(ui);
                self.adress = address - 1;

                ui.label("Size:");
                if DragValue::new(&mut self.size)
                    .clamp_range(0..=512)
                    .speed(1.0)
                    .ui(ui)
                    .changed()
                {
                    self.values.resize(self.size as usize, 0);
                    self.timelines.resize(self.size as usize, Timeline::new(0));
                }
            });

            ui.horizontal(|ui| {
                ui.label("Cycle");
                DragValue::new(&mut self.cycle_length).speed(0.01).ui(ui);
            });

            for (index, value) in &mut self.values.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Channel {}", index + 1));
                    let mut temp_value = *value as i32;
                    if DragValue::new(&mut temp_value)
                        .clamp_range(0..=255)
                        .speed(1.0)
                        .ui(ui)
                        .changed()
                    {
                        *value = temp_value as u8;
                    }
                });

                let timeline = &mut self.timelines[index];
                CollapsingHeader::new(format!(
                    "{}: Timeline ({})",
                    index + 1,
                    if timeline.red.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                ))
                .default_open(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(if timeline.red.enabled {
                            "Enabled"
                        } else {
                            "Disabled"
                        });
                        if ui.button("Toggle").clicked() {
                            timeline.red.enabled = !timeline.red.enabled;
                        }

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

                    timeline.red.ui(ui);
                    ui.add(egui::Separator::default());
                });
            }
        });

        delete
    }
}
