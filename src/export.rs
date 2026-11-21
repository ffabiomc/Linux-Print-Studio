//! Experimental rasterized PDF imposition exporter.
//! Requires Poppler's pdftoppm. Text is rasterized and is no longer searchable.
use crate::imposition::{self, SheetSide};
use lopdf::{dictionary, Document, Object, Stream};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy)]
pub enum ExportLayout {
    Booklet,
    Multiple(usize),
}

pub fn export_pdf(source: &Path, destination: &Path, pages: usize, layout: ExportLayout) -> Result<(), String> {
    if source == destination {
        return Err("Source and destination must differ".to_owned());
    }
    if pages > 1000 {
        return Err("Export is limited to 1000 source pages".to_owned());
    }
    let sides = match layout {
        ExportLayout::Booklet => imposition::booklet(pages),
        ExportLayout::Multiple(n) => imposition::n_up(pages, n),
    }
    .map_err(str::to_owned)?;
    let landscape = matches!(layout, ExportLayout::Booklet);
    let (paper_w, paper_h) = if landscape { (842.0_f32, 595.0_f32) } else { (595.0, 842.0) };
    let mut pdf = Document::with_version("1.5");
    let pages_id = pdf.new_object_id();
    let mut page_ids = Vec::new();
    for side in &sides {
        let (columns, rows) = grid(side, landscape);
        let cell_w = paper_w / columns as f32;
        let cell_h = paper_h / rows as f32;
        let mut xobjects = lopdf::Dictionary::new();
        let mut commands = String::new();
        for (index, source_page) in side.slots.iter().enumerate() {
            let Some(source_page) = source_page else { continue };
            let (width, height, pixels) = rasterize(source, *source_page)?;
            let image_id = pdf.add_object(Stream::new(
                dictionary! {
                    "Type" => "XObject",
                    "Subtype" => "Image",
                    "Width" => width as i64,
                    "Height" => height as i64,
                    "ColorSpace" => "DeviceRGB",
                    "BitsPerComponent" => 8,
                },
                pixels,
            ));
            let name = format!("Im{index}");
            xobjects.set(name.as_bytes(), image_id);
            let col = index % columns;
            let row = index / columns;
            let margin = 8.0_f32;
            let available_w = (cell_w - margin * 2.0).max(1.0);
            let available_h = (cell_h - margin * 2.0).max(1.0);
            let scale = (available_w / width as f32).min(available_h / height as f32);
            let draw_w = width as f32 * scale;
            let draw_h = height as f32 * scale;
            let x = col as f32 * cell_w + (cell_w - draw_w) / 2.0;
            let y = paper_h - (row as f32 + 1.0) * cell_h + (cell_h - draw_h) / 2.0;
            commands.push_str(&format!(
                "q {draw_w:.3} 0 0 {draw_h:.3} {x:.3} {y:.3} cm /{name} Do Q\n"
            ));
        }
        let resources_id = pdf.add_object(dictionary! { "XObject" => xobjects });
        let content_id = pdf.add_object(Stream::new(dictionary! {}, commands.into_bytes()));
        let page_id = pdf.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), Object::Real(paper_w), Object::Real(paper_h)],
            "Resources" => resources_id,
            "Contents" => content_id,
        });
        page_ids.push(page_id);
    }
    pdf.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
            "Count" => page_ids.len() as i64,
        }),
    );
    let catalog_id = pdf.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    pdf.trailer.set("Root", catalog_id);
    pdf.compress();
    pdf.save(destination).map_err(|error| error.to_string())?;
    Ok(())
}

fn grid(side: &SheetSide, booklet: bool) -> (usize, usize) {
    if booklet {
        return (2, 1);
    }
    match side.slots.len() {
        1 => (1, 1),
        2 => (1, 2),
        4 => (2, 2),
        6 => (2, 3),
        9 => (3, 3),
        16 => (4, 4),
        _ => (1, 1),
    }
}

fn rasterize(path: &Path, page: usize) -> Result<(u32, u32, Vec<u8>), String> {
    let output = Command::new("pdftoppm")
        .args(["-f", &page.to_string(), "-l", &page.to_string(), "-singlefile", "-scale-to", "1200", "-png"])
        .arg(path)
        .arg("-")
        .output()
        .map_err(|error| format!("Cannot launch pdftoppm: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    if output.stdout.len() > 32 * 1024 * 1024 {
        return Err("Rendered PNG exceeds 32 MiB".to_owned());
    }
    let image = image::load_from_memory_with_format(&output.stdout, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?
        .to_rgb8();
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 16_000_000 {
        return Err("Rendered page dimensions are invalid".to_owned());
    }
    Ok((width, height, image.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grids_match_supported_n_up_modes() {
        for (n, expected) in [(1, (1, 1)), (2, (1, 2)), (4, (2, 2)), (6, (2, 3)), (9, (3, 3)), (16, (4, 4))] {
            assert_eq!(grid(&SheetSide { slots: vec![None; n] }, false), expected);
        }
        assert_eq!(grid(&SheetSide { slots: vec![None; 2] }, true), (2, 1));
    }
}
