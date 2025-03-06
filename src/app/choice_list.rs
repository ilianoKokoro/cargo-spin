use egui::{Context, Frame, Label, RichText, Rounding};
use egui_modal::Modal;

use crate::app::constants;

use super::{wheel::Wheel, Choice, WheelChoices};

pub struct ChoiceList {
    choice_to_rename: Option<Choice>,
    rename_input: String,
}
impl ChoiceList {
    pub fn new() -> Self {
        Self {
            choice_to_rename: None,
            rename_input: String::new(),
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &Context,
        wheel_choices: &mut WheelChoices,
        wheel: &mut Wheel,
    ) {
        let enabled = !wheel.spinning;
        let modal = Modal::new(ctx, "my_dialog");

        modal.show(|ui| {
            modal.title(ui, "Edit this choice");
            modal.frame(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.spacing().interact_size.y * 3.0)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.rename_input)
                                .hint_text("Rename the choice")
                                .char_limit(constants::MAX_INPUT_SIZE)
                                .desired_rows(1),
                        )
                    })
            });

            modal.buttons(ui, |ui| {
                modal.button(ui, "Cancel");
                if modal.button(ui, "Confirm").clicked() {
                    if let Some(choice) = &self.choice_to_rename {
                        wheel_choices.rename_choice(choice.id, self.rename_input.clone());
                    }
                }
            });
        });

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() * 0.75)
            .show(ui, |ui| {
                let mut choice_to_remove: Option<Choice> = None;

                let buttons_width: f32 = ui.spacing().interact_size.x * 4.0;
                let available_width: f32 = ui.available_width() - buttons_width;

                for choice in wheel_choices.choices.iter_mut() {
                    ui.horizontal(|ui| {
                        Frame::default()
                            .fill(ui.style().visuals.widgets.active.bg_fill)
                            .rounding(Rounding::same(4.0))
                            .show(ui, |ui| {
                                ui.add_sized(
                                    [available_width, ui.spacing().interact_size.y],
                                    Label::new(
                                        RichText::new(&choice.label)
                                            .color(ui.style().visuals.widgets.active.text_color()),
                                    )
                                    .truncate(),
                                );
                            });

                        ui.add(Label::new(format!("Weight : {}", choice.weight)));

                        if ui
                            .add_enabled(
                                enabled && choice.weight < constants::MAX_SEGMENT_WEIGHT,
                                egui::Button::new("+"),
                            )
                            .clicked()
                        {
                            choice.weight += 1;
                        }

                        if ui
                            .add_enabled(enabled && choice.weight > 1, egui::Button::new("-"))
                            .clicked()
                        {
                            choice.weight -= 1;
                        }

                        if ui.add_enabled(enabled, egui::Button::new("✏")).clicked() {
                            self.choice_to_rename = Some(choice.clone());
                            self.rename_input = choice.label.clone();
                            modal.open();
                        }

                        if ui.add_enabled(enabled, egui::Button::new("🗑")).clicked() {
                            choice_to_remove = Some(choice.clone());
                        }
                    });
                }

                if let Some(choice) = choice_to_remove {
                    wheel_choices.remove_segment(choice.id, wheel);
                }
            });
    }
}
