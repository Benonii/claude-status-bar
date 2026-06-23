# Maintainer: benoni <benoni@abyot.com>
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

    # Rust binary: `hook`, `sessions`, `install`/`uninstall`, and the tray fallback.
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

    # Plasma 6 plasmoid, with the absolute `sessions` command baked into the QML
    # (replaces the __SESSIONS_CMD__ placeholder that the dev script substitutes).
    local pdir="$pkgdir/usr/share/plasma/plasmoids/com.abyot.claudestatusbar"
    install -Dm644 plasmoid/metadata.json            "$pdir/metadata.json"
    install -Dm644 plasmoid/contents/ui/main.qml     "$pdir/contents/ui/main.qml"
    install -Dm644 plasmoid/contents/icons/claude.png "$pdir/contents/icons/claude.png"
    sed -i "s|__SESSIONS_CMD__|/usr/bin/$pkgname sessions|g" "$pdir/contents/ui/main.qml"

    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 LICENSE   "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
