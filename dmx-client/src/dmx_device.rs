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

#[derive(Serialize, Deserialize, Default)]
pub struct DmxDevice {
    pub enabled: bool,
    adress: u16,
    size: u16,
    name: String,
    cycle_length: f32,
    timelines: Vec<Timeline>,
    values: Vec<u8>,
    #[serde(skip)]
    time: f32,
}

impl DmxDevice {
    pub fn update(&mut self, ui: &mut Ui, index: usize, dmx_message: &mut DmxMessage, dt: f32) {
        let speed = 1000.0 / self.cycle_length;

        self.time += speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        CollapsingHeader::new(format!("Device {index} ({})", self.name))
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(if self.enabled { "Enabled" } else { "Disabled" });
                    if ui.button("Toggle").clicked() {
                        self.enabled = !self.enabled;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Cycle");
                    DragValue::new(&mut self.cycle_length).speed(0.01).ui(ui);
                });

                ui.horizontal(|ui| {
                    ui.label("Address:");
                    DragValue::new(&mut self.adress)
                        .clamp_range(0..=512)
                        .speed(1.0)
                        .ui(ui);
                });
                ui.horizontal(|ui| {
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

                    CollapsingHeader::new(format!("{}: Timeline", index + 1))
                        .default_open(false)
                        .show(ui, |ui| {
                            let timeline = &mut self.timelines[index];
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

                            *value = (timeline.red.ui(ui, self.time + timeline.offset)
                                * timeline.gain
                                * 255.0) as u8;
                            /*timeline.color.rgb[1] =
                                (timeline.green.ui(ui, self.time + timeline.offset)
                                    * timeline.gain
                                    * 255.0) as u8;
                            timeline.color.rgb[2] =
                                (timeline.blue.ui(ui, self.time + timeline.offset)
                                    * timeline.gain
                                    * 255.0) as u8;
                            if ui.button("delete track").clicked() {
                                timeline.id = -1;
                            }*/
                            ui.add(egui::Separator::default());
                        });
                }
            });

        if self.enabled && self.size as usize == self.values.len() {
            for i in 0..self.size {
                dmx_message.buffer[(self.adress + i) as usize] = self.values[i as usize];
            }
        }
    }
}
