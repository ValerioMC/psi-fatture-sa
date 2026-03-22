#!/usr/bin/env bash
# Avvia il dev server Vite e attende che sia pronto su porta 1420.
# Usato come prerequisito per il debug Rust in RustRover.

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Termina eventuali processi Vite già in ascolto sulla porta
if lsof -ti:1420 > /dev/null 2>&1; then
    echo "[vite] porta 1420 già occupata, la libero..."
    lsof -ti:1420 | xargs kill -9 2>/dev/null
    sleep 1
fi

echo "[vite] avvio dev server..."
npm run dev &
VITE_PID=$!

echo "[vite] attendo che la porta 1420 sia pronta..."
ATTEMPTS=0
MAX_ATTEMPTS=30
until curl -s -o /dev/null -w "%{http_code}" http://localhost:1420 | grep -q "200\|304"; do
    ATTEMPTS=$((ATTEMPTS + 1))
    if [ $ATTEMPTS -ge $MAX_ATTEMPTS ]; then
        echo "[vite] timeout: il server non risponde dopo ${MAX_ATTEMPTS} secondi"
        kill $VITE_PID 2>/dev/null
        exit 1
    fi
    sleep 1
done

echo "[vite] pronto su http://localhost:1420 (PID: $VITE_PID)"
