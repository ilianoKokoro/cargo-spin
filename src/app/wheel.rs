use super::constants;
use super::{Choice, WheelChoices};
use eframe::{
    egui::{self, Color32, Context, FontId, Painter, Pos2, Stroke},
    epaint::PathShape,
};
use egui::{epaint::TextShape, Align2};
use egui_modal::Modal;
use rand::Rng;
use std::f32::consts::PI;

pub struct Wheel {
    pub radius: f32,
    pub center: Pos2,
    pub spinning: bool,
    rotation: f32,
    spin_velocity: f32,
    winner: Option<Choice>,
}
struct Point {
    x: f32,
    y: f32,
}

impl Wheel {
    pub fn new() -> Self {
        Self {
            center: Pos2::new(0.0, 0.0),
            radius: 0.0,
            rotation: 0.0,
            spinning: false,
            spin_velocity: 0.0,
            winner: None,
        }
    }

    pub fn clear(&mut self) {
        self.center = Pos2::new(0.0, 0.0);
        self.radius = 0.0;
        self.rotation = 0.0;
        self.spinning = false;
        self.spin_velocity = 0.0;
    }

    pub fn do_spin(&mut self, ctx: &Context, wheel_choices: &mut WheelChoices) {
        let modal = Modal::new(ctx, "winner_modal");
        if !self.spinning {
            modal.show(|ui| {
                modal.frame(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(ui.spacing().interact_size.y * 10.0)
                        .show(ui, |ui: &mut egui::Ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(match &self.winner {
                                    Some(choice) => choice.label.clone(),
                                    None => String::from(""),
                                })
                                .font(FontId::proportional(constants::TITLE_SIZE)),
                            ))
                        })
                });
                modal.buttons(ui, |ui| {
                    if modal.button(ui, "Close").clicked() {
                        self.winner = None;
                    }
                    if modal.button(ui, "Remove the winner").clicked() {
                        if let Some(choice) = &self.winner {
                            wheel_choices.remove_segment(choice.id, self);
                            self.winner = None;
                        }
                    };
                });
            });
        } else {
            self.rotation += self.spin_velocity;
            self.spin_velocity *= constants::BREAKING_PERCENT;
            // Stop
            if self.spin_velocity.abs() < constants::MIN_SPEED {
                self.spinning = false;

                if let Some(choice) = self.get_winner(&wheel_choices) {
                    self.winner = Some(choice);
                    modal.open();
                }
            }
            ctx.request_repaint();
        }
    }

    pub fn start_spin(&mut self) {
        if self.spinning {
            return;
        }
        self.spin_velocity =
            rand::rng().random_range(constants::SPIN_VELOCITY_MIN..constants::SPIN_VELOCITY_MAX);
        self.spinning = true;
    }

    pub fn draw(&mut self, painter: &Painter, wheel_choices: &mut WheelChoices) {
        // Colors
        let colors = [
            Color32::from_rgb(51, 105, 232),
            Color32::from_rgb(213, 15, 37),
            Color32::from_rgb(238, 178, 17),
            Color32::from_rgb(0, 153, 37),
        ];

        // Error message
        if wheel_choices.choices.is_empty() {
            painter.text(
                self.center,
                Align2::CENTER_CENTER,
                "Add options to spin the wheel !",
                FontId::proportional(30.0),
                Color32::WHITE,
            );
            return;
        }

        let total_weight = Wheel::get_total_weight(&wheel_choices);

        let number_of_segments = wheel_choices.choices.len();
        let angle_step = 2.0 * PI / total_weight as f32;

        let mut last_angle: f32 = self.rotation;

        for (i, choice) in wheel_choices.choices.iter_mut().enumerate() {
            let angle_occupied = angle_step * choice.weight as f32;
            // Start and end angle of the current segment
            let start_angle: f32 = last_angle;
            let end_angle = start_angle + angle_occupied;

            last_angle = end_angle;

            // Find the color of the segment
            // (Skip a color to prevent 2 from being next to each-other)
            let mut color_index = i;
            if (number_of_segments % colors.len() == 1) && (i + 1 == number_of_segments) {
                color_index += 1;
            }
            let color: Color32 = colors[color_index % colors.len()];

            // Calculate the points of the segment to draw
            let mut points = vec![self.center];

            let mut side_points: (Point, Point) = (Point::new(), Point::new());

            let actual_steps: u8 =
                ((constants::STEPS / total_weight as u8) * choice.weight as u8) as u8;

            for j in 0..=actual_steps as u8 {
                let t: f32 = j as f32 / actual_steps as f32;
                let angle: f32 = start_angle + t * (end_angle - start_angle);
                let x: f32 = self.center.x + self.radius * angle.cos();
                let y: f32 = self.center.y + self.radius * angle.sin();
                let pos = egui::pos2(x, y);
                points.push(egui::pos2(x, y));
                if j == 0 {
                    side_points.0 = Point::from(pos);
                } else if j == actual_steps {
                    side_points.1 = Point::from(pos);
                }
            }
            let segment_width = side_points.0.distance_to(&side_points.1);

            // Find the center of the segment
            let mut x_sum = 0.0;
            let mut y_sum = 0.0;
            let point_amount = points.len() as f32;

            for point in points.iter() {
                x_sum += point.x;
                y_sum += point.y;
            }
            choice.center = Pos2::new(x_sum / point_amount, y_sum / point_amount);
            points.push(self.center);

            // Draw the segment
            let path = PathShape::convex_polygon(
                points,
                color,
                Stroke::NONE, // Stroke::new(2 as f32, Color32::from_rgb(0, 0, 0)),
            );
            painter.add(path);

            // Draw the text
            let text_angle: f32 = start_angle + angle_occupied / 2.0;
            painter.add(Wheel::create_text_shape(
                choice.label.to_owned(),
                painter,
                text_angle,
                self.radius,
                self.center,
                segment_width,
            ));
        }
    }

    pub fn get_triangle_center(&self) -> Pos2 {
        egui::pos2(self.center.x + self.radius, self.center.y)
    }

    pub fn reset_rotation(&mut self, choices: &Vec<Choice>) {
        self.rotation = PI / choices.len() as f32
    }

    fn get_winner(&self, choices: &WheelChoices) -> Option<Choice> {
        if self.spinning {
            return None;
        }
        let triangle_center = Point::from(self.get_triangle_center());

        let mut winner: Option<Choice> = None;
        let mut min_distance: Option<f32> = None;
        for segment in choices.choices.iter() {
            let new_distance = Point::from(segment.center).distance_to(&triangle_center);

            if min_distance.is_none() || new_distance < min_distance.unwrap() {
                min_distance = Some(new_distance);
                winner = Some(segment.clone()); // Use a reference
            }
        }

        winner
    }

    fn create_text_shape(
        text: String,
        painter: &Painter,
        text_angle: f32,
        wheel_radius: f32,
        text_center: Pos2,
        segment_width: f32,
    ) -> TextShape {
        let actual_label: String = if text.len() > constants::MAX_RANGE_TEXT_LENGTH {
            format!("{}..", text[..constants::MAX_RANGE_TEXT_LENGTH].to_string())
        } else {
            text
        };

        let text_radius: f32 = wheel_radius * 0.6;
        let real_width = if segment_width < 1.0 {
            wheel_radius
        } else {
            segment_width
        };

        let mut galley;
        let mut current_text_size: usize = constants::MAX_TEXT_SIZE;
        let mut text_size;
        loop {
            galley = {
                painter.layout_no_wrap(
                    actual_label.clone(),
                    FontId::proportional(current_text_size as f32),
                    Color32::WHITE,
                )
            };
            text_size = galley.size();
            if (text_size.x <= text_radius && text_size.y <= (real_width * 0.9))
                || current_text_size <= constants::MIN_TEXT_SIZE
            {
                break;
            }
            current_text_size = current_text_size - 1;
        }

        let text_center = Pos2::new(
            text_center.x + text_radius * text_angle.cos(),
            text_center.y + text_radius * text_angle.sin(),
        );

        // Center the text
        let text_offset = Pos2::new(text_size.x / 2.0, text_size.y / 2.0);

        let rotated_offset = Pos2::new(
            text_offset.x * text_angle.cos() - text_offset.y * text_angle.sin(),
            text_offset.x * text_angle.sin() + text_offset.y * text_angle.cos(),
        );

        let centered_point = Pos2::new(
            text_center.x - rotated_offset.x,
            text_center.y - rotated_offset.y,
        );

        TextShape {
            angle: text_angle,
            ..TextShape::new(centered_point, galley, Color32::WHITE)
        }
    }

    fn get_total_weight(choices: &WheelChoices) -> u32 {
        let mut total: u32 = 0;
        for choice in choices.choices.iter() {
            total += choice.weight;
        }
        total
    }
}

impl Point {
    fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn from(pos: Pos2) -> Self {
        Self { x: pos.x, y: pos.y }
    }

    fn distance_to(&self, other: &Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt() as f32
    }
}
