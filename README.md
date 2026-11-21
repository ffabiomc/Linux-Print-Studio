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

## Saved manual-duplex profiles (experimental)

The manual-duplex panel lets you name a printer profile and save its back-side page order (same or reversed). Saved profiles can be selected again later. Data is stored locally at `$XDG_CONFIG_HOME/linux-print-studio/printer-profiles.txt` or `~/.config/linux-print-studio/printer-profiles.txt` if XDG_CONFIG_HOME is not set. No account or network service is required. Saving a profile with the same name updates its page order.

**Limitations:** these are user-named presets, not discovered CUPS printers. They store only back-side page order; they do not yet record paper orientation, rotation, face-up/face-down output, tray, or automatic printing. Always verify paper handling on the physical printer. The format is an internal tab-delimited text file; do not edit it while the application is running.

## Paper refeed orientation presets

The manual duplex panel now records one of four physical refeed positions (printed face up/down and top/bottom edge first), or **Not calibrated**. Select the position only after a physical test; it is stored with the named printer profile and restored on selection. Older profiles without this field load as **Not calibrated**. This setting provides a reminder only: it does not yet modify PDF rotation, printer trays, or CUPS print jobs. The feed orientation of a stack and the back-side page order are independent settings, and both must be tested before relying on multi-sheet duplex.

## CUPS printer discovery (read-only, experimental)

Click **Find CUPS printers** in the printer panel to query local CUPS printer destinations using `lpstat -e`. The application also attempts `lpstat -d` to identify the system default printer. Select a destination in the drop-down. This requires the `lpstat` client utility and an accessible CUPS configuration/service; on Debian/Ubuntu it is commonly provided by `cups-client`. Discovery is triggered by the user and may briefly block the UI while the command runs. No jobs are submitted, printer settings are changed, or documents are sent. The Print button remains disabled. Printer status and capabilities are not yet queried.

## CUPS status and capabilities (read-only, experimental)

After selecting a discovered printer, click **Check printer status and capabilities**. The application runs `lpstat -p PRINTER` and `lpoptions -p PRINTER -l`, passing the printer name as a separate command argument (not through a shell). The output is displayed as reported by CUPS/driver, including available options when exposed. Driverless printers and some configurations may provide limited or no options. This does not guarantee support for any specific paper size, color mode, resolution, or duplex setting. The command may briefly block the UI; no print job is submitted.

## CUPS driver option selectors (experimental)

After **Check printer status and capabilities**, expand **Available printer settings** to select among the choices exposed by `lpoptions -l`. Defaults marked with an asterisk are preselected. These selections are in-memory only; switching printers or refreshing the list clears them. They do **not** change system printer defaults or submit a job. Some CUPS/IPP drivers expose differently formatted or incomplete option descriptions; those lines remain visible in the raw options section but may not produce selectors. Printing remains disabled.

## Direct PDF printing through CUPS (experimental)

With a PDF loaded, a discovered printer selected, and **Normal** layout active, click **Print** to review the document path, printer, copies, and selected CUPS options. Nothing is sent until you click **Confirm printing**. Submission uses the local `lp` utility and reports its output or error. Install the CUPS client tools (`lp`, `lpstat`, and `lpoptions`) and ensure the printer is configured and reachable.

**Important:** this sends the **original PDF** only; booklet, poster, and N-up layouts are not yet connected to direct printing. The command runs synchronously and may briefly block the interface. A successful `lp` response means the job was accepted by the spooler, **not** that physical printing completed. Test with a small document first.

## Print queue management (experimental)

Select a CUPS printer and click **Refresh queue** to list pending jobs reported by `lpstat -o PRINTER`. Each job has a **Cancel job** action with a second confirmation dialog. Cancellation uses `cancel JOB-ID`, with job IDs validated against the selected printer's queue prefix. CUPS permissions still apply: the system may refuse to cancel jobs belonging to other users. Queue data is a manual snapshot; refresh to see current state. The command output is shown as provided by CUPS and is not a detailed progress indicator. The app does not automatically cancel or alter any jobs.

## Direct booklet and N-up printing (experimental)

The confirmed CUPS print action now supports **Normal**, **Booklet**, and **Multiple pages per sheet**. Booklet and N-up first generate a rasterized PDF using the existing exporter into a temporary file, then submit it to `lp`. The temporary file is removed when the submission attempt finishes. Poppler's `pdftoppm` is required. **Poster** is not supported for direct printing. The booklet's front/back ordering does not automatically configure duplex: choose compatible printer duplex settings and test with a short document. For manual duplex, use the separate front/back export workflow instead. PDF rasterization may lose selectable text and increase processing time. A successful CUPS submission is not proof that the physical job printed.

## Poster printing (experimental)

Select **Poster** and choose independent horizontal and vertical tile counts (1–4 each, at least two sheets total), including 2×1, 1×2, 3×2, and 4×4. The **currently displayed source page only** is rasterized and expanded across the chosen tiles, with configurable 0–20 mm overlap between adjacent sheets (default 4 mm). Use **Export prepared PDF** to inspect the tiles before printing, or submit them through the CUPS confirmation dialog. Print at **actual size / 100%** with no fit-to-page scaling. Tile ordering is row-major, from top left to bottom right. Optional dashed cutting guides are available along internal overlap boundaries; the UI shows a numbered tile grid and supports rendering one selected sheet from the generated PDF on demand. Guides can be clipped by printers with nonprintable margins. Printer nonprintable margins may cut off content near sheet edges; test with inexpensive paper before a final poster.

