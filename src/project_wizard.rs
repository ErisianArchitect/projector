use eframe::{
    egui::*,
};

use crate::{ext::UiExt, settings::{Closer, DialogCloser, Settings}};

pub struct ProjectWizard {

}

impl ProjectWizard {
    pub fn show(
        &mut self,
        closer: Closer<'_>,
        settings: &Settings,
        ui: &mut Ui,
    ) {
        Modal::new(Id::new("project_wizard_modal"))
            .area(
                Area::new(Id::new("project_wizard_modal_area"))
                    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                    .constrain(true)
                    .kind(UiKind::Modal)
                    .order(Order::Foreground)
            )
            .frame(
                Frame::NONE
                    .fill(ui.style().visuals.window_fill)
            )
            .show(ui.ctx(), move |ui| {
                ui.set_size(vec2(700.0, 700.0));
                ui.bottom_up(Align::Min, |ui| {
                    ui.with_inner_margin(Margin::same(8), |ui| {
                        // let inner_rect = ui.response().rect + Margin::same(8);
                        // ui.painter().rect_filled(inner_rect, CornerRadius::ZERO, Color32::LIGHT_GRAY);
                        menu::bar(ui, |ui| {
                            ui.right_to_left(Align::Center, |ui| {
                                ui.label("💾🕖 7:28 PM 6/27/2025");
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    if ui.button("Close").clicked() {
                                        closer.close();
                                    }
                                    if ui.button("Save").clicked() {

                                    }
                                    if ui.button("Save and Close").clicked() {

                                    }
                                    if ui.button("Discard Changes").clicked() {

                                    }
                                    ui.separator();
                                    ui.label("Modified");
                                });
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.with_inner_margin(Margin::same(8), |ui| {
                            ui.setting_ui(
                                180.0,
                                "Test",
                                "This is a test",
                                Color32::TRANSPARENT,
                                |ui| {
                                    if ui.clicked("Click me!") {
                                        println!("Clicked.");
                                    }
                                }
                            );
                            if ui.clicked("Close") {
                                closer.close();
                            }
                        });
                    });
                });
            });
    }
}