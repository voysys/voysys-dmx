use crate::{
    dmx_device::px_hex_5::{PxHex5, PxHex5Strobe},
    timeline::Timeline,
};
use eframe::{
    egui::Ui,
    egui::{self, CollapsingHeader, DragValue, Slider, SliderOrientation, Widget},
};

fn slider_percent(ui: &mut Ui, value: &mut u8, value_range: f32, text: &str) {
    let mut temp_value = ((*value as f32 / value_range) * 100.0) as i32;
    if ui
        .add(
            Slider::new(&mut temp_value, 0..=100)
                .orientation(SliderOrientation::Horizontal)
                .text(text)
                .suffix("%"),
        )
        .is_pointer_button_down_on()
    {
        *value = ((temp_value as f32 / 100.0) * value_range).clamp(0.0, value_range) as u8;
    }
}

pub fn generic_gui(
    ui: &mut Ui,
    cycle_length: &mut f32,
    values: &mut [u8],
    timelines: &mut [Timeline],
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

            slider_percent(ui, &mut values[0], 255.0, "Intensity");
        });
    });
}

pub fn px_hex(ui: &mut Ui, values: &mut [u8], inner_data: &mut PxHex5) {
    let mut rgb = [
        values[0] as f32 / 255.0,
        values[1] as f32 / 255.0,
        values[2] as f32 / 255.0,
    ];
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            slider_percent(ui, &mut values[6], 255.0, "Master");

            ui.color_edit_button_rgb(&mut rgb);

            values[0] = (rgb[0] * 255.0) as u8;
            values[1] = (rgb[1] * 255.0) as u8;
            values[2] = (rgb[2] * 255.0) as u8;
        });

        slider_percent(ui, &mut values[3], 255.0, "White");
        slider_percent(ui, &mut values[4], 255.0, "Amber");
        slider_percent(ui, &mut values[5], 255.0, "UV");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source("Strobe")
                .selected_text(inner_data.strobe.name())
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::NoStrobe,
                        PxHex5Strobe::NoStrobe.name(),
                    );
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::Strobe,
                        PxHex5Strobe::Strobe.name(),
                    );
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::StrobePuls,
                        PxHex5Strobe::StrobePuls.name(),
                    );
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::StrobeRandom,
                        PxHex5Strobe::StrobeRandom.name(),
                    );
                });

            if inner_data.strobe != PxHex5Strobe::NoStrobe {
                slider_percent(ui, &mut inner_data.strobe_value, 31.0, "Intensity");

                if ui.button("Strobe").is_pointer_button_down_on() {
                    values[7] = inner_data.to_dmx_value();
                } else {
                    values[7] = 33;
                }
            } else {
                values[7] = 33;
            }
        });
    });
}

/*pub fn px_hex(ui: &mut Ui, values: &mut [u8], inner_data: &mut PxHex5) {
    let mut rgb = [
        values[0] as f32 / 255.0,
        values[1] as f32 / 255.0,
        values[2] as f32 / 255.0,
    ];
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            slider_percent(ui, &mut values[6], 255.0, "Master");

            ui.color_edit_button_rgb(&mut rgb);

            values[0] = (rgb[0] * 255.0) as u8;
            values[1] = (rgb[1] * 255.0) as u8;
            values[2] = (rgb[2] * 255.0) as u8;
        });

        slider_percent(ui, &mut values[3], 255.0, "White");
        slider_percent(ui, &mut values[4], 255.0, "Amber");
        slider_percent(ui, &mut values[5], 255.0, "UV");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source("Strobe")
                .selected_text(inner_data.strobe.name())
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::NoStrobe,
                        PxHex5Strobe::NoStrobe.name(),
                    );
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::Strobe,
                        PxHex5Strobe::Strobe.name(),
                    );
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::StrobePuls,
                        PxHex5Strobe::StrobePuls.name(),
                    );
                    ui.selectable_value(
                        &mut inner_data.strobe,
                        PxHex5Strobe::StrobeRandom,
                        PxHex5Strobe::StrobeRandom.name(),
                    );
                });

            if inner_data.strobe != PxHex5Strobe::NoStrobe {
                slider_percent(ui, &mut inner_data.strobe_value, 31.0, "Intensity");

                if ui.button("Strobe").is_pointer_button_down_on() {
                    values[7] = inner_data.to_dmx_value();
                } else {
                    values[7] = 0;
                }
            } else {
                values[7] = 0;
            }
        });
    });
}*/
