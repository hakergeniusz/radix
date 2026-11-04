# Radix RPM spec — builds from a source archive of this repository.
Name:           radix
Version:        0.1.0
Release:        1%{?dist}
Summary:        An independent desktop calculator

License:        MIT
URL:            https://github.com/hakergeniusz/radix
Source0:        %{name}-%{version}.tar.xz

ExclusiveArch:  x86_64 aarch64
BuildRequires:  rustc >= 1.85
BuildRequires:  cargo
BuildRequires:  gtk4-devel
BuildRequires:  libadwaita1-devel
BuildRequires:  pkgconfig

Requires:       gtk4
Requires:       libadwaita
Requires:       glib-networking

%description
Radix is a history-centric desktop calculator with Basic, Advanced,
Financial, Programming, Keyboard and Conversion modes, arbitrary-precision
arithmetic, exact rational results where possible, trigonometry with
configurable angle units, bitwise operations with selectable word size,
live unit and currency conversion, and four result formats.

%prep
%setup -q

%build
cargo build --release --locked

%install
install -Dm755 target/release/radix %{buildroot}%{_bindir}/radix
install -Dm644 data/io.github.hakergeniusz.Radix.desktop.in \
  %{buildroot}%{_datadir}/applications/io.github.hakergeniusz.Radix.desktop
install -Dm644 data/io.github.hakergeniusz.Radix.metainfo.xml \
  %{buildroot}%{_metainfodir}/io.github.hakergeniusz.Radix.metainfo.xml
for s in 16 24 32 48 64 128 256 512; do
  install -Dm644 data/icons/hicolor/${s}x${s}/apps/io.github.hakergeniusz.Radix.png \
    %{buildroot}%{_datadir}/icons/hicolor/${s}x${s}/apps/io.github.hakergeniusz.Radix.png
done
install -Dm644 data/icons/hicolor/scalable/apps/io.github.hakergeniusz.Radix.svg \
  %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.hakergeniusz.Radix.svg
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE

%files
%license LICENSE
%{_bindir}/radix
%{_datadir}/applications/io.github.hakergeniusz.Radix.desktop
%{_metainfodir}/io.github.hakergeniusz.Radix.metainfo.xml
%{_datadir}/icons/hicolor/*/apps/io.github.hakergeniusz.Radix.*

%changelog
* Tue Nov 03 2026 hakergeniusz <radix@users.noreply.github.com> - 0.1.0-1
- Initial independent clean-room implementation of Radix.
