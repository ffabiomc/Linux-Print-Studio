#!/usr/bin/env bash
# Non-destructive Linux Print Studio runtime preflight.
# Does not submit or cancel print jobs.
set -u

missing=0
for command in lp lpstat lpoptions cancel pdftoppm pdfseparate pdfunite; do
    if command -v "$command" >/dev/null 2>&1; then
        printf '[OK] %s\n' "$command"
    else
        printf '[MISSING] %s\n' "$command"
        missing=1
    fi
done

printf '\nCUPS destinations:\n'
if command -v lpstat >/dev/null 2>&1; then
    if ! lpstat -e; then
        printf 'WARNING: Cannot query CUPS. Check the service and printer configuration.\n'
    fi
    printf '\nDefault printer:\n'
    lpstat -d || true
fi

printf '\nRuntime requirements:\n'
printf 'Debian/Ubuntu: sudo apt install cups-client poppler-utils\n'
printf 'This check never sends data to a printer.\n'
exit "$missing"
