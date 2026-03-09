use eframe::egui::Color32;

use crate::util::atom::Atom;


pub fn tag_text_color(background_color: Color32, light: Color32, dark: Color32) -> Color32 {
    fn normalize_u8(value: u8) -> f32 {
        value as f32 / 255.0
    }
    fn normalize_color(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        (
            normalize_u8(r),
            normalize_u8(g),
            normalize_u8(b),
        )
    }
    fn lrgb(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        fn select(value: f32) -> f32 {
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        let (r, g, b) = normalize_color(r, g, b);
        (
            select(r),
            select(g),
            select(b),
        )

    }
    let [r, g, b, _a] = background_color.to_srgba_unmultiplied();
    let (r, g, b) = lrgb(r, g, b);
    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if lum <= 0.5 {
        light
    } else {
        dark
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tag {
    atom: Atom,
    color: Color32,
}

#[derive(Debug, Clone, Copy)]
pub struct WeightedTag {
    atom: Atom,
    color: Color32,
    weight: f32,
}