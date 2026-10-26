# Maintainer: tu-usuario <tu-email@example.com>
pkgname=ani-es-rs
pkgver=0.1.0
pkgrel=1
pkgdesc="Cliente de anime en terminal para jkanime.net"
arch=('x86_64')
url="https://github.com/olalekanob6/ani-es-rs"
license=('MIT')
depends=('mpv' 'fzf')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/olalekanob6/ani-es-rs/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm0755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm0644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm0644 config.example.toml "$pkgdir/usr/share/doc/$pkgname/config.example.toml"
  install -Dm0644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
