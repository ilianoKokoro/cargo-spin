mod choice_list;
mod constants;
mod wheel;

use choice_list::ChoiceList;
use eframe::{
    egui::{self, Color32, FontId, Pos2, Stroke},
    epaint::PathShape,
};
use egui::Key;
use wheel::Wheel;

pub struct App {
    wheel: Wheel,
    input_text: String,
    wheel_choices: WheelChoices,
    choices_ui: ChoiceList,
}

#[derive(Debug, Clone)]
struct Choice {
    id: u32,
    label: String,
    center: Pos2,
    weight: u32,
}

struct WheelChoices {
    choices: Vec<Choice>,
    current_id: u32,
}

impl WheelChoices {
    fn new() -> Self {
        Self {
            choices: Vec::new(),
            current_id: 0,
        }
    }

    fn full(&self) -> bool {
        self.choices.len() >= constants::MAX_CHOICES
    }

    fn add_segment(&mut self, label: String, wheel: &mut Wheel) {
        let new_choice = self.create_choice(label);
        self.choices.push(new_choice);
        wheel.reset_rotation(&self.choices);
    }

    fn remove_segment(&mut self, id: u32, wheel: &mut Wheel) {
        let segment_index = self
            .choices
            .iter()
            .position(|segment_found| segment_found.id == id);

        if let Some(index) = segment_index {
            self.choices.remove(index);
        }
        wheel.reset_rotation(&self.choices);
    }

    fn empty(&mut self) -> bool {
        self.choices.is_empty()
    }

    fn create_choice(&mut self, label: String) -> Choice {
        self.current_id = self.current_id + 1;
        Choice::new(label, self.current_id)
    }
    fn rename_choice(&mut self, id: u32, new_name: String) {
        let segment_index = self
            .choices
            .iter()
            .position(|segment_found| segment_found.id == id);

        if let Some(index) = segment_index {
            self.choices[index].label = new_name;
        }
    }
}

impl Choice {
    fn new(label: String, id: u32) -> Self {
        Self {
            id,
            label: label.to_string(),
            center: Pos2::new(0.0, 0.0),
            weight: 1,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            wheel: Wheel::new(),
            input_text: String::new(),
            wheel_choices: WheelChoices::new(),
            choices_ui: ChoiceList::new(),
        }
    }
}

impl App {
    fn can_add_segment(&self) -> bool {
        self.can_type_segment() && !self.input_text.is_empty()
    }

    fn can_type_segment(&self) -> bool {
        !self.wheel.spinning && !self.wheel_choices.full()
    }

    fn add_segment_ui(&mut self) {
        if self.can_add_segment() {
            self.wheel_choices
                .add_segment(self.input_text.trim().replace("\n", " "), &mut self.wheel);
            self.input_text.clear();
        }
    }
}

// Main loop
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let available_rect = ui.max_rect();
            let painter = ui.painter();

            // When wheel is spinning
            self.wheel.do_spin(ctx, &mut self.wheel_choices);

            // Wheel
            self.wheel.center = egui::pos2(
                available_rect.width() * 0.25 + constants::WHEEL_OFFSET,
                available_rect.center().y,
            );

            let available_width = available_rect.width() / 4.0;
            let available_height = available_rect.height() / 2.0;
            self.wheel.radius = f32::min(available_width, available_height);

            self.wheel.draw(painter, &mut self.wheel_choices);

            // Triangle
            if !self.wheel_choices.empty() {
                let triangle_center = self.wheel.get_triangle_center();
                let triangle_points: Vec<Pos2> = vec![
                    egui::pos2(triangle_center.x - 15.0, triangle_center.y),
                    egui::pos2(triangle_center.x + 30.0, triangle_center.y + 20.0),
                    egui::pos2(triangle_center.x + 30.0, triangle_center.y - 20.0),
                ];
                let path = PathShape::convex_polygon(
                    triangle_points,
                    Color32::from_rgb(200, 200, 200),
                    Stroke::NONE,
                );
                painter.add(path);
            }

            // Inputs
            let inputs_center = egui::pos2(
                available_rect.width() * 0.75 + constants::WHEEL_OFFSET,
                available_rect.center().y,
            );

            let inputs_width = available_rect.width() / 2.0 * 0.9 - constants::WHEEL_OFFSET;
            let inputs_height = available_rect.height() * 0.9;

            let inputs_pos = egui::pos2(
                inputs_center.x - inputs_width / 2.0,
                inputs_center.y - inputs_height / 2.0,
            );
            // Add input UI to the right side of the container
            egui::Area::new(egui::Id::new("inputs"))
                .fixed_pos(inputs_pos)
                .show(ctx, |ui| {
                    ui.set_height(inputs_height);
                    ui.set_width(inputs_width);

                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        // Add

                        ui.horizontal(|ui| {
                            let button_width: f32 = ui.spacing().interact_size.x;
                            let input_width: f32 = ui.available_width() - button_width;

                            egui::ScrollArea::vertical()
                                .max_height(ui.spacing().interact_size.y * 3.0)
                                .show(ui, |ui: &mut egui::Ui| {
                                    let can_type = self.can_type_segment();
                                    ui.add_enabled(
                                        can_type,
                                        egui::TextEdit::multiline(&mut self.input_text)
                                            .hint_text(if can_type {
                                                "Add a choice".to_owned()
                                            } else {
                                                format!(
                                                    "Max amount of choices reached : {}",
                                                    constants::MAX_CHOICES
                                                )
                                            })
                                            .char_limit(constants::MAX_INPUT_SIZE)
                                            .desired_width(input_width)
                                            .desired_rows(1),
                                    );
                                    if ctx.input(|i| i.key_pressed(Key::Enter)) {
                                        self.add_segment_ui();
                                    }
                                });

                            if ui
                                .add_enabled(self.can_add_segment(), egui::Button::new("Add"))
                                .clicked()
                            {
                                self.add_segment_ui();
                            }
                        });

                        ui.add_space(constants::SPACER_AMOUNT);

                        // Choices
                        self.choices_ui
                            .draw(ui, &ctx, &mut self.wheel_choices, &mut self.wheel);

                        ui.add_space(60.0);
                        // Spin button
                        if ui
                            .add_enabled(
                                !self.wheel.spinning && !self.wheel_choices.empty(),
                                egui::Button::new(
                                    egui::RichText::new("Spin the Wheel !")
                                        .font(FontId::proportional(constants::TITLE_SIZE)),
                                ),
                            )
                            .clicked()
                        {
                            self.wheel.start_spin();
                        }
                        ui.add_space(constants::SPACER_AMOUNT);

                        // Clear
                        if ui
                            .add(egui::Button::new(
                                egui::RichText::new("Clear the wheel")
                                    .font(FontId::proportional(constants::TITLE_SIZE / 2.0)),
                            ))
                            .clicked()
                        {
                            self.wheel_choices.choices = vec![];
                            self.wheel.clear();
                        }
                    });
                });
        });
    }
}
