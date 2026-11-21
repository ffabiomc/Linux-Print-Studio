# Linux Print Studio — Functional acceptance checklist

This is a **manual hardware acceptance test**, not a claim that the application has been tested with a physical printer. Use a non-confidential, short sample PDF and an A4 printer. Record the Linux distribution, printer model, CUPS version, and application commit.

## Before printing

1. Run `bash scripts/check-print-environment.sh`. Resolve missing `cups-client` or `poppler-utils` commands.
2. Run `cargo test` and `cargo run`.
3. Discover the CUPS printer, select it, and inspect status and supported driver options.
4. Use a short sample PDF (for example, four pages numbered 1–4). Confirm page navigation and preview.

## Acceptance cases

| ID | Test | Expected outcome | Result |
| --- | --- | --- | --- |
| F01 | Print one-page PDF in Normal layout | Correct page, paper size, and orientation; one CUPS job | Not run |
| F02 | Print 4-page Booklet | Correct imposed order and folding orientation after a physical test | Not run |
| F03 | Print 4-page N-up (2 and 4 per sheet) | All pages legible and in expected order | Not run |
| F04 | Export and print Poster 2×1 | Exactly two A4 tiles, correctly aligned at selected overlap | Not run |
| F05 | Export and print Poster 1×2 | Exactly two vertical tiles, correctly aligned | Not run |
| F06 | Manual duplex, calibrated 2-sheet test | Fronts finish, paper is reinserted correctly, backs align | Not run |
| F07 | Refresh front CUPS queue while job pending | Back submission remains blocked | Not run |
| F08 | Cancel tracked front job in app | Back submission remains blocked; start a new session | Not run |
| F09 | Disconnect printer or stop CUPS before queue refresh | Error is reported and earlier refeed confirmation is cleared | Not run |
| F10 | Try opening another PDF during manual duplex | Active session remains unchanged | Not run |
| F11 | Cancel a queued test job with confirmation | Only the chosen job is canceled if CUPS permits | Not run |
| F12 | Resize settings panel and navigate first/last pages | Controls remain usable; navigation buttons disable at boundaries | Not run |

## Recording results

For each case, change **Not run** to **Pass**, **Fail**, or **Blocked**. Record the CUPS job ID and relevant error message, but do not upload confidential PDFs or printer credentials. For duplex, record which way the stack exits and how it was reinserted.

**Important:** A successful `lp` submission only confirms spooler acceptance. A job disappearing from `lpstat -o` does **not** prove that it printed; it may have failed or been canceled outside the app. Physically inspect every sheet before confirming the back pass. Do not use the manual-duplex feature for important documents before calibration.

## Release gate

Do not declare version 1.0 physically validated until F01–F12 have been executed on at least one actual CUPS-configured printer, with failures triaged. CI compilation and unit tests are separate from this checklist.
