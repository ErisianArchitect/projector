#![windows_subsystem = "windows"]

use eframe::{
    NativeOptions,
    egui::ViewportBuilder,
};
use projector::app::*;

fn main() -> Result<(), eframe::Error> {
    let title = format!("Projector v{}{}", projector::VERSION, if projector::IS_DEBUG {
        " [DEBUG]"
    } else {
        ""
    });
    eframe::run_native(
        "projector",
        NativeOptions {
            centered: true,
            persist_window: false,
            viewport: ViewportBuilder::default()
                .with_inner_size((800.0, 800.0))
                .with_resizable(false)
                .with_maximize_button(false)
                .with_title(title),
            ..Default::default()
        },
        Box::new(|cc| Ok(ProjectorApp::boxed_new(cc)))
    )
}