The poster preview's **Preview sheet** button generates a temporary poster PDF and renders the selected numbered sheet with Poppler. Change the sheet number or poster settings and press the button again. This operation is synchronous and may take time for large PDFs; a simultaneous multi-sheet thumbnail view is not yet available.

## Manual duplex guided workflow

For booklet and N-up imposition, the manual duplex panel now displays the number of physical sheets, identifies a blank final back when the imposed side count is odd, and warns when paper refeed orientation has not been calibrated. Follow the three displayed steps: print fronts, reinsert the stack according to a tested printer profile, and then print backs. These are **instructions only**: the application does not yet detect when a physical print pass is complete or automatically submit the next pass. Verify the paper feed and back-side order using the calibration PDF and a short test document.

## Two-pass manual duplex via CUPS (experimental)

For Booklet and N-up layouts, after selecting a printer and a calibrated refeed orientation, choose **Prepare manual duplex via CUPS**. The application exports an imposed PDF, splits it into front and back passes, and keeps the temporary files alive while the dialog is open. Submit the front pass first; wait for **physical printing to finish**, reinsert the paper according to the calibrated printer profile, tick the confirmation, and only then submit the back pass. Both submissions explicitly request `sides=one-sided` and one copy; user-selected driver options are not forwarded. Closing/canceling the dialog discards temporary files, and restarting the app does not restore an interrupted session. CUPS acceptance does not guarantee physical completion. The application does not automatically detect completion, prevent refeed mistakes, or verify printer support for the simplex setting. Test with inexpensive paper first.

The manual duplex dialog can now display the CUPS job identifier returned for the front pass and refresh whether it remains in the printer queue. A job disappearing from `lpstat -o` is **not proof** that the physical printing succeeded: it may also have been canceled or failed. The user must still verify the output and explicitly confirm paper refeeding before submitting backs. This status check is manual, not live polling.

### Interrupted manual duplex jobs

When the user cancels the tracked front job through the application's queue dialog, the current duplex session blocks the back pass; start a fresh session instead. While the tracked front job remains in the CUPS queue, the paper-refeed confirmation is disabled. Before submitting the backs, the application queries CUPS again and refuses to continue if the front job is still listed or the query fails. **Important:** an absent job is not proof of successful printing; canceled/failed jobs may disappear too. Always inspect the physical output. External cancellation or printer failures are not yet reliably distinguishable from completion, so this is a safeguard rather than automatic completion verification.

## Pre-release functional testing

Run `bash scripts/check-print-environment.sh` to verify that CUPS client and Poppler commands are installed and to inspect available printer destinations **without sending any jobs**. Follow [the functional acceptance checklist](docs/functional-test-plan.md) for real-printer tests covering normal printing, booklet, N-up, poster 2×1 and 1×2, manual duplex, cancellation, failure handling, and UI navigation. These cases are not considered passed until physically executed and recorded. Rust CI does not validate printer hardware.

## Linux packaging (pre-release)

GitHub Actions → **Linux packages** → **Run workflow** builds an amd64 Debian package and an x86_64 AppImage. Download the `linux-print-studio-linux-x86_64` artifact from the completed run. A tag starting with `v` also triggers this workflow. The workflow compiles on Ubuntu 22.04 and packages the executable, desktop launcher, and SVG icon.

For Debian/Ubuntu, install the `.deb` using `sudo apt install ./linux-print-studio_<version>_amd64.deb` (replace `<version>` with the actual filename). For the AppImage, run `chmod +x Linux-Print-Studio-x86_64.AppImage` and then `./Linux-Print-Studio-x86_64.AppImage`. The AppImage currently packages the app executable and launcher but **does not bundle every shared library**: compatibility with other distributions is not yet guaranteed. Both formats still depend on system CUPS and Poppler command-line tools. On Debian/Ubuntu, install `cups-client` and `poppler-utils` if needed.

**Release gate:** verify the workflow actually produces both artifacts and test installation, startup, PDF preview, and printing on real Linux machines before publishing a stable 1.0 release. These packages are pre-release builds, not validated release binaries.

## RPM packaging (experimental)

The **Linux packages** workflow also builds an x86_64 `.rpm` with `rpmbuild`, placing it alongside the Debian package and AppImage in the workflow artifact. RPM shared-library dependencies are detected from the Ubuntu-built binary, so **installation compatibility with Fedora, Rocky Linux, AlmaLinux and openSUSE has not yet been validated**. Test the RPM in a clean target distribution before relying on it; some distributions may need a native RPM build and adjusted dependency names. The package uses system CUPS and Poppler tools.

## Package integrity checks

The Linux packages workflow runs `packaging/verify-packages.sh` after building DEB, RPM and AppImage. The script inspects the package payloads, validates desktop launchers, checks the AppImage type-2 signature, and writes `SHA256SUMS` into the downloadable artifact. On Linux, verify downloaded files with `sha256sum -c SHA256SUMS` from the same directory. This is a corruption/integrity check, **not** a cryptographic publisher signature or proof that the software is safe. Package inspection does not test runtime compatibility or physical printing.
