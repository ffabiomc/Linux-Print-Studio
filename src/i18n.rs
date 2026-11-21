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
            (true, "refresh_printers") => "Buscar impressoras CUPS",
            (false, "refresh_printers") => "Find CUPS printers",
            (true, "no_printers") => "Nenhuma impressora encontrada.",
            (false, "no_printers") => "No printers found.",
            (true, "default_printer") => "Impressora padrão do sistema",
            (false, "default_printer") => "System default printer",
            (true, "print_queue") => "Fila de impressão",
            (false, "print_queue") => "Print queue",
            (true, "refresh_queue") => "Atualizar fila",
            (false, "refresh_queue") => "Refresh queue",
            (true, "cancel_job") => "Cancelar trabalho",
            (false, "cancel_job") => "Cancel job",
            (true, "confirm_cancel_job") => "Confirmar cancelamento do trabalho",
            (false, "confirm_cancel_job") => "Confirm job cancellation",
            (true, "job_cancelled") => "Cancelamento solicitado ao CUPS.",
            (false, "job_cancelled") => "Cancellation requested from CUPS.",
            (true, "confirm_print") => "Confirmar impressão",
            (false, "confirm_print") => "Confirm printing",
            (true, "cancel") => "Cancelar",
            (false, "cancel") => "Cancel",
            (true, "poster_select_tile") => "Folha do pôster",
            (false, "poster_select_tile") => "Poster sheet",
            (true, "poster_render_tile") => "Visualizar folha",
            (false, "poster_render_tile") => "Preview sheet",
            (true, "poster_preview_note") => "Clique em Visualizar folha após alterar a grade, a sobreposição ou as guias. A prévia é gerada do PDF final.",
            (false, "poster_preview_note") => "Click Preview sheet after changing the grid, overlap or guides. The preview is rendered from the final PDF.",
            (true, "poster_crop_marks") => "Guias tracejadas para corte",
            (false, "poster_crop_marks") => "Dashed cutting guides",
            (true, "poster_overlap") => "Sobreposição entre folhas",
            (false, "poster_overlap") => "Overlap between sheets",
            (true, "poster_sheets") => "Total de folhas",
            (false, "poster_sheets") => "Total sheets",
            (true, "poster_columns") => "Folhas na horizontal",
            (false, "poster_columns") => "Sheets horizontally",
            (true, "poster_rows") => "Folhas na vertical",
            (false, "poster_rows") => "Sheets vertically",
            (true, "poster_grid") => "Grade do pôster (folhas A4)",
            (false, "poster_grid") => "Poster grid (A4 sheets)",
            (true, "poster_note") => "Apenas a página exibida; sobreposição ajustável de 0 a 20 mm. Imprima em tamanho real (100%).",
            (false, "poster_note") => "Current page only; adjustable 0–20 mm overlap. Print at actual size (100%).",
            (true, "print_layout_warning") => "Livreto e múltiplas páginas geram PDF rasterizado temporário. Confira orientação e frente e verso no teste físico. Pôster usa a página atual em uma grade A4 configurável.",
            (false, "print_layout_warning") => "Booklet and N-up use a temporary rasterized PDF. Verify orientation and duplex with a physical test. Poster tiles the current page onto A4 sheets.",
            (true, "print_normal_only") => "Somente o PDF original será enviado. Layouts especiais não estão disponíveis para impressão direta.",
            (false, "print_normal_only") => "Only the original PDF will be sent. Special layouts are not available for direct printing.",
            (true, "printer_settings") => "Configurações disponíveis da impressora",
            (false, "printer_settings") => "Available printer settings",
            (true, "settings_preview_only") => "Seleção experimental: ainda não envia nem altera trabalhos de impressão.",
            (false, "settings_preview_only") => "Experimental selection: does not submit or modify print jobs yet.",
            (true, "printer_details") => "Consultar estado e recursos",
            (false, "printer_details") => "Check printer status and capabilities",
            (true, "printer_options") => "Opções disponíveis (CUPS)",
            (false, "printer_options") => "Available options (CUPS)",
            (true, "printer") => "Impressora",
            (false, "printer") => "Printer",
            (true, "layout") => "Layout",
            (false, "layout") => "Layout",
            (true, "split_duplex") => "Separar PDF em frentes e versos",
            (false, "split_duplex") => "Split PDF into fronts and backs",
            (true, "split_success") => "Arquivos gerados. Confira a pasta escolhida e teste a ordem das folhas.",
            (false, "split_success") => "Files generated. Check the selected folder and test the sheet order.",
            (true, "calibration_pdf") => "Gerar PDF de calibração",
            (false, "calibration_pdf") => "Generate calibration PDF",
            (true, "calibration_saved") => "PDF de calibração salvo. Imprima apenas a página 1, recoloque a folha e imprima a página 2.",
            (false, "calibration_saved") => "Calibration PDF saved. Print page 1, reinsert the sheet, then print page 2.",
            (true, "calibration_instructions") => "Teste com uma folha A4: imprima a página 1, anote a orientação de saída, recoloque-a e imprima a página 2. Confira o verso antes de escolher a ordem.",
            (false, "calibration_instructions") => "Test with one A4 sheet: print page 1, note its exit orientation, reinsert it and print page 2. Check the back before choosing the order.",
            (true, "refeed_orientation") => "Como recolocar o papel após o teste",
            (false, "refeed_orientation") => "How to reinsert paper after testing",
            (true, "refeed_unknown") => "Não calibrado",
            (false, "refeed_unknown") => "Not calibrated",
            (true, "up_top") => "Face impressa para cima, topo primeiro",
            (false, "up_top") => "Printed side up, top edge first",
            (true, "up_bottom") => "Face impressa para cima, base primeiro",
            (false, "up_bottom") => "Printed side up, bottom edge first",
            (true, "down_top") => "Face impressa para baixo, topo primeiro",
            (false, "down_top") => "Printed side down, top edge first",
            (true, "down_bottom") => "Face impressa para baixo, base primeiro",
            (false, "down_bottom") => "Printed side down, bottom edge first",
            (true, "refeed_caution") => "Registre somente a posição que funcionou no teste físico. Esta opção não altera a impressão automaticamente.",
            (false, "refeed_caution") => "Record only the position verified by a physical test. This setting does not automatically change printing.",
            (true, "profile_name") => "Nome do perfil da impressora",
            (false, "profile_name") => "Printer profile name",
            (true, "save_profile") => "Salvar perfil",
            (false, "save_profile") => "Save profile",
            (true, "load_profile") => "Selecionar perfil salvo",
            (false, "load_profile") => "Select saved profile",
            (true, "profile_saved") => "Perfil salvo localmente.",
            (false, "profile_saved") => "Profile saved locally.",
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
