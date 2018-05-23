Name: sudo-pair
Version: v0.9.2
Release: 1
Summary: Plugin for sudo that requires another human to approve and monitor privileged sudo sessions.
Group: tem Environment/Libraries
License: Apache Software License 2.0
Url: https://github.com/square/sudo_pair
Source: https://github.com/square/sudo_pair/archive/sudo_pair-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-build
BuildRequires: cargo
BuildRequires: clang-devel
BuildRequires: git
Requires: sudo

%description
Plugin for sudo that requires another human to approve and monitor privileged sudo sessions

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

%changelog
* Wed May 23 2018 - robert (at) meinit.nl
- Initial release.
