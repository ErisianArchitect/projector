#![cfg_attr(not(feature = "use_terminal"), windows_subsystem = "windows")]
// #![windows_subsystem = "windows"]

use eframe::{
    NativeOptions,
    egui::ViewportBuilder,
};
use image::GenericImageView;
use projector::app::*;

#[must_use]
#[inline(always)]
pub const fn const_or_empty<const CONDITION: bool>(s: &str) -> &str {
    if CONDITION {
        s
    } else {
        ""
    }
}

fn main() -> Result<(), eframe::Error> {
    const ICON: &[u8] = include_bytes!("resources/icon.ico");
    let (width, height, img) = if let Ok(icon) = image::load_from_memory(ICON) {
        let (width, height) = icon.dimensions();
        (
            width,
            height,
            icon.to_rgba8()
        )
    } else {
        panic!("Failed to load image.");
    };
    
    let icon = eframe::egui::IconData {
        width,
        height,
        rgba: img.into_vec()
    };
    
    // This print call is used to show when the subsystem is not "windows".
    println!("Program started.");
    let title = format!(
        "Projector v{}{}",
        projector::VERSION,
        const_or_empty::<{projector::IS_DEBUG}>(" [DEBUG]")
    );
    eframe::run_native(
        "projector",
        NativeOptions {
            centered: true,
            persist_window: false,
            viewport: ViewportBuilder::default()
                .with_inner_size((800.0, 800.0))
                .with_resizable(false)
                .with_maximize_button(false)
                .with_title(title)
                .with_icon(icon),
            ..Default::default()
        },
        Box::new(|cc| Ok(ProjectorApp::boxed_new(cc)))
    )
}