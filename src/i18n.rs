#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Auto,
    English,
    PortugueseBrazil,
}

pub fn system_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            if value.is_empty() { continue; }
            let code = value.split(['.', '@']).next().unwrap_or("").replace('_', "-").to_lowercase();
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
        if self == Self::Auto { system_language() } else { self }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic / Automático",
            Self::English => "English",
            Self::PortugueseBrazil => "Português (Brasil)",
        }
    }

    pub fn text(self, key: &'static str) -> &'static str {
        let pt = self.resolved() == Self::PortugueseBrazil;
        match (pt, key) {
            (true, "subtitle") => "Gerenciador de impressão para Linux",
            (false, "subtitle") => "Print manager for Linux",
            (true, "language") => "Idioma",
            (false, "language") => "Language",
            (true, "document") => "Documento",
            (false, "document") => "Document",
            (true, "printer") => "Impressora",
            (false, "printer") => "Printer",
            (true, "layout") => "Layout",
            (false, "layout") => "Layout",
            (true, "status") => "Estrutura inicial: impressão ainda não implementada.",
            (false, "status") => "Initial scaffold: printing is not implemented yet.",
            (true, "open") => "Abrir PDF (em breve)",
            (false, "open") => "Open PDF (coming soon)",
            (true, "print") => "Imprimir (em breve)",
            (false, "print") => "Print (coming soon)",
            _ => key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_languages_resolve() {
        assert_eq!(Language::English.resolved(), Language::English);
        assert_eq!(Language::PortugueseBrazil.resolved(), Language::PortugueseBrazil);
    }
}
