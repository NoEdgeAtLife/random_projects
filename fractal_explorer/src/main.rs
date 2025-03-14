mod app;
mod fractal;
mod color_palette;

use app::FractalExplorer;
use eframe::{egui, NativeOptions};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 800.0)),
        maximized: false,
        vsync: true,
        ..Default::default()
    };
    
    eframe::run_native(
        "Fractal Explorer",
        options,
        Box::new(|cc| Box::new(FractalExplorer::new(cc)))
    )
}