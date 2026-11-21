mod app;
mod calibration;
mod document;
mod duplex;
mod duplex_export;
mod export;
mod i18n;
mod imposition;
mod preview;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Linux Print Studio",
        options,
        Box::new(|_cc| Ok(Box::new(app::PrintStudioApp::default()))),
    )
}
