#![allow(unused)]
use std::{any, collections::VecDeque, io::Write, ops::BitOrAssign, os::windows::process::CommandExt, path::{
    Path, PathBuf,
}, process::{Command, CommandArgs}, time::{Duration, Instant}};
use eframe::{
    egui::{self, Style, *}, epaint::tessellator::path, App, CreationContext
};
use crate::{appdata::AppData, dgui::recents::Recent, ext::{BoolExt, CloserBoolExt, UiExt}, project_wizard::ProjectWizard, projects::ProjectPath, util::marker::Marker};
use crate::settings::*;

use crate::{settings::Settings, dgui::{self, tabs::{Tab, TabSizeMode, Tabs}}, projects::ProjectType};

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
pub enum MainTab {
    Main,
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
            MainTab::Main => "Main",
            MainTab::Project(ProjectType::Rust) => "Rust",
            MainTab::Project(ProjectType::Python) => "Python",
            MainTab::Project(ProjectType::Web) => "Web",
            MainTab::Project(ProjectType::Other) => "Other",
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


pub enum ModalUi {
    None,
    Settings(SettingsDialog),
    ProjectWizard(ProjectWizard),
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

    #[inline]
    pub fn settings_tab(settings: Settings, tab: SettingsTab) -> Self {
        Self::Settings(SettingsDialog::from_settings_tab(settings, tab))
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct Persist {
    recent_projects: VecDeque<ProjectPath>
}

impl Default for Persist {
    fn default() -> Self {
        Self {
            recent_projects: VecDeque::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct RecentProjectContext {
    open_editor: bool,
    open_shell: bool,
    open_explorer: bool,
}

impl RecentProjectContext {
    const OFF: Self = Self { open_editor: false, open_explorer: false, open_shell: false };

    #[inline]
    pub const fn clear(&mut self) {
        *self = Self::OFF;
    }

    #[inline]
    pub const fn any(&self) -> bool {
        self.open_editor || self.open_shell || self.open_explorer
    }
}

#[derive(Debug, Default)]
pub struct Runtime {
    recents_search_text: String,
    recent_project_context: RecentProjectContext,
}

pub struct ProjectorApp {
    settings: Settings,
    language_tab: MainTab,
    dialog: ModalUi,
    tab_index: usize,
    quick_edit_text: String,
    app_data: AppData,
    persist: Persist,
    runtime: Runtime,
}

impl ProjectorApp {
    const QUICK_EDIT_CAPACITY: usize = 8192;
    pub fn boxed_new(cc: &CreationContext<'_>) -> Box<Self> {
        cc.egui_ctx.style_mut(|style| {
            set_style(style);
        });
        let app_data = AppData::from("com", "erisianarchitect", "projector").expect("Failed to create AppData object.");
        app_data.ensure_dirs();
        let settings = match app_data.config().load_settings() {
            Ok(settings) => settings,
            Err(err) => {
                eprintln!("Failed to load settings. Loading default settings instead. {err}");
                Settings::default()
            },
        };
        let persist = match app_data.config().load::<_, Persist>(".persist") {
            Ok(mut persist) => {
                // persist.recent_projects.push_back(ProjectPath::Python(PathBuf::from(r#"C:\Users\derek\Documents\code\python\hydra"#)));
                // persist.recent_projects.push_back(ProjectPath::Web(PathBuf::from(r#"C:\Users\derek\Documents\code\web\erisianarchitect"#)));
                // persist.recent_projects.push_back(ProjectPath::Other(PathBuf::from(r#"C:\Users\derek\Documents\code\writeups\region_files"#)));
                persist
            },
            Err(err) => {
                eprintln!("Failed to load persisted data. Loading defaults instead. {err}");
                Persist::default()
            }
        };
        Box::new(Self {
            tab_index: match settings.general.default_projects_tab {
                MainTab::Main => 0,
                MainTab::Project(project_type) => match project_type {
                    ProjectType::Rust => 1,
                    ProjectType::Python => 2,
                    ProjectType::Web => 3,
                    ProjectType::Other => 4,
                },
                MainTab::Text => 5,
            },
            language_tab: settings.general.default_projects_tab,
            settings,
            dialog: ModalUi::None,
            quick_edit_text: String::with_capacity(Self::QUICK_EDIT_CAPACITY),
            app_data,
            persist,
            runtime: Runtime::default(),
        })
    }
}

impl ProjectorApp {
    fn save_internal(&self) {
        match self.app_data.config().save(".persist", &self.persist) {
            Ok(()) => (),
            Err(err) => {
                panic!("Failed to save persist data: {err}");
            },
        }
    }

    fn open_in_editor<P: AsRef<Path>>(&self, path: P) {
        fn inner(app: &ProjectorApp, path: &Path) {
            let editor_cmd = &app.settings.general.editor_command;
            let path_str = format!(r#""{}""#, path.display());
            use strfmt::strfmt;
            let edit_cmd = strfmt!(editor_cmd, path => path_str).unwrap();
            crate::util::execute::execute(&edit_cmd);
        }
        inner(self, path.as_ref());
    }

    fn open_terminal_here<P: AsRef<Path>>(&self, path: P) {
        fn inner(app: &ProjectorApp, path: &Path) {
            let path = if path.is_file() {
                path.parent().expect("Path has no parent.")
            } else {
                path
            };
            let shell_cmd = &app.settings.general.shell_command;
            let path_str = format!(r#""{}""#, path.display());
            use strfmt::strfmt;
            let shell_cmd = strfmt!(shell_cmd, path => path_str).unwrap();
            crate::util::execute::execute(&shell_cmd);
        }
        inner(self, path.as_ref());
    }

    fn reveal_in_file_explorer<P: AsRef<Path>>(&self, path: P) {
        fn inner(app: &ProjectorApp, path: &Path) {
            let path = if path.is_file() {
                path.parent().expect("Path has no parent.")
            } else {
                path
            };
            let explorer_cmd = &app.settings.general.explorer_command;
            let path_str = format!(r#""{}""#, path.display());
            use strfmt::strfmt;
            let explorer_cmd = strfmt!(explorer_cmd, path => path_str).unwrap();
            crate::util::execute::execute(&explorer_cmd);
        }
        inner(self, path.as_ref());
    }
}

impl App for ProjectorApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) { 
        println!("Program Data Saved.");
        self.save_internal();
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        panel::TopBottomPanel::bottom("bottom_panel")
            .frame(Frame::new().stroke(Stroke::NONE))
            .show(ctx, |ui| {
                let frame_bottom = ui.available_rect_before_wrap().bottom();
                ui.horizontal(|ui| {
                    menu::bar(ui, |ui| {
                        let (gear_rect, gear_btn) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                        let gear_style = ui.style().visuals.widgets.style(&gear_btn);
                        ui.painter().text(gear_rect.center(), Align2::CENTER_CENTER, crate::charcons::GEAR2, FontId::monospace(24.0), gear_style.text_color());
                        if gear_btn.clicked() {
                            self.dialog = ModalUi::settings(self.settings.clone());
                        }
                        let gear_btn = gear_btn
                            .on_hover_cursor(CursorIcon::PointingHand);
                        gear_btn.context_menu(|ui| {
                            if ui.button("General").clicked() {
                                self.dialog = ModalUi::settings_tab(self.settings.clone(), SettingsTab::General);
                                ui.close_menu();
                            }
                            if ui.button("Projects").clicked() {
                                self.dialog = ModalUi::settings_tab(self.settings.clone(), SettingsTab::Projects);
                                ui.close_menu();
                            }
                            if ui.button("Licenses").clicked() {
                                self.dialog = ModalUi::settings_tab(self.settings.clone(), SettingsTab::Licenses);
                                ui.close_menu();
                            }
                            if ui.button("Templates").clicked() {
                                self.dialog = ModalUi::settings_tab(self.settings.clone(), SettingsTab::Templates);
                                ui.close_menu();
                            }
                            if ui.button("Style").clicked() {
                                self.dialog = ModalUi::settings_tab(self.settings.clone(), SettingsTab::Style);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Close").clicked() {
                                ui.close_menu();
                            }
                        });

                        if ui.button("Restart").clicked() {
                            self.save_internal();
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                            let curr_exe = std::env::current_exe().expect("Failed to get current exe.");
                            std::process::Command::new(curr_exe).spawn().expect("Failed to spawn process.");
                        }
                        if ui.button("Exit").clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                        if ui.button("Create Project").clicked() {
                            self.dialog = ModalUi::ProjectWizard(ProjectWizard {

                            });
                        }
                    });
                });
            });
        CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
            let close = OwnedCloser::new();
            let mut closer = close.make_closer();
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
                ModalUi::ProjectWizard(wizard) => {
                    wizard.show(
                        closer,
                        &self.settings,
                        ui,
                    );
                }
            }
            if close.is_closed() {
                self.dialog.close();
            }
            const TABS: &[Tab<'static, MainTab>] = &[
                Tab::new("Main", MainTab::Main),
                Tab::new("Rust", MainTab::Project(ProjectType::Rust)),
                Tab::new("Python", MainTab::Project(ProjectType::Python)),
                Tab::new("Web", MainTab::Project(ProjectType::Web)),
                Tab::new("Other", MainTab::Project(ProjectType::Other)),
                Tab::new("Text", MainTab::Text),
            ];
            let mut tab_index = self.tab_index;
            dgui::tabs::Tabs::new(&mut tab_index, TABS)
                .with_size_mode(self.settings.style.tab_size_mode)
                .with_text_align(Align::Center)
                .show(ui, |index, tab, ui| {
                    match tab {
                        MainTab::Main => {
                            if ui.button("Add Directory").clicked() {
                                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                    self.persist.recent_projects.push_back(ProjectPath::Other(dir));
                                }
                            }
                            ui.with_inner_margin(Margin { left: 16, right: 16, top: 16, bottom: 8 }, |ui| {
                                menu::bar(ui, |ui| {
                                    ui.menu_button(crate::charcons::PUSHPIN, |ui| {
                                        if ui.clicked("Test") {
                                            println!("Test");
                                        }
                                    });
                                    ui.pin_btn(ui.spacing().interact_size.y, Color32::WHITE);
                                });
                            });
                            let recents_search = Frame::NONE
                                .inner_margin(Margin { top: 0, left: 16, right: 16, bottom: 0 })
                                .show(ui, |ui| {
                                    Frame::NONE
                                    .stroke(Stroke::new(1.0, Color32::WHITE))
                                    .show(ui, |ui| {
                                        TextEdit::singleline(&mut self.runtime.recents_search_text)
                                            .desired_width(ui.available_width())
                                            .hint_text("Filter")
                                            .show(ui)
                                    }).inner;
                                });
                            ScrollArea::new(Vec2b::new(false, true))
                            .auto_shrink(Vec2b::FALSE)
                            .show(ui, |ui| {
                                ui.with_inner_margin(Margin::symmetric(16, 0), |ui| {
                                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                                    let mut open_editor_toggle = self.runtime.recent_project_context.open_editor;
                                    let mut open_shell_toggle = self.runtime.recent_project_context.open_shell;
                                    let mut open_explorer_toggle = self.runtime.recent_project_context.open_explorer;
                                    let mut remove_index = None;
                                    self.persist.recent_projects.iter().enumerate().for_each(|(index, proj)| {
                                        let path = match proj {
                                            ProjectPath::Rust(path_buf) => path_buf.as_path(),
                                            ProjectPath::Python(path_buf) => path_buf.as_path(),
                                            ProjectPath::Web(path_buf) => path_buf.as_path(),
                                            ProjectPath::Other(path_buf) => path_buf.as_path(),
                                        };
                                        let recent = Recent::new(proj);
                                        let recent_resp = recent.ui(ui);
                                        if recent_resp.clicked() {
                                            self.open_in_editor(path);
                                        }
                                        if recent_resp.clicked_by(PointerButton::Secondary) {
                                            open_editor_toggle = false;
                                            open_shell_toggle = false;
                                            open_explorer_toggle = false;
                                        }
                                        recent_resp.context_menu(|ui| {
                                            ui.horizontal(|ui| {
                                                let close_resp = ui.button("❎");
                                                if close_resp.clicked() {
                                                    ui.close_menu();
                                                }
                                                close_resp.on_hover_text("Close Menu");
                                                if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                                                    ui.add(Label::new(format!("{}", name))
                                                        .halign(Align::Center).selectable(false));
                                                } else {
                                                    ui.colored_label(Color32::RED, "<invalid>");
                                                }
                                            });

                                            ui.separator();
                                            
                                            let mut exec_actions = false;

                                            let reveal_in_explorer = ui.add(
                                                Button::new("🗀 Reveal in File Explorer")
                                                    .corner_radius(CornerRadius::ZERO)
                                                    .selected(open_explorer_toggle)
                                            );
                                            if reveal_in_explorer.secondary_clicked() {
                                                open_explorer_toggle.toggle();
                                            } else if reveal_in_explorer.clicked() {
                                                open_explorer_toggle = true;
                                                exec_actions = true;
                                            }
                                            reveal_in_explorer.on_hover_text(&self.settings.general.explorer_command);
                                            
                                            let open_terminal_here = ui.add(
                                                Button::new("🗖 Open Terminal Here")
                                                    .corner_radius(CornerRadius::ZERO)
                                                    .selected(open_shell_toggle)
                                            );
                                            if open_terminal_here.secondary_clicked() {
                                                open_shell_toggle.toggle();
                                            } else if open_terminal_here.clicked() {
                                                open_shell_toggle = true;
                                                exec_actions = true;
                                            }
                                            open_terminal_here.on_hover_text(&self.settings.general.shell_command);

                                            let open_in_editor = ui.add(
                                                Button::new("✏ Open in Editor")
                                                    .corner_radius(CornerRadius::ZERO)
                                                    .selected(open_editor_toggle)
                                            );
                                            if open_in_editor.secondary_clicked() {
                                                open_editor_toggle.toggle();
                                            }else if open_in_editor.clicked() {
                                                open_editor_toggle = true;
                                                exec_actions = true;
                                            }
                                            open_in_editor.on_hover_text(&self.settings.general.editor_command);
                                            
                                            if exec_actions {
                                                if open_editor_toggle {
                                                    self.open_in_editor(path);
                                                }
                                                if open_explorer_toggle {
                                                    self.reveal_in_file_explorer(path);
                                                }
                                                if open_shell_toggle {
                                                    self.open_terminal_here(path);
                                                }
                                                ui.close_menu();
                                            }
                                            ui.separator();

                                            if ui.button("🗐 Copy Path").clicked() {
                                                ui.ctx().copy_text(format!("{}", path.display()));
                                                ui.close_menu();
                                            }
                                            ui.separator();

                                            if ui.clicked("🗑 Remove") {
                                                remove_index.replace(index);
                                                ui.close_menu();
                                            }
                                        });
                                        recent_resp.on_hover_ui(move |ui| {
                                            let path_str = format!("{}", path.display());
                                            ui.label(&path_str);
                                        });
                                    });
                                    if let Some(index) = remove_index {
                                        self.persist.recent_projects.remove(index);
                                    }

                                    self.runtime.recent_project_context = RecentProjectContext {
                                        open_editor: open_editor_toggle,
                                        open_shell: open_shell_toggle,
                                        open_explorer: open_explorer_toggle,
                                    };
                                });
                                // Frame::NONE
                                // .inner_margin(Margin::symmetric(16, 0))
                                // .show(ui, |ui| {

                                // });
                            });
                        }
                        MainTab::Project(ProjectType::Rust) => {
                            ui.with_inner_margin(Margin::same(16), |ui| {
                                
                            });
                        }
                        MainTab::Project(ProjectType::Python) => {
                            // let (bar_rect, _) = ui.allocate_exact_size(vec2(ui.available_width() / 2.0, 24.0), Sense::empty());
                            // ui.painter().rect_filled(bar_rect, CornerRadius::ZERO, Color32::DARK_GREEN);
                            // fn cont<F: FnOnce(&mut Ui) -> Response>(add_contents: F) -> F {
                            //     add_contents
                            // }
                            ui.with_inner_margin(Margin::same(16), |ui| {
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
                            ui.label("Unfinished.");
                        }
                        MainTab::Project(ProjectType::Other) => {
                            ui.label("Unfinished.");
                        }
                        MainTab::Text => {
                            ui.centered_and_justified(|ui| {
                                Frame::NONE
                                    .inner_margin(Margin::same(8))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.with_layout(Layout::bottom_up(Align::Center).with_cross_justify(true), |ui| {
                                            ui.button("Save");
                                            Frame::NONE
                                            .stroke(Stroke::new(1.0, Color32::DARK_GRAY))
                                            .show(ui, |ui| {
                                                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                                                    ScrollArea::both().show(ui, |ui| {
                                                        TextEdit::multiline(&mut self.quick_edit_text)
                                                            .font(FontId::monospace(16.0))
                                                            // .frame(false)
                                                            .desired_width(ui.available_width())
                                                            .code_editor()
                                                            .hint_text("Enter text here...")
                                                            .lock_focus(true)
                                                            .show(ui);
                                                    });
                                                });
                                            });
                                        });
                                    });
                            });
                        }
                    }
                });
            self.tab_index = tab_index;
        });
    }
}