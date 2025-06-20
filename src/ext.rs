use eframe::{
    egui::{self, *},
};

use crate::settings::DialogCloser;


pub trait UiExt {
    fn set_size(&mut self, size: Vec2);
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response;
    fn allocate_blank_response(&mut self) -> Response;
    fn debug_rect(&mut self, rect: Rect);
    fn setting_ui<R, F: FnOnce(&mut Ui) -> R>(&mut self, label_width: f32, text: impl Into<WidgetText>, info: impl Into<WidgetText>, color: impl Into<Color32>, add_contents: F) -> InnerResponse<R>;
    fn toggle_box(&mut self, toggle: &mut bool) -> Response;
    fn clicked(&mut self, text: impl Into<WidgetText>) -> bool;
}

impl UiExt for Ui {
    #[inline]
    fn set_size(&mut self, size: Vec2) {
        self.set_min_size(size);
        self.set_max_size(size);
    }

    fn toggle_box(&mut self, toggle: &mut bool) -> Response {
        let int_height = self.spacing().interact_size.y;
        let toggle_size = Vec2::splat(int_height);
        let (rect, mut resp) = self.allocate_exact_size(toggle_size, Sense::click());

        if resp.clicked() {
            toggle.toggle();
            resp.mark_changed();
        }
        let style = self.style().visuals.widgets.style(&resp);
        let p = self.painter();
        p.rect(rect, CornerRadius::ZERO, style.bg_fill, style.fg_stroke, StrokeKind::Inside);

        if *toggle {
            let toggle_rect = rect.shrink(4.0);
            let fill_color: Color32 = style.text_color();
            p.rect(toggle_rect, CornerRadius::ZERO, fill_color, style.bg_stroke, StrokeKind::Inside);
        }

        resp
    }

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

    fn setting_ui<R, F: FnOnce(&mut Ui) -> R>(&mut self, label_width: f32, text: impl Into<WidgetText>, info: impl Into<WidgetText>, color: impl Into<Color32>, add_contents: F) -> InnerResponse<R> {
        let resp = self.horizontal(|ui| {
            Frame::NONE
            .inner_margin(Margin::symmetric(0, 4))
            .show(ui, |ui|{
                Frame::NONE
                .show(ui, |ui| {
                    ui.set_width(label_width);
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        ui.label(text);
                        let int_height = ui.style().spacing.interact_size.y;
                        let info_size = Vec2::splat(int_height);
                        let (info_rect, info_resp) = ui.allocate_exact_size(info_size, Sense::hover());
                        let info_color = if info_resp.hovered() {
                            Color32::LIGHT_GRAY
                        } else {
                            Color32::DARK_GRAY
                        };
                        ui.painter().text(info_rect.center(), Align2::CENTER_CENTER, crate::charcons::INFO, FontId::monospace(12.0), info_color);
                        info_resp.on_hover_text(info);
                        // let btn = Button::new(crate::charcons::INFO)
                        //     .frame(false);
                        // let btn_resp = ui.add(btn);
                        // btn_resp
                        //     .on_hover_text(info);
                    });
                });
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.set_width(ui.available_width());
                    add_contents(ui)
                }).inner
            }).inner
        });
        self.painter().rect_filled(resp.response.rect, CornerRadius::ZERO, color);
        resp
    }

    fn clicked(&mut self, text: impl Into<WidgetText>) -> bool {
        let text: WidgetText = text.into();
        self.button(text).clicked()
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