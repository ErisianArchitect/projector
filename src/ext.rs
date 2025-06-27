use eframe::{
    egui::{self, *},
};

use crate::settings::DialogCloser;


pub trait UiExt {
    fn set_size(&mut self, size: Vec2);
    fn rtl_label(&mut self, valign: Align, text: impl Into<WidgetText>) -> Response;
    fn allocate_blank_response(&mut self) -> Response;
    fn debug_rect(&mut self, rect: Rect);
    fn setting_ui<R, F: FnOnce(&mut Ui) -> R>(&mut self, label_width: f32, label: impl Into<WidgetText>, info: impl Into<WidgetText>, color: impl Into<Color32>, add_contents: F) -> InnerResponse<R>;
    fn toggle_box(&mut self, toggle: &mut bool) -> Response;
    fn clicked(&mut self, text: impl Into<WidgetText>) -> bool;
    fn pin_btn(&mut self, size: f32, color: Color32) -> Response;
    fn with_inner_margin<R, F>(&mut self, margin: impl Into<Margin>, add_contents: F) -> InnerResponse<R>
    where F: FnOnce(&mut Ui) -> R;
    fn right_to_left<R, F>(&mut self, align: Align, add_contents: F) -> InnerResponse<R>
    where F: FnOnce(&mut Ui) -> R;
    fn bottom_up<R, F>(&mut self, align: Align, add_contents: F) -> InnerResponse<R>
    where F: FnOnce(&mut Ui) -> R;
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

    fn setting_ui<R, F: FnOnce(&mut Ui) -> R>(&mut self, label_width: f32, label: impl Into<WidgetText>, info: impl Into<WidgetText>, color: impl Into<Color32>, add_contents: F) -> InnerResponse<R> {
        self.horizontal(|ui| {
            ui.spacing_mut().item_spacing = ui.ctx().style().spacing.item_spacing;
            // This must be here because if you put it inside the Frame, the rect is affected by the inner margin.
            let contains_pointer = ui.response().contains_pointer();
            Frame::NONE
            .fill(color.into())
            .inner_margin(Margin::symmetric(4, 4))
            .show(ui, |ui| {
                Frame::NONE
                .show(ui, |ui| {
                    ui.set_width(label_width);
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        let label: WidgetText = label.into();
                        let font_id = ui.style().text_styles[&TextStyle::Body].clone();
                        let label_job = label.into_layout_job(ui.style(), font_id.into(), Align::Min);

                        let label_galley = ui.fonts(|fonts| {
                            fonts.layout_job(label_job)
                        });
                        let label_size = label_galley.size();
                        let int_height = ui.style().spacing.interact_size.y;

                        let (label_rect, _) = ui.allocate_exact_size(vec2(label_size.x, int_height), Sense::empty());

                        ui.painter().galley(label_rect.left_center() - vec2(0.0, label_size.y * 0.5), label_galley, ui.style().visuals.text_color());

                        if contains_pointer {

                            let info_size = Vec2::splat(int_height);
                            let (info_rect, info_resp) = ui.allocate_exact_size(info_size, Sense::hover());
                            let info_color = if info_resp.hovered() {
                                Color32::LIGHT_GRAY
                            } else {
                                Color32::DARK_GRAY
                            };
                            ui.painter().text(info_rect.center(), Align2::CENTER_CENTER, crate::charcons::INFO, FontId::monospace(12.0), info_color);
                            info_resp.on_hover_text(info);
                        }

                    });
                });
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.set_width(ui.available_width());
                    add_contents(ui)
                }).inner
            }).inner
        })
    }

    fn pin_btn(&mut self, size: f32, color: Color32) -> Response {
        let (rect, resp) = self.allocate_exact_size(Vec2::splat(size), Sense::click());
        let p = self.painter();
        p.text(rect.center(), Align2::CENTER_CENTER, crate::charcons::PUSHPIN, FontId::monospace(16.0), color);
        resp
    }

    #[inline]
    fn clicked(&mut self, text: impl Into<WidgetText>) -> bool {
        let text: WidgetText = text.into();
        self.button(text).clicked()
    }

    fn with_inner_margin<R, F>(&mut self, inner_margin: impl Into<Margin>, add_contents: F) -> InnerResponse<R>
    where F: FnOnce(&mut Ui) -> R {
        let margin: Margin = inner_margin.into();
        Frame::NONE
        .inner_margin(margin)
        .show(self, add_contents)
    }

    #[inline]
    fn right_to_left<R, F>(&mut self, align: Align, add_contents: F) -> InnerResponse<R>
    where F: FnOnce(&mut Ui) -> R {
        self.with_layout(Layout::right_to_left(align), add_contents)
    }

    #[inline]
    fn bottom_up<R, F>(&mut self, align: Align, add_contents: F) -> InnerResponse<R>
    where F: FnOnce(&mut Ui) -> R {
        self.with_layout(Layout::bottom_up(align), add_contents)
    }
}

