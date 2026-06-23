# Distro-agnostic build/install. Works on any Linux (Arch, Debian/Ubuntu, Fedora…).
#
#   make                      # build the release binary
#   sudo make install         # install system-wide (PREFIX=/usr/local by default)
#   make install PREFIX=/usr DESTDIR=pkgroot   # staged install for deb/rpm/PKGBUILD
#   sudo make uninstall
#
# After installing, each user runs:  claude-status-bar install   (wires hooks)

PREFIX  ?= /usr/local
DESTDIR ?=
BIN      = claude-status-bar
PLASMOID = com.abyot.claudestatusbar

BINDIR      = $(DESTDIR)$(PREFIX)/bin
PLASMOIDDIR = $(DESTDIR)$(PREFIX)/share/plasma/plasmoids/$(PLASMOID)
DOCDIR      = $(DESTDIR)$(PREFIX)/share/doc/$(BIN)
LICENSEDIR  = $(DESTDIR)$(PREFIX)/share/licenses/$(BIN)

.PHONY: all build install install-plasmoid uninstall clean

all: build

build:
	cargo build --release --locked

# Binary + the KDE plasmoid (with the installed binary path baked into the QML).
install: build
	install -Dm755 target/release/$(BIN) $(BINDIR)/$(BIN)
	install -Dm644 plasmoid/metadata.json            $(PLASMOIDDIR)/metadata.json
	install -Dm644 plasmoid/contents/ui/main.qml     $(PLASMOIDDIR)/contents/ui/main.qml
	install -Dm644 plasmoid/contents/icons/claude.png $(PLASMOIDDIR)/contents/icons/claude.png
	sed -i "s|__SESSIONS_CMD__|$(PREFIX)/bin/$(BIN) sessions|g" $(PLASMOIDDIR)/contents/ui/main.qml
	install -Dm644 README.md $(DOCDIR)/README.md
	install -Dm644 LICENSE   $(LICENSEDIR)/LICENSE

uninstall:
	rm -f  $(BINDIR)/$(BIN)
	rm -rf $(PLASMOIDDIR)
	rm -rf $(DOCDIR)
	rm -rf $(LICENSEDIR)

clean:
	cargo clean
