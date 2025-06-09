use std::path::{Path, PathBuf};
use eframe::{
    egui::{self, *},
};

use crate::{
    app::{
        MainTab, ModalUi, ProjectType
    }, appdata::{AppData, SettingsSaver}, dgui::tabs::{Tab, TabSizeMode, Tabs}, uiext::UiExt
};

struct ResponseUpdater {
    response: Option<Response>,
}

impl ResponseUpdater {
    fn new() -> Self {
        Self { response: None }
    }

    fn merge(&mut self, response: Response) {
        self.response = Some(if let Some(current) = self.response.take() {
            current.union(response)
        } else {
            response
        });
    }

    fn finish(self) -> Response {
        self.response.expect("No response!")
    }
}

macro_rules! settings_structs {
    (
        $(
            $(#[$attr:meta])*
            pub struct $name:ident {
                $(
                    pub $field:ident : $type:ty = $default:expr
                ),*
                $(,)?
            }
        )+
    ) => {
        $(
            $(#[$attr])*
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }
    
            impl Default for $name {
                fn default() -> $name {
                    $name {
                        $(
                            $field: $default,
                        )*
                    }
                }
            }
        )*
    };
}

settings_structs!{
    #[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
    pub struct General {
        pub open_after_create: bool = true,
        pub close_after_open: bool = true,
        pub default_projects_tab: MainTab = MainTab::Project(ProjectType::Rust),
    }

    #[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
    pub struct Rust {
        pub project_directories: Vec<PathBuf> = Vec::new(),
    }

    #[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
    pub struct Python {
        pub project_directories: Vec<PathBuf> = Vec::new(),
    }

    #[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
    pub struct Web {
        pub project_directories: Vec<PathBuf> = Vec::new(),
    }
    
    #[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
    pub struct Projects {
        pub rust: Rust = Rust::default(),
        pub python: Python = Python::default(),
        pub web: Web = Web::default(),
    }

    #[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
    pub struct Settings {
        pub general: General = General::default(),
        pub projects: Projects = Projects::default(),

    }
}

impl Settings {
    pub fn create_settings_modal(&self) -> ModalUi {
        ModalUi::Settings(SettingsDialog::from_settings(self.clone()))
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        self.clone_from(settings);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettingsTab {
    General,
    Projects,
    Plugins,
    Templates,
    Style,
}

pub struct DialogCloser<'a> {
    close: &'a mut bool,
}

impl<'a> DialogCloser<'a> {
    pub fn new(close: &'a mut bool) -> Self {
        Self { close }
    }

    pub fn close(&mut self) {
        *self.close = true;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditState {
    Modified = 0,
    #[default]
    Unaltered = 1,
    Synced = 2,
}

impl EditState {
    #[inline]
    pub const fn needs_update(self) -> bool {
        matches!(self, EditState::Modified)
    }

    #[inline]
    pub const fn synced(self) -> bool {
        !matches!(self, EditState::Modified)
    }
}

pub struct SettingsDialog {
    settings_copy: Settings,
    settings_tab_index: usize,
    edit_state: EditState,
    request_close: bool,
    general_gui: GeneralGui,
    projects_gui: ProjectsGui,
}

pub struct SettingsDialogResponse {
    pub inner: Response,

}

impl SettingsDialog {
    pub fn from_settings(settings: Settings) -> Self {
        Self {
            settings_copy: settings,
            settings_tab_index: 0,
            edit_state: EditState::Unaltered,
            request_close: false,
            general_gui: GeneralGui {

            },
            projects_gui: ProjectsGui {
                projects_tab: ProjectType::Rust,
                tab_index: 0,
            },
        }
    }

    pub fn show(
        &mut self,
        mut closer: DialogCloser<'_>,
        app_data: &mut AppData,
        original_settings: &mut Settings,
        ui: &mut Ui,
    ) -> Response {
        fn apply_settings(
            target: &mut Settings,
            settings: &Settings,
            app_data: &mut AppData,
        ) -> crate::error::Result<()> {
            target.apply_settings(&settings);
            app_data.config().save_settings(target)
        }
        modal::Modal::new(Id::new("settings_dialog_modal"))
            .area(
                Area::new(Id::new("settings_dialog_modal_-_area"))
                    .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            )
            .frame(
                Frame::new()
                    .corner_radius(CornerRadius::ZERO)
                    // .stroke(Stroke::new(1.0, Color32::GRAY))
            )
            .show(ui.ctx(), |ui| {
                // let avail = ui.available_rect_before_wrap();
                if self.request_close {
                    if self.edit_state.needs_update() {
                        Modal::new(Id::new("request_close_modal"))
                            .show(ui.ctx(), |ui| {
                                ui.label("Settings have been modified.");
                                ui.horizontal_centered(|ui| {
                                    if ui.button("Apply").clicked() {
                                        if let Ok(_) = apply_settings(original_settings, &self.settings_copy, app_data) {
                                            self.edit_state = EditState::Synced;
                                        } else {
                                            eprintln!("Failed to save settings.");
                                        }
                                    }
                                    if ui.button("Discard").clicked() {
                                        closer.close();
                                    }
                                    if ui.button("Cancel").clicked() {
                                        self.request_close = false;
                                    }
                                });
                            });
                    } else {
                        closer.close();
                    }
                }
                // ui.horizontal(|ui| {
                //     ui.style_mut().spacing.item_spacing = Vec2::ZERO;
                //     ui.allocate_exact_size(vec2(0.0, 400.0), Sense::empty());
                // }).inner
                const TABS: &[Tab<'static, SettingsTab>] = &[
                    Tab::new("General", SettingsTab::General),
                    Tab::new("Projects", SettingsTab::Projects),
                    Tab::new("Plugins", SettingsTab::Plugins),
                    Tab::new("Templates", SettingsTab::Templates),
                    Tab::new("Style", SettingsTab::Style),
                ];
                let tabs_resp = Tabs::new(&mut self.settings_tab_index, TABS)
                    .with_text_align(Align::Center)
                    .with_size_mode(TabSizeMode::Grow)
                    .show(ui, |index, tab, ui| {
                        crate::app::set_style(ui.style_mut());
                        // ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                        // ui.set_min_height(300.0);
                        // ui.set_min_size(ui.max_rect().size());
                        let (avail, _) = ui.allocate_exact_size(vec2(600.0, 460.0), Sense::empty());

                        let center_rect = Rect::from_min_max(
                            avail.min,
                            avail.right_bottom() - vec2(0.0, 24.0),
                        );

                        let bottom_rect = Rect::from_min_max(
                            center_rect.left_bottom(),
                            avail.right_bottom(),
                        );

                        let top_resp = ui.put(center_rect, |ui: &mut Ui| {
                            let final_resp = ui.with_layout(Layout::default(), |ui| {
                                // let resp = ui.allocate_response(Vec2::ZERO, Sense::empty());
                                let resp = match tab {
                                    SettingsTab::General => {
                                        // ui.add(&mut self.settings_copy.general)
                                        self.general_gui.ui(&mut self.settings_copy.general, ui)
                                    }
                                    SettingsTab::Projects => {
                                        // ui.add(&mut self.settings_copy.projects)
                                        self.projects_gui.ui(&mut self.settings_copy.projects, ui)
                                    }
                                    SettingsTab::Plugins => {
                                        ui.allocate_response(Vec2::ZERO, Sense::empty())
                                    }
                                    SettingsTab::Templates => {
                                        ui.allocate_response(Vec2::ZERO, Sense::empty())
                                    }
                                    SettingsTab::Style => {
                                        ui.allocate_response(Vec2::ZERO, Sense::empty())
                                    }
                                };
                                resp
                            }).inner;
                            final_resp
                        });
                        
                        let bottom_shrink = bottom_rect.shrink(4.0);
                        let bottom_resp = ui.put(bottom_shrink, |ui: &mut Ui| {
                            ui.horizontal(|ui| {
                                let apply = ui.add_enabled_ui(self.edit_state.needs_update(), |ui| ui.button("Apply")).inner;
                                if apply.clicked() {
                                    if self.edit_state.needs_update() {
                                        if let Ok(_) = apply_settings(original_settings, &self.settings_copy, app_data) {
                                            self.edit_state = EditState::Synced;
                                        } else {
                                            eprintln!("Failed to save settings.");
                                        }
                                    }
                                }
                                let close = ui.button("Close");
                                if close.clicked() {
                                    self.request_close = true;
                                }
                                // let defaults = ui.button("Defaults");
                                // if defaults.clicked() {
                                //     self.settings_copy.apply_settings(&Settings::default());
                                //     self.edit_state = EditState::Modified;
                                // }
                                let reset = if self.edit_state.needs_update() {
                                    let reset = ui.button("Reset");
                                    if reset.clicked() {
                                        if self.edit_state.needs_update() {
                                            self.settings_copy.apply_settings(&original_settings);
                                        }
                                        self.edit_state = EditState::Unaltered;
                                    }
                                    reset
                                } else {
                                    ui.allocate_response(Vec2::ZERO, Sense::empty())
                                };
                                match self.edit_state {
                                    EditState::Modified => {
                                        ui.label("Settings have changed. Apply settings before closing.");
                                    },
                                    EditState::Unaltered => (),
                                    EditState::Synced => {
                                        ui.label("Synced");
                                    },
                                }
                                apply.union(reset.union(close))
                            }).inner
                        });
                            
                        top_resp
                    });
                if tabs_resp.changed() {
                    if *original_settings != self.settings_copy {
                        self.edit_state = EditState::Modified;
                    } else {
                        self.edit_state = EditState::Unaltered;
                    }
                }
                // ui.painter().rect_stroke(avail, CornerRadius::ZERO, Stroke::new(1.0, Color32::GRAY), StrokeKind::Inside);
                tabs_resp
            }).inner
    }
}

pub struct GeneralGui {

}

pub struct ProjectsGui {
    projects_tab: ProjectType,
    tab_index: usize,
}

impl GeneralGui {
    pub fn ui(&mut self, general: &mut General, ui: &mut Ui) -> Response {
        let avail = ui.available_rect_before_wrap();
        ScrollArea::vertical()
            .auto_shrink(Vec2b::FALSE)
            .show(ui, |ui| {
                let grid_resp = Grid::new("general_settings")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        let mut resp = ResponseUpdater::new();

                        ui.rtl_label(Align::Center, "Open After Create")
                            .on_hover_text("Open projects in VS Code after they are created.");
                        let open_after_create = ui.checkbox(&mut general.open_after_create, "");
                        resp.merge(open_after_create);
                        ui.end_row();

                        ui.rtl_label(Align::Center, "Close After Open")
                            .on_hover_text("Close the window after opening a project");
                        let close_after_open = ui.checkbox(&mut general.close_after_open, "");
                        resp.merge(close_after_open);
                        ui.end_row();
            
                        ui.rtl_label(Align::Center, "Startup Projects Tab")
                            .on_hover_text("The default projects tab.");
                        // let before = general.default_projects_tab;
                        let startup_projects_tab = ComboBox::new("start_projects_tab_combo", "")
                            .selected_text(general.default_projects_tab.text())
                            .show_ui(ui, |ui| {
                                let rust_label = ui.selectable_value(&mut general.default_projects_tab, MainTab::Project(ProjectType::Rust), "Rust");
                                let python_label = ui.selectable_value(&mut general.default_projects_tab, MainTab::Project(ProjectType::Python), "Python");
                                let web_label = ui.selectable_value(&mut general.default_projects_tab, MainTab::Project(ProjectType::Web), "Web");
                                rust_label.union(python_label).union(web_label)
                            }).inner;
                        // if general.default_projects_tab != before {
                        //     startup_projects_tab.mark_changed();
                        // }
                        if let Some(slt_resp) = startup_projects_tab {
                            resp.merge(slt_resp);
                        }
                        // resp.merge(startup_projects_tab);
                        ui.end_row();
                        
                        
                        resp.finish()
                    }).inner;
                let p = ui.painter().with_clip_rect(avail);
                let (rect, _) = ui.allocate_exact_size(vec2(50.0, 600.0), Sense::empty());
                p.rect_filled(rect, CornerRadius::ZERO, Color32::LIGHT_RED);
                grid_resp
            }).inner
    }
}

impl ProjectsGui {
    pub fn ui(&mut self, projects: &mut Projects, ui: &mut Ui) -> Response {
        const TABS: &[Tab<'static, ProjectType>] = &[
            Tab::new("Rust", ProjectType::Rust),
            Tab::new("Python", ProjectType::Python),
            Tab::new("Web", ProjectType::Web),
        ];
        let tab_index_id = Id::new("projects_gui_tab_index");
        let tab_resp = Tabs::new(&mut self.tab_index, TABS)
            .with_text_align(Align::Center)
            .with_size_mode(TabSizeMode::Grow)
            .show(ui, |tab_index, project_type, ui| {
                match project_type {
                    ProjectType::Rust => {
                        Grid::new("projects_settings")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            let mut resp_updater = ResponseUpdater::new();
                            // resp.merge(ui.allocate_response(Vec2::ZERO, Sense::empty()));

                            ui.label("Mark changed.");
                            let mut btn = ui.button("Mark Changed");
                            if btn.clicked() {
                                btn.mark_changed();
                            }
                            resp_updater.merge(btn);
                            
                            ui.end_row();

                            ui.rtl_label(Align::Center, "Test Scroll Area");
                            ui.vertical(|ui| {
                                let _u = ScrollArea::new(Vec2b::new(false, true))
                                .auto_shrink(Vec2b::new(false, true))
                                // .max_width(200.0)
                                .show_rows(ui, 32.0, projects.rust.project_directories.len(), |ui, range| {
                                    for i in range {
                                        ui.label(format!("{}", projects.rust.project_directories[i].display()));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if ui.button("Add Path").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            projects.rust.project_directories.push(path);
                                            let mut resp = ui.allocate_response(Vec2::ZERO, Sense::empty());
                                            resp.mark_changed();
                                            resp_updater.merge(resp);
                                        }
                                    }
                                });
                            });
                            ui.end_row();

                            resp_updater.finish()
                        }).inner
                    }
                    ProjectType::Python => {
                        ui.allocate_response(Vec2::ZERO, Sense::empty())
                    }
                    ProjectType::Web => {
                        ui.allocate_response(Vec2::ZERO, Sense::empty())
                    }
                }
            });
        tab_resp
    }
}

