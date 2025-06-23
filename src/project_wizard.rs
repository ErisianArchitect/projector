use eframe::{
    egui::*,
};

use crate::{ext::UiExt, settings::{DialogCloser, Settings}};

pub struct ProjectWizard {

}

impl ProjectWizard {
    pub fn show(
        &mut self,
        mut closer: DialogCloser<'_>,
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
                    .inner_margin(Margin::same(16))
            )
            .show(ui.ctx(), move |ui| {
                ui.set_size(vec2(700.0, 700.0));
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
    }
}