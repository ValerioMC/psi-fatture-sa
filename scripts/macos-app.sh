#!/usr/bin/env bash
# Mounts the disk image `make build` just produced and runs PSI Fatture straight from it.
#
#   scripts/macos-app.sh [path/to/PSI Fatture_*.dmg]
#
# Nothing is copied into /Applications: the build you are trying is the one on the disk image,
# exactly what another machine would get, and an installed copy (if there is one) is left alone.
# A running PSI Fatture is quit first and a disk image from an earlier build is ejected, so the
# one that opens is always the one just built.
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

echo "==> Running $MOUNT/PSI Fatture.app"
open "$MOUNT/PSI Fatture.app"
echo
echo "    Running from the disk image; eject \"$(basename "$MOUNT")\" in Finder when you are done."
