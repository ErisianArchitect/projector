
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use chrono::Timelike;
use eframe::{
    egui::*,
};
use crate::projects::ProjectPath;

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum Order {
    #[default]
    Ascending = 0,
    Descending = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum Recency {
    Most,
    Least,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum RecentsSort {
    #[default]
    Default,
    NameAscending,
    NameDescending,
    MostRecent,
    LeastRecent,
}

impl RecentsSort {
    pub fn default_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
        let lhs = lhs.0;
        let rhs = rhs.0;
        lhs.cmp(&rhs)
    }

    pub fn ascending_name_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
        let lhs = lhs.1.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        let rhs = rhs.1.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        lhs.cmp(rhs)
    }

    pub fn descending_name_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
        let lhs = lhs.1.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        let rhs = rhs.1.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        rhs.cmp(lhs)
    }

    pub fn most_recent_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
        let lhs = &lhs.1.time;
        let rhs = &rhs.1.time;
        rhs.cmp(lhs)
    }

    pub fn least_recent_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
        let lhs = &lhs.1.time;
        let rhs = &rhs.1.time;
        lhs.cmp(rhs)
    }

    pub fn sort_by_fn(self) -> fn((usize, &RecentEntry), (usize, &RecentEntry)) -> Ordering {
        match self {
            RecentsSort::Default => Self::default_sort,
            RecentsSort::NameAscending => Self::ascending_name_sort,
            RecentsSort::NameDescending => Self::descending_name_sort,
            RecentsSort::MostRecent => Self::most_recent_sort,
            RecentsSort::LeastRecent => Self::least_recent_sort,
        }
    }

    pub fn sort(self, recents: &[RecentEntry], order: &mut [u16]) {
        let sort_by = self.sort_by_fn();
        order.sort_by(move |&lhs, &rhs| {
            let l_index = lhs as usize;
            let r_index = rhs as usize;
            let l_obj = &recents[l_index];
            let r_obj = &recents[r_index];
            let lhs = (l_index, l_obj);
            let rhs = (r_index, r_obj);
            sort_by(lhs, rhs)
        });
    }
}

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

pub struct RecentEntryTimeCurry {
    time: chrono::DateTime<chrono::Utc>,
}

impl RecentEntryTimeCurry {
    #[must_use]
    #[inline]
    pub fn now() -> Self {
        Self {
            time: chrono::Utc::now(),
        }
    }

    #[inline]
    pub fn with(self, path: ProjectPath) -> RecentEntry {
        RecentEntry {
            path,
            time: self.time,
        }
    }

    #[inline]
    pub fn rust<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Rust(path.into()),
            time: self.time,
        }
    }

    #[inline]
    pub fn python<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Python(path.into()),
            time: self.time,
        }
    }

    #[inline]
    pub fn web<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Python(path.into()),
            time: self.time,
        }
    }

    #[inline]
    pub fn other<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Other(path.into()),
            time: self.time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecentEntry {
    path: ProjectPath,
    time: chrono::DateTime<chrono::Utc>,
}

impl RecentEntry {
    #[must_use]
    #[inline]
    pub fn now_curry() -> RecentEntryTimeCurry {
        RecentEntryTimeCurry::now()
    }

    #[must_use]
    #[inline]
    pub fn now(path: ProjectPath) -> Self {
        Self {
            path,
            time: chrono::Utc::now(),
        }
    }

    #[must_use]
    #[inline]
    pub fn new(path: ProjectPath, time: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            path,
            time,
        }
    }
}

impl bincode::Encode for RecentEntry {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.path.encode(encoder)?;
        // let time: SystemTime = self.time.into();
        // time.encode(encoder)
        let seconds = self.time.timestamp();
        let nsecs = self.time.timestamp_subsec_nanos();
        seconds.encode(encoder)?;
        nsecs.encode(encoder)
    }
}

impl<Ctx> bincode::Decode<Ctx> for RecentEntry {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let path = ProjectPath::decode(decoder)?;
        let seconds = i64::decode(decoder)?;
        let nsecs = u32::decode(decoder)?;
        Ok(Self {
            path,
            time: chrono::DateTime::from_timestamp(seconds, nsecs).unwrap_or_default(),
        })
    }
}

// Hmm. What do I need for this?
// I need there to be a list that has all of the recents
// Then I also need another list for the recents that are to be displayed (controlled by a filter/order)
// I need the ordered list to know the index in the original list so that removals can happen
// When updates (such as removals or refreshes) happen, the display list must be refreshed.
/// Not to be confused with [Recent].
pub struct Recents {
    recents: Vec<RecentEntry>,
    order: Vec<u16>,
    sort: RecentsSort,
}

impl Recents {
    pub fn new(recents: Vec<RecentEntry>) -> Self {
        Self {
            order: (0..recents.len()).map(|i| i as u16).collect(),
            recents,
            sort: RecentsSort::Default,
        }
    }

    pub fn set_sort(&mut self, new_sort: RecentsSort) {
        let Self {
            recents,
            order,
            sort,
        } = self;
        *sort = new_sort;
        new_sort.sort(recents, order);
    }

    pub fn order_by_name(&mut self, sort: Order) {
        let sort = match sort {
            Order::Ascending => RecentsSort::NameAscending,
            Order::Descending => RecentsSort::NameDescending,
        };
        self.set_sort(sort);
    }

    pub fn order_by_time(&mut self, recency: Recency) {
        let sort = match recency {
            Recency::Most => RecentsSort::MostRecent,
            Recency::Least => RecentsSort::LeastRecent,
        };
        self.set_sort(sort);
    }
}

impl std::ops::Index<usize> for Recents {
    type Output = RecentEntry;

    fn index(&self, index: usize) -> &Self::Output {
        let entry_index = self.order[index] as usize;
        &self.recents[entry_index]
    }
}

impl bincode::Encode for Recents {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.recents.encode(encoder)
    }
}

impl<Ctx> bincode::Decode<Ctx> for Recents {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let recents = Vec::<RecentEntry>::decode(decoder)?;
        let order = (0..recents.len()).map(|i| i as u16).collect::<Vec<_>>();
        Ok(Self {
            recents,
            order,
            // It doesn't make sense to me to persist the sort, so it won't be persisted.
            sort: RecentsSort::Default,
        })
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