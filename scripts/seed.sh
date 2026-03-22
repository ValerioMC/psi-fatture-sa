#!/usr/bin/env bash
# seed.sh — Wrapper per lanciare il seed del database PSI Fatture SA
#
# Uso:
#   bash scripts/seed.sh            # usa il DB di default
#   bash scripts/seed.sh --db PATH  # specifica un percorso custom
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v python3 &>/dev/null; then
    echo "❌  python3 non trovato. Installalo per continuare."
    exit 1
fi

exec python3 "$SCRIPT_DIR/seed_db.py" "$@"
