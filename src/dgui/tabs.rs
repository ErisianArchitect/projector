use eframe::egui::{self, *};


#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
    padding: Vec2,
}

impl<'a, T: Copy> Tabs<'a, T> {
    pub const fn new(
        tab_index: &'a mut usize,
        tabs: &'a [Tab<'a, T>],
    ) -> Self {
        Self {
            tab_index,
            tabs,
            size_mode: TabSizeMode::Equal,
            text_align: Align::Min,
            padding: Vec2::new(16.0, 8.0),
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

    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }

    fn draw_titles(&mut self, ui: &mut Ui) -> Response {
        const MONO: FontId = FontId::monospace(16.0);
        let area_rect = ui.available_rect_before_wrap();
        let titles = self.tabs;
        let (
            galleys,
            row_height,
            max_width,
        ) = ui.ctx().fonts(|fonts| {
            let mut max_width = <f32>::MIN;
            let galleys: Vec<_> = titles.iter().map(|title| {
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
        let tabs_size = vec2(area_rect.width(), row_height + self.padding.y * 2.0);
        let (tabs_rect, mut resp) = ui.allocate_exact_size(tabs_size, Sense::empty());
        ui.painter().rect_filled(tabs_rect, CornerRadius::ZERO, Color32::from_gray(30));
        let equal_width = area_rect.width() / titles.len() as f32;
        let mut x = tabs_rect.min.x;
        let mut tab_index_edit = *self.tab_index;
        let tab_index_edit = &mut tab_index_edit;
        galleys.into_iter().enumerate().for_each(|(index, galley)| {
            let tab_width = match self.size_mode {
                TabSizeMode::Equal => max_width + self.padding.x * 2.0,
                TabSizeMode::Shrink => galley.size().x + self.padding.x * 2.0,
                TabSizeMode::Grow => equal_width,
                TabSizeMode::Exact(width) => width,
                TabSizeMode::ShrinkMin(min) => (galley.size().x + self.padding.x * 2.0).max(min),
            };
            let tab_rect = Rect::from_min_size(pos2(x, tabs_rect.min.y), vec2(tab_width, tabs_rect.height()));
            let text_rect = tab_rect.shrink2(self.padding);
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
            let (unselected, fill_color) = if index == *tab_index_edit {
                (false, ui.style().visuals.extreme_bg_color)
            } else {
                (true, style.bg_fill)
            };
            let p = ui.painter_at(tab_rect);
            p.rect_filled(tab_rect, CornerRadius::ZERO, fill_color);
            if unselected {
                p.rect_stroke(tab_rect, CornerRadius::ZERO, style.fg_stroke, StrokeKind::Inside);
            }
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
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
            Frame::new().show(ui, |ui| {
                let spacing = ui.spacing().item_spacing;
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                self.draw_titles(ui);
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    ui.spacing_mut().item_spacing = spacing;
                    ui.set_min_size(ui.available_size());
                    Frame::new()
                        .fill(ui.style().visuals.extreme_bg_color)
                        .corner_radius(CornerRadius::ZERO)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.with_layout(Layout::default(), |ui| {
                                f(*self.tab_index, self.tabs[*self.tab_index].value, ui)
                            }).inner
                        }).inner
                }).inner
            }).inner
        }).inner
    }

}