pub trait DirectionExt {
    fn offset(self, dist: f32) -> Vec2;
}

impl DirectionExt for Direction {
    #[inline]
    fn offset(self, dist: f32) -> Vec2 {
        match self {
            Direction::LeftToRight => vec2(dist, 0.0),
            Direction::RightToLeft => vec2(-dist, 0.0),
            Direction::TopDown => vec2(0.0, dist),
            Direction::BottomUp => vec2(0.0, -dist),
        }
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
    fn toggle(&mut self) -> Self;
    fn toggle_if(&mut self, condition: bool) -> Self;
    fn select<T>(self, _false: T, _true: T) -> T;
}

impl BoolExt for bool {
    #[inline]
    fn toggle(&mut self) -> Self {
        *self ^= true;
        *self
    }

    #[inline]
    fn toggle_if(&mut self, condition: bool) -> Self {
        *self ^= condition;
        *self
    }

    #[inline]
    fn select<T>(self, _true: T, _false: T) -> T {
        if self {
            _true
        } else {
            _false
        }
    }
}

pub trait ToSome: Sized {
    fn some(self) -> Option<Self>;
}

impl<T: Sized> ToSome for T {
    #[inline]
    fn some(self) -> Option<Self> {
        Some(self)
    }
}

pub trait ArrayOfOne: Sized {
    fn array_of_one(self) -> [Self; 1];
}

impl<T: Sized> ArrayOfOne for T {
    fn array_of_one(self) -> [Self; 1] {
        [self]
    }
}

pub trait AsSliceOfOne: Sized {
    fn as_slice_of_one<'a>(&'a self) -> &'a [Self];
}

impl<T: Sized> AsSliceOfOne for T {
    fn as_slice_of_one<'a>(&'a self) -> &'a [Self] {
        unsafe {
            std::slice::from_raw_parts(self, 1)
        }
    }
}

pub trait AsSliceOfOneMut: Sized {
    fn as_slice_of_one_mut<'a>(&'a mut self) -> &'a mut [Self];
}

impl<T: Sized> AsSliceOfOneMut for T {
    fn as_slice_of_one_mut<'a>(&'a mut self) -> &'a mut [Self] {
        unsafe {
            std::slice::from_raw_parts_mut(self, 1)
        }
    }
}

pub trait TupleOfOne {
    fn tuple_of_one(self) -> (Self,);
}

impl<T: Sized> TupleOfOne for T {
    fn tuple_of_one(self) -> (Self,) {
        (self,)
    }
}

/// [Replace] allows for in-place replacement of values.
pub trait Replace: Sized {
    fn replace(&mut self, src: Self) -> Self;
}

impl<T: Sized> Replace for T {
    /// Replaces `self` with `src`.
    #[inline(always)]
    fn replace(&mut self, src: Self) -> Self {
        std::mem::replace(self, src)
    }
}

/// [ReplaceWith] allows for in-place replacement of values using a transformer function.
pub trait ReplaceWith: Sized {
    fn replace_with<F: FnOnce(Self) -> Self>(&mut self, replace: F);
}

impl<T: Sized> ReplaceWith for T {
    /// Takes the value and replaces it using a function that takes the value as input and returns the new value.
    #[inline(always)]
    fn replace_with<F: FnOnce(Self) -> Self>(&mut self, replace: F) {
        unsafe {
            std::ptr::write(self, replace(std::ptr::read(self)));
        }
    }
}

pub trait InstantExt {
    fn start() -> Self;
    fn reset(&mut self) -> std::time::Duration;
}

impl InstantExt for std::time::Instant {
    #[inline]
    fn start() -> Self {
        Self::now()
    }

    #[inline]
    fn reset(&mut self) -> std::time::Duration {
        self.replace(Self::now())
            .elapsed()
    }
}