use crate::document::PdfDocument;
use crate::i18n::Language;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    Normal,
    Booklet,
    Poster,
    Multiple,
}

pub struct PrintStudioApp {
    language: Language,
    dark_mode: bool,
    document: Option<PdfDocument>,
    page: usize,
    layout: Layout,
    copies: u32,
    status: Option<String>,
}

impl Default for PrintStudioApp {
    fn default() -> Self {
        Self {
            language: Language::Auto,
            dark_mode: true,
            document: None,
            page: 1,
            layout: Layout::Normal,
            copies: 1,
            status: None,
        }
    }
}

impl PrintStudioApp {
    fn open_document(&mut self, path: &std::path::Path) {
        match PdfDocument::open(path) {
            Ok(document) => {
                self.page = 1;
                self.status = None;
                self.document = Some(document);
            }
            Err(error) => self.status = Some(error),
        }
    }
}

impl eframe::App for PrintStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = file.path {
                self.open_document(&path);
            }
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Linux Print Studio");
                ui.separator();
                ui.label(self.language.text("subtitle"));
            });
            ui.horizontal(|ui| {
                ui.label(self.language.text("language"));
                egui::ComboBox::from_id_salt("language")
                    .selected_text(self.language.label())
                    .show_ui(ui, |ui| {
                        for language in [
                            Language::Auto,
                            Language::English,
                            Language::PortugueseBrazil,
                        ] {
                            ui.selectable_value(&mut self.language, language, language.label());
                        }
                    });
                ui.checkbox(&mut self.dark_mode, self.language.text("dark"));
            });
        });

        egui::SidePanel::left("settings")
            .min_width(245.0)
            .show(ctx, |ui| {
                ui.heading(self.language.text("document"));
                ui.label(self.language.text("drop"));
                if let Some(document) = &self.document {
                    ui.label(document.path.display().to_string());
                    ui.label(format!(
                        "{}: {}",
                        self.language.text("pages"),
                        document.pages
                    ));
                }
                ui.separator();
                ui.heading(self.language.text("layout"));
                for (layout, key) in [
                    (Layout::Normal, "normal"),
                    (Layout::Booklet, "booklet"),
                    (Layout::Poster, "poster"),
                    (Layout::Multiple, "multiple"),
                ] {
                    ui.radio_value(&mut self.layout, layout, self.language.text(key));
                }
                ui.separator();
                ui.label(self.language.text("copies"));
                ui.add(egui::DragValue::new(&mut self.copies).range(1..=999));
                ui.separator();
                ui.heading(self.language.text("printer"));
                ui.label(self.language.text("not_ready"));
                ui.add_enabled(false, egui::Button::new(self.language.text("print")));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.language.text("preview"));
            if let Some(document) = &self.document {
                let pages = document.pages;
                ui.horizontal(|ui| {
                    if ui.button("◀").clicked() && self.page > 1 {
                        self.page -= 1;
                    }
                    ui.label(format!("{} / {}", self.page, pages));
                    if ui.button("▶").clicked() && self.page < pages {
                        self.page += 1;
                    }
                });
                ui.separator();
                ui.label(self.language.text("preview_pending"));
            } else {
                ui.label(self.language.text("drop"));
            }
            if let Some(error) = &self.status {
                ui.separator();
                ui.colored_label(egui::Color32::RED, error);
            }
        });
    }
}
