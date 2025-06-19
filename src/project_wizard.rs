use eframe::{
    egui::*,
};

use crate::settings::{DialogCloser, Settings};

pub struct ProjectWizard {

}

impl ProjectWizard {
    pub fn show(
        &mut self,
        mut closer: DialogCloser<'_>,
        settings: &Settings,
        ui: &mut Ui,
    ) {
        // Modal::new(Id::new("project_wizard_modal"))
        //     .area(
        //         Area::new(Id::new("project_wizard_modal_area"))
        //             .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        //             .constrain(true)
        //             .kind(UiKind::Modal)
        //             .order(Order::Foreground)
        //             .
        //     )
    }
}