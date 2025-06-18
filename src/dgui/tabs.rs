use eframe::{egui::{self, *}, glow::ZERO};

use crate::eguiext::UiExt;


#[derive(Debug, Default, Clone, Copy, PartialEq, bincode::Decode, bincode::Encode)]
pub enum TabSizeMode {
    /// All tabs are equal sized, based on the largest required size.
    #[default]
    Equal,
    /// Tabs shrink to fit their content.
    Shrink,
    /// Tabs grow so that all tabs fill the available width with equal sizing.
    Grow,
    /// All tabs are the exact size.
    Exact(f32),
    /// Tabs will shrink to the minimum value at most.
    ShrinkMin(f32),
}

#[derive(Debug)]
pub struct Tab<'a, T: Copy> {
    value: T,
    title: &'a str,
}

impl<'a, T: Copy> Tab<'a, T> {
    pub const fn new(title: &'a str, value: T) -> Self {
        Self {
            value,
            title,
        }
    }

    pub const fn copy_value(&self) -> T {
        self.value
    }

    pub const fn title(&self) -> &str {
        self.title
    }
}

#[derive(Debug)]
pub struct Tabs<'a, T: Copy> {
    tab_index: &'a mut usize,
    tabs: &'a [Tab<'a, T>],
    size_mode: TabSizeMode,
    text_align: Align,
    title_padding: Vec2,
    padding: Margin,
}

impl<'a, T: Copy> Tabs<'a, T> {
    pub fn new(
        tab_index: &'a mut usize,
        tabs: &'a [Tab<'a, T>],
    ) -> Self {
        Self {
            tab_index,
            tabs,
            size_mode: TabSizeMode::Equal,
            text_align: Align::Min,
            title_padding: Vec2::new(16.0, 8.0),
            padding: Margin::from(vec2(0.0, 0.0)),
        }
    }

    pub fn with_size_mode(mut self, size_mode: TabSizeMode) -> Self {
        self.size_mode = size_mode;
        self
    }

    pub fn with_text_align(mut self, align: Align) -> Self {
        self.text_align = align;
        self
    }

    pub fn with_title_padding(mut self, padding: Vec2) -> Self {
        self.title_padding = padding;
        self
    }

    pub fn with_padding(mut self, padding: impl Into<Margin>) -> Self {
        self.padding = padding.into();
        self
    }

    fn draw_titles(&mut self, ui: &mut Ui) -> Response {
        const MONO: FontId = FontId::monospace(16.0);
        let avail_rect = ui.available_rect_before_wrap();
        // let avail_width = avail_rect.width();
        let avail_width = avail_rect.width();
        let alloc_width = avail_width;
        // ui.set_max_width(alloc_width);
        let tabs = self.tabs;
        let (
            galleys,
            row_height,
            max_width,
        ) = ui.ctx().fonts(|fonts| {
            let mut max_width = <f32>::MIN;
            let galleys: Vec<_> = tabs.iter().map(|title| {
                let galley = fonts.layout_no_wrap(title.title.into(), MONO, Color32::WHITE);
                max_width = max_width.max(galley.size().x);
                galley
            }).collect();
            (
                galleys,
                fonts.row_height(&MONO),
                max_width,
            )
        });
        let tab_bar_size = vec2(alloc_width, row_height + self.title_padding.y * 2.0);
        // This is where the space for the entire tab bar is allocated in the Ui.
        let (tab_bar_rect, mut resp) = ui.allocate_exact_size(tab_bar_size, Sense::empty());
        // tab_bar_rect.set_width(avail_width);
        ui.painter().rect_filled(tab_bar_rect, CornerRadius::ZERO, Color32::from_gray(30));
        let mut x = tab_bar_rect.min.x;
        let mut tab_index_edit = *self.tab_index;
        let tab_index_edit = &mut tab_index_edit;
        galleys.into_iter().enumerate().for_each(|(index, galley)| {
            let tab_width = match self.size_mode {
                TabSizeMode::Equal => max_width + self.title_padding.x * 2.0,
                TabSizeMode::Shrink => galley.size().x + self.title_padding.x * 2.0,
                TabSizeMode::Grow => alloc_width / tabs.len() as f32,
                TabSizeMode::Exact(width) => width,
                TabSizeMode::ShrinkMin(min) => (galley.size().x + self.title_padding.x * 2.0).max(min),
            };
            let tab_rect = Rect::from_min_size(pos2(x, tab_bar_rect.min.y), vec2(tab_width, tab_bar_rect.height()));
            let text_rect = tab_rect.shrink2(self.title_padding);
            let align = self.text_align;
            let text_pos = match align {
                Align::Min => {
                    text_rect.min
                },
                Align::Center => {
                    let half_width = galley.size().x * 0.5;
                    text_rect.center_top() - vec2(half_width, 0.0)
                },
                Align::Max => {
                    text_rect.right_top() - vec2(galley.size().x, 0.0)
                },
            }; 
            
            let resp = ui.allocate_rect(tab_rect, Sense::click());
            if resp.clicked() {
                *tab_index_edit = index;
            }
            let style = ui.style().visuals.widgets.style(&resp);
            if (galley.size().x + style.fg_stroke.width * 2.0) > tab_width {
                resp.on_hover_text(galley.text());
            }
            // color for selected/unselected tabs.
            let (unselected, fill_color) = if index == *tab_index_edit {
                (false, ui.style().visuals.extreme_bg_color)
            } else {
                (true, style.bg_fill)
            };

            let p = ui.painter_at(tab_rect);
            // paint tab fill color
            p.rect_filled(tab_rect, CornerRadius::ZERO, fill_color);
            if unselected {
                // paint tab stroke only when tab is unselected.
                p.rect_stroke(tab_rect, CornerRadius::ZERO, style.fg_stroke, StrokeKind::Inside);
            } else {
                let select_rect = Rect::from_min_size(tab_rect.min, vec2(tab_rect.width(), 2.0));
                p.rect_filled(select_rect, CornerRadius::ZERO, Color32::from_rgb(68, 166, 198)); //rgb(68,166,198)
            }
            // paint tab title.
            p.galley(text_pos, galley, style.text_color());

            x += tab_width;
        });
        if *tab_index_edit != *self.tab_index {
            *self.tab_index = *tab_index_edit;
            resp.mark_changed();
        }
        resp
    }

    pub fn show<R, F: FnOnce(usize, T, &mut Ui) -> R>(&mut self, ui: &mut Ui, f: F) -> R {
        let avail_rect = ui.available_rect_before_wrap();
        let max_width = avail_rect.width();
        ui.set_max_width(max_width);
        let result = ui.allocate_new_ui(
            UiBuilder::new()
                .layout(Layout::top_down(Align::Min))
                // .max_rect(avail_rect)
                .sizing_pass(),
            |ui| {
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.set_min_width(max_width);
                ui.set_max_width(max_width);
                // let spacing = ui.spacing().item_spacing;
                // ui.spacing_mut().item_spacing = Vec2::ZERO;
                // ui.spacing_mut().window_margin = Margin::ZERO;
                Frame::NONE.show(ui, |ui| {
                    let remaining_rect = ui.available_rect_before_wrap();
                    ui.allocate_new_ui(UiBuilder::new().max_rect(remaining_rect).layout(Layout::default()), |ui| {
                        // ui.spacing_mut().item_spacing = spacing;
                        self.draw_titles(ui);
                        Frame::NONE
                        .fill(ui.style().visuals.extreme_bg_color)
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = spacing;
                            ui.set_min_width(ui.available_width());
                            ui.set_max_width(ui.available_width());
                            f(*self.tab_index, self.tabs[*self.tab_index].value, ui)
                        }).inner
                    }).inner
                }).inner
            }).inner;
        result
    }
}