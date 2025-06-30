
use std::{
    cmp::Ordering, collections::HashMap, ops::Index, path::{Path, PathBuf}
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum RecentsSort {
    NameAscending,
    NameDescending,
    MostRecent,
    LeastRecent,
}

impl RecentsSort {
    // pub fn default_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
    //     let lhs = lhs.0;
    //     let rhs = rhs.0;
    //     lhs.cmp(&rhs)
    // }
    #[inline]
    pub const fn is_time_based(self) -> bool {
        matches!(self, Self::MostRecent | Self::LeastRecent)
    }

    #[inline]
    pub const fn is_name_based(self) -> bool {
        matches!(self, Self::NameAscending | Self::NameDescending)
    }

    #[inline]
    fn make_search_fn<'a, 'b: 'a>(find: &'b RecentEntry, sorter: fn(&RecentEntry, &RecentEntry) -> Ordering) -> impl FnMut(&'a RecentEntry) -> Ordering {
        move |entry: &'a RecentEntry| {
            sorter(entry, find)
        }
    }

    fn ascending_name_sort(lhs: &RecentEntry, rhs: &RecentEntry) -> Ordering {
        let lhs = lhs.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        let rhs = rhs.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        lhs.cmp(rhs)
    }

    fn ascending_name_search<'a, 'b: 'a>(find: &'b RecentEntry) -> impl FnMut(&'a RecentEntry) -> Ordering {
        Self::make_search_fn(find, Self::ascending_name_sort)
    }

    fn descending_name_sort(lhs: &RecentEntry, rhs: &RecentEntry) -> Ordering {
        let lhs = lhs.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        let rhs = rhs.path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("");
        rhs.cmp(lhs)
    }

    fn descending_name_search<'a, 'b: 'a>(find: &'b RecentEntry) -> impl FnMut(&'a RecentEntry) -> Ordering {
        Self::make_search_fn(find, Self::descending_name_sort)
    }

    fn most_recent_sort(lhs: &RecentEntry, rhs: &RecentEntry) -> Ordering {
        let lhs = &lhs.last_open_time;
        let rhs = &rhs.last_open_time;
        rhs.cmp(lhs)
    }

    fn most_recent_search<'a, 'b: 'a>(find: &'b RecentEntry) -> impl FnMut(&'a RecentEntry) -> Ordering {
        Self::make_search_fn(find, Self::most_recent_sort)
    }

    fn least_recent_sort(lhs: &RecentEntry, rhs: &RecentEntry) -> Ordering {
        let lhs = &lhs.last_open_time;
        let rhs = &rhs.last_open_time;
        lhs.cmp(rhs)
    }

    fn least_recent_search<'a, 'b: 'a>(find: &'b RecentEntry) -> impl FnMut(&'a RecentEntry) -> Ordering {
        Self::make_search_fn(find, Self::least_recent_sort)
    }

    fn sort_by_fn(self) -> fn(&RecentEntry, &RecentEntry) -> Ordering {
        match self {
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
            let l_entry = &recents[l_index];
            let r_entry = &recents[r_index];
            sort_by(l_entry, r_entry)
        });
    }

    pub fn partition_point(self, recents: &[RecentEntry], order: &[u16], find: &RecentEntry) -> usize {
        match self {
            RecentsSort::NameAscending => {
                let mut search = Self::ascending_name_search(find);
                order.partition_point(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry) != Ordering::Greater
                })
            },
            RecentsSort::NameDescending => {
                let mut search = Self::descending_name_search(find);
                order.partition_point(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry) != Ordering::Greater
                })
            },
            RecentsSort::MostRecent => {
                let mut search = Self::most_recent_search(find);
                order.partition_point(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry) != Ordering::Greater
                })
            },
            RecentsSort::LeastRecent => {
                let mut search = Self::least_recent_search(find);
                order.partition_point(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry) != Ordering::Greater
                })
            },
        }
    }

    pub fn search(self, recents: &[RecentEntry], order: &[u16], find: &RecentEntry) -> Result<usize, usize> {
        match self {
            RecentsSort::NameAscending => {
                let mut search = Self::ascending_name_search(find);
                order.binary_search_by(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry)
                })
            },
            RecentsSort::NameDescending => {
                let mut search = Self::descending_name_search(find);
                order.binary_search_by(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry)
                })
            },
            RecentsSort::MostRecent => {
                let mut search = Self::most_recent_search(find);
                order.binary_search_by(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry)
                })
            },
            RecentsSort::LeastRecent => {
                let mut search = Self::least_recent_search(find);
                order.binary_search_by(move |&index| {
                    let entry = &recents[index as usize];
                    search(entry)
                })
            },
        }
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
            last_open_time: self.time,
        }
    }

    #[inline]
    pub fn rust<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Rust(path.into()),
            last_open_time: self.time,
        }
    }

    #[inline]
    pub fn python<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Python(path.into()),
            last_open_time: self.time,
        }
    }

    #[inline]
    pub fn web<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Python(path.into()),
            last_open_time: self.time,
        }
    }

    #[inline]
    pub fn other<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Other(path.into()),
            last_open_time: self.time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecentEntry {
    path: ProjectPath,
    last_open_time: chrono::DateTime<chrono::Utc>,
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
            last_open_time: chrono::Utc::now(),
        }
    }

    #[must_use]
    #[inline]
    pub fn new(path: ProjectPath, time: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            path,
            last_open_time: time,
        }
    }
}

impl bincode::Encode for RecentEntry {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.path.encode(encoder)?;
        let seconds = self.last_open_time.timestamp();
        let nsecs = self.last_open_time.timestamp_subsec_nanos();
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
            last_open_time: chrono::DateTime::from_timestamp(seconds, nsecs).unwrap_or_default(),
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
    pub fn new(recents: Vec<RecentEntry>, sort: RecentsSort) -> Self {
        let mut order: Vec<u16> = (0..recents.len()).map(|i| i as u16).collect();
        sort.sort(recents.as_slice(), order.as_mut_slice());
        Self {
            order,
            recents,
            sort,
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

    #[inline]
    pub const fn sort(&self) -> RecentsSort {
        self.sort
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.order.len()
    }

    /// Sets the entry's time to Utc::now() then bumps it in the order if the order depends on the time.
    #[inline]
    pub fn bump(&mut self, index: usize) {
        // Bumps the time for an entry, and may move it in the order if the sort is time based.
        self.recents[self.order[index] as usize].last_open_time = chrono::Utc::now();
        if matches!(self.sort, RecentsSort::MostRecent | RecentsSort::LeastRecent) {
            let recent_index = self.order.remove(index);
            let entry = &self.recents[recent_index as usize];
            let insert_index = self.sort.partition_point(&self.recents, &self.order, entry);
            self.order.insert(insert_index, recent_index);
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        let recents_index = self.order.remove(index);
        self.recents.remove(recents_index as usize);
        // Adjust indices to account for the removed item.
        self.order.iter_mut().for_each(move |index| if *index > recents_index { *index -= 1 });
    }

    /// Finds the index in the `self.order` list where the index to this path exists in `self.recents` or returns None if it doesn't exist.
    /// This is a linear search because each path needs to be checked individually.
    fn order_entry_index(&self, path: &Path) -> Option<usize> {
        self.order.iter().cloned().enumerate().find_map(move |(i, entry_index)| {
            let entry = &self.recents[entry_index as usize];
            if same_file::is_same_file(path, &entry.path).unwrap_or(false) {
                Some(i)
            } else {
                None
            }
        })
    }

    pub fn push_now(&mut self, path: ProjectPath) {
        if let Some(entry_index) = self.order_entry_index(&path) {
            self.bump(entry_index);
            return;
        }
        let entry = RecentEntry::now(path);
        let index = self.recents.len();
        let insert_index = self.sort.partition_point(&self.recents, &self.order, &entry);
        self.recents.push(entry);
        self.order.insert(insert_index, index as u16);
    }

    /// Purges all paths that are not found on the file system.
    pub fn purge_not_found(&mut self) {
        let purge_list = self.order.iter().enumerate().filter_map(|(i, &entry_index)| {
            let entry = &self.recents[entry_index as usize];
            if entry.path.exists() {
                None
            } else {
                Some(i)
            }
        }).collect::<Vec<_>>();
        for &purge_index in purge_list.iter().rev() {
            self.remove(purge_index)
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &RecentEntry> {
        self.order.iter().map(move |&index| &self.recents[index as usize])
    }

    #[inline]
    pub fn clear(&mut self) {
        self.recents.clear();
        self.order.clear();
    }
}

impl std::ops::Index<usize> for Recents {
    type Output = RecentEntry;

    fn index(&self, index: usize) -> &Self::Output {
        let entry_index = self.order[index] as usize;
        &self.recents[entry_index]
    }
}

impl std::ops::IndexMut<usize> for Recents {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let entry_index = self.order[index] as usize;
        &mut self.recents[entry_index]
    }
}

impl bincode::Encode for Recents {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.recents.encode(encoder)
    }
}

impl<Ctx> bincode::Decode<Ctx> for Recents {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::new(
            Vec::<RecentEntry>::decode(decoder)?,
            RecentsSort::MostRecent,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEP: &'static str = "********************************";
    fn sep() {
        println!("{}", SEP);
    }

    #[test]
    fn recents_test() {
        fn print_recents(recents: &Recents) {
            for i in 0..recents.len() {
                let entry = &recents[i];
                println!("{}", entry.path.display());
            }
        }
        sep();
        let mut recents = Recents::new(vec![], RecentsSort::LeastRecent);
        println!("{:?}", recents.sort);
        recents.push_now(ProjectPath::other("./ignore/sub/a.txt"));
        recents.push_now(ProjectPath::other("./ignore/sub/b.txt"));
        recents.push_now(ProjectPath::other("./ignore/a.txt"));
        recents.push_now(ProjectPath::other("./ignore/c.txt"));
        recents.push_now(ProjectPath::other("./ignore/b.txt"));
        print_recents(&recents);
        sep();
        recents.purge_not_found();
        print_recents(&recents);
        sep();
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