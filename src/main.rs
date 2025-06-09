use eframe::{
    NativeOptions,
    egui::{self, *},
};
use projector::app::*;

fn main() {
    eframe::run_native("projector", NativeOptions {
        centered: true,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(vec2(800.0, 600.0)),
        ..Default::default()
    }, Box::new(|cc| Ok(ProjectorApp::boxed_new(cc))))
    .expect("Failed to run app.");
}
