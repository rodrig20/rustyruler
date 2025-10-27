# Maintainer: rodrig20
pkgname=rustyruler
pkgver=0.1.0
pkgrel=1
pkgdesc="A lightweight and efficient ruler tool built with Rust and GTK4"
arch=('x86_64')
url="https://github.com/rodrig20/rustyruler"
license=('MIT')
depends=('gtk4' 'glib2' 'librsvg' 'cairo' 'pango' 'atk' 'gdk-pixbuf2')
makedepends=('rust' 'cargo')
options=(!strip)

build() {
    cd $srcdir
    cargo build --release
}

package() {
    cd $srcdir/..
    install -Dm755 target/release/rustyruler "$pkgdir/usr/bin/rustyruler"
}