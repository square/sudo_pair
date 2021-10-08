.DEFAULT_GOAL := all

SHELL = /bin/sh

INSTALL         := install
INSTALL_PROGRAM := $(INSTALL)
INSTALL_DATA    := $(INSTALL) -m 644

prefix        := /usr/local
exec_prefix   := $(prefix)
bindir        := $(exec_prefix)/bin
libdir        := $(exec_prefix)/lib
sysconfdir    := $(prefix)/etc
localstatedir := $(prefix)/var
runstatedir   := $(localstatedir)/run

CARGO_TARGET_DIR ?= $(realpath ./target)
CARGOFLAGS       ?= --features slog/release_max_level_trace

nogroup_gid := $(shell getent group nogroup | awk -F: '{print $$3}')

.PHONY: all install

all: $(CARGO_TARGET_DIR)/$(PROFILE)/libsudopair.so

install: $(CARGO_TARGET_DIR)/$(PROFILE)/libsudopair.so
	$(INSTALL_DATA) -d $(DESTDIR)$(runstatedir)/sudo_pair

	$(INSTALL_PROGRAM)    ./bin/sudo_approve                          $(DESTDIR)$(bindir)
	$(INSTALL_DATA)       $(CARGO_TARGET_DIR)/release/libsudo_pair.so $(DESTDIR)$(libdir)/sudo/sudo_pair.so
	$(INSTALL_DATA)       ./etc/sudo_pair.prompt.pair                 $(DESTDIR)$(sysconfdir)
	$(INSTALL_DATA)       ./etc/sudo_pair.prompt.user                 $(DESTDIR)$(sysconfdir)
	$(INSTALL_DATA) -m440 ./etc/sudoers.d/sudo_pair                   $(DESTDIR)$(sysconfdir)/sudoers.d

	echo "Plugin sudo_pair sudo_pair.so gids_enforced=$(nogroup_gid)" >> $(DESTDIR)$(sysconfdir)/sudo.conf

$(CARGO_TARGET_DIR)/$(PROFILE)/libsudopair.so:
	export CARGO_TARGET_DIR

	cargo build --verbose --manifest-path ../Cargo.toml --release $(CARGOFLAGS)

# TODO: express rust dependencies
# TODO: target as dylib/so
# TODO: pre/post install command categories?
