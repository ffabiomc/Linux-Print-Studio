use crate::calibration;
use crate::cups::{self, PrintJob, PrinterDetails, Printers};
use crate::document::PdfDocument;
use crate::duplex::{self, FeedOrder};
use crate::duplex_export;
use crate::export::{self, ExportLayout};
use crate::i18n::Language;
use crate::imposition;
use crate::preview;
use crate::profiles::{self, PrinterProfile, Refeed};
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
    poster_tiles: usize,
    back_feed: FeedOrder,
    refeed: Refeed,
    profiles: Vec<PrinterProfile>,
    profile_name: String,
    cups_printers: Option<Printers>,
    selected_printer: String,
    cups_error: Option<String>,
    printer_details: Option<PrinterDetails>,
    selected_options: std::collections::BTreeMap<String, String>,
    confirm_print: bool,
    queue_jobs: Vec<PrintJob>,
    cancel_target: Option<String>,
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
            poster_tiles: 2,
            back_feed: FeedOrder::Reversed,
            refeed: Refeed::Unknown,
            profiles: profiles::config_path()
                .and_then(|path| profiles::load(&path))
                .unwrap_or_default(),
            profile_name: String::new(),
            cups_printers: None,
            selected_printer: String::new(),
            cups_error: None,
            printer_details: None,
            selected_options: std::collections::BTreeMap::new(),
            confirm_print: false,
            queue_jobs: Vec::new(),
            cancel_target: None,
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

    fn submit_current_document(&self) -> Result<String, String> {
        let document = self.document.as_ref().ok_or("No PDF selected")?;
        let choices = self
            .printer_details
            .as_ref()
            .map(|details| details.choices.as_slice())
            .unwrap_or(&[]);
        let temporary = match self.layout {
            Layout::Normal => None,
            Layout::Booklet | Layout::Multiple => {
                let layout = match self.layout {
                    Layout::Booklet => ExportLayout::Booklet,
                    Layout::Multiple => ExportLayout::Multiple(self.pages_per_sheet),
                    _ => unreachable!(),
                };
                let file = tempfile::Builder::new()
                    .prefix("linux-print-studio-")
                    .suffix(".pdf")
                    .tempfile()
                    .map_err(|error| error.to_string())?;
                export::export_pdf(&document.path, file.path(), document.pages, layout)?;
                Some(file)
            }
            Layout::Poster => {
                let file = tempfile::Builder::new()
                    .prefix("linux-print-studio-poster-")
                    .suffix(".pdf")
                    .tempfile()
                    .map_err(|error| error.to_string())?;
                export::export_poster(&document.path, file.path(), self.page, self.poster_tiles)?;
                Some(file)
            }
        };
        let pdf = temporary
            .as_ref()
            .map_or(document.path.as_path(), |file| file.path());
        cups::submit(
            &self.selected_printer,
            pdf,
            self.copies,
            choices,
            &self.selected_options,
        )
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
                if self.layout == Layout::Poster {
                    ui.label(self.language.text("poster_grid"));
                    egui::ComboBox::from_id_salt("poster_tiles")
                        .selected_text(format!("{} × {}", self.poster_tiles, self.poster_tiles))
                        .show_ui(ui, |ui| {
                            for count in [2, 3, 4] {
                                ui.selectable_value(
                                    &mut self.poster_tiles,
                                    count,
                                    format!("{count} × {count}"),
                                );
                            }
                        });
                    ui.small(self.language.text("poster_note"));
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
                    Layout::Poster => None,
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
                if self.layout == Layout::Poster {
                    if ui
                        .add_enabled(
                            self.document.is_some(),
                            egui::Button::new(self.language.text("export_pdf")),
                        )
                        .clicked()
                    {
                        if let Some(document) = &self.document {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PDF", &["pdf"])
                                .set_file_name("poster.pdf")
                                .save_file()
                            {
                                self.status = Some(
                                    match export::export_poster(
                                        &document.path,
                                        &path,
                                        self.page,
                                        self.poster_tiles,
                                    ) {
                                        Ok(()) => format!("PDF saved: {}", path.display()),
                                        Err(error) => format!("Poster export failed: {error}"),
                                    },
                                );
                            }
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
                        ui.label(self.language.text("refeed_orientation"));
                        egui::ComboBox::from_id_salt("refeed_orientation")
                            .selected_text(match self.refeed {
                                Refeed::Unknown => self.language.text("refeed_unknown"),
                                Refeed::FaceUpTopFirst => self.language.text("up_top"),
                                Refeed::FaceUpBottomFirst => self.language.text("up_bottom"),
                                Refeed::FaceDownTopFirst => self.language.text("down_top"),
                                Refeed::FaceDownBottomFirst => self.language.text("down_bottom"),
                            })
                            .show_ui(ui, |ui| {
                                for (value, key) in [
                                    (Refeed::Unknown, "refeed_unknown"),
                                    (Refeed::FaceUpTopFirst, "up_top"),
                                    (Refeed::FaceUpBottomFirst, "up_bottom"),
                                    (Refeed::FaceDownTopFirst, "down_top"),
                                    (Refeed::FaceDownBottomFirst, "down_bottom"),
                                ] {
                                    ui.selectable_value(
                                        &mut self.refeed,
                                        value,
                                        self.language.text(key),
                                    );
                                }
                            });
                        ui.small(self.language.text("refeed_caution"));
                        ui.label(self.language.text("profile_name"));
                        ui.text_edit_singleline(&mut self.profile_name);
                        if ui.button(self.language.text("save_profile")).clicked() {
                            let name = self.profile_name.trim().to_owned();
                            let mut updated = self.profiles.clone();
                            if let Some(profile) =
                                updated.iter_mut().find(|profile| profile.name == name)
                            {
                                profile.back_feed = self.back_feed;
                                profile.refeed = self.refeed;
                            } else {
                                updated.push(PrinterProfile {
                                    name,
                                    back_feed: self.back_feed,
                                    refeed: self.refeed,
                                });
                            }
                            self.status = Some(
                                match profiles::config_path()
                                    .and_then(|path| profiles::save(&path, &updated))
                                {
                                    Ok(()) => {
                                        self.profiles = updated;
                                        self.language.text("profile_saved").to_owned()
                                    }
                                    Err(error) => format!("Profile save failed: {error}"),
                                },
                            );
                        }
                        egui::ComboBox::from_id_salt("printer_profile")
                            .selected_text(self.language.text("load_profile"))
                            .show_ui(ui, |ui| {
                                for profile in &self.profiles {
                                    if ui.selectable_label(false, &profile.name).clicked() {
                                        self.profile_name = profile.name.clone();
                                        self.back_feed = profile.back_feed;
                                        self.refeed = profile.refeed;
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
                if ui.button(self.language.text("refresh_printers")).clicked() {
                    self.printer_details = None;
                    self.queue_jobs.clear();
                    self.selected_options.clear();
                    match cups::discover() {
                        Ok(printers) => {
                            if !printers.names.contains(&self.selected_printer) {
                                self.selected_printer =
                                    printers.default.clone().unwrap_or_default();
                            }
                            self.cups_printers = Some(printers);
                            self.cups_error = None;
                        }
                        Err(error) => {
                            self.cups_printers = None;
                            self.cups_error = Some(error);
                        }
                    }
                }
                if let Some(printers) = &self.cups_printers {
                    if printers.names.is_empty() {
                        ui.label(self.language.text("no_printers"));
                    } else {
                        egui::ComboBox::from_id_salt("cups_printers")
                            .selected_text(&self.selected_printer)
                            .show_ui(ui, |ui| {
                                for name in &printers.names {
                                    if ui
                                        .selectable_value(
                                            &mut self.selected_printer,
                                            name.clone(),
                                            name,
                                        )
                                        .changed()
                                    {
                                        self.printer_details = None;
                                        self.queue_jobs.clear();
                                        self.selected_options.clear();
                                    }
                                }
                            });
                        if printers.default.as_ref() == Some(&self.selected_printer) {
                            ui.small(self.language.text("default_printer"));
                        }
                    }
                }
                if let Some(error) = &self.cups_error {
                    ui.small(error);
                }
                if !self.selected_printer.is_empty()
                    && ui.button(self.language.text("printer_details")).clicked()
                {
                    match cups::details(&self.selected_printer) {
                        Ok(details) => {
                            self.selected_options = details
                                .choices
                                .iter()
                                .filter_map(|option| {
                                    option
                                        .default
                                        .as_ref()
                                        .map(|value| (option.key.clone(), value.clone()))
                                })
                                .collect();
                            self.printer_details = Some(details);
                            self.cups_error = None;
                        }
                        Err(error) => {
                            self.printer_details = None;
                            self.cups_error = Some(error);
                        }
                    }
                }
                if let Some(details) = &self.printer_details {
                    ui.label(&details.status);
                    ui.collapsing(self.language.text("printer_settings"), |ui| {
                        for option in &details.choices {
                            ui.label(&option.label);
                            let selected = self
                                .selected_options
                                .entry(option.key.clone())
                                .or_insert_with(|| option.values[0].clone());
                            egui::ComboBox::from_id_salt(("cups_option", &option.key))
                                .selected_text(selected.as_str())
                                .show_ui(ui, |ui| {
                                    for value in &option.values {
                                        ui.selectable_value(selected, value.clone(), value);
                                    }
                                });
                        }
                        ui.small(self.language.text("settings_preview_only"));
                    });
                    ui.collapsing(self.language.text("printer_options"), |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(170.0)
                            .show(ui, |ui| {
                                for option in &details.options {
                                    ui.small(option);
                                }
                            });
                    });
                }
                ui.separator();
                ui.heading(self.language.text("print_queue"));
                if !self.selected_printer.is_empty()
                    && ui.button(self.language.text("refresh_queue")).clicked()
                {
                    match cups::jobs(&self.selected_printer) {
                        Ok(jobs) => {
                            self.queue_jobs = jobs;
                            self.cups_error = None;
                        }
                        Err(error) => self.cups_error = Some(error),
                    }
                }
                for job in &self.queue_jobs {
                    ui.horizontal(|ui| {
                        ui.label(&job.description);
                        if ui.button(self.language.text("cancel_job")).clicked() {
                            self.cancel_target = Some(job.id.clone());
                        }
                    });
                }
                ui.label(self.language.text("not_ready"));
                let can_print = self.document.is_some() && !self.selected_printer.is_empty();
                if ui
                    .add_enabled(can_print, egui::Button::new(self.language.text("print")))
                    .clicked()
                {
                    self.confirm_print = true;
                }
            });

        if let Some(job_id) = self.cancel_target.clone() {
            egui::Window::new(self.language.text("confirm_cancel_job"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&job_id);
                    ui.horizontal(|ui| {
                        if ui.button(self.language.text("cancel")).clicked() {
                            self.cancel_target = None;
                        }
                        if ui
                            .button(self.language.text("confirm_cancel_job"))
                            .clicked()
                        {
                            self.cancel_target = None;
                            self.status =
                                Some(match cups::cancel_job(&self.selected_printer, &job_id) {
                                    Ok(()) => {
                                        self.queue_jobs.retain(|job| job.id != job_id);
                                        self.language.text("job_cancelled").to_owned()
                                    }
                                    Err(error) => error,
                                });
                        }
                    });
                });
        }

        if self.confirm_print {
            egui::Window::new(self.language.text("confirm_print"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(document) = &self.document {
                        ui.label(document.path.display().to_string());
                    }
                    ui.label(format!(
                        "{}: {}",
                        self.language.text("printer"),
                        self.selected_printer
                    ));
                    ui.label(format!("{}: {}", self.language.text("copies"), self.copies));
                    for (key, value) in &self.selected_options {
                        ui.label(format!("{key}: {value}"));
                    }
                    ui.small(self.language.text("print_layout_warning"));
                    ui.horizontal(|ui| {
                        if ui.button(self.language.text("cancel")).clicked() {
                            self.confirm_print = false;
                        }
                        if ui.button(self.language.text("confirm_print")).clicked() {
                            self.confirm_print = false;
                            self.status = Some(match self.submit_current_document() {
                                Ok(message) => format!("CUPS: {message}"),
                                Err(error) => format!("CUPS: {error}"),
                            });
                        }
                    });
                });
        }

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
