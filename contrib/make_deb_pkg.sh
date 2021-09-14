#!/usr/bin/env bash

REPO_ROOT=$(git rev-parse --show-toplevel)
CURRENT_DIR=$(pwd)

# switch to root of repo
cd "${REPO_ROOT}"

# make release build
cargo build --release

DEB_TMPL_DIR="${REPO_ROOT}/contrib/DEBIAN"
OUT_DIR="${REPO_ROOT}/out"
DEB_PKG_DIR="${OUT_DIR}/debian"

# create packagind directory
mkdir -p ${DEB_PKG_DIR}

# copy templates
cp -R "${DEB_TMPL_DIR}" "${DEB_PKG_DIR}/"

# shared library
mkdir -p "${DEB_PKG_DIR}/usr/lib/sudo"
cp "${REPO_ROOT}/target/release/libsudo_pair.so" "${DEB_PKG_DIR}/usr/lib/sudo"

# socket directory
mkdir -p "${DEB_PKG_DIR}/var/run/sudo_pair"

# approval script
mkdir -p "${DEB_PKG_DIR}/usr/bin"
cp "${REPO_ROOT}/sample/bin/sudo_approve" "${DEB_PKG_DIR}/usr/bin/sudo_approve"

# sudo.conf
mkdir "${DEB_PKG_DIR}/etc"
cp "${REPO_ROOT}/sample/etc/sudo.conf" "${DEB_PKG_DIR}/etc/sudo.conf_for_sudo_pair_sample"

# prompts
cp "${REPO_ROOT}/sample/etc/sudo.prompt.user" "${DEB_PKG_DIR}/etc/sudo.prompt.user"
cp "${REPO_ROOT}/sample/etc/sudo.prompt.pair" "${DEB_PKG_DIR}/etc/sudo.prompt.pair"

VERSION=$(cat "${DEB_TMPL_DIR}/control" | grep Version | cut -d ' ' -f 2)
dpkg-deb -b "${DEB_PKG_DIR}" "${OUT_DIR}/sudo-pair-${VERSION}.deb"

# switch back
cd "${CURRENT_DIR}"
