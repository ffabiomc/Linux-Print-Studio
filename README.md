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
