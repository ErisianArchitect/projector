
use std::{
    cmp::{Ordering, Reverse}, collections::{BTreeMap, BTreeSet, HashSet}, ops::Range, path::{Path, PathBuf}
};

use bincode::Decode;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveWeek, Utc};
use eframe::{
    egui::*,
};
use crate::{projects::{ProjectPath, ProjectType}, util::atom::Atom};

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum Order {
    #[default]
    Ascending = 0,
    Descending = 1,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum Recency {
    #[default]
    Most,
    Least,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum RecentsSort {
    NameAscending,
    NameDescending,
    #[default]
    MostRecent,
    LeastRecent,
}

impl RecentsSort {
    // pub fn default_sort(lhs: (usize, &RecentEntry), rhs: (usize, &RecentEntry)) -> Ordering {
    //     let lhs = lhs.0;
    //     let rhs = rhs.0;
    //     lhs.cmp(&rhs)
    // }

    /// Returns false if the values are equal.
    #[must_use]
    #[inline]
    pub fn exchange(&mut self, sort: Self) -> bool {
        if *self == sort {
            return false;
        }
        *self = sort;
        true
    }

    #[must_use]
    #[inline]
    pub const fn text(self) -> &'static str {
        match self {
            RecentsSort::NameAscending => "Name Ascending",
            RecentsSort::NameDescending => "Name Descending",
            RecentsSort::MostRecent => "Most Recent",
            RecentsSort::LeastRecent => "Least Recent",
        }
    }

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
            starred: false,
            // tags: BTreeSet::new(),
        }
    }

    #[inline]
    pub fn rust<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Rust(path.into()),
            last_open_time: self.time,
            starred: false,
            // tags: BTreeSet::new(),
        }
    }

    #[inline]
    pub fn python<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Python(path.into()),
            last_open_time: self.time,
            starred: false,
            // tags: BTreeSet::new(),
        }
    }

    #[inline]
    pub fn web<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Python(path.into()),
            last_open_time: self.time,
            starred: false,
            // tags: BTreeSet::new(),
        }
    }

    #[inline]
    pub fn other<P: Into<PathBuf>>(self, path: P) -> RecentEntry {
        RecentEntry {
            path: ProjectPath::Other(path.into()),
            last_open_time: self.time,
            starred: false,
            // tags: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecentEntry {
    path: ProjectPath,
    last_open_time: chrono::DateTime<chrono::Utc>,
    starred: bool,
    // TODO implement tagging system. Consider using Atom system that is
    //      in development.
    // tags: Vec<Atom>,
}

pub struct Date {
    year: i32,
    month: u8,
    day: u8,
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
            starred: false,
            // tags: BTreeSet::new(),
        }
    }

    #[must_use]
    #[inline]
    pub fn new(
        path: ProjectPath,
        time: chrono::DateTime<chrono::Utc>,
        starred: bool,
        // tags: BTreeSet<String>,
    ) -> Self {
        Self {
            path,
            last_open_time: time,
            starred,
            // tags,
        }
    }

    #[must_use]
    #[inline]
    pub const fn starred(&self) -> bool {
        self.starred
    }

    #[must_use]
    #[inline]
    pub const fn with_star(mut self) -> Self {
        self.starred = true;
        self
    }

    #[must_use]
    #[inline]
    pub const fn path(&self) -> &ProjectPath {
        &self.path
    }

    #[must_use]
    pub fn name(&self) -> &str {
        if let Some(name) = self.path.file_name()
        && let Some(name) = name.to_str() {
            name
        } else {
            ""
        }
    }

    #[must_use]
    #[inline]
    pub fn last_open_time_utc(&self) -> DateTime<Utc> {
        self.last_open_time
    }

    #[must_use]
    #[inline]
    pub fn last_open_time_local(&self) -> DateTime<Local> {
        self.last_open_time.into()
    }

    // #[must_use]
    // #[inline]
    // pub fn date(&self) -> 
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GroupRange {
    start: u16,
    end: u16,
}

impl GroupRange {
    #[must_use]
    #[inline]
    pub const fn new(range: std::ops::Range<u16>) -> Self {
        debug_assert!(range.start <= range.end);
        Self {
            start: range.start,
            end: range.end,
        }
    }

    #[must_use]
    #[inline]
    pub const fn range_u16(self) -> Range<u16> {
        self.start..self.end
    }

    #[must_use]
    #[inline]
    pub const fn range(self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct ProjectTypeGroupSort {
    rust: i8,
    python: i8,
    web: i8,
    other: i8,
}

impl ProjectTypeGroupSort {
    const DEFAULT: Self = Self::new(0, 1, 2, 3);
    const DEFAULT_REV: Self = Self::new(3, 2, 1, 0);
    #[must_use]
    #[inline]
    pub const fn new(rust: i8, python: i8, web: i8, other: i8) -> Self {
        Self {
            rust,
            python,
            web,
            other,
        }
    }

    #[must_use]
    #[inline]
    pub const fn order_of(&self, project_type: ProjectType) -> i8 {
        match project_type {
            ProjectType::Rust => self.rust,
            ProjectType::Python => self.python,
            ProjectType::Web => self.web,
            ProjectType::Other => self.other,
        }
    }
}

impl Default for ProjectTypeGroupSort {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub enum GroupBy {
    Ungrouped,
    Day,
    Week,
    Month,
    Year,
    ProjectType(ProjectTypeGroupSort),
}

impl GroupBy {
    #[must_use]
    #[inline]
    pub const fn is_grouped(self) -> bool {
        !matches!(self, Self::Ungrouped)
    }

    #[must_use]
    #[inline]
    fn exchange(&mut self, group_by: Self) -> bool {
        if self == &group_by {
            return false;
        }
        *self = group_by;
        true
    }

    #[must_use]
    #[inline]
    pub const fn text(self) -> &'static str {
        match self {
            GroupBy::Ungrouped => "Ungrouped",
            GroupBy::Day => "Day",
            GroupBy::Week => "Week",
            GroupBy::Month => "Month",
            GroupBy::Year => "Year",
            GroupBy::ProjectType(_) => "Project Type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupTag {
    Day(NaiveDate),
    Week(NaiveWeek),
    /// DateTime is first day of the month.
    Month(NaiveDate),
    /// DateTime is first day of the year.
    Year(NaiveDate),
    ProjectType(ProjectType),
}

#[derive(Debug)]
struct RecentsGroup {
    range: GroupRange,
    tag: GroupTag,
}

impl RecentsGroup {

    #[must_use]
    #[inline]
    const fn new(range: Range<u16>, tag: GroupTag) -> Self {
        Self {
            range: GroupRange::new(range),
            tag,
        }
    }

    // #[must_use]
    // #[inline]
    // const fn range(&self) -> Range<usize> {
    //     self.range.range()
    // }

    // #[must_use]
    // #[inline]
    // const fn tag(&self) -> &GroupTag {
    //     &self.tag
    // }

    #[must_use]
    #[inline]
    const fn replace_new(&mut self, range: Range<u16>, tag: GroupTag) -> Self {
        std::mem::replace(self, Self::new(range, tag))
    }
}

/// Not to be confused with [Recent].
#[derive(Debug)]
pub struct Recents {
    recents: Vec<RecentEntry>,
    order: Vec<u16>,
    groups: Vec<RecentsGroup>,
    search_results: Vec<u16>,
    search_string: String,
    sort: RecentsSort,
    group_by: GroupBy,
}

impl Recents {
    #[must_use]
    pub fn new(recents: Vec<RecentEntry>, sort: RecentsSort, group_by: GroupBy) -> Self {
        assert!(recents.len() <= u16::MAX as usize);
        let order: Vec<u16> = (0..recents.len() as u16).collect();
        let search_results = order.clone();
        Self {
            order,
            recents,
            groups: Vec::new(),
            search_results,
            search_string: String::with_capacity(128),
            sort,
            group_by,
        }.init_group_and_sort()
    }

    fn sort_search(&mut self) {
        if self.search_string.is_empty() {
            return;
        }
        let Self {
            recents,
            order,
            search_string,
            search_results,
            ..
        } = self;
        let search_string_lower = search_string.to_lowercase();
        search_results.sort_by_key(move |&order_index| {
            let entry_index = order[order_index as usize];
            let entry = &recents[entry_index as usize];
            if let Some(name) = entry.path.file_name()
            && let Some(name) = name.to_str() {
                let name_lower = name.to_lowercase();
                let search_bias = strsim::jaro_winkler(&search_string_lower, &name_lower);
                let mult = u16::MAX as f64;
                let bias_uint = (search_bias * mult).round() as u16;
                Reverse(bias_uint)
            } else {
                Reverse(0)
            }
        });
    }

    fn init_group_and_sort(mut self) -> Self {
        self.group_and_sort();
        self
    }

    /// This method should be called after assigning the sort/group
    fn group_and_sort(&mut self) {
        let sort = self.sort;
        let group_by = self.group_by;
        let Self {
            recents,
            order,
            groups,
            ..
        } = self;
        match (group_by, sort) {
            (GroupBy::Ungrouped, RecentsSort::NameAscending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    (),
                    if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    }
                )
            }),
            (GroupBy::Ungrouped, RecentsSort::NameDescending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    (),
                    Reverse(if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    })
                )
            }),
            (GroupBy::Ungrouped, RecentsSort::MostRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    (),
                    Reverse(recent.last_open_time),
                )
            }),
            (GroupBy::Ungrouped, RecentsSort::LeastRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    (),
                    recent.last_open_time
                )
            }),
            (GroupBy::Day, RecentsSort::NameAscending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let local_time = recent.last_open_time_local();
                let year = local_time.year();
                let month = local_time.month();
                let day = local_time.day();
                (
                    Reverse((year, month, day)),
                    if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    }
                )
            }),
            (GroupBy::Day, RecentsSort::NameDescending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let local_time = recent.last_open_time_local();
                let year = local_time.year();
                let month = local_time.month();
                let day = local_time.day();
                (
                    Reverse((year, month, day)),
                    Reverse(if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    })
                )
            }),
            (GroupBy::Day, RecentsSort::MostRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let local_time = recent.last_open_time_local();
                let year = local_time.year();
                let month = local_time.month();
                let day = local_time.day();
                (
                    Reverse((year, month, day)),
                    Reverse(recent.last_open_time),
                )
            }),
            (GroupBy::Day, RecentsSort::LeastRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let local_time = recent.last_open_time_local();
                let year = local_time.year();
                let month = local_time.month();
                let day = local_time.day();
                (
                    Reverse((year, month, day)),
                    recent.last_open_time,
                )
            }),
            (GroupBy::Week, RecentsSort::NameAscending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let naive_date = recent.last_open_time.naive_local();
                let week = naive_date.date().week(chrono::Weekday::Sun);
                (
                    Reverse(week.first_day()),
                    if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    }
                )
            }),
            (GroupBy::Week, RecentsSort::NameDescending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let naive_date = recent.last_open_time.naive_local();
                let week = naive_date.date().week(chrono::Weekday::Sun);
                (
                    Reverse(week.first_day()),
                    Reverse(if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    })
                )
            }),
            (GroupBy::Week, RecentsSort::MostRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let last_open_time = &recent.last_open_time;
                let naive_date = last_open_time.naive_local();
                let week = naive_date.date().week(chrono::Weekday::Sun);
                (
                    Reverse(week.first_day()),
                    Reverse(last_open_time)
                )
            }),
            (GroupBy::Week, RecentsSort::LeastRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let last_open_time = &recent.last_open_time;
                let naive_date = last_open_time.naive_local();
                let week = naive_date.date().week(chrono::Weekday::Sun);
                (
                    Reverse(week.first_day()),
                    last_open_time
                )
            }),
            (GroupBy::Month, RecentsSort::NameAscending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let naive_date = recent.last_open_time.naive_local();
                (
                    Reverse((naive_date.year(), naive_date.month())),
                    if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    }
                )
            }),
            (GroupBy::Month, RecentsSort::NameDescending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let naive_date = recent.last_open_time.naive_local();
                (
                    Reverse((naive_date.year(), naive_date.month())),
                    Reverse(if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    })
                )
            }),
            (GroupBy::Month, RecentsSort::MostRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let open_time = recent.last_open_time;
                let naive_date = open_time.naive_local();
                (
                    Reverse((naive_date.year(), naive_date.month())),
                    Reverse(open_time)
                )
            }),
            (GroupBy::Month, RecentsSort::LeastRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let open_time = recent.last_open_time;
                let naive_date = open_time.naive_local();
                (
                    Reverse((naive_date.year(), naive_date.month())),
                    open_time
                )
            }),
            (GroupBy::Year, RecentsSort::NameAscending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let open_time = recent.last_open_time;
                let naive_date = open_time.naive_local();
                (
                    Reverse(naive_date.year()),
                    if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    }
                )
            }),
            (GroupBy::Year, RecentsSort::NameDescending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let open_time = recent.last_open_time;
                let naive_date = open_time.naive_local();
                (
                    Reverse(naive_date.year()),
                    Reverse(if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    })
                )
            }),
            (GroupBy::Year, RecentsSort::MostRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let open_time = recent.last_open_time;
                let naive_date = open_time.naive_local();
                (
                    Reverse(naive_date.year()),
                    Reverse(open_time)
                )
            }),
            (GroupBy::Year, RecentsSort::LeastRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                let open_time = recent.last_open_time;
                let naive_date = open_time.naive_local();
                (
                    Reverse(naive_date.year()),
                    open_time
                )
            }),
            (GroupBy::ProjectType(project_type_group_sort), RecentsSort::NameAscending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    project_type_group_sort.order_of(recent.path.project_type()),
                    if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    }
                )
            }),
            (GroupBy::ProjectType(project_type_group_sort), RecentsSort::NameDescending) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    project_type_group_sort.order_of(recent.path.project_type()),
                    Reverse(if let Some(name) = recent.path.file_name()
                    && let Some(name) = name.to_str() {
                        name
                    } else {
                        ""
                    })
                )
            }),
            (GroupBy::ProjectType(project_type_group_sort), RecentsSort::MostRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    project_type_group_sort.order_of(recent.path.project_type()),
                    Reverse(recent.last_open_time)
                )
            }),
            (GroupBy::ProjectType(project_type_group_sort), RecentsSort::LeastRecent) => order.sort_by_key(|&entry_index| {
                let recent = &recents[entry_index as usize];
                (
                    project_type_group_sort.order_of(recent.path.project_type()),
                    recent.last_open_time
                )
            }),
        }
        groups.clear();
        match group_by {
            GroupBy::Ungrouped => (),
            GroupBy::Day => {
                let Some((&first, rest)) = order.split_first() else {
                    return;
                };
                let first_entry = &recents[first as usize];
                let mut cur_group = RecentsGroup::new(0..1, GroupTag::Day(first_entry.last_open_time_local().date_naive()));
                for entry_index in rest.iter().cloned() {
                    let entry = &recents[entry_index as usize];
                    let GroupTag::Day(group_date) = &cur_group.tag else {
                        unreachable!("This should not be possible.");
                    };
                    let date = entry.last_open_time_local().date_naive();
                    if group_date == &date {
                        cur_group.range.end += 1;
                    } else {
                        let range = cur_group.range.end..cur_group.range.end + 1;
                        groups.push(cur_group.replace_new(range, GroupTag::Day(date)));
                    }
                }
                groups.push(cur_group);
            },
            GroupBy::Week => {
                let Some((&first, rest)) = order.split_first() else {
                    return;
                };
                let first_entry = &recents[first as usize];
                let mut cur_group = RecentsGroup::new(0..1, GroupTag::Week(first_entry.last_open_time.naive_local().date().week(chrono::Weekday::Sun)));
                for entry_index in rest.iter().cloned() {
                    let entry = &recents[entry_index as usize];
                    let GroupTag::Week(group_week) = &cur_group.tag else {
                        unreachable!("This should not be possible.");
                    };
                    let week = entry.last_open_time.naive_local().date().week(chrono::Weekday::Sun);
                    if group_week == &week {
                        cur_group.range.end += 1;
                    } else {
                        let range = cur_group.range.end..cur_group.range.end + 1;
                        groups.push(cur_group.replace_new(range, GroupTag::Week(week)));
                    }
                }
                groups.push(cur_group);
            },
            GroupBy::Month => {
                let Some((&first, rest)) = order.split_first() else {
                    return;
                };
                let first_entry = &recents[first as usize];
                let first_time = first_entry.last_open_time.naive_local();
                let year = first_time.year();
                let month = first_time.month();
                let first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let mut cur_group = RecentsGroup::new(0..1, GroupTag::Month(first_of_month));
                for entry_index in rest.iter().cloned() {
                    let entry = &recents[entry_index as usize];
                    let GroupTag::Month(first_day) = &cur_group.tag else {
                        unreachable!();
                    };
                    let time = entry.last_open_time.naive_local();
                    let year = time.year();
                    let month = time.month();
                    let cur_first_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                    if first_day == &cur_first_of_month {
                        cur_group.range.end += 1;
                    } else {
                        let range = cur_group.range.end..cur_group.range.end + 1;
                        groups.push(cur_group.replace_new(range, GroupTag::Month(cur_first_of_month)));
                    }
                }
                groups.push(cur_group);
            },
            GroupBy::Year => {
                let Some((&first, rest)) = order.split_first() else {
                    return;
                };
                let first_entry = &recents[first as usize];
                let first_time = first_entry.last_open_time.naive_local();
                let year = first_time.year();
                let first_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let mut cur_group = RecentsGroup::new(0..1, GroupTag::Year(first_of_year));
                for entry_index in rest.iter().cloned() {
                    let entry = &recents[entry_index as usize];
                    let GroupTag::Year(first_of_year) = &cur_group.tag else {
                        unreachable!();
                    };
                    let time = entry.last_open_time.naive_local();
                    let year = time.year();
                    let cur_first_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                    if first_of_year == &cur_first_of_year {
                        cur_group.range.end += 1;
                    } else {
                        let range = cur_group.range.end..cur_group.range.end + 1;
                        groups.push(cur_group.replace_new(range, GroupTag::Year(cur_first_of_year)));
                    }
                }
                groups.push(cur_group);
            },
            GroupBy::ProjectType(_) => {
                let Some((&first, rest)) = order.split_first() else {
                    return;
                };
                let first_entry = &recents[first as usize];
                let first_proj_ty = first_entry.path.project_type();
                let mut cur_group = RecentsGroup::new(0..1, GroupTag::ProjectType(first_proj_ty));
                for entry_index in rest.iter().cloned() {
                    let entry = &recents[entry_index as usize];
                    let GroupTag::ProjectType(proj_ty) = &cur_group.tag else {
                        unreachable!();
                    };
                    let cur_proj_ty = entry.path.project_type();
                    if proj_ty == &cur_proj_ty {
                        cur_group.range.end += 1;
                    } else {
                        let range = cur_group.range.end..cur_group.range.end + 1;
                        groups.push(cur_group.replace_new(range, GroupTag::ProjectType(cur_proj_ty)));
                    }
                }
                groups.push(cur_group);
            },
        }
        self.sort_search();
    }

    pub fn set_sort(&mut self, new_sort: RecentsSort) {
        if !self.sort.exchange(new_sort) {
            return;
        }
        self.group_and_sort();
    }

    pub fn set_group_by(&mut self, group_by: GroupBy) {
        if !self.group_by.exchange(group_by) {
            return;
        }
        self.group_and_sort();
    }

    pub fn set_group_by_and_sort(&mut self, sort: RecentsSort, group_by: GroupBy) {
        if !self.sort.exchange(sort) && !self.group_by.exchange(group_by) {
            // both remain unchanged, so return.
            return;
        }
        self.group_and_sort();
    }

    pub fn set_search(&mut self, search: &str) {
        if self.search_string == search {
            return;
        }
        self.search_string.clear();
        self.search_string.push_str(search);
        self.sort_search();
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

    #[must_use]
    #[inline]
    pub const fn sort(&self) -> RecentsSort {
        self.sort
    }

    #[must_use]
    #[inline]
    pub const fn group_by(&self) -> GroupBy {
        self.group_by
    }

    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.order.len()
    }

    fn index_at(&self, index: usize) -> usize {
        if self.search_string.is_empty() {
            self.order[index] as usize
        } else {
            let order_index = self.search_results[index] as usize;
            let entry_index = self.order[order_index as usize];
            entry_index as usize
        }
    }

    fn group_at(&self, index: u16) -> Option<&RecentsGroup> {
        if self.groups.is_empty() {
            return None;
        }
        let found = self.groups.binary_search_by(|group| {
            if index < group.range.start {
                Ordering::Less
            } else if index >= group.range.end {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }).ok()?;
        Some(&self.groups[found])
    }

    /// Sets the entry's time to Utc::now() then bumps it in the order if the order depends on the time.
    #[inline]
    pub fn bump(&mut self, index: usize) {
        // Bumps the time for an entry, and may move it in the order if the sort is time based.
        // the index in self.recents
        let unordered_index = self.index_at(index);
        self.recents[unordered_index].last_open_time = chrono::Utc::now();
        self.group_and_sort();
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> RecentEntry {
        let order_index = if self.search_string.is_empty() {
            index
        } else {
            self.search_results[index] as usize
        };
        let recents_index = self.order.remove(order_index);
        let entry = self.recents.remove(recents_index as usize);
        // It's alright to subtract one here because it should be known that the len is > 0.
        let highest = (self.search_results.len() - 1) as u16;
        self.search_results.retain(move |&order_index| order_index != highest);
        // Adjust indices to account for the removed item.
        self.order.iter_mut().for_each(move |index| if *index > recents_index { *index -= 1 });
        // I'm lazy, so this is how it's gonna be.
        // self.search_results.clear();
        // self.search_results.extend(0u16..self.order.len() as u16);
        // let highest = self.search_results.len() - 1;
        // let (max_index, _) = self.search_results.iter().enumerate().fold((0usize, 0u16), |(fold_index, fold_order_index), (index, &order_index)| {
        //     if order_index > fold_order_index {
        //         (index, order_index)
        //     } else {
        //         (fold_index, fold_order_index)
        //     }
        // });
        // self.search_results.remove(max_index);
        self.group_and_sort();
        entry
    }

    /// Finds the index in the `self.order` list where the index to this path exists in `self.recents` or returns None if it doesn't exist.
    /// This is a linear search because each path needs to be checked individually.
    #[must_use]
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

    pub fn insert_now(&mut self, path: ProjectPath) {
        if let Some(entry_index) = self.order_entry_index(&path) {
            self.bump(entry_index);
            return;
        }
        let entry = RecentEntry::now(path);
        let index = self.recents.len();
        self.recents.push(entry);
        let order_index = self.order.len();
        self.order.push(index as u16);
        self.search_results.push(order_index as u16);
        self.group_and_sort();
    }

    // We'll comment this out for now.
    // /// Purges all paths that are not found on the file system.
    // pub fn purge_not_found(&mut self) {
    //     let purge_list = self.order.iter().enumerate().filter_map(|(i, &entry_index)| {
    //         let entry = &self.recents[entry_index as usize];
    //         if entry.path.exists() {
    //             None
    //         } else {
    //             Some(i)
    //         }
    //     }).collect::<Vec<_>>();
    //     for &purge_index in purge_list.iter().rev() {
    //         self.remove(purge_index);
    //     }
    // }

    #[must_use]
    #[inline]
    pub fn iter_order(&self) -> impl Iterator<Item = &RecentEntry> {
        self.order.iter().map(move |&index| &self.recents[index as usize])
    }

    #[must_use]
    #[inline]
    pub fn iter_search(&self) -> impl Iterator<Item = &RecentEntry> {
        self.search_results.iter().map(move |&index| &self.recents[self.order[index as usize] as usize])
    }

    #[inline]
    pub fn clear(&mut self) {
        self.recents.clear();
        self.order.clear();
        self.search_results.clear();
        self.search_string.clear();
        self.groups.clear();
    }
}

impl std::ops::Index<usize> for Recents {
    type Output = RecentEntry;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let entry_index = self.index_at(index);
        &self.recents[entry_index]
    }
}

