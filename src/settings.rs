use std::{path::{Path, PathBuf}, sync::atomic::{AtomicBool, Ordering}};
use eframe::{
    egui::{self, *},
};

use crate::{
    app::{
        MainTab, ModalUi,
    }, appdata::AppData, dgui::tabs::{Tab, TabSizeMode, Tabs}, ext::UiExt,
    util::{
        alt::Alternator, marker::*
    },
    projects::{
        ProjectType,
    },
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Encode, bincode::Decode)]
pub enum IncludePathTypes {
    Files = 1,
    Directories = 2,
    FilesAndDirectories = 3,
}

impl IncludePathTypes {
    #[inline]
    pub const fn text(self) -> &'static str {
        match self {
            IncludePathTypes::Files => "Files",
            IncludePathTypes::Directories => "Directories",
            IncludePathTypes::FilesAndDirectories => "Files and Directories",
        }
    }

    #[inline]
    pub const fn include_files(self) -> bool {
        ((self as u8) & 1) != 0
    }

    #[inline]
    pub const fn include_directories(self) -> bool {
        ((self as u8) & 2) != 0
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
    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct General {
        pub open_after_create: bool = true,
        pub close_after_open: bool = false,
        pub default_projects_tab: MainTab = MainTab::Main,
        pub editor_command: String = String::from("code {path}"),
        pub shell_command: String = String::from(if cfg!(target_os = "windows") {
            "wt.exe --startingDirectory {path}"
        } else if cfg!(target_os = "linux") {
            "konsole --workdir {path}"
        } else {
            ""
        }),
        pub explorer_command: String = String::from(if cfg!(target_os = "windows") {
            "explorer.exe {path}"
        } else {
            ""
        }),
        pub dummy_string: String = String::from("dummy"),
        pub dummy_toggle: bool = false,
        pub clicker_counter: u64 = 0,
        pub dummy_number: u64 = 0,
    }

    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct Rust {
        pub editor_command: String = String::new(),
        pub project_directories: Vec<PathBuf> = Vec::new(),
        pub include_path_types: IncludePathTypes = IncludePathTypes::Directories,
        pub restrict_extensions: bool = false,
        pub include_extensions: Vec<String> = vec![
            String::from("rs"),
        ],
    }

    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct Python {
        pub editor_command: String = String::new(),
        pub project_directories: Vec<PathBuf> = Vec::new(),
        pub include_path_types: IncludePathTypes = IncludePathTypes::FilesAndDirectories,
        pub include_extensions: Vec<String> = vec![
            String::from("py"),
            String::from("pyw"),
            String::from("pyi"),
        ],
    }

    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct Web {
        pub editor_command: String = String::new(),
        pub project_directories: Vec<PathBuf> = Vec::new(),
        pub include_path_types: IncludePathTypes = IncludePathTypes::FilesAndDirectories,
        pub include_extensions: Vec<String> = vec![
            String::from("html"),
            String::from("htm"),
            String::from("xhtml"),
            String::from("js"),
            String::from("mjs"),
            String::from("ts"),
            String::from("jsx"),
            String::from("tsx"),
            String::from("css"),
            String::from("scss"),
            String::from("sass"),
            String::from("less"),
            String::from("json"),
            String::from("xml"),
            String::from("yaml"),
            String::from("yml"),
            String::from("env"),
            String::from("wasm"),
            String::from("php"),
            String::from("asp"),
            String::from("aspx"),
            String::from("jsp"),
            String::from("cgi"),
            String::from("py"),
        ],
    }
    
    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct Projects {
        pub rust: Rust = Rust::default(),
        pub python: Python = Python::default(),
        pub web: Web = Web::default(),
    }

    // plugins
    // templates,
    // style
    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct Style {
        pub tab_size_mode: TabSizeMode = TabSizeMode::Grow,
    }

    #[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
    pub struct Settings {
        pub general: General = General::default(),
        pub projects: Projects = Projects::default(),
        // plugins
        // templates
        // style
        pub style: Style = Style::default(),
    }
}

impl Settings {
    #[inline]
    pub fn create_settings_modal(&self) -> ModalUi {
        ModalUi::Settings(SettingsDialog::from_settings(self.clone()))
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        self.clone_from(settings);
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettingsTab {
    General = 0,
    Projects = 1,
    Licenses = 2,
    Templates = 3,
    Style = 4,
}

impl SettingsTab {
    pub const fn tab_index(self) -> usize {
        self as usize
    }
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

#[repr(transparent)]
pub struct BaseCloser<T> {
    close: T,
}

pub type OwnedCloser = BaseCloser<AtomicBool>;
pub type Closer<'a> = BaseCloser<&'a BaseCloser<AtomicBool>>;

impl OwnedCloser {
    #[inline]
    pub fn new() -> Self {
        Self {
            close: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn close(&self) -> bool {
        !self.close.swap(true, Ordering::Relaxed)
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.close.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn make_closer(&self) -> Closer {
        Closer::new(self)
    }
}

impl<'a> Closer<'a> {
    pub fn new(closer: &'a OwnedCloser) -> Self {
        Self {
            close: closer,
        }
    }

    #[inline]
    pub fn close(self) -> bool {
        self.close.close()
    }

    #[inline]
    pub fn is_closed(self) -> bool {
        self.close.is_closed()
    }
}

impl<'a> Clone for Closer<'a> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            close: self.close,
        }
    }
}

impl<'a> Copy for Closer<'a> {}

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
    pub settings_copy: Settings,
    pub settings_tab_index: usize,
    pub edit_state: EditState,
    pub request_close: bool,
    pub general_gui: GeneralGui,
    pub projects_gui: ProjectsGui,
    pub style_gui: StyleGui,
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
                // projects_tab: ProjectType::Rust,
                tab_index: 0,
            },
            style_gui: StyleGui {

            }
        }
    }

    pub fn from_settings_tab(settings: Settings, tab: SettingsTab) -> Self {
        Self {
            settings_copy: settings,
            settings_tab_index: tab.tab_index(),
            edit_state: EditState::Unaltered,
            request_close: false,
            general_gui: GeneralGui {

            },
            projects_gui: ProjectsGui {
                tab_index: 0,
            },
            style_gui: StyleGui {

            },
        }
    }

    pub fn show(
        &mut self,
        closer: Closer<'_>,
        app_data: &mut AppData,
        original_settings: &mut Settings,
        ui: &mut Ui,
    ) -> bool {
        fn apply_settings(
            target: &mut Settings,
            settings: &Settings,
            app_data: &mut AppData,
        ) -> crate::error::Result<()> {
            target.apply_settings(&settings);
            app_data.config().save_settings(target)
        }
        let save = ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::S))
        });
        if save {
            if self.edit_state.needs_update() {
                if let Ok(_) = apply_settings(original_settings, &self.settings_copy, app_data) {
                    self.edit_state = EditState::Synced;
                } else {
                    eprintln!("Failed to save settings.");
                }
            }
        }
        let change_marker = marker();
        let changed = change_marker.mark_only();
        modal::Modal::new(Id::new("settings_dialog_modal"))
            .area(
                Area::new(Id::new("settings_dialog_modal_area"))
                    .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            )
            .frame(
                Frame::NONE
                    .fill(Color32::ORANGE)
                    // .stroke(Stroke::new(1.0, Color32::GRAY))
            )
            .show(ui.ctx(), |ui| {
                // settings dialog size
                const SETTINGS_DIALOG_SIZE: Vec2 = Vec2::new(700.0, 700.0);
                ui.set_min_size(SETTINGS_DIALOG_SIZE);
                ui.set_max_size(SETTINGS_DIALOG_SIZE);
                const TABS: &[Tab<'static, SettingsTab>] = &[
                    Tab::new("General", SettingsTab::General),
                    Tab::new("Projects", SettingsTab::Projects),
                    Tab::new("Licenses", SettingsTab::Licenses),
                    Tab::new("Templates", SettingsTab::Templates),
                    Tab::new("Style", SettingsTab::Style),
                    // Tab::new("Other", SettingsTab::Extended("other")),
                    // Tab::new("End", SettingsTab::Extended("end")),
                ];
                Tabs::new(&mut self.settings_tab_index, TABS)
                    .with_text_align(Align::Center)
                    .with_size_mode(TabSizeMode::Grow)
                    .show(ui, |_index, tab, ui| {
                        // crate::app::set_style(ui.style_mut());
                        // ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                        // ui.set_min_height(300.0);
                        // ui.set_min_size(ui.max_rect().size());
                        // let (avail, _) = ui.allocate_exact_size(vec2(750.0, 460.0), Sense::empty());
                        let avail = ui.available_rect_before_wrap();

                        let center_rect = Rect::from_min_max(
                            avail.min,
                            avail.right_bottom() - vec2(0.0, 24.0),
                        );

                        let bottom_rect = Rect::from_min_max(
                            center_rect.left_bottom(),
                            avail.right_bottom(),
                        );

                        ui.allocate_new_ui(UiBuilder::new().max_rect(center_rect), |ui| {
                            match tab {
                                SettingsTab::General => {
                                    self.general_gui.ui(changed, &mut self.settings_copy.general, ui);
                                }
                                SettingsTab::Projects => {
                                    self.projects_gui.ui(changed, &mut self.settings_copy.projects, ui);
                                }
                                SettingsTab::Licenses => {}
                                SettingsTab::Templates => {}
                                SettingsTab::Style => {
                                    Frame::NONE
                                        .inner_margin(Margin::same(8))
                                        .show(ui, |ui| {
                                            self.style_gui.ui(changed, &mut self.settings_copy.style, ui)
                                        });
                                }
                            }
                            // let final_resp = ui.with_layout(Layout::default(), |ui| {
                            //     // let resp = ui.allocate_response(Vec2::ZERO, Sense::empty());
                            // }).inner;
                            // final_resp
                        });
                        
                        let bottom_shrink = bottom_rect.shrink(4.0);
                        ui.allocate_new_ui(UiBuilder::new().max_rect(bottom_shrink), |ui| {
                            ui.horizontal(|ui| {
                                let win_pos = bottom_rect.left_bottom();
                                let close = ui.button("Close");
                                let esc_pressed = ui.input_mut(|i| {
                                    i.consume_key(Modifiers::NONE, Key::Escape)
                                });
                                if esc_pressed || close.clicked() {
                                    self.request_close = true;
                                }
                                if self.request_close {
                                    if self.edit_state.needs_update() {
                                        let frame = Frame::NONE.fill(ui.style().visuals.panel_fill).corner_radius(CornerRadius::ZERO);
                                        // let padding = frame.inner_margin.sum().y + frame.outer_margin.sum().y;
                                        let int_size = ui.style().spacing.interact_size;
                                        Modal::new(Id::new("request_close_modal"))
                                            .area(Area::new(Id::new("request_close_modal_area"))
                                                // .anchor(Align2::LEFT_TOP, win_pos - vec2(0.0, 60.0))
                                                .fixed_pos(win_pos)
                                                .pivot(Align2::LEFT_BOTTOM)
                                            ).frame(frame)
                                            .show(ui.ctx(), |ui| {
                                                // crate::app::set_style(ui.style_mut());
                                                ui.set_min_size(vec2(240.0, 60.0));
                                                ui.set_max_size(vec2(240.0, 60.0));
                                                let avail = ui.available_rect_before_wrap();
                                                let msg_rect = Rect::from_min_max(
                                                    avail.min,
                                                    pos2(avail.max.x, avail.max.y - (int_size.y + 8.0)),
                                                );
                                                let btns_rect = Rect::from_min_max(
                                                    msg_rect.left_bottom(),
                                                    avail.max,
                                                );

                                                let btn_width = btns_rect.width() / 3.0;

                                                let save_rect = Rect::from_min_max(
                                                    btns_rect.min,
                                                    pos2(btns_rect.min.x + btn_width, btns_rect.max.y),
                                                );

                                                let discard_rect = Rect::from_min_max(
                                                    save_rect.right_top(),
                                                    pos2(save_rect.max.x + btn_width, btns_rect.max.y),
                                                );

                                                let cancel_rect = Rect::from_min_max(
                                                    discard_rect.right_top(),
                                                    btns_rect.max,
                                                );

                                                let save_btn = ui.put(save_rect.shrink(4.0), Button::new("Save"));
                                                let discard_btn = ui.put(discard_rect.shrink(4.0), Button::new("Discard"));
                                                let cancel_btn = ui.put(cancel_rect.shrink(4.0), Button::new("Cancel"));

                                                if save_btn.clicked() {
                                                    if let Ok(_) = apply_settings(original_settings, &self.settings_copy, app_data) {
                                                        self.edit_state = EditState::Synced;
                                                    } else {
                                                        eprintln!("Failed to save settings.");
                                                    }
                                                }
                                                if discard_btn.clicked() {
                                                    closer.close();
                                                }
                                                if cancel_btn.clicked() {
                                                    self.request_close = false;
                                                }

                                                let p = ui.painter_at(msg_rect);
                                                p.text(msg_rect.center(), Align2::CENTER_CENTER, "Settings have been modified.", FontId::proportional(16.0), ui.style().visuals.widgets.active.text_color());
                                            });
                                    } else {
                                        closer.close();
                                    }
                                }
                                // let defaults = ui.button("Defaults");
                                // if defaults.clicked() {
                                //     self.settings_copy.apply_settings(&Settings::default());
                                //     self.edit_state = EditState::Modified;
                                // }
                                if self.edit_state.needs_update() {
                                    let save_btn = ui.button("Save");
                                    if save_btn.clicked() {
                                        if self.edit_state.needs_update() {
                                            if let Ok(_) = apply_settings(original_settings, &self.settings_copy, app_data) {
                                                self.edit_state = EditState::Synced;
                                            } else {
                                                eprintln!("Failed to save settings.");
                                            }
                                        }
                                    }

                                    let save_and_close_btn = ui.button("Save and Close");
                                    if save_and_close_btn.clicked() {
                                        if self.edit_state.needs_update() {
                                            if let Ok(_) = apply_settings(original_settings, &self.settings_copy, app_data) {
                                                self.edit_state = EditState::Synced;
                                                self.request_close = true;
                                            } else {
                                                eprintln!("Failed to save settings.");
                                            }
                                        }
                                    }

                                    let discard_changes = ui.button("Discard Changes");
                                    if discard_changes.clicked() {
                                        if self.edit_state.needs_update() {
                                            self.settings_copy.apply_settings(&original_settings);
                                        }
                                        self.edit_state = EditState::Unaltered;
                                    }
                                }
                                match self.edit_state {
                                    EditState::Modified => {
                                        ui.add(
                                            Label::new("Modified")
                                                .selectable(false)
                                        ).on_hover_cursor(CursorIcon::Default);
                                    },
                                    EditState::Unaltered => (),
                                    EditState::Synced => {
                                        ui.add(
                                            Label::new("Synced")
                                                .selectable(false)
                                        ).on_hover_cursor(CursorIcon::Default);
                                    },
                                }
                            });
                        });
                    });
                if changed.is_marked() {
                    if *original_settings != self.settings_copy {
                        self.edit_state = EditState::Modified;
                    } else {
                        self.edit_state = EditState::Unaltered;
                    }
                }
                // ui.painter().rect_stroke(avail, CornerRadius::ZERO, Stroke::new(1.0, Color32::GRAY), StrokeKind::Inside);
            }); // Modal [...] .show(ui.ctx(), |ui| {
        changed.is_marked()
    }
}

