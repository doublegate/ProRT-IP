Name:           prtip
Version:        1.0.0
Release:        1%{?dist}
Summary:        Modern high-performance network scanner

License:        GPL-3.0-or-later
URL:            https://github.com/doublegate/ProRT-IP
Source0:        https://github.com/doublegate/ProRT-IP/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  rust >= 1.85
BuildRequires:  cargo
BuildRequires:  libpcap-devel
BuildRequires:  openssl-devel
BuildRequires:  gcc
BuildRequires:  make

Requires:       libpcap

%description
ProRT-IP WarScan is a modern network scanner written in Rust that combines
the speed of Masscan (10M+ packets/second) with the depth of Nmap's service
detection and OS fingerprinting capabilities.

Features:
- TCP SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle scan types
- Service version detection (85-90% accuracy, 187 Nmap probes)
- OS fingerprinting (16-probe technique, 2,600+ signatures)
- Evasion techniques (fragmentation, decoys, TTL manipulation)
- TUI dashboard with real-time visualization
- Full IPv6 support
- Nmap-compatible CLI flags

%prep
%autosetup -n ProRT-IP-%{version}

%build
cargo build --release --locked

%install
install -Dm755 target/release/prtip %{buildroot}%{_bindir}/prtip
install -Dm644 man/prtip.1 %{buildroot}%{_mandir}/man1/prtip.1
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 LICENSE %{buildroot}%{_docdir}/%{name}/LICENSE
install -Dm644 CHANGELOG.md %{buildroot}%{_docdir}/%{name}/CHANGELOG.md

%check
cargo test --release --locked

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/prtip
%{_mandir}/man1/prtip.1*

%changelog
* Sat Jan 25 2025 ProRT-IP Contributors <noreply@github.com> - 1.0.0-1
- Initial release
- TCP SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle scan types
- Service version detection (85-90% accuracy)
- OS fingerprinting (2,600+ signatures)
- TUI dashboard with real-time visualization
- Full IPv6 support
- Nmap-compatible CLI flags
