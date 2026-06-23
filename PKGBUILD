# Maintainer: benoni <besckinder@gmail.com>
# Builds the Rust binary (Claude Code hooks + `sessions` query + SNI tray fallback)
# and installs the Plasma 6 panel widget system-wide.
#
# Local install (from this repo):   makepkg -si
# For the AUR, replace the build/package `$startdir` usage with a real source=()
# (e.g. source=("$pkgname::git+https://github.com/<you>/claude-status-bar-arch.git")).

pkgname=claude-status-bar
pkgver=1.0.0
pkgrel=1
pkgdesc="Claude Code activity in the KDE Plasma panel (Rust hooks + QML plasmoid)"
arch=('x86_64')
url="https://github.com/m1ckc3s/claude-status-bar"
license=('MIT')
# dbus: SNI tray fallback links libdbus-1.  plasma-workspace: provides the panel
# (plasmoid host) and the plasma5support executable datasource the widget reads with.
depends=('dbus' 'plasma-workspace' 'gcc-libs' 'glibc')
makedepends=('cargo')
install="$pkgname.install"

build() {
    cd "$startdir"
    export CARGO_TARGET_DIR=target
    cargo build --release --frozen
}

package() {
    cd "$startdir"
    # The Makefile bakes /usr/bin/claude-status-bar into the plasmoid's QML and
    # lays out binary + plasmoid + docs + license under $pkgdir.
    make install PREFIX=/usr DESTDIR="$pkgdir"
    # Panel-config snippets for non-KDE desktops.
    install -Dm644 packaging/panels.md "$pkgdir/usr/share/doc/$pkgname/panels.md"
}
