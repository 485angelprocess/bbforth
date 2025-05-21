mod forth;
mod editor;

use eframe::egui::{Style, Visuals};

use editor::Editor;

use eframe;
use egui;

fn main() -> eframe::Result{
    
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "bbforth interpreter",
        options,
        Box::new(|cc| {
            
            let style = Style {
                visuals: Visuals::dark(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            
            Ok(Box::<Editor>::default())
        }),
    )
    
}
