#!/usr/bin/env bash
set -euo pipefail
app_root=$(cd "$(dirname "$0")/.." && pwd)
repository_root=$(cd "$app_root/.." && pwd)
cd "$app_root"
profile=${1:-release}
if [[ "$profile" != release && "$profile" != debug ]]; then
    printf 'Usage: %s [release|debug]\n' "$0" >&2
    exit 2
fi
cargo test --locked -p tremors-core
arguments=(build --locked -p tremors-music)
if [[ "$profile" == release ]]; then arguments+=(--release); fi
cargo "${arguments[@]}"
version=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -1)
architecture=$(uname -m)
case "$architecture" in
    x86_64) architecture=x64 ;;
    aarch64) architecture=arm64 ;;
esac
suffix=""
if [[ "$profile" == debug ]]; then suffix="-debug"; fi
name="TremorsMusic-$version-linux-$architecture$suffix"
mkdir -p dist
staging=$(mktemp -d "$PWD/dist/linux-package.XXXXXX")
trap 'rm -rf -- "$staging"' EXIT
mkdir "$staging/$name"
target_directory=${CARGO_TARGET_DIR:-target}
install -m 755 "$target_directory/$profile/tremors-music" "$staging/$name/tremors-music"
cp "$repository_root/"{LICENSE.md,README.md,DEVELOPMENT.md,CHANGELOG.md,TASKS.md,PRIVACY.md} "$staging/$name/"
cp "$repository_root/assets/TremorsMusic.png" "$staging/$name/tremorsmusic.png"
# Ship the exact local source, including new files and uncommitted edits.
git -C "$repository_root" ls-files --cached --others --exclude-standard -z > "$staging/source-candidates"
while IFS= read -r -d '' file; do
    if [[ -f "$repository_root/$file" ]]; then printf '%s\0' "$file"; fi
done < "$staging/source-candidates" > "$staging/source-files"
mkdir "$staging/source"
tar -C "$repository_root" --null -T "$staging/source-files" -cf - | tar -C "$staging/source" -xf -
(
    cd "$staging/source/tremors-music-app"
    cargo vendor --locked --versioned-dirs vendor > "$staging/vendor-config"
    printf '\n' >> .cargo/config.toml
    cat "$staging/vendor-config" >> .cargo/config.toml
)
source_name="$name-source.tar.gz"
tar -C "$staging/source" -czf "dist/$source_name" .
(cd dist && sha256sum "$source_name" > "$source_name.sha256")
cat > "$staging/$name/SOURCE.txt" <<INFO
Tremors Music $version is licensed GPL-3.0-only; see LICENSE.md.
Complete corresponding source for this build is available separately in:
$source_name
Download it from the same release:
https://github.com/qtremors/tremors-music/releases/tag/v$version

The archive includes application and dependency sources, notices, build scripts,
lockfile and shaders. Publish this matching source archive alongside binary
downloads at no extra charge when redistributing under GPL v3.
INFO
cat > "$staging/$name/tremors-music.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Tremors Music
Comment=Your music. Your space.
Exec=tremors-music
Icon=tremorsmusic
Terminal=false
Categories=AudioVideo;Audio;Player;
DESKTOP
cat > "$staging/$name/START-HERE.txt" <<'INFO'
Run ./tremors-music from an extracted folder on a Linux desktop.
ALSA, Fontconfig/Freetype, X11/Wayland/XKB and a Vulkan driver must be installed.
A session D-Bus enables media controls. A desktop portal enables the folder picker;
typed folder paths also work. See DEVELOPMENT.md for build/runtime requirements, checks, and dependency notices.

This is a native tar package, not a self-contained AppImage. It was built locally;
compatibility with other Linux distributions must be checked.

Licensed under GNU GPL v3; see LICENSE.md. Complete corresponding application
source, build scripts and vendored dependency sources are available as a separate
source archive on the same GitHub release. See SOURCE.txt for its exact filename.
Dependencies retain their own licenses; their notices are included with their sources.
INFO
tar -C "$staging" -czf "dist/$name.tar.gz" "$name"
(cd dist && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
printf 'Created dist/%s.tar.gz\n' "$name"
printf 'Created dist/%s\n' "$source_name"
