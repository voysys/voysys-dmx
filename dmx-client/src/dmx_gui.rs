use crate::timeline::Timeline;
use eframe::{
    egui::Ui,
    egui::{self, CollapsingHeader, DragValue, Widget},
};

pub fn generic_gui(
    ui: &mut Ui,
    cycle_length: &mut f32,
    values: &mut Vec<u8>,
    timelines: &mut Vec<Timeline>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Cycle");
            DragValue::new(cycle_length).speed(0.01).ui(ui);
        });

        for (index, value) in &mut values.iter_mut().enumerate() {
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

            let timeline = &mut timelines[index];
            CollapsingHeader::new(format!(
                "{}: Timeline ({})",
                index + 1,
                if timeline.channel.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            ))
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(if timeline.channel.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    });
                    if ui.button("Toggle").clicked() {
                        timeline.channel.enabled = !timeline.channel.enabled;
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

                timeline.channel.ui(ui);
                ui.add(egui::Separator::default());
            });
        }
    });
}

pub fn generic_smoke(
    ui: &mut Ui,
    cycle_length: &mut f32,
    values: &mut Vec<u8>,
    timelines: &mut Vec<Timeline>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Cycle");
            DragValue::new(cycle_length).speed(0.01).ui(ui);
        });
    });
}
