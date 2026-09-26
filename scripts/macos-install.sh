#!/usr/bin/env bash
# Mounts the disk image `make build` just produced, copies PSI Fatture.app into /Applications,
# and ejects the disk image again.
#
#   scripts/macos-install.sh [path/to/PSI Fatture_*.dmg]
#
# Unlike scripts/macos-app.sh, this replaces whatever is at /Applications/PSI Fatture.app. A PSI
# Fatture that is already running (from /Applications or from a mounted disk image) is quit
# first, so the copy is never in use while it is replaced.
set -euo pipefail

if [ "$(uname -s)" != "Darwin" ]; then
    echo "This mounts a macOS disk image and only runs on macOS." >&2
    exit 1
fi

cd "$(dirname "$0")/.."

DMG="${1:-$(ls -t src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1 || true)}"
if [ -z "$DMG" ] || [ ! -f "$DMG" ]; then
    echo "No disk image under src-tauri/target/release/bundle/dmg. Run 'make build' first." >&2
    exit 1
fi

if pgrep -xq "psi-fatture-sa" || pgrep -xq "PSI Fatture"; then
    echo "==> Quitting the running PSI Fatture"
    osascript -e 'quit app "PSI Fatture"' >/dev/null 2>&1 || true
    for _ in $(seq 1 40); do
        { pgrep -xq "psi-fatture-sa" || pgrep -xq "PSI Fatture"; } || break
        sleep 0.25
    done
fi

# Every volume a PSI Fatture disk image was mounted on earlier ("PSI Fatture", "PSI Fatture 1", ...).
for volume in "/Volumes/PSI Fatture"*; do
    [ -d "$volume/PSI Fatture.app" ] || continue
    echo "==> Ejecting $volume"
    hdiutil detach "$volume" -quiet || hdiutil detach "$volume" -force -quiet || true
done

echo "==> Mounting $(basename "$DMG")"
MOUNT="$(hdiutil attach -noverify -noautoopen "$DMG" | awk -F'\t' '/\/Volumes\// { print $NF }' | tail -1)"
if [ -z "$MOUNT" ] || [ ! -d "$MOUNT/PSI Fatture.app" ]; then
    echo "The disk image mounted, but no PSI Fatture.app was found on it." >&2
    exit 1
fi

echo "==> Installing into /Applications"
rm -rf "/Applications/PSI Fatture.app"
cp -R "$MOUNT/PSI Fatture.app" "/Applications/PSI Fatture.app"
xattr -dr com.apple.quarantine "/Applications/PSI Fatture.app" 2>/dev/null || true

echo "==> Ejecting $MOUNT"
hdiutil detach "$MOUNT" -quiet || hdiutil detach "$MOUNT" -force -quiet || true

echo
echo "    Installed at /Applications/PSI Fatture.app"
