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
git tag v0.9.0
git push origin v0.9.0
```

In alternativa: Actions > Release > *Run workflow*, indicando la versione.

Il tag determina la versione del bundle: `scripts/set-version.mjs` la scrive in
`package.json`, `src-tauri/tauri.conf.json` e `src-tauri/Cargo.toml` prima della build,
quindi i valori committati in quei file sono solo un segnaposto.

GitHub Actions avvia le build su runner macOS e Windows, crea la release come draft e la
pubblica come **latest** solo quando tutte e tre le piattaforme hanno caricato il bundle.

**Output generato per release:**

| Piattaforma | Asset |
|-------------|-------|
| macOS Apple Silicon | `PSI-Fatture-macOS-arm64.dmg` |
| macOS Intel | `PSI-Fatture-macOS-x64.dmg` |
| Windows x64 | `PSI-Fatture-Windows-x64-setup.exe` (installer NSIS) |

I nomi degli asset sono fissi e non contengono la versione: il sito vetrina vi punta con
URL stabili nella forma

```
https://github.com/ValerioMC/psi-fatture-sa/releases/latest/download/<asset>
```

che GitHub redirige sempre all'ultima release pubblicata. Perche' il redirect funzioni la
release deve essere pubblicata e non marcata come *pre-release*: il workflow lo garantisce.

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
| Configurazione professionale | Dott.ssa Maria Demo (psicoterapeuta e psicoanalista, regime forfettario) |
| Pazienti | 50 |
| Prestazioni | 20 (€70–€120) |
| Appuntamenti | ~1.500 (2025 → dicembre 2026, dimensionati sul budget mensile) |
| Fatture mensili | ~320, dal 2025 fino al mese corrente (pagate/emesse/bozza) |
| Fatturato | ~€6.300–6.950 netti/mese, sempre < €7.000/mese e < €85.000/anno (tetto forfettario) |

Il caricamento è rapido (~2 s) perché scrive direttamente nel SQLite bypassando l'app.

## Interfaccia

Il sistema visivo si chiama "Carta e inchiostro": fondo carta caldo, testo inchiostro
e un solo colore d'accento (indaco inchiostro) per selezione, azione primaria e focus.

- **Token**: `src/style.css` definisce una volta sola colori, tipografia, due raggi,
  tre ombre, la scala z-index e le transizioni. La palette standard di Tailwind è
  disattivata (`--color-*: initial`): una classe come `text-slate-500` non produce nulla.
- **Componenti base**: `src/components/ui/` (AppButton, AppCard, FormField, ComboBox,
  SegmentedControl, ToggleSwitch, AppDialog, ConfirmDialog, ToastHost, SkeletonRows,
  EmptyState, InvoiceSeal…). Le schermate usano solo questi.
- **Sigillo della fattura**: `InvoiceSeal.vue` disegna lo stato di una fattura
  (bozza, emessa con l'arco verso la scadenza, pagata, scaduta, annullata); la logica
  sta in `src/utils/invoiceSeal.ts`.
- **Tema**: chiaro, scuro o come il sistema, da Impostazioni. La scelta è salvata in
  `localStorage` (chiave `psi-fatture.theme`).
- **Scorciatoie**: ⌘K (Ctrl K su Windows) apre la ricerca di pazienti, azioni e sezioni.
- **Finestra macOS**: `titleBarStyle: "Overlay"` in `tauri.conf.json`; i semafori stanno
  sopra la barra laterale e intestazioni e barra laterale trascinano la finestra
  (permesso `core:window:allow-start-dragging`).

## Sistema Tessera Sanitaria

Le spese sanitarie delle fatture pagate si trasmettono al Sistema TS (Sogei) tramite il
**web service sincrono** `DocumentoSpesa730p`, un documento per chiamata, con l'esito
nella risposta. Tutto gira nel binario Rust: nessun servizio di terze parti.

- **Dove**: la pagina *Sistema TS* (da trasmettere, trasmissioni con esito, contenuto
  del Sistema TS mese per mese), il pannello nel dettaglio di ogni fattura (stato,
  scadenze, invio, sostituzione, annullamento, verifica online) e un segno a forma di
  tessera nella lista fatture.
- **Credenziali**: in Impostazioni → *Sistema Tessera Sanitaria*: ambiente, codice
  fiscale di accesso, partita IVA, password e PINCODE. Password e PINCODE stanno solo
  nel portachiavi del sistema operativo (crate `keyring`, servizio
  `it.psifatture.sistema-ts`); il frontend sa solo se ci sono. "Verifica credenziali"
  fa una chiamata reale.
- **Ambienti**: *Produzione* (`invioSS730p.sanita.finanze.it`) e *Test Sogei*
  (`invioSS730pTest…`, senza valore fiscale). In test si può compilare con un clic
  l'utenza pubblica "Psicologo" del kit Sogei. Ogni trasmissione ricorda il proprio
  ambiente.
- **Protocollo**: Basic auth con CF e password; PINCODE, CF del professionista e del
  paziente cifrati RSA PKCS#1 v1.5 con il certificato `SanitelCF`
  (`src-tauri/resources/sistema_ts/SanitelCF.pem`, **scade il 23/01/2027**: va
  sostituito con quello del nuovo kit). L'host di test usa la CA *Sogei Certification
  Authority Test*, inclusa in `SogeiTestCA.pem` e accettata solo per quell'ambiente.
- **Cosa si invia**: tipo spesa `SP`, importo = onorario + contributo ENPAP (senza
  bollo), natura IVA `N2.2` (forfettario) o `N4` (ordinario, esente), pagamento
  tracciato per bonifico e POS, opposizione del paziente senza codice fiscale.
- **Coda offline**: tabella `ts_submissions` (`invio` / `sostituzione` /
  `annullamento`; stati `non_inviata → inviata → accettata | scartata`, poi
  `annullata` / `sostituita`). Un worker la svuota ogni minuto; errori di rete e
  `WS99` si ritentano con backoff da 1 minuto a 6 ore; credenziali rifiutate o
  servizio irraggiungibile fermano il giro senza scartare nulla.
- **Lista e verifica**: report mensile (`ReportMensile730`, per data di invio o di
  pagamento) e interrogazione puntuale del singolo documento.
- **Scadenze**: invio entro il 31 gennaio dell'anno dopo il pagamento, correzioni
  gratuite nei 5 giorni successivi (spostati al lunedì se cadono nel weekend).
- **Cancellazione fatture**: non si elimina una fattura i cui dati sono sul Sistema TS
  di produzione; prima va annullato l'invio.

Test contro l'ambiente di test Sogei (servono rete e l'utenza pubblica del kit):

```bash
cd src-tauri && cargo test live_ -- --ignored
```

## Screenshot per il sito vetrina

`scripts/screenshots/capture.mjs` genera le immagini WebP di psifatture.it dal dev
server, con l'IPC di Tauri sostituito da dati demo (`mock-tauri.js`). Non pilota la
finestra nativa, quindi macOS non chiede permessi di registrazione schermo.

```bash
npm run dev -- --port 1421        # in un altro terminale
npm install --no-save playwright sharp
node scripts/screenshots/capture.mjs ../psi-fatture-brochure/public/screenshots
```

## Struttura del progetto

```
psi-fatture-sa/
├── src/                    # Frontend Vue 3 + TypeScript
│   ├── api.ts              # Wrapper chiamate Tauri invoke
│   ├── types.ts            # Tipi condivisi frontend
│   ├── style.css           # Token del design system, campi, movimenti
│   ├── views/              # Pagine dell'applicazione
│   ├── components/
│   │   ├── ui/             # Componenti base (bottoni, campi, dialoghi, sigillo…)
│   │   ├── layout/         # Barra laterale, layout, palette ⌘K
│   │   ├── dashboard/      # Grafico mensile, soglia forfettario, stima fiscale
│   │   ├── profile/        # Sezioni del profilo, credenziali Sistema TS
│   │   └── sts/            # Pannelli del Sistema Tessera Sanitaria
│   ├── composables/        # Tema, focus trap, form profilo, smooth scroll
│   ├── stores/             # State management (Pinia), notifiche
│   └── utils/              # Formattazione, fisco, validazione, stato fatture
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

