mod app;
mod i18n;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Linux Print Studio",
        options,
        Box::new(|_cc| Ok(Box::new(app::PrintStudioApp::default()))),
    )
}
