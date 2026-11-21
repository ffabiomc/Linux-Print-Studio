#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
debs=(linux-print-studio_*_amd64.deb)
rpms=(linux-print-studio-*.x86_64.rpm)
images=(Linux-Print-Studio-x86_64.AppImage)
if (( ${#debs[@]} != 1 || ${#rpms[@]} != 1 )) || [[ ! -s "${images[0]}" ]]; then
  echo "Expected one DEB, one RPM, and one AppImage" >&2
  exit 1
fi
deb="${debs[0]}"
rpm="${rpms[0]}"
appimage="${images[0]}"
for item in "$deb" "$rpm" "$appimage"; do test -s "$item"; file "$item"; done
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/deb" "$tmp/rpm"
dpkg-deb --extract "$deb" "$tmp/deb"
rpm2cpio "$rpm" | (cd "$tmp/rpm" && cpio -idm --quiet --no-absolute-filenames)
for root in "$tmp/deb" "$tmp/rpm"; do
  test -x "$root/usr/bin/linux-print-studio"
  test -s "$root/usr/share/applications/linux-print-studio.desktop"
  test -s "$root/usr/share/icons/hicolor/scalable/apps/linux-print-studio.svg"
  desktop-file-validate "$root/usr/share/applications/linux-print-studio.desktop"
done
test "$(dd if="$appimage" bs=1 count=4 2>/dev/null | od -An -tx1 | tr -d ' \n')" = "7f454c46"
test "$(dd if="$appimage" bs=1 skip=8 count=3 2>/dev/null | od -An -tx1 | tr -d ' \n')" = "414902"
dpkg-deb --field "$deb" Package Version Architecture
rpm -qp --queryformat '%{NAME} %{VERSION} %{ARCH}\n' "$rpm"
sha256sum "$deb" "$rpm" "$appimage" > SHA256SUMS
cat SHA256SUMS
echo "Package structure verified; physical printing and GUI startup not tested."
