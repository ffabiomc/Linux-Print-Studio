use eframe::egui;
use std::path::Path;
use std::process::Command;

pub fn render_page(path: &Path, page: usize) -> Result<egui::ColorImage, String> {
    let output = Command::new("pdftoppm")
        .arg("-f")
        .arg(page.to_string())
        .arg("-l")
        .arg(page.to_string())
        .arg("-singlefile")
        .arg("-scale-to")
        .arg("1200")
        .arg("-png")
        .arg(path)
        .arg("-")
        .output()
        .map_err(|error| format!("pdftoppm unavailable: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    if output.stdout.len() > 32 * 1024 * 1024 {
        return Err("Rendered page is too large".to_owned());
    }
    let decoded = image::load_from_memory_with_format(&output.stdout, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?
        .to_rgba8();
    let (width, height) = decoded.dimensions();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        decoded.as_raw(),
    ))
}