// impl Widget for &mut General {
//     fn ui(self, ui: &mut Ui) -> Response {
//         let avail = ui.available_rect_before_wrap();
//         ScrollArea::vertical()
//             .auto_shrink(Vec2b::FALSE)
//             .show(ui, |ui| {
//                 let grid_resp = Grid::new("general_settings")
//                     .num_columns(2)
//                     .striped(true)
//                     .show(ui, |ui| {
//                         let mut resp = ResponseUpdater::new();
                        
//                         ui.rtl_label(Align::Center, "Close After Open")
//                             .on_hover_text("Close the window after opening a project");
//                         let close_after_open = ui.checkbox(&mut self.close_after_open, "");
//                         resp.merge(close_after_open);
//                         ui.end_row();
            
//                         ui.rtl_label(Align::Center, "Startup Projects Tab")
//                             .on_hover_text("The default projects tab.");
//                         let startup_language_tab = ComboBox::new("start_projects_tab_combo", "")
//                             .selected_text(self.default_language_tab.text())
//                             .show_ui(ui, |ui| {
//                                 let rust_label = ui.selectable_value(&mut self.default_language_tab, MainTab::Project(ProjectType::Rust), "Rust");
//                                 let python_label = ui.selectable_value(&mut self.default_language_tab, MainTab::Project(ProjectType::Python), "Python");
//                                 rust_label.union(python_label)
//                             }).inner;
//                         if let Some(slt_resp) = startup_language_tab {
//                             resp.merge(slt_resp);
//                         }
//                         ui.end_row();
                        
                        
//                         resp.finish()
//                     }).inner;
//                 let p = ui.painter().with_clip_rect(avail);
//                 let (rect, _) = ui.allocate_exact_size(vec2(50.0, 600.0), Sense::empty());
//                 p.rect_filled(rect, CornerRadius::ZERO, Color32::LIGHT_RED);
//                 grid_resp
//             }).inner
//     }
// }

