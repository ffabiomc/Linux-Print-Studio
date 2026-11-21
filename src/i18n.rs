#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Auto,
    English,
    PortugueseBrazil,
}

pub fn system_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            if value.is_empty() {
                continue;
            }
            let code = value
                .split(['.', '@'])
                .next()
                .unwrap_or("")
                .replace('_', "-")
                .to_lowercase();
            return if code == "pt" || code.starts_with("pt-") {
                Language::PortugueseBrazil
            } else {
                Language::English
            };
        }
    }
    Language::English
}

impl Language {
    pub fn resolved(self) -> Self {
        if self == Self::Auto {
            system_language()
        } else {
            self
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic / Automático",
            Self::English => "English",
            Self::PortugueseBrazil => "Português (Brasil)",
        }
    }

    pub fn text(self, key: &str) -> &'static str {
        let pt = self.resolved() == Self::PortugueseBrazil;
        match (pt, key) {
            (true, "subtitle") => "Gerenciador de impressão para Linux",
            (false, "subtitle") => "Print manager for Linux",
            (true, "language") => "Idioma",
            (false, "language") => "Language",
            (true, "document") => "Documento PDF",
            (false, "document") => "PDF document",
            (true, "printer") => "Impressora",
            (false, "printer") => "Printer",
            (true, "layout") => "Layout",
            (false, "layout") => "Layout",
            (true, "manual_duplex") => "Frente e verso manual",
            (false, "manual_duplex") => "Manual duplex",
            (true, "reverse_backs") => "Versos em ordem inversa",
            (false, "reverse_backs") => "Reverse back-side order",
            (true, "same_backs") => "Versos na mesma ordem",
            (false, "same_backs") => "Keep back-side order",
            (true, "front_pass") => "Primeira passagem (frentes)",
            (false, "front_pass") => "First pass (fronts)",
            (true, "back_pass") => "Segunda passagem (versos)",
            (false, "back_pass") => "Second pass (backs)",
            (true, "duplex_warning") => "Use os números no PDF exportado. Teste com duas folhas: a posição e a ordem dependem da impressora. Não há impressão automática.",
            (false, "duplex_warning") => "Use page numbers from the exported PDF. Test with two sheets: feed direction and order depend on your printer. Printing is not automated.",
            (true, "export_pdf") => "Exportar PDF preparado",
            (false, "export_pdf") => "Export prepared PDF",
            (true, "open") => "Abrir PDF...",
            (false, "open") => "Open PDF...",
            (true, "drop") => "Arraste um PDF para esta janela",
            (false, "drop") => "Drop a PDF onto this window",
            (true, "pages") => "Páginas",
            (false, "pages") => "Pages",
            (true, "copies") => "Cópias",
            (false, "copies") => "Copies",
            (true, "dark") => "Tema escuro",
            (false, "dark") => "Dark theme",
            (true, "normal") => "Normal",
            (false, "normal") => "Normal",
            (true, "booklet") => "Livreto",
            (false, "booklet") => "Booklet",
            (true, "poster") => "Pôster",
            (false, "poster") => "Poster",
            (true, "multiple") => "Múltiplas páginas",
            (false, "multiple") => "Multiple pages",
            (true, "preview") => "Pré-visualização",
            (false, "preview") => "Preview",
            (true, "preview_pending") => "PDF carregado. Renderização visual em desenvolvimento.",
            (false, "preview_pending") => "PDF loaded. Visual rendering is under development.",
            (true, "not_ready") => "Integração CUPS ainda não disponível.",
            (false, "not_ready") => "CUPS integration is not available yet.",
            (true, "print") => "Imprimir (em breve)",
            (false, "print") => "Print (coming soon)",
            _ => "Unknown translation",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_languages_resolve() {
        assert_eq!(Language::English.resolved(), Language::English);
        assert_eq!(
            Language::PortugueseBrazil.resolved(),
            Language::PortugueseBrazil
        );
    }
}
