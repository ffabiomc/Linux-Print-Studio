//! Load installed Linux fonts as fallbacks for translated interface text.
//!
//! Fontconfig resolves the actual files so Debian, Fedora and other distributions
//! can use their own Noto (or equivalent) packages without bundled font assets.
use eframe::egui::{self, FontData, FontFamily};
use std::collections::HashSet;
use std::process::Command;

const LANGUAGE_SAMPLES: &[(&str, &str)] = &[
    ("zh-cn", "汉"),
    ("ja", "日"),
    ("hi", "अ"),
    ("bn", "ব"),
    ("ar", "ع"),
    ("ur", "ا"),
    ("el", "Ω"),
    ("ru", "Я"),
];

fn fontconfig_file(language: &str) -> Option<std::path::PathBuf> {
    let output = Command::new("fc-match")
        .args(["-f", "%{file}\n", &format!(":lang={language}")])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8(output.stdout).ok()?;
    let path = std::path::PathBuf::from(path.lines().next()?.trim());
    path.is_file().then_some(path)
}

pub fn install(ctx: &egui::Context) {
    let mut definitions = egui::FontDefinitions::default();
    let mut seen = HashSet::new();
    let mut fallback_names = Vec::new();

    // Use the desktop's preferred sans-serif font for Latin UI text.
    // Retain egui's built-in font as a fallback.
    if let Some(path) = fontconfig_file("pt") {
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("ttf" | "otf")
        ) {
            if let Ok(bytes) = std::fs::read(&path) {
                definitions
                    .font_data
                    .insert("desktop-sans".to_owned(), FontData::from_owned(bytes).into());
                if let Some(fonts) = definitions.families.get_mut(&FontFamily::Proportional) {
                    fonts.insert(0, "desktop-sans".to_owned());
                }
                seen.insert(path);
            }
        }
    }


    for (language, sample) in LANGUAGE_SAMPLES {
        let Some(path) = fontconfig_file(language) else {
            continue;
        };
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        // Avoid registering unsupported font formats; egui parses OpenType/TrueType.
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("ttf" | "otf")
        ) {
            continue;
        }
        let name = format!("system-{language}");
        definitions
            .font_data
            .insert(name.clone(), FontData::from_owned(bytes).into());
        fallback_names.push((name, sample.to_string()));
    }

    // Preserve the default Latin font first and append international fallbacks.
    // Each font is added to both families so buttons and monospace widgets work.
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let fonts = definitions.families.entry(family).or_default();
        for (name, _) in &fallback_names {
            fonts.push(name.clone());
        }
    }
    ctx.set_fonts(definitions);
}

#[cfg(test)]
mod tests {
    #[test]
    fn language_samples_cover_non_latin_scripts() {
        let languages: Vec<_> = super::LANGUAGE_SAMPLES
            .iter()
            .map(|(lang, _)| *lang)
            .collect();
        for required in ["zh-cn", "ja", "hi", "bn", "ar", "ur", "el", "ru"] {
            assert!(languages.contains(&required));
        }
    }
}