impl Widget for &mut Projects {
    fn ui(self, ui: &mut Ui) -> Response {
        Grid::new("projects_settings")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let mut resp_updater = ResponseUpdater::new();
            // resp.merge(ui.allocate_response(Vec2::ZERO, Sense::empty()));

            ui.label("Mark changed.");
            let mut btn = ui.button("Mark Changed");
            if btn.clicked() {
                btn.mark_changed();
            }
            resp_updater.merge(btn);
            
            ui.end_row();

            ui.rtl_label(Align::Center, "Test Scroll Area");
            ui.vertical(|ui| {
                let _u = ScrollArea::new(Vec2b::new(false, true))
                .auto_shrink(Vec2b::new(false, true))
                // .max_width(200.0)
                .show_rows(ui, 32.0, self.rust.project_directories.len(), |ui, range| {
                    for i in range {
                        ui.label(format!("{}", self.rust.project_directories[i].display()));
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Add Path").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.rust.project_directories.push(path);
                            let mut resp = ui.allocate_response(Vec2::ZERO, Sense::empty());
                            resp.mark_changed();
                            resp_updater.merge(resp);
                        }
                    }
                });
            });
            ui.end_row();

            resp_updater.finish()
        }).inner
    }
}

// impl Widget for &mut Settings {
//     fn ui(self, ui: &mut Ui) -> Response {
//         let mut resp = ResponseUpdater::new();

//         let general_response = ui.add(&mut self.general);
//         resp.merge(general_response);

//         resp.finish()
//     }
// }