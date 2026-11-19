# Maintainer: olal3
pkgname=ani-es-rs
pkgver=0.1.3
pkgrel=1
pkgdesc="Cliente de anime en terminal para jkanime.net"
arch=('x86_64')
url="https://github.com/olalekanob6/ani-es-rs"
license=('MIT')
depends=('mpv' 'fzf')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/olalekanob6/ani-es-rs/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('3eb0396ec2d4d287a48ecad28f9811079219cae6f9577950f6eb7b1541c806ee')

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
