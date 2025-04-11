use std::io::Result;

mod process;
mod window;

fn main() -> Result<()> {    
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Rem Engine",
        options,
        Box::new(|_cc| Ok(Box::<window::MyApp>::default())),
    );    
    Ok(())
}
