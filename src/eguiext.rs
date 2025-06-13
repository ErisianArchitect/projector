use eframe::{
    egui::{self, *},
};


pub trait UiExt {
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response;
    fn allocate_blank_response(&mut self) -> Response;
}

impl UiExt for Ui {
    #[inline]
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response {
        self.with_layout(Layout::right_to_left(valign), |ui| {
            ui.label(text)
        }).inner
    }

    #[inline]
    fn allocate_blank_response(&mut self) -> Response {
        self.allocate_exact_size(Vec2::ZERO, Sense::empty()).1
    }
}

pub trait ResponseExt {
    fn merge(&mut self, response: Response);
}

impl ResponseExt for Response {
    fn merge(&mut self, response: Response) {
        *self = self.union(response)
    }
}