pub struct GeneralGui {

}

pub struct ProjectsGui {
    // projects_tab: ProjectType,
    tab_index: usize,
}

pub struct StyleGui {

}

const LABEL_WIDTH: f32 = 180.0;

impl GeneralGui {
    pub fn ui(
        &mut self,
        changed: MarkOnly,
        general: &mut General,
        ui: &mut Ui,
    ) {
        // let avail = ui.available_rect_before_wrap();
        let record_change = changed.response_marker_fn();
        ui.centered_and_justified(|ui| {
            Frame::NONE
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let mut alt = Alternator::new(Color32::TRANSPARENT, ui.style().visuals.faint_bg_color);
                // alt.alternate();
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Open After Create",
                    "Open projects in editor after they are created.",
                    alt.next(),
                    |ui| {
                        record_change(&ui.toggle_box(&mut general.open_after_create));
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Close After Open",
                    "Close the window after opening a project",
                    alt.next(),
                    |ui| {
                        record_change(&ui.toggle_box(&mut general.close_after_open));
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Startup Tab",
                    "The tab that is open when the program starts.",
                    alt.next(),
                    |ui| {
                        record_change(&ComboBox::new("start_projects_tab_combo", "")
                            .selected_text(general.default_projects_tab.text())
                            .show_ui(ui, |ui| {
                                let projects_tab = &mut general.default_projects_tab;
                                let mut selectable = move |tab: MainTab| {
                                    let select_resp = ui.selectable_value(projects_tab, tab, tab.text());
                                    record_change(&select_resp);
                                };
                                selectable(MainTab::Main);
                                selectable(MainTab::Project(ProjectType::Rust));
                                selectable(MainTab::Project(ProjectType::Python));
                                selectable(MainTab::Project(ProjectType::Web));
                                selectable(MainTab::Project(ProjectType::Other));
                                selectable(MainTab::Text);
                            }).response);
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Editor Command",
                    "The command that is executed to open a project path.\nUse `{path}` (without backticks) as a placeholder for the formatter.\nUse `{{` and `}}` to escape `{` and `}`.\nYou do not need to put quotes around `{path}`.",
                    alt.next(),
                    |ui| {
                        let edit = TextEdit::singleline(&mut general.editor_command)
                            .desired_width(ui.available_width());
                        record_change(&ui.add(edit));
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Open Shell Command",
                    "The command to open an external shell.\nUse `{path}` (without backticks) as a placeholder for the formatter.\nUse `{{` and `}}` to escape `{` and `}`.\nYou do not need to put quotes around `{path}`.",
                    alt.next(),
                    |ui| {
                        let edit = TextEdit::singleline(&mut general.shell_command)
                            .desired_width(ui.available_width());
                        record_change(&ui.add(edit));
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "File Explorer Command",
                    "The command that is executed to open the file explorer.\nUse `{path}` (without backticks) as a placeholder for the formatter.\nUse `{{` and `}}` to escape `{` and `}`.\nYou do not need to put quotes around `{path}`.",
                    alt.next(),
                    |ui| {
                        let edit = TextEdit::singleline(&mut general.explorer_command)
                            .desired_width(ui.available_width());
                        record_change(&ui.add(edit));
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Click Counter",
                    format!("This just counts how many times you've clicked the button. You've clicked it {} times.",general.clicker_counter),
                    alt.next(),
                    |ui| {
                        let text = format!("{}", general.clicker_counter);
                        if ui.clicked(text) {
                            general.clicker_counter += 1;
                            changed.mark();
                        }
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Dummy Text",
                    "This field is not used for anything. It is for testing/entertainment purposes.",
                    alt.next(),
                    |ui| {
                        let edit = TextEdit::multiline(&mut general.dummy_string)
                            .desired_width(ui.available_width())
                            .desired_rows(8);
                        record_change(&ui.add(edit));
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Dummy",
                    "Dummy",
                    alt.next(),
                    |ui| {
                        let dummy = ui.toggle_box(&mut general.dummy_toggle);
                        record_change(&dummy);
                    }
                );
                ui.setting_ui(
                    LABEL_WIDTH,
                    "Debug",
                    "Debug",
                    alt.next(),
                    |ui| {
                        let dummy = ui.button("Mark Changed");
                        _=ui.button("Test");
                        if dummy.clicked() {
                            general.dummy_number = general.dummy_number.wrapping_add(1);
                            changed.mark();
                        }
                    }
                );
            });
        });
    }
}

impl ProjectsGui {
    pub fn ui(&mut self, changed: MarkOnly, projects: &mut Projects, ui: &mut Ui) {
        // ui.spacing_mut().window_margin = Margin::ZERO;
        // ui.spacing_mut().menu_margin = Margin::ZERO;
        // let record_change = |resp: &Response| {
        //     changed.mark_if(resp.changed())
        // };
        const TABS: &[Tab<'static, ProjectType>] = &[
            Tab::new("Rust", ProjectType::Rust),
            Tab::new("Python", ProjectType::Python),
            Tab::new("Web", ProjectType::Web),
            Tab::new("Other", ProjectType::Other),
        ];
        Tabs::new(&mut self.tab_index, TABS)
            .with_text_align(Align::Center)
            .with_size_mode(TabSizeMode::Grow)
            .show(ui, |_tab_index, project_type, ui| {
                // ui.set_min_width(ui.available_width());
                match project_type {
                    ProjectType::Rust => {
                        ScrollArea::vertical()
                        .auto_shrink(Vec2b::FALSE)
                        .show(ui, |ui| {
                            Frame::NONE
                            .inner_margin(Margin::same(8))
                            .show(ui, |ui| {
                                let mut alt = Alternator::new(Color32::TRANSPARENT, ui.style().visuals.faint_bg_color);
                                ui.setting_ui(
                                    LABEL_WIDTH,
                                    "Editor Command",
                                    "The editor command used to open Rust projects.\nLeave blank to use the default editor command.",
                                    alt.next(),
                                    |ui| {
                                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                            let clear_btn = Button::new(crate::charcons::XBOX)
                                                .frame(false);

                                            let clear_btn_resp = ui.add(clear_btn);
                                            if clear_btn_resp.clicked() {
                                                projects.rust.editor_command.clear();
                                                changed.mark();
                                            }
                                            clear_btn_resp.on_hover_text("Clear the text field.");
                                            let text_edit = TextEdit::singleline(&mut projects.rust.editor_command)
                                                .desired_width(ui.available_width());
                                            changed.record_change(ui.add(text_edit));
                                        });
                                    }
                                );
                                ui.setting_ui(
                                    LABEL_WIDTH,
                                    "Project Directories",
                                    "The directories that will be searched for sub-directories/files to add to the project browser.",
                                    alt.next(),
                                    |ui| {
                                        let _u = ScrollArea::new(Vec2b::new(false, true))
                                        .auto_shrink(Vec2b::new(false, true))
                                        // .max_width(200.0)
                                        .show(ui, |ui| {
                                            let dirs = projects.rust.project_directories.as_slice();
                                            for dir in dirs {
                                                ui.label(format!("{}", dir.display())).on_hover_cursor(CursorIcon::Default);
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            if ui.button("Add Path").clicked() {
                                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                                    projects.rust.project_directories.push(path);
                                                    changed.mark();
                                                }
                                            }
                                        });
                                    }
                                );
                                ui.setting_ui(
                                    LABEL_WIDTH,
                                    "Include Path Types",
                                    "The types of paths to include.",
                                    alt.next(),
                                    |ui| {
                                        changed.record_change(ComboBox::new("inc_path_ty_combo", "")
                                            .selected_text(projects.rust.include_path_types.text())
                                            .show_ui(ui, |ui| {
                                                changed.record_change(
                                                    ui.selectable_value(&mut projects.rust.include_path_types, IncludePathTypes::Files, "Files")
                                                );
                                                changed.record_change(
                                                    ui.selectable_value(&mut projects.rust.include_path_types, IncludePathTypes::Directories, "Directories")
                                                );
                                                changed.record_change(
                                                    ui.selectable_value(&mut projects.rust.include_path_types, IncludePathTypes::FilesAndDirectories, "Files and Directories")
                                                );
                                            }).response);
                                    }
                                );
                                if projects.rust.include_path_types.include_files() {
                                    ui.setting_ui(
                                        LABEL_WIDTH,
                                        "Restrict Extensions",
                                        "If this is set, that means that only the specified extensions will be included.",
                                        alt.next(),
                                        |ui| {
                                            changed.record_change(ui.toggle_box(&mut projects.rust.restrict_extensions));
                                        }
                                    );
                                    if projects.rust.restrict_extensions {
                                        ui.setting_ui(
                                            LABEL_WIDTH,
                                            "Included Extensions",
                                            "The file extensions that are included.",
                                            alt.next(),
                                            |ui| {
                                                ui.label("Work in progress...");
                                            }
                                        );
                                    }
                                }
                                // Grid::new("projects_settings")
                                // .num_columns(2)
                                // .striped(true)
                                // .show(ui, |ui| {
                                    
                                //     ui.rtl_label(Align::Center, "Project Directories")
                                //         .on_hover_text("The directories that will be searched for sub-directories/files to add to the project browser.");
                                //     ui.vertical(|ui| {
                                //         let _u = ScrollArea::new(Vec2b::new(false, true))
                                //         .auto_shrink(Vec2b::new(false, true))
                                //         // .max_width(200.0)
                                //         .show(ui, |ui| {
                                //             let dirs = projects.rust.project_directories.as_slice();
                                //             for dir in dirs {
                                //                 ui.label(format!("{}", dir.display())).on_hover_cursor(CursorIcon::Default);
                                //             }
                                //         });
                                //         ui.horizontal(|ui| {
                                //             if ui.button("Add Path").clicked() {
                                //                 if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                //                     projects.rust.project_directories.push(path);
                                //                     changed.mark();
                                //                 }
                                //             }
                                //         });
                                //     });
                                //     ui.end_row();
                                // });
                            });
                        });
                    }
                    ProjectType::Python => {}
                    ProjectType::Web => {}
                    ProjectType::Other => {}
                }
            });
    }
}

impl StyleGui {
    pub fn ui(&mut self, changed: MarkOnly, style: &mut Style, ui: &mut Ui) {
        let record_change = move |resp: &Response| {
            changed.mark_if(resp.changed())
        };
        ScrollArea::vertical()
            .auto_shrink(Vec2b::FALSE)
            .show(ui, |ui| {
                ui.setting_ui(LABEL_WIDTH, "Tab Size Mode", "The size mode for the tabs on the main screen.", Color32::TRANSPARENT, |ui| {
                    let tab_size_mode = ComboBox::new("tab_size_combobox", "")
                    .selected_text(match style.tab_size_mode {
                        TabSizeMode::Equal => "Equal",
                        TabSizeMode::Shrink => "Shrink",
                        TabSizeMode::Grow => "Grow",
                        TabSizeMode::Exact(_) => "Exact",
                        TabSizeMode::ShrinkMin(_) => "Shrink Min",
                    })
                    .show_ui(ui, |ui| {
                        let grow = ui.selectable_value(&mut style.tab_size_mode, TabSizeMode::Grow, "Grow");
                        record_change(&grow);
                        let equal = ui.selectable_value(&mut style.tab_size_mode, TabSizeMode::Equal, "Equal");
                        record_change(&equal);
                        let shrink = ui.selectable_value(&mut style.tab_size_mode, TabSizeMode::Shrink, "Shrink");
                        record_change(&shrink);
                    }).response;
                    record_change(&tab_size_mode);
                });
            });
    }
}