impl std::ops::IndexMut<usize> for Recents {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let entry_index = self.index_at(index);
        &mut self.recents[entry_index]
    }
}

impl bincode::Encode for RecentEntry {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.path.encode(encoder)?;
        let seconds = self.last_open_time.timestamp();
        let nsecs = self.last_open_time.timestamp_subsec_nanos();
        seconds.encode(encoder)?;
        nsecs.encode(encoder)?;
        self.starred.encode(encoder)?;
        Ok(())
    }
}

impl<Ctx> bincode::Decode<Ctx> for RecentEntry {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let path = ProjectPath::decode(decoder)?;
        let seconds = i64::decode(decoder)?;
        let nsecs = u32::decode(decoder)?;
        let starred = bool::decode(decoder)?;
        Ok(Self {
            path,
            last_open_time: chrono::DateTime::from_timestamp(seconds, nsecs).unwrap_or_default(),
            starred,
            // TODO
            // tags: BTreeSet::new(),
        })
    }
}

impl<'de, Ctx> bincode::BorrowDecode<'de, Ctx> for RecentEntry {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Ctx>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
        let path = ProjectPath::decode(decoder)?;
        let seconds = i64::decode(decoder)?;
        let nsecs = u32::decode(decoder)?;
        let starred = bool::decode(decoder)?;
        Ok(Self {
            path,
            last_open_time: chrono::DateTime::from_timestamp(seconds, nsecs).unwrap_or_default(),
            starred,
            // TODO
            // tags: BTreeSet::new(),
        })
    }
}

