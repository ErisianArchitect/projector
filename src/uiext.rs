use eframe::{
    egui::{self, *},
};


pub trait UiExt {
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response;
}

impl UiExt for Ui {
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response {
        self.with_layout(Layout::right_to_left(valign), |ui| {
            ui.label(text)
        }).inner
    }
}