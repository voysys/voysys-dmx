use dmx_shared::DmxMessage;
use eframe::{
    egui::Ui,
    egui::{CollapsingHeader, DragValue, Widget},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct DmxDevice {
    pub enabled: bool,
    adress: u16,
    size: u16,
    name: String,

    #[serde(skip_serializing)]
    values: Vec<u8>,
    timelines: Vec<Timeline>,
    //lights: [i32; 5],
}

impl DmxDevice {
    pub fn update(&mut self, ui: &mut Ui, index: usize, dmx_message: &mut DmxMessage) {
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
                    }
                });

                for (index, value) in &mut self.values.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Channel {index}"));
                        let mut temp_value = *value as i32;
                        DragValue::new(&mut temp_value)
                            .clamp_range(0..=255)
                            .speed(1.0)
                            .ui(ui);
                        *value = temp_value as u8;
                    });
                }
            });

        if self.enabled {
            for i in 0..self.size {
                dmx_message.buffer[(self.adress + i) as usize] = self.values[i as usize];
            }
        }
    }
}
