#![allow(unused)]
use std::{path::{
    Path, PathBuf,
}, time::{Duration, Instant}};
use eframe::{
    egui::{self, *},
    App, CreationContext,
};
use crate::{appdata::AppData, eguiext::UiExt};
use crate::settings::*;

use crate::{settings::Settings, dgui::{self, tabs::{Tab, TabSizeMode, Tabs}}};

pub fn set_style(style: &mut Style) {
    style.visuals.widgets.active.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::ZERO;
    style.visuals.widgets.open.corner_radius = CornerRadius::ZERO;
    style.visuals.menu_corner_radius = CornerRadius::ZERO;
    style.visuals.window_corner_radius = CornerRadius::ZERO;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum ProjectType {
    Rust,
    Python,
    Web,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum MainTab {
    RecentProjects,
    Project(ProjectType),
    Text,
}

impl Default for MainTab {
    fn default() -> Self {
        Self::Project(ProjectType::Rust)
    }
}

impl MainTab {
    pub const fn text(self) -> &'static str {
        match self {
            MainTab::RecentProjects => "Recent",
            MainTab::Project(ProjectType::Rust) => "Rust",
            MainTab::Project(ProjectType::Python) => "Python",
            MainTab::Project(ProjectType::Web) => "Web",
            MainTab::Text => "Text",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgramState {
    Projects(MainTab),
    Settings,
}

impl ProgramState {
    #[inline]
    pub const fn initial() -> Self {
        Self::Projects(MainTab::Project(ProjectType::Rust))
    }
}

impl Default for ProgramState {
    fn default() -> Self {
        Self::initial()
    }
}

pub struct CreateProjectDialog {

}


pub enum ModalUi {
    None,
    Settings(SettingsDialog),
    CreateProject(CreateProjectDialog),
}

impl ModalUi {
    #[inline]
    pub fn close(&mut self) {
        *self = Self::None;
    }

    #[inline]
    pub fn settings(settings: Settings) -> Self {
        Self::Settings(SettingsDialog::from_settings(settings))
    }
}

pub struct ProjectorApp {
    settings: Settings,
    language_tab: MainTab,
    dialog: ModalUi,
    tab_index: usize,
    size_mode: TabSizeMode,
    quick_edit_text: String,
    app_data: AppData,
}

impl ProjectorApp {
    const QUICK_EDIT_CAPACITY: usize = 8192;
    pub fn boxed_new(cc: &CreationContext<'_>) -> Box<Self> {
        let app_data = AppData::from("com", "erisianarchitect", "projector").expect("Failed to create AppData object.");
        app_data.ensure_dirs();
        let settings = match app_data.config().load_settings() {
            Ok(settings) => settings,
            Err(err) => {
                eprintln!("Failed to load settings. Loading default settings instead. {err}");
                Settings::default()
            },
        };
        Box::new(Self {
            tab_index: match settings.general.default_projects_tab {
                MainTab::RecentProjects => 0,
                MainTab::Project(project_type) => match project_type {
                    ProjectType::Rust => 1,
                    ProjectType::Python => 2,
                    ProjectType::Web => 3,
                },
                MainTab::Text => 4,
            },
            language_tab: settings.general.default_projects_tab,
            settings,
            dialog: ModalUi::None,
            size_mode: TabSizeMode::Equal,
            quick_edit_text: String::with_capacity(Self::QUICK_EDIT_CAPACITY),
            app_data,
        })
    }
}

impl App for ProjectorApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        std::fs::write("output.txt", "Saved or something.").expect("Failed");
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        panel::TopBottomPanel::bottom("bottom_panel")
            .frame(Frame::new().stroke(Stroke::NONE))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (gear_rect, gear_btn) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                    let gear_style = ui.style().visuals.widgets.style(&gear_btn);
                    ui.painter().rect(gear_rect, CornerRadius::ZERO, gear_style.bg_fill, gear_style.bg_stroke, StrokeKind::Inside);
                    ui.painter().text(gear_rect.center(), Align2::CENTER_CENTER, "⚙", FontId::monospace(24.0), gear_style.text_color());
                    if gear_btn.clicked() {
                        self.dialog = ModalUi::settings(self.settings.clone());
                    }
                });
            });
        CentralPanel::default().frame(Frame::new().inner_margin(0.0)).show(ctx, |ui| {
            let mut close = false;
            let mut closer = DialogCloser::new(&mut close);
            match &mut self.dialog {
                ModalUi::None => (),
                ModalUi::Settings(settings_dialog) => {
                    settings_dialog.show(
                        closer,
                        &mut self.app_data,
                        &mut self.settings,
                        ui,
                    );
                },
                ModalUi::CreateProject(create_project_dialog) => {

                },
            }
            if close {
                self.dialog.close();
            }
            // ui.style_mut().spacing.window_margin = Margin::ZERO;
            // ui.style_mut().spacing.menu_margin = Margin::ZERO;
            set_style(ui.style_mut());
            let tabs = &[
                Tab::new("Recent", MainTab::RecentProjects),
                Tab::new("Rust", MainTab::Project(ProjectType::Rust)),
                Tab::new("Python", MainTab::Project(ProjectType::Python)),
                Tab::new("Web", MainTab::Project(ProjectType::Web)),
                Tab::new("Text", MainTab::Text),
            ];
            // menu::bar(ui, |ui| {
            //     ui.small_button("Settings");
            // });
            dgui::tabs::Tabs::new(&mut self.tab_index, tabs)
                .with_size_mode(self.size_mode)
                .with_text_align(Align::Center)
                .show(ui, |index, tab, ui| {
                    // ui.painter().rect_stroke(ui.max_rect(), CornerRadius::ZERO, Stroke::new(2.0, Color32::RED), StrokeKind::Outside);
                    match tab {
                        MainTab::RecentProjects => {
                            
                        }
                        MainTab::Project(ProjectType::Rust) => {
                            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                                Grid::new("rust_project_tab_grid")
                                    .num_columns(2)
                                    .min_col_width(250.0)
                                    .show(ui, |ui| {
                                        ui.rtl_label(Align::Center, "Test Button");
                                        ui.button("Test Button");
                                        ui.end_row();
                                        ui.rtl_label(Align::Center, "Tab Size Mode");
                                        ComboBox::new("size_mode_combo", "Tab Size Mode")
                                            .selected_text(match self.size_mode {
                                                TabSizeMode::Equal => "Equal",
                                                TabSizeMode::Shrink => "Shrink",
                                                TabSizeMode::Grow => "Grow",
                                                TabSizeMode::Exact(_) => "Exact",
                                                TabSizeMode::ShrinkMin(_) => "Shrink Min",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut self.size_mode, TabSizeMode::Equal, "Equal");
                                                ui.selectable_value(&mut self.size_mode, TabSizeMode::Grow, "Grow");
                                                ui.selectable_value(&mut self.size_mode, TabSizeMode::Shrink, "Shrink");
                                                ui.selectable_value(&mut self.size_mode, TabSizeMode::ShrinkMin(120.0), "Min");
                                            });
                                        ui.end_row();
                                    });
                            });
                        }
                        MainTab::Project(ProjectType::Python) => {
                            // let (bar_rect, _) = ui.allocate_exact_size(vec2(ui.available_width() / 2.0, 24.0), Sense::empty());
                            // ui.painter().rect_filled(bar_rect, CornerRadius::ZERO, Color32::DARK_GREEN);
                            // fn cont<F: FnOnce(&mut Ui) -> Response>(add_contents: F) -> F {
                            //     add_contents
                            // }
                            ui.spacing_mut().menu_margin = Margin::ZERO;
                            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                ui.spacing_mut().item_spacing = Vec2::ZERO;
                                let (btn_rect, btn) = ui.allocate_exact_size(vec2(100.0, 24.0), Sense::click());
                                let style = ui.style().visuals.widgets.style(&btn);
                                ui.painter().rect(btn_rect, CornerRadius::ZERO, style.bg_fill, style.bg_stroke, StrokeKind::Inside);
                                if btn.clicked() {
                                    println!("Test click.");
                                }
                                let (btn_rect, btn) = ui.allocate_exact_size(vec2(100.0, 24.0), Sense::click());
                                let style = ui.style().visuals.widgets.style(&btn);
                                ui.painter().rect(btn_rect, CornerRadius::ZERO, style.bg_fill, style.bg_stroke, StrokeKind::Inside);
                                if btn.clicked() {
                                    println!("Test click.");
                                }
                                btn
                            });
                        }
                        MainTab::Project(ProjectType::Web) => {
                            
                        }
                        MainTab::Text => {
                            ui.with_layout(Layout::bottom_up(Align::Center).with_cross_justify(true), |ui| {
                                ui.button("Save");
                                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                                    ScrollArea::both().show(ui, |ui| {
                                        TextEdit::multiline(&mut self.quick_edit_text)
                                            .font(FontId::monospace(16.0))
                                            // .frame(false)
                                            .code_editor()
                                            .show(ui);
                                    });
                                });
                            });
                            
                        }
                    }
                });
        });
    }
}