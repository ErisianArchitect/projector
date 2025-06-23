
use eframe::{
    egui::*,
};
use crate::projects::{ProjectPath, ProjectType};


/// Not to be confused with [Recents].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Recent<'a> {
    path: &'a ProjectPath,
}

impl<'a> Recent<'a> {

    #[inline]
    pub const fn new(path: &'a ProjectPath) -> Self {
        Self { path }
    }

    pub fn ui(&self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        const HEIGHT: f32 = 32.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(width, HEIGHT), Sense::click());
        let style = ui.style().visuals.widgets.style(&resp);
        let p = ui.painter().with_clip_rect(rect);
        p.rect(rect, CornerRadius::ZERO, style.bg_fill, style.bg_stroke, StrokeKind::Inside);

        let left_rect = Rect::from_min_max(
            rect.min,
            pos2(rect.right() - 120.0, rect.max.y),
        );
        let right_rect = Rect::from_min_max(
            left_rect.right_top(),
            rect.max,
        );

        
        let name_rect = left_rect.shrink(4.0);
        let type_rect = right_rect.shrink(4.0);
        
        let (path, type_name, type_color) = match self.path {
            ProjectPath::Rust(path_buf) => (path_buf, "Rust", Color32::from_rgb(185, 71, 0)),
            ProjectPath::Python(path_buf) => (path_buf, "Python", Color32::from_rgb(53, 113, 163)),
            ProjectPath::Web(path_buf) => (path_buf, "Web", Color32::from_rgb(0, 190, 255)),
            ProjectPath::Other(path_buf) => (path_buf, "Other", Color32::from_rgb(255, 220, 196)),
        };

        p.rect(right_rect, CornerRadius::ZERO, type_color, style.bg_stroke, StrokeKind::Inside);
        
        let text_p = p.with_clip_rect(name_rect);
        let path_name = if let Some(path_name) = path.file_name() {
            path_name.to_str().unwrap_or("")
        } else {
            ""
        };
        text_p.text(name_rect.left_center(), Align2::LEFT_CENTER, path_name, FontId::monospace(16.0), Color32::WHITE);

        let type_p = p.with_clip_rect(type_rect);
        type_p.text(type_rect.center(), Align2::CENTER_CENTER, type_name, FontId::monospace(16.0), Color32::BLACK);

        resp
    }
}

// Hmm. What do I need for this?
// I need there to be a list that has all of the recents
// Then I also need another list for the recents that are to be displayed (controlled by a filter/order)
// I need the ordered list to know the index in the original list so that removals can happen
// When updates (such as removals or refreshes) happen, the display list must be refreshed.
/// Not to be confused with [Recent].
pub struct Recents {
    recents: Vec<ProjectPath>,
    order: Vec<u16>,
}

impl Recents {
    pub fn new(recents: Vec<ProjectPath>) -> Self {
        Self {
            order: (0..recents.len()).map(|_| 0u16).collect(),
            recents,
        }
    }
}

/*
group_by month {
    order: [Rust, Python, Web, Other] {
        group_by type {
            order_by name {
                group_by directory {
                    order_by name {

                    }
                }
            }
        }
    }
}
group_by type {
    group_by month {

    }
}
*/

// I want to be able to order the Recents in various ways:
// - Alphabetically Ascending/Descending
// - Time, Most/Least Recently Opened
// In addition to being able to order them, I also want to be able to group them:
// - By Day/Month/Year
// - Project Type (Rust, Python, Web, Other) (and order the project types as well)
// - Parent Directory
// 

// pub struct RecentsOrder {
//     rust: i8,
//     python: i8,
//     web: i8,
//     other: i8,
// }

// impl RecentsOrder {
//     pub const DEFAULT: RecentsOrder = RecentsOrder {
//         rust: 0,
//         python: 1,
//         web: 2,
//         other: 3,
//     };
// }