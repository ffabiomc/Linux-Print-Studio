//! A two-page calibration document for testing paper-feed direction.
//! It does not infer printer behavior or send jobs to a printer.
use lopdf::{dictionary, Document, Object, Stream};
use std::path::Path;

pub fn create_calibration_pdf(destination: &Path) -> Result<(), String> {
    if destination.exists() {
        return Err("Output already exists".to_owned());
    }
    let mut pdf = Document::with_version("1.5");
    let pages_id = pdf.new_object_id();
    let font_id = pdf.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = pdf.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let mut page_ids = Vec::new();
    for (number, title) in [(1, "FRONT / FRENTE"), (2, "BACK / VERSO")] {
        let content = format!(
            "0.8 w 36 36 523 770 re S\n             36 421 m 559 421 l S\n             297 36 m 297 806 l S\n             BT /F1 22 Tf 80 720 Td ({title}) Tj ET\n             BT /F1 14 Tf 80 680 Td (TOP / TOPO - THIS EDGE FIRST) Tj ET\n             BT /F1 14 Tf 80 140 Td (BOTTOM / BASE) Tj ET\n             BT /F1 48 Tf 260 420 Td ({number}) Tj ET\n             BT /F1 12 Tf 80 90 Td (Test with one sheet only.) Tj ET\n"
        );
        let content_id = pdf.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let page_id = pdf.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), Object::Integer(595), Object::Integer(842)],
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
            "Count" => 2,
        }),
    );
    let catalog_id = pdf.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    pdf.trailer.set("Root", catalog_id);
    pdf.save(destination).map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_two_page_pdf() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("calibration.pdf");
        create_calibration_pdf(&path).unwrap();
        let doc = Document::load(&path).unwrap();
        assert_eq!(doc.get_pages().len(), 2);
        assert!(create_calibration_pdf(&path).is_err());
    }
}