impl bincode::Encode for Recents {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        self.recents.encode(encoder)?;
        self.sort.encode(encoder)?;
        self.group_by.encode(encoder)?;
        Ok(())
    }
}

impl<Ctx> bincode::Decode<Ctx> for Recents {
    fn decode<D: bincode::de::Decoder<Context = Ctx>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::new(
            Vec::<RecentEntry>::decode(decoder)?,
            RecentsSort::decode(decoder)?,
            GroupBy::decode(decoder)?,
        ))
    }
}

impl<'de, Ctx> bincode::BorrowDecode<'de, Ctx> for Recents {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Ctx>>(
            decoder: &mut D,
        ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::new(
            Vec::<RecentEntry>::decode(decoder)?,
            RecentsSort::decode(decoder)?,
            GroupBy::decode(decoder)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

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
        let mut recents = Recents::new(vec![], RecentsSort::LeastRecent, GroupBy::Ungrouped);
        println!("{:?}", recents.sort);
        let start = Instant::now();
        recents.insert_now(ProjectPath::other("./ignore/sub/a.txt"));
        recents.insert_now(ProjectPath::other("./ignore/sub/b.txt"));
        recents.insert_now(ProjectPath::other("./ignore/a.txt"));
        recents.insert_now(ProjectPath::other("./ignore/c.txt"));
        recents.insert_now(ProjectPath::other("./ignore/b.txt"));
        let elapsed = start.elapsed();
        println!("Inserted in {elapsed:?}");
        print_recents(&recents);
        sep();
        let start = Instant::now();
        // recents.purge_not_found();
        let elapsed = start.elapsed();
        println!("Purged in {elapsed:?}");
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