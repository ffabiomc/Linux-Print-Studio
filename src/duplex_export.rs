//! Split an already-imposed PDF into front and back passes.
//! Uses Poppler pdfseparate/pdfunite to preserve vector content.
use crate::duplex::{plan_manual_duplex, FeedOrder};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn split_pdf(
    source: &Path,
    fronts: &Path,
    backs: &Path,
    order: FeedOrder,
) -> Result<(), String> {
    if source == fronts || source == backs || fronts == backs {
        return Err("Input and output paths must all be different".to_owned());
    }
    if fronts.exists() || backs.exists() {
        return Err("Output files already exist; choose a new name".to_owned());
    }
    let document = lopdf::Document::load(source).map_err(|error| error.to_string())?;
    let count = document.get_pages().len();
    if count == 0 || count > 1000 {
        return Err("Expected between 1 and 1000 imposed pages".to_owned());
    }
    let plan = plan_manual_duplex(count, order).map_err(str::to_owned)?;
    let temporary = tempfile::tempdir().map_err(|error| error.to_string())?;
    let pattern = temporary.path().join("page-%d.pdf");
    run("pdfseparate", &[source.to_path_buf(), pattern])?;
    let front_temp = temporary.path().join("front-pass.pdf");
    let back_temp = temporary.path().join("back-pass.pdf");
    merge(temporary.path(), &plan.fronts, &front_temp)?;
    // A one-sided document has no backs: only produce a front PDF.
    if !plan.backs.is_empty() {
        merge(temporary.path(), &plan.backs, &back_temp)?;
    }
    std::fs::rename(&front_temp, fronts).map_err(|error| error.to_string())?;
    if !plan.backs.is_empty() {
        if let Err(error) = std::fs::rename(&back_temp, backs) {
            let _ = std::fs::remove_file(fronts);
            return Err(error.to_string());
        }
    }
    Ok(())
}

fn merge(directory: &Path, numbers: &[usize], destination: &Path) -> Result<(), String> {
    let mut args: Vec<PathBuf> = numbers
        .iter()
        .map(|page| directory.join(format!("page-{page}.pdf")))
        .collect();
    args.push(destination.to_path_buf());
    run("pdfunite", &args)
}

fn run(program: &str, args: &[PathBuf]) -> Result<(), String> {
    let result = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("{program} unavailable: {error}"))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_same_path() {
        let path = Path::new("same.pdf");
        assert!(split_pdf(path, path, Path::new("back.pdf"), FeedOrder::Same).is_err());
    }
}
