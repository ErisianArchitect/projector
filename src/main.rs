// #![windows_subsystem = "windows"]

use eframe::{
    NativeOptions,
    egui::{self, *},
};
use projector::app::*;

fn main() {
    eframe::run_native("projector", NativeOptions {
        centered: true,
        persist_window: false,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(vec2(800.0, 800.0))
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    }, Box::new(|cc| Ok(ProjectorApp::boxed_new(cc))))
    .expect("Failed to run app.");
}
