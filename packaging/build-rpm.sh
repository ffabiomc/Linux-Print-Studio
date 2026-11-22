#!/usr/bin/env bash
set -euo pipefail
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)"
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid package version: $version" >&2
  exit 1
fi
root="$(mktemp -d)"
trap 'rm -rf "$root"' EXIT
mkdir -p "$root"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
cat > "$root/SPECS/linux-print-studio.spec" <<'SPEC'
Name: linux-print-studio
Version: %{?lps_version}
Release: 1%{?dist}
Summary: Linux desktop PDF print preparation and CUPS printing
License: LicenseRef-Not-Specified
URL: https://github.com/ffabiomc/Linux-Print-Studio
BuildArch: x86_64
Requires: cups-client
Requires: poppler-utils
Requires: fontconfig

%description
Desktop print utility with booklet, poster, N-up and manual duplex workflows.
Uses system CUPS and Poppler utilities.

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
install -m 755 %{_sourcedir}/linux-print-studio %{buildroot}/usr/bin/linux-print-studio
install -m 644 %{_sourcedir}/linux-print-studio.desktop %{buildroot}/usr/share/applications/linux-print-studio.desktop
install -m 644 %{_sourcedir}/linux-print-studio.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/linux-print-studio.svg

%files
/usr/bin/linux-print-studio
/usr/share/applications/linux-print-studio.desktop
/usr/share/icons/hicolor/scalable/apps/linux-print-studio.svg
SPEC
install -m755 target/release/linux-print-studio "$root/SOURCES/linux-print-studio"
install -m644 packaging/linux-print-studio.desktop "$root/SOURCES/linux-print-studio.desktop"
install -m644 packaging/linux-print-studio.svg "$root/SOURCES/linux-print-studio.svg"
rpmbuild -bb --define "_topdir $root" --define "lps_version $version" "$root/SPECS/linux-print-studio.spec"
install -m644 "$root/RPMS/x86_64/"*.rpm "./linux-print-studio-$version-1.x86_64.rpm"
rpm -qip "./linux-print-studio-$version-1.x86_64.rpm"
rpm -qlp "./linux-print-studio-$version-1.x86_64.rpm"
