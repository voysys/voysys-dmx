use crate::timeline::Timeline;
use eframe::{
    egui::Ui,
    egui::{self, CollapsingHeader, DragValue, Slider, SliderOrientation, Widget},
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

pub fn smoke(ui: &mut Ui, values: &mut Vec<u8>, enabled: &mut bool) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if values.is_empty() {
            return;
        }
        ui.horizontal(|ui| {
            *enabled = ui.button("Deploy Smoke").is_pointer_button_down_on();

            let mut value = ((values[0] as f32 / 255.0) * 100.0) as i32;

            ui.add(
                Slider::new(&mut value, 0..=100)
                    .clamp_to_range(true)
                    .orientation(SliderOrientation::Horizontal)
                    .text("Amount")
                    .step_by(1.0)
                    .suffix("%"),
            );

            values[0] = ((value as f32 / 100.0) * 255.0).clamp(0.0, 255.0) as u8;
        });
    });
}