## Test

```bash
# Frontend (vitest): logica fiscale, validazione (codice fiscale, P.IVA, IBAN…),
# sigillo delle fatture, soglia forfettario, date locali, ricorrenze, tema, notifiche
npm test

# Backend (cargo): calcolo totali fattura, validazione input, Sistema TS (buste SOAP,
# risposte reali catturate, coda, invio con gateway finto). I test usano un portachiavi
# in memoria: non toccano quello di sistema
cd src-tauri && cargo test
```

## Qualità del codice Rust

Il progetto integra **Rustfmt** (formatter) e **Clippy** (linter), entrambi strumenti ufficiali del toolchain Rust. Non richiedono installazione separata: sono già inclusi se hai Rust via `rustup`.

Il workflow `.github/workflows/lint.yml` esegue entrambi i check automaticamente ad ogni push e pull request su `main`. Un warning Clippy non risolto blocca il check CI.

### Rustfmt — formattazione

```bash
cd src-tauri

# Formatta tutti i file .rs
cargo fmt

# Verifica senza modificare (usato in CI)
cargo fmt --check
```

La configurazione si trova in `src-tauri/rustfmt.toml`:

| Opzione | Valore | Effetto |
|---------|--------|---------|
| `edition` | `"2021"` | Allineato all'edizione Rust del progetto |
| `max_width` | `100` | Lunghezza massima di riga (default 80) |
| `imports_granularity` | `"Module"` | Raggruppa gli `use` per modulo |
| `group_imports` | `"StdExternalCrate"` | Ordine: std → crate esterne → crate interne |

### Clippy — linting

```bash
cd src-tauri

# Analisi con tutti i warning (stessa modalità del CI)
cargo clippy -- -D warnings

# Analisi permissiva (solo output, non blocca)
cargo clippy
```

### VS Code

Il file `.vscode/settings.json` già presente nel repo configura:

- **Format-on-save** automatico per i file `.rs`
- **Clippy come checker** in tempo reale al posto del semplice `cargo check` — i warning appaiono direttamente nell'editor mentre scrivi

## Stack tecnologico

- **Frontend**: Vue 3, TypeScript, Tailwind CSS 4, Pinia, Vite
- **Backend**: Rust, Tauri 2, SeaORM, SQLite, keyring (portachiavi di sistema),
  reqwest + rustls, quick-xml, rsa (Sistema TS)
- **Build**: Tauri CLI, Vite, vue-tsc
- **Test**: Vitest (frontend), cargo test (backend)

I font (Geist, Geist Mono, Newsreader) sono self-hosted via `@fontsource-variable`:
l'app rende correttamente anche offline, senza dipendere da Google Fonts.
