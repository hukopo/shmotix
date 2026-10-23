#!/usr/bin/env bash
# Сборка, релиз на GitHub и обновление cask в hukopo/homebrew-tap.
# Версию берёт из src-tauri/tauri.conf.json — подними её перед запуском.
# Подпись — постоянным сертификатом из связки ключей: тогда macOS не сбрасывает
# разрешение «Запись экрана» после обновления.
set -euo pipefail
cd "$(dirname "$0")/.."
V=$(grep -m1 '"version"' src-tauri/tauri.conf.json | cut -d'"' -f4)
export APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:-Shmotix Self-Signed}"

npx tauri build --bundles dmg
mkdir -p dist
DMG="dist/Shmotix-$V-macos-arm64.dmg"
cp "src-tauri/target/release/bundle/dmg/Shmotix_${V}_aarch64.dmg" "$DMG"
SHA=$(shasum -a 256 "$DMG" | cut -d' ' -f1)

gh release create "v$V" "$DMG" -t "v$V" --generate-notes

TAP=$(mktemp -d)
gh repo clone hukopo/homebrew-tap "$TAP" -- -q
sed -i '' -e "s/version \".*\"/version \"$V\"/" -e "s/sha256 \".*\"/sha256 \"$SHA\"/" "$TAP/Casks/shmotix.rb"
git -C "$TAP" commit -qam "shmotix $V" && git -C "$TAP" push -q
echo "Готово: brew upgrade --cask shmotix"
