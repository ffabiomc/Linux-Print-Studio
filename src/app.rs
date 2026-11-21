use crate::calibration;
use crate::document::PdfDocument;
use crate::duplex::{self, FeedOrder};
use crate::duplex_export;
use crate::export::{self, ExportLayout};
use crate::i18n::Language;
use crate::imposition;
use crate::preview;
use crate::profiles::{self, PrinterProfile};
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
    pages_per_sheet: usize,
    back_feed: FeedOrder,
    profiles: Vec<PrinterProfile>,
    profile_name: String,
    status: Option<String>,
    preview: Option<egui::TextureHandle>,
    preview_page: usize,
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
            pages_per_sheet: 4,
            back_feed: FeedOrder::Reversed,
            profiles: profiles::config_path()
                .and_then(|path| profiles::load(&path))
                .unwrap_or_default(),
            profile_name: String::new(),
            status: None,
            preview: None,
            preview_page: 0,
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
                self.preview = None;
                self.preview_page = 0;
            }
            Err(error) => self.status = Some(error),
        }
    }

    fn refresh_preview(&mut self, ctx: &egui::Context) {
        if self.preview_page == self.page {
            return;
        }
        self.preview = None;
        self.preview_page = self.page;
        if let Some(document) = &self.document {
            match preview::render_page(&document.path, self.page) {
                Ok(image) => {
                    self.preview =
                        Some(ctx.load_texture("pdf-page", image, egui::TextureOptions::LINEAR));
                    self.status = None;
                }
                Err(error) => self.status = Some(error),
            }
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
                if ui.button(self.language.text("open")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .pick_file()
                    {
                        self.open_document(&path);
                    }
                }
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
                if self.layout == Layout::Multiple {
                    egui::ComboBox::from_id_salt("pages_per_sheet")
                        .selected_text(format!("{} / sheet", self.pages_per_sheet))
                        .show_ui(ui, |ui| {
                            for count in [1, 2, 4, 6, 9, 16] {
                                ui.selectable_value(
                                    &mut self.pages_per_sheet,
                                    count,
                                    count.to_string(),
                                );
                            }
                        });
                }
                if let Some(document) = &self.document {
                    let plan = match self.layout {
                        Layout::Booklet => imposition::booklet(document.pages),
                        Layout::Multiple => imposition::n_up(document.pages, self.pages_per_sheet),
                        _ => Ok(Vec::new()),
                    };
                    if let Ok(sides) = plan {
                        if !sides.is_empty() {
                            ui.label(format!("Planned sheet sides: {}", sides.len()));
                            egui::ScrollArea::vertical()
                                .max_height(110.0)
                                .show(ui, |ui| {
                                    for (index, side) in sides.iter().take(20).enumerate() {
                                        let pages = side
                                            .slots
                                            .iter()
                                            .map(|page| {
                                                page.map_or("-".to_owned(), |n| n.to_string())
                                            })
                                            .collect::<Vec<_>>()
                                            .join(" | ");
                                        ui.label(format!("{}: {}", index + 1, pages));
                                    }
                                });
                            ui.small("Experimental raster PDF export available below.");
                        }
                    }
                }
                ui.separator();
                let export_layout = match self.layout {
                    Layout::Booklet => Some(ExportLayout::Booklet),
                    Layout::Multiple => Some(ExportLayout::Multiple(self.pages_per_sheet)),
                    _ => None,
                };
                if ui
                    .add_enabled(
                        self.document.is_some() && export_layout.is_some(),
                        egui::Button::new(self.language.text("export_pdf")),
                    )
                    .clicked()
                {
                    if let (Some(document), Some(layout)) = (&self.document, export_layout) {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PDF", &["pdf"])
                            .set_file_name("print-ready.pdf")
                            .save_file()
                        {
                            self.status = Some(
                                match export::export_pdf(
                                    &document.path,
                                    &path,
                                    document.pages,
                                    layout,
                                ) {
                                    Ok(()) => format!("PDF saved: {}", path.display()),
                                    Err(error) => format!("PDF export failed: {error}"),
                                },
                            );
                        }
                    }
                }
                if let Some(document) = &self.document {
                    let sides = match self.layout {
                        Layout::Booklet => imposition::booklet(document.pages).ok(),
                        Layout::Multiple => {
                            imposition::n_up(document.pages, self.pages_per_sheet).ok()
                        }
                        _ => None,
                    };
                    if let Some(sides) = sides {
                        ui.separator();
                        ui.heading(self.language.text("manual_duplex"));
                        ui.label(self.language.text("profile_name"));
                        ui.text_edit_singleline(&mut self.profile_name);
                        if ui.button(self.language.text("save_profile")).clicked() {
                            let name = self.profile_name.trim().to_owned();
                            let mut updated = self.profiles.clone();
                            if let Some(profile) = updated.iter_mut().find(|profile| profile.name == name) {
                                profile.back_feed = self.back_feed;
                            } else {
                                updated.push(PrinterProfile {
                                    name,
                                    back_feed: self.back_feed,
                                });
                            }
                            self.status = Some(match profiles::config_path()
                                .and_then(|path| profiles::save(&path, &updated)) {
                                Ok(()) => {
                                    self.profiles = updated;
                                    self.language.text("profile_saved").to_owned()
                                }
                                Err(error) => format!("Profile save failed: {error}"),
                            });
                        }
                        egui::ComboBox::from_id_salt("printer_profile")
                            .selected_text(self.language.text("load_profile"))
                            .show_ui(ui, |ui| {
                                for profile in &self.profiles {
                                    if ui.selectable_label(false, &profile.name).clicked() {
                                        self.profile_name = profile.name.clone();
                                        self.back_feed = profile.back_feed;
                                    }
                                }
                            });
                        if ui.button(self.language.text("calibration_pdf")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .set_file_name("duplex-calibration.pdf")
                                .save_file()
                            {
                                self.status =
                                    Some(match calibration::create_calibration_pdf(&path) {
                                        Ok(()) => {
                                            self.language.text("calibration_saved").to_owned()
                                        }
                                        Err(error) => format!("Calibration PDF failed: {error}"),
                                    });
                            }
                        }
                        ui.small(self.language.text("calibration_instructions"));
                        ui.radio_value(
                            &mut self.back_feed,
                            FeedOrder::Reversed,
                            self.language.text("reverse_backs"),
                        );
                        ui.radio_value(
                            &mut self.back_feed,
                            FeedOrder::Same,
                            self.language.text("same_backs"),
                        );
                        if let Ok(plan) = duplex::plan_manual_duplex(sides.len(), self.back_feed) {
                            ui.label(format!(
                                "{}: {}",
                                self.language.text("front_pass"),
                                duplex::page_range(&plan.fronts)
                            ));
                            ui.label(format!(
                                "{}: {}",
                                self.language.text("back_pass"),
                                duplex::page_range(&plan.backs)
                            ));
                            ui.small(self.language.text("duplex_warning"));
                            if ui.button(self.language.text("split_duplex")).clicked() {
                                if let Some(input) = rfd::FileDialog::new()
                                    .add_filter("PDF", &["pdf"])
                                    .pick_file()
                                {
                                    if let Some(base) = rfd::FileDialog::new()
                                        .add_filter("PDF", &["pdf"])
                                        .set_file_name("duplex.pdf")
                                        .save_file()
                                    {
                                        let stem = base.with_extension("");
                                        let front = stem.with_file_name(format!(
                                            "{}-fronts.pdf",
                                            stem.file_name()
                                                .and_then(|name| name.to_str())
                                                .unwrap_or("duplex")
                                        ));
                                        let back = stem.with_file_name(format!(
                                            "{}-backs.pdf",
                                            stem.file_name()
                                                .and_then(|name| name.to_str())
                                                .unwrap_or("duplex")
                                        ));
                                        self.status = Some(
                                            match duplex_export::split_pdf(
                                                &input,
                                                &front,
                                                &back,
                                                self.back_feed,
                                            ) {
                                                Ok(()) => {
                                                    self.language.text("split_success").to_owned()
                                                }
                                                Err(error) => format!("PDF split failed: {error}"),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                ui.separator();
                ui.label(self.language.text("copies"));
                ui.add(egui::DragValue::new(&mut self.copies).range(1..=999));
                ui.separator();
                ui.heading(self.language.text("printer"));
                ui.label(self.language.text("not_ready"));
                ui.add_enabled(false, egui::Button::new(self.language.text("print")));
            });

        self.refresh_preview(ctx);
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
                if let Some(texture) = &self.preview {
                    egui::ScrollArea::both().show(ui, |ui| {
                        let available = ui.available_width().max(1.0);
                        let size = texture.size_vec2();
                        let scale = (available / size.x).min(1.0);
                        ui.image((texture.id(), size * scale));
                    });
                } else {
                    ui.label(self.language.text("preview_pending"));
                }
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
