use eframe::egui;
use crate::i18n::Language;

pub struct PrintStudioApp {
    language: Language,
}

impl Default for PrintStudioApp {
    fn default() -> Self {
        Self {
            language: Language::Auto,
        }
    }
}

impl eframe::App for PrintStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Linux Print Studio");
            ui.label(self.language.text("subtitle"));
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
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.language.text("document"));
            ui.add_enabled(false, egui::Button::new(self.language.text("open")));
            ui.separator();
            ui.heading(self.language.text("printer"));
            ui.label(self.language.text("status"));
            ui.separator();
            ui.heading(self.language.text("layout"));
            ui.add_enabled(false, egui::Button::new(self.language.text("print")));
        });
    }
}
