mod app;
mod calibration;
mod cups;
mod document;
mod duplex;
mod duplex_export;
mod export;
mod fonts;
mod i18n;
mod imposition;
mod preview;
mod profiles;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Linux Print Studio",
        options,
        Box::new(|cc| {
            fonts::install(&cc.egui_ctx);
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::PrintStudioApp::default()))
        }),
    )
}
