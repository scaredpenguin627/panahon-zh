use eframe::egui;
use panahon_zh::PanahonApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 760.0])
            .with_min_inner_size([360.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "PANaHON 中文天气",
        options,
        Box::new(|cc| Ok(Box::new(PanahonApp::new(cc)))),
    )
}
