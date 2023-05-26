use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use eframe::egui::color_picker::color_edit_button_srgb;
use eframe::egui::{DragValue, Sense, Widget};
use eframe::epaint::{self, Color32, PathShape, Pos2, Rect, Shape, Stroke, Vec2};
use eframe::{egui, emath};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Voysys DMX controller",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

fn tcp_thread(rx: Receiver<DmxColor>, run: Arc<AtomicBool>) {
    match TcpStream::connect("127.0.0.1:33333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 33333");
            while run.load(Ordering::SeqCst) {
                if let Ok(msg) = rx.recv_timeout(Duration::from_millis(10)) {
                    let data = [
                        msg.rgb[0], msg.rgb[1], msg.rgb[2], msg.white, msg.amber, msg.uv,
                    ];

                    stream.write_all(&data).unwrap();
                }
            }
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Closing TCP Thread.");
}

#[derive(Debug, Default, Clone)]
struct DmxColor {
    rgb: [u8; 3],
    white: u8,
    amber: u8,
    uv: u8,
}

struct MyApp {
    colors: DmxColor,
    tcp_thread: Option<JoinHandle<()>>,
    run: Arc<AtomicBool>,
    tx: Sender<DmxColor>,

    next_id: i32,
    control_points: Vec<(Pos2, i32)>,

    last_frame_time: Instant,
    time: f32,
    speed: f32,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let run = Arc::new(AtomicBool::new(true));

        let tcp_thread = {
            let run = run.clone();
            Some(thread::spawn(move || tcp_thread(rx, run)))
        };

        Self {
            colors: DmxColor::default(),
            tcp_thread,
            tx,
            run,
            next_id: 3,
            control_points: vec![
                (Pos2::new(0.0, 0.0), 0),
                (Pos2::new(100.0, 64.0), 1),
                (Pos2::new(1000.0, 0.0), 2),
            ],
            last_frame_time: Instant::now(),
            time: 0.0,
            speed: 10.0,
        }
    }
}

impl eframe::App for MyApp {
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

        self.time += self.speed * dt;

        if self.time > 1000.0 {
            self.time = 0.0;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            DragValue::new(&mut self.speed).speed(0.01).ui(ui);

            let mut send = false;
            send |= color_edit_button_srgb(ui, &mut self.colors.rgb).changed();
            send |= ui
                .add(egui::Slider::new(&mut self.colors.white, 0..=255).text("White"))
                .changed();
            send |= ui
                .add(egui::Slider::new(&mut self.colors.amber, 0..=255).text("Amber"))
                .changed();
            send |= ui
                .add(egui::Slider::new(&mut self.colors.uv, 0..=255).text("UV"))
                .changed();

            if ui.button("Add Point").clicked() {
                self.control_points
                    .push((Pos2::new(10.0, 0.0), self.next_id));
                self.next_id += 1;
            }

            let (response, painter) = ui.allocate_painter(Vec2::new(1000.0, 100.0), Sense::hover());

            painter.add(epaint::RectShape::stroke(
                response.rect,
                0.0,
                Stroke::new(2.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
            ));

            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                response.rect,
            );

            let control_point_radius = 5.0;

            let control_point_shapes: Vec<Shape> = self
                .control_points
                .iter_mut()
                .map(|(point, i)| {
                    let size = Vec2::splat(2.0 * control_point_radius);

                    let point_in_screen = to_screen.transform_pos(*point);
                    let point_rect = Rect::from_center_size(point_in_screen, size);
                    let point_id = response.id.with(*i);
                    let point_response = ui.interact(point_rect, point_id, Sense::drag());

                    *point += point_response.drag_delta();
                    *point = to_screen.from().clamp(*point);

                    let point_in_screen = to_screen.transform_pos(*point);
                    let stroke = ui.style().interact(&point_response).fg_stroke;

                    Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
                })
                .collect();

            self.control_points
                .sort_by(|a, b| ((a.0.x * 1000.0) as i32).cmp(&((b.0.x * 1000.0) as i32)));

            {
                let mut before = Pos2::new(0.0, 100.0);
                let mut after = Pos2::new(1000.0, 100.0);

                for points in self.control_points.windows(2) {
                    if points[0].0.x < self.time && points[1].0.x > self.time {
                        before = points[0].0;
                        after = points[1].0;
                    }  
                }

                println!(
                    "{:?} {:?}, {:?}: {:?}",
                    before, after, &self.control_points, self.time
                );
            }

            {
                let points_in_screen: Vec<Pos2> = self
                    .control_points
                    .iter()
                    .map(|p| to_screen * p.0)
                    .collect();
                painter.add(PathShape::line(
                    points_in_screen,
                    Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
                ));
            }

            {
                let points_in_screen: Vec<Pos2> =
                    [Pos2::new(self.time, 0.0), Pos2::new(self.time, 100.0)]
                        .iter()
                        .map(|p| to_screen * *p)
                        .collect();

                painter.add(PathShape::line(
                    points_in_screen,
                    Stroke::new(2.0, Color32::WHITE.linear_multiply(0.25)),
                ));
            }

            painter.extend(control_point_shapes);

            if send {
                self.tx.send(self.colors.clone()).ok();
            }
        });
    }
}

impl Drop for MyApp {
    fn drop(&mut self) {
        self.run.store(false, Ordering::SeqCst);
        if let Some(thread) = self.tcp_thread.take() {
            thread.join().ok();
        }
    }
}
