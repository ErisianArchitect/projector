use eframe::egui::{Layout, TextEdit, Widget};


pub struct PathInput<'a> {
    value: &'a mut String,
}

impl<'a> PathInput<'a> {
    pub fn new(value: &'a mut String) -> Self {
        Self { value }
    }
}

pub struct OptionalPathInput<'a> {
    value: &'a mut Option<String>,
}

impl<'a> OptionalPathInput<'a> {
    pub fn new(value: &'a mut Option<String>) -> Self {
        Self { value }
    }
}

impl<'a> Widget for PathInput<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
            let btn_resp = ui.button("...");
            if btn_resp.clicked() {
                if let Some(result) = rfd::FileDialog::new().pick_file() {
                    *self.value = result.display().to_string();
                }
            }
            let text_input = TextEdit::singleline(self.value)
                .desired_width(ui.available_width())
                .show(ui);
            btn_resp.union(text_input.response)
        }).inner
    }
}