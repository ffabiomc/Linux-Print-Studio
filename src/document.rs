use std::path::{Path, PathBuf};

pub struct PdfDocument {
    pub path: PathBuf,
    pub pages: usize,
}

impl PdfDocument {
    pub fn open(path: &Path) -> Result<Self, String> {
        if !path.is_file() {
            return Err("File does not exist".to_owned());
        }
        let document = lopdf::Document::load(path).map_err(|error| error.to_string())?;
        let pages = document.get_pages().len();
        if pages == 0 {
            return Err("PDF has no pages".to_owned());
        }
        Ok(Self {
            path: path.to_path_buf(),
            pages,
        })
    }
}
