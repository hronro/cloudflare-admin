#!/bin/sh

set -euo pipefail

echo 'Build executable...'
cargo build -r
echo 'Executable built.'
echo 'Preparing macOS app bundle...'
rm -rf target/release/macos
mkdir -p target/release/macos
cp -r assets/macOS/Cloudflare\ Admin.app target/release/macos/
mkdir -p target/release/macos/Cloudflare\ Admin.app/Contents/MacOS
cp -fp target/release/cloudflare-admin target/release/macos/Cloudflare\ Admin.app/Contents/MacOS/
touch -r target/release/cloudflare-admin target/release/macos/Cloudflare\ Admin.app
codesign --remove-signature target/release/macos/Cloudflare\ Admin.app
codesign --force --deep --sign - target/release/macos/Cloudflare\ Admin.app
echo 'macOS app bundle is ready at target/release/macos/Cloudflare\\ Admin.app'
echo 'Preparing DMG installer...'
ln -sf /Applications target/release/macos/
hdiutil create target/release/macos/cloudflare-admin.dmg \
	-volname 'Cloudflare Admin' \
	-fs HFS+ \
	-srcfolder target/release/macos \
	-ov -format UDZO
echo 'DMG installer is ready at target/release/macos/cloudflare-admin.dmg'
