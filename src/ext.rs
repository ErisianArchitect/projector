use eframe::{
    egui::{self, *},
};

use crate::settings::DialogCloser;


pub trait UiExt {
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response;
    fn allocate_blank_response(&mut self) -> Response;
    fn debug_rect(&mut self, rect: Rect);
}

impl UiExt for Ui {
    #[inline]
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response {
        let text: WidgetText = text.into();
        self.with_layout(Layout::right_to_left(valign), |ui| {
            ui.label(text)
        }).inner
    }

    #[inline]
    fn allocate_blank_response(&mut self) -> Response {
        self.allocate_exact_size(Vec2::ZERO, Sense::empty()).1
    }

    #[inline]
    fn debug_rect(&mut self, rect: Rect) {
        self.painter().rect_stroke(rect, CornerRadius::ZERO, Stroke::new(1.0, Color32::RED), StrokeKind::Inside);
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

pub trait CloserBoolExt {
    fn closer(&mut self) -> DialogCloser<'_>;
}

impl CloserBoolExt for bool {
    #[inline]
    fn closer(&mut self) -> DialogCloser<'_> {
        DialogCloser::new(self)
    }
}

pub trait BoolExt {
    fn toggle(&mut self);
}

impl BoolExt for bool {
    #[inline]
    fn toggle(&mut self) {
        *self = !*self;
    }
}