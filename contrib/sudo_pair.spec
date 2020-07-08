Name: sudo_pair
Version: 1.0.0
Release: 1
Summary: Plugin for sudo that requires another human to approve and monitor privileged sudo sessions.
Group: System Environment/Libraries
License: Apache Software License 2.0
Url: https://github.com/square/sudo_pair
Source: https://github.com/square/sudo_pair/archive/sudo_pair-v%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-build
BuildRequires: cargo
BuildRequires: clang-devel
BuildRequires: git
Requires: sudo

%description
Plugin for sudo that requires another human to approve and monitor privileged sudo sessions

%global debug_package %{nil}

%prep

%setup -n sudo_pair-sudo_pair-%{version}

%build
cargo build --release

%install
mkdir -p %{buildroot}/usr/libexec/sudo
%{__cp} target/release/libsudo_pair.so %{buildroot}/usr/libexec/sudo/

%clean
rm -rf %{buildroot}

%files
/usr/libexec/sudo/libsudo_pair.so
%doc README.md
%doc sample/etc/sudo.conf
%doc sample/etc/sudo.prompt.pair
%doc sample/etc/sudo.prompt.user
%doc sample/bin/sudo_approve

%changelog
* Wed May 23 2018 - robert (at) meinit.nl
- Initial release.
* Fri Jul 3 2020 - robert (at) meinit.nl
- Bump version to 1.0.0
