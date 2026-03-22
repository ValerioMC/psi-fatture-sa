# PSI Fatture

Gestionale fatture per psicologi — applicazione desktop per macOS e Windows costruita con Tauri 2, Vue 3 e SQLite.

## Prerequisiti

| Strumento | Versione minima | Installazione |
|-----------|----------------|---------------|
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org/) oppure `brew install node` |
| **Rust** | 1.77+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Xcode Command Line Tools** | — | `xcode-select --install` |

> Xcode CLT è necessario perché Tauri usa il toolchain nativo Apple per compilare il binario macOS.

## Setup locale

```bash
# 1. Clona il repository
git clone <repo-url> && cd psi-fatture-sa

# 2. Installa le dipendenze frontend
npm install

# 3. Avvia in modalità sviluppo (hot-reload)
npm run tauri dev
```

L'applicazione si apre automaticamente in una finestra nativa.

## Build per distribuzione macOS

### Build non firmata (uso personale / test)

```bash
npm run tauri build
```

L'output si trova in:

```
src-tauri/target/release/bundle/
├── macos/
│   └── PSI Fatture.app        ← applicazione .app
└── dmg/
    └── PSI Fatture_0.1.0_aarch64.dmg   ← installer DMG
```

Puoi copiare `PSI Fatture.app` nella cartella `/Applications` o distribuire il `.dmg`.

> **Nota**: Senza firma digitale, macOS Gatekeeper blocca l'app al primo avvio.
> L'utente dovrà: tasto destro → Apri → confermare. Oppure da terminale:
> ```bash
> xattr -cr "/Applications/PSI Fatture.app"
> ```

### Build firmata (distribuzione a terzi)

Per distribuire l'app senza avvisi di sicurezza serve un **Apple Developer Account** (99 $/anno).

#### 1. Configura i certificati

1. Accedi a [developer.apple.com](https://developer.apple.com) → Certificates, IDs & Profiles
2. Crea un certificato **Developer ID Application**
3. Scaricalo e installalo nel Keychain (doppio clic sul `.cer`)
4. Verifica che sia presente:
   ```bash
   security find-identity -v -p codesigning
   ```
   Dovresti vedere una riga tipo:
   ```
   "Developer ID Application: Nome Cognome (TEAM_ID)"
   ```

#### 2. Configura le variabili d'ambiente

```bash
# Identità di firma (il nome esatto dal passo precedente)
export APPLE_SIGNING_IDENTITY="Developer ID Application: Nome Cognome (TEAM_ID)"

# Per la notarizzazione Apple (necessaria per distribuzione fuori dal Mac App Store)
export APPLE_ID="tua@email.com"
export APPLE_PASSWORD="app-specific-password"    # Genera da appleid.apple.com → Sicurezza → Password per le app
export APPLE_TEAM_ID="XXXXXXXXXX"                # Il tuo Team ID (visibile nel portale developer)
```

#### 3. Build + firma + notarizzazione

```bash
npm run tauri build
```

Tauri rileva automaticamente le variabili d'ambiente e:
- firma il `.app` con il certificato Developer ID
- invia il DMG ad Apple per la notarizzazione
- "pinza" il ticket di notarizzazione al DMG

Il DMG risultante può essere distribuito liberamente — macOS lo aprirà senza avvisi.

### Build per architettura specifica

```bash
# Solo Apple Silicon (M1/M2/M3/M4)
npm run tauri build -- --target aarch64-apple-darwin

# Solo Intel
npm run tauri build -- --target x86_64-apple-darwin

# Universal binary (entrambe le architetture)
npm run tauri build -- --target universal-apple-darwin
```

> Per il target Intel su un Mac Apple Silicon devi prima aggiungere il target:
> ```bash
> rustup target add x86_64-apple-darwin
> ```

## Build per Windows

Tauri **non supporta la cross-compilazione**: non puoi generare un `.exe` da macOS. Hai due opzioni.

### Opzione 1 — GitHub Actions (consigliata)

Il progetto include un workflow CI/CD che builda automaticamente per macOS e Windows.

**Come usarlo:**

```bash
# Crea un tag di versione e pusha
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions avvia le build su runner macOS e Windows. Al termine, trovi i file nella sezione **Releases** del repository (come draft da pubblicare).

**Output generato per release:**

| Piattaforma | File |
|-------------|------|
| macOS Apple Silicon | `PSI Fatture_0.1.0_aarch64.dmg` |
| macOS Intel | `PSI Fatture_0.1.0_x64.dmg` |
| Windows x64 | `PSI Fatture_0.1.0_x64-setup.exe` (installer NSIS) |

> Il workflow si trova in `.github/workflows/release.yml`.

### Opzione 2 — Build locale su Windows

Se hai accesso a una macchina Windows (fisica, VM o Parallels):

**Prerequisiti Windows:**
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/)
- [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (seleziona "Sviluppo di applicazioni desktop con C++")
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (già incluso in Windows 10 21H2+ e Windows 11)

```bash
npm install
npm run tauri build
```

L'output:
```
src-tauri/target/release/bundle/
└── nsis/
    └── PSI Fatture_0.1.0_x64-setup.exe   ← installer Windows
```

## Dati demo (seed)

Lo script `scripts/seed_db.py` popola il database locale con dati realistici, sovrascrivendo quelli esistenti.

**Prerequisito:** avviare l'app almeno una volta (crea e inizializza il DB).

```bash
bash scripts/seed.sh
# oppure
python3 scripts/seed_db.py

# percorso DB custom
python3 scripts/seed_db.py --db /percorso/database.db
```

Dati generati:

| Dato | Quantità |
|------|----------|
| Configurazione professionale | Dott.ssa Maria Demo (regime forfettario) |
| Pazienti | 50 |
| Prestazioni | 20 (€70–€120) |
| Appuntamenti | ~5.500 (≥6/giorno, 2025–2026) |
| Fatture mensili | ~750 (pagate/emesse/bozza per anno) |

Il caricamento è rapido (~2 s) perché scrive direttamente nel SQLite bypassando l'app.

## Struttura del progetto

```
psi-fatture-sa/
├── src/                    # Frontend Vue 3 + TypeScript
│   ├── api.ts              # Wrapper chiamate Tauri invoke
│   ├── types.ts            # Tipi condivisi frontend
│   ├── views/              # Pagine dell'applicazione
│   ├── components/         # Componenti UI riutilizzabili
│   ├── stores/             # State management (Pinia)
│   └── utils/              # Utility (formattazione, ecc.)
├── src-tauri/              # Backend Rust + Tauri
│   ├── src/
│   │   ├── app/
│   │   │   ├── controller/ # Comandi Tauri (API layer)
│   │   │   ├── service/    # Logica di business
│   │   │   ├── repository/ # Accesso dati (SeaORM)
│   │   │   ├── entity/     # Entità database
│   │   │   └── model/      # DTO e tipi condivisi
│   │   └── migration/      # Migrazioni schema SQLite
│   ├── tauri.conf.json     # Configurazione Tauri
│   └── Cargo.toml          # Dipendenze Rust
└── package.json            # Dipendenze frontend
```

## Stack tecnologico

- **Frontend**: Vue 3, TypeScript, Tailwind CSS 4, Pinia, Vite
- **Backend**: Rust, Tauri 2, SeaORM, SQLite
- **Build**: Tauri CLI, Vite, vue-tsc
