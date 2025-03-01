#![windows_subsystem = "windows"]
mod app;

use crate::app::MyApp;
use eframe::egui::{self, Vec2};

const APP_TITLE: &str = "Fortune wheel";

fn main() -> eframe::Result<()> {
    // Options
    let window_size = Vec2 {
        x: 1200.0,
        y: 750.0,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(window_size)
            .with_min_inner_size(window_size)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
