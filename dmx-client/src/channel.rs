use eframe::{
    egui::{PointerButton, Sense, Ui},
    emath,
    epaint::{self, Color32, PathShape, Pos2, Rect, Shape, Stroke, Vec2},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelWidget {
    next_id: i32,
    control_points: Vec<(Pos2, i32)>,

    #[serde(default)]
    pub enabled: bool,

    #[serde(skip)]
    current_point: Pos2,
    #[serde(skip)]
    current_time: f32,
}

impl ChannelWidget {
    pub fn new() -> Self {
        Self {
            next_id: 3,
            control_points: vec![
                (Pos2::new(0.0, 0.0), 0),
                (Pos2::new(100.0, 64.0), 1),
                (Pos2::new(1000.0, 0.0), 2),
            ],
            current_point: Pos2::ZERO,
            current_time: 0.0,
            enabled: false,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(Vec2::new(1000.0, 100.0), Sense::click());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let to_painter = emath::RectTransform::from_to(
            response.rect,
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
        );

        if response.clicked() {
            let pos = response.hover_pos().unwrap();

            self.control_points
                .push((to_painter.transform_pos(pos), self.next_id));
            self.next_id += 1;
        }

        painter.add(epaint::RectShape::stroke(
            response.rect,
            0.0,
            Stroke::new(2.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        ));

        let mut remove_list = Vec::new();

        let control_point_radius = 5.0;
        let points = self.control_points.len();

        let mut first_edited = false;
        let mut last_edited = false;

        for (index, (point, id)) in self.control_points.iter_mut().enumerate() {
            let size = Vec2::splat(2.0 * control_point_radius);

            let point_in_screen = to_screen.transform_pos(*point);
            let point_rect = Rect::from_center_size(point_in_screen, size);
            let point_id = response.id.with(*id);
            let point_response = ui.interact(point_rect, point_id, Sense::click_and_drag());

            let mut delta = point_response.drag_delta();

            if point_response.dragged() {
                if index == 0 {
                    first_edited = true;
                }
                if index == points - 1 {
                    last_edited = true;
                }
            }

            if index == 0 || index == points - 1 {
                delta.x = 0.0;
            } else if point_response.clicked_by(PointerButton::Secondary) {
                remove_list.push(*id);
            }

            *point += delta;
            *point = to_screen.from().clamp(*point);

            let point_in_screen = to_screen.transform_pos(*point);
            let stroke = ui.style().interact(&point_response).fg_stroke;

            painter.add(Shape::circle_stroke(
                point_in_screen,
                control_point_radius,
                stroke,
            ));
        }

        {
            let len: usize = self.control_points.len();

            if first_edited {
                self.control_points[len - 1].0.y = self.control_points[0].0.y;
            }

            if last_edited {
                self.control_points[0].0.y = self.control_points[len - 1].0.y;
            }
        }

        for i in remove_list {
            self.control_points.retain_mut(|(_, j)| i != *j);
        }

        self.control_points
            .sort_by(|a, b| ((a.0.x * 1000.0) as i32).cmp(&((b.0.x * 1000.0) as i32)));

        {
            let len = self.control_points.len();
            self.control_points[0].0.x = 0.0;
            self.control_points[len - 1].0.x = 1000.0;
        }

        let pos: Pos2 = to_screen * self.current_point;

        painter.add(Shape::circle_stroke(
            pos,
            control_point_radius,
            Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
        ));

        {
            let points_in_screen: Vec<Pos2> = self
                .control_points
                .iter()
                .map(|p| to_screen * p.0)
                .collect();
            painter.add(PathShape::line(
                points_in_screen,
                Stroke::new(2.0, Color32::RED.linear_multiply(0.25)),
            ));
        }

        {
            let points_in_screen: Vec<Pos2> = [
                Pos2::new(self.current_time, 0.0),
                Pos2::new(self.current_time, 100.0),
            ]
            .iter()
            .map(|p| to_screen * *p)
            .collect();

            painter.add(PathShape::line(
                points_in_screen,
                Stroke::new(2.0, Color32::WHITE.linear_multiply(0.25)),
            ));
        }
    }

    pub fn update(&mut self, time: f32) -> f32 {
        let time = time % 1000.0;
        self.current_time = time;

        let mut before = Pos2::new(0.0, 100.0);
        let mut after = Pos2::new(1000.0, 100.0);

        for points in self.control_points.windows(2) {
            if points[0].0.x <= time && points[1].0.x > time {
                before = points[0].0;
                after = points[1].0;
            }
        }

        let range = after.x - before.x;
        let pos = time - before.x;

        let ratio = pos / range;

        let x = time;
        let y = before.y * (1.0 - ratio) + after.y * ratio;

        self.current_point = Pos2::new(x, y);

        1.0 - y / 100.0
    }
}
