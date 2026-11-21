# Linux Print Studio

Open-source Linux print management application built with Rust and egui.

> Status: early development / experimental scaffold. Printing interception, virtual printer integration and PDF composition are not implemented yet.

## Vision

A consistent print workflow across Linux applications through a CUPS virtual queue, a separate job service and an egui desktop application. The virtual queue will receive jobs explicitly routed to it (or sent via the system default printer). It cannot intercept applications bypassing CUPS.

## Current milestone (0.1.0-dev)

- Native egui desktop shell
- Language detection from `LC_ALL`, `LC_MESSAGES`, `LANG` with English fallback
- English and Brazilian Portuguese translations (initial UI strings)
- Architectural boundaries for document composition, job intake and printing

## Build

Install the Rust stable toolchain and Linux development dependencies required by eframe/winit, then run:

```bash
cargo run
```

## Planned architecture

- `src/main.rs`: desktop entry point
- `src/i18n.rs`: localization
- `src/app.rs`: egui UI
- Future: CUPS integration, print job service, PDF processing and secure temporary job storage

## Security principles

Incoming print jobs can contain sensitive information. A future service must isolate users, restrict spool permissions, validate input, limit resource usage, and clean up temporary files. No print job is to be silently redirected to a physical printer.

## Contributing

Contributions and translations are welcome once contribution guidelines and licensing have been finalized.

## PDF preview (0.2.1 experimental)

The desktop app now supports opening PDFs using a native file picker or drag-and-drop, inspecting page counts and displaying rendered page previews.

**Linux runtime requirement:** install Poppler's `pdftoppm` command (for example `sudo apt install poppler-utils` on Debian/Ubuntu). The preview invokes this executable and displays its PNG output. This is an interim implementation, not a bundled PDF renderer.

Printing, PDF layout transformations, and CUPS virtual printer integration are **not implemented**. Selecting a layout currently changes only the selection control. Preview rendering runs synchronously and may temporarily block the interface for large PDFs; a background worker and input resource limits are planned before production use.

## Imposition planning (0.3.0 development)

The `src/imposition.rs` module calculates booklet page order and N-up sheet-side slots (1, 2, 4, 6, 9 or 16 pages per side). The UI shows a limited textual plan. This is **planning only**: it does not yet generate imposed PDFs, rotate duplex backs, handle poster tiling, or print. Output must not be treated as print-ready. Unit tests cover common page counts and blank padding.

## Experimental PDF export (0.3.0)

Select **Booklet** or **Multiple pages**, open a PDF, and choose **Export prepared PDF**. The exporter produces an A4 PDF with imposed pages using Poppler `pdftoppm` and `lopdf`.

**Limitations:** export is raster-based (text selection/search and vector fidelity are lost); it may take time and memory for large documents and currently runs on the UI thread. Source files are limited to 1000 pages; Poppler must be installed. This does not support posters, duplex back rotation, automatic/manual printing, custom paper sizes, color profiles, crop marks, bleed, or CUPS yet. Booklet order must be checked against the printer's duplex flipping behavior before printing. Do not overwrite the source file. Physical printing is not yet implemented.

The Rust CI validates compilation and unit tests, not actual printer output or rendered-page fidelity.

## Manual duplex planning (experimental)

For an exported booklet or N-up PDF, the interface shows page numbers for a front-side pass and a back-side pass. Choose same or reversed order for the backs. **This does not print, split the PDF into two files, rotate backs, or control the paper feed.** The user must print the listed pages using an external PDF viewer and perform a short two-sheet test to establish the correct stack orientation and back-side order for their specific printer. Page numbers refer to the **exported imposed PDF**, not the source PDF. The final sheet can have a blank back.

## Split imposed PDF into two manual duplex passes (experimental)

In the manual duplex section, choose **Split PDF into fronts and backs**, select an **already imposed** PDF (not the original source), then choose a base output name. The app writes `*-fronts.pdf` and `*-backs.pdf` using Poppler `pdfseparate` and `pdfunite`. Both executables must be installed (Debian/Ubuntu: `sudo apt install poppler-utils`). Output files must not already exist. A single-page imposed PDF produces only a fronts file. The app does not print or automatically detect paper-feed orientation.

**Important:** the split assumes the imposed PDF has alternating front/back pages in physical sheet order, as produced by the booklet planner. For N-up layouts, an odd final face may require an empty back for a complete stack; test with a small document first. Some printers reverse the delivered stack; select the matching back order after a physical two-sheet calibration. Splitting preserves PDF objects better than rasterizing again, but Poppler may not preserve every interactive feature. This is an experimental implementation and has not been physically validated.

## Manual duplex calibration PDF (experimental)

The manual duplex panel now offers **Generate calibration PDF**. It creates a two-page A4 vector PDF with a border, center lines, TOP/BOTTOM labels, and FRONT/BACK page identifiers. Print only page 1, observe how the sheet exits, reinsert the same sheet, and print page 2. Inspect whether the second page lands on the opposite side and whether it is upright. Use a single test sheet to avoid wasting paper.

This tool **does not** detect the printer, infer the correct refeed orientation, save printer profiles, automatically print, or calibrate stack order. Those require a guided user-confirmation workflow and real-device testing. The file will not overwrite an existing output. CI checks compilation and a two-page PDF creation unit test, not physical printer behavior.
