use eframe::egui;
use panahon_zh::PanahonApp;

fn main() -> eframe::Result {
    // 桌面预览按 Pixel 10a 的逻辑分辨率开窗，方便和真机截图对齐。
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([412.0, 760.0])
            .with_position([8.0, 30.0])
            .with_min_inner_size([360.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "菲律宾天气",
        options,
        Box::new(|cc| Ok(Box::new(PanahonApp::new(cc)))),
    )
}
