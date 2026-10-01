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
- **Componenti base**: `src/components/ui/` (AppButton, ActionButton, AppCard, FormField,
  ComboBox, SegmentedControl, ToggleSwitch, AppDialog, ConfirmDialog, ToastHost,
  SkeletonRows, EmptyState, InvoiceSeal, TsMark, MailMark…). Le schermate usano solo questi;
  varianti e misure dei bottoni stanno una volta sola in `buttonStyles.ts`.
- **Bottoni che mostrano l'esito**: `ActionButton.vue` esegue l'azione e la racconta nel
  bottone stesso, nel suo colore: con `motion="send"` l'aeroplano di carta esce a destra,
  poi tre punti finché il server non risponde, poi un cerchio con la spunta che si
  disegna. Lo usano l'invio email, i salvataggi della casella e del modello, "Salva PDF"
  e le verifiche di collegamento (email e Sistema TS).
- **Impostazioni per contesto**: `/settings/profile`, `/invoicing`, `/email`, `/sts`,
  `/appearance`, con una navigazione laterale che segnala con un punto ciò che manca
  (casella da configurare, credenziali TS). Il modulo del profilo è condiviso da Profilo e
  Fatturazione (`useSettingsProfile.ts`) e si salva da un'unica barra.
- **Sigillo della fattura**: `InvoiceSeal.vue` disegna lo stato di una fattura
  (bozza, emessa con l'arco verso la scadenza, pagata, scaduta, annullata); la logica
  sta in `src/utils/invoiceSeal.ts`.
- **Tema**: chiaro, scuro o come il sistema, da Impostazioni → Aspetto. La scelta è salvata in
  `localStorage` (chiave `psi-fatture.theme`).
- **Scorciatoie**: ⌘K (Ctrl K su Windows) apre la ricerca di pazienti, azioni e sezioni.
- **Finestra macOS**: `titleBarStyle: "Overlay"` in `tauri.conf.json`; i semafori stanno
  sopra la barra laterale e intestazioni e barra laterale trascinano la finestra
  (permesso `core:window:allow-start-dragging`).

## Calcolo della fattura

Il backend (`src-tauri/src/app/service/tax_service.rs`) calcola i totali salvati;
`src/utils/tax.ts` ripete la stessa logica per l'anteprima nel modulo e va tenuto allineato.

- **Base ENPAP 2%**: nel forfettario la marca da bollo addebitata al paziente è compenso
  (Interpello AdE) e concorre alla base; nell'ordinario la base è il solo imponibile.
  Impostazioni → Fatturazione → *Escludi la marca da bollo dal calcolo ENPAP* toglie il
  bollo dalla base anche nel forfettario.
- **Importo a mano**: ogni riga ha un campo *Importo* che vale quantità × prezzo finché non
  lo si scrive a mano; da lì l'importo scritto (`invoice_lines.amount_override`) sostituisce
  il prodotto in tutti i calcoli, e in fattura la riga non mostra il prezzo unitario.
  *Ricalcola* torna al prodotto.
- **Quantità e prezzo unitario in fattura**: la scelta è salvata sulla fattura
  (`invoices.hide_quantity`). Una fattura nuova parte dall'impostazione del paziente
  (scheda paziente → *Fattura*), se c'è, altrimenti da quella in Impostazioni → Fatturazione.

## Sistema Tessera Sanitaria

Le spese sanitarie delle fatture pagate si trasmettono al Sistema TS (Sogei) tramite il
**web service sincrono** `DocumentoSpesa730p`, un documento per chiamata, con l'esito
nella risposta. Tutto gira nel binario Rust: nessun servizio di terze parti.

- **Dove**: la pagina *Sistema TS* (da trasmettere, trasmissioni con esito, contenuto
  del Sistema TS mese per mese), il pannello nel dettaglio di ogni fattura (stato,
  scadenze, invio, sostituzione, annullamento, verifica online) e un segno a forma di
  tessera nella lista fatture.
- **Credenziali**: in Impostazioni → *Sistema TS*: codice
  fiscale di accesso, partita IVA, password e PINCODE. Una guida passo passo nella card
  spiega dove trovarli (sistemats.it → *Profilo utente → Stampa credenziali*); finché
  mancano, la pagina *Sistema TS* e il pannello della fattura lo segnalano con un
  rimando diretto alla pagina (`/settings/sts`). Password e PINCODE li custodisce
  l'app in `secrets.json`, accanto al database: AES-256-GCM con una chiave derivata
  (HKDF-SHA256) dall'identificativo del computer e da un sale casuale, file leggibile
  solo dall'utente. Mai in chiaro e nessun accesso al portachiavi di sistema: copiato
  su un altro computer il file non si decifra, e l'app chiede di reinserirli. Il
  frontend sa solo se ci sono. "Verifica credenziali"
  fa una chiamata reale.
- **Ambienti**: *Produzione* (`invioSS730p.sanita.finanze.it`) e *Test Sogei*
  (`invioSS730pTest…`, senza valore fiscale). Il test esiste solo nelle build per lo
  sviluppatore: `npm run tauri dev` / `make dev` (build debug), `make build-sogei`
  (release con la feature Cargo `sogei-test`) e `make install`, che installa in
  `/Applications` proprio quella release. Il bundle che lasciano in
  `src-tauri/target/release/bundle/` non va distribuito. Per installare in locale
  invece la build cliente, senza ambiente di test, c'è `make install-prod` (usa
  `make build`).
  Lì un riquadro tratteggiato in cima alla card permette di passare a *Test Sogei* e
  compilare con un clic l'utenza pubblica "Psicologo" del kit. La build distribuita
  (`make build`, CI di release) conosce solo la produzione: il backend rifiuta di
  salvare l'ambiente di test, ignora un ambiente di test salvato in precedenza (torna
  ai dati del profilo) e il gateway non chiama l'host di test. Ogni trasmissione
  ricorda il proprio ambiente.
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

## Fatture via email ai pazienti

Ogni fattura emessa si può mandare al paziente con il PDF allegato, dalla fattura o, per
più fatture insieme, dalla barra di selezione della lista. Tutto passa dal binario Rust:
SMTP diretto verso il provider della casella, nessun servizio di terze parti.

- **Casella**: Impostazioni → *Email*. I provider noti portano con sé il server:
  **psypec.it** (la PEC gratuita dell'Ordine, gestita da Namirial:
  `smtps.sicurezzapostale.it`, porta 465, SSL/TLS), PEC Aruba (`smtps.pec.aruba.it`),
  Gmail (`smtp.gmail.com`, solo con password per le app) e *Altro provider* con server,
  porta, sicurezza e utente a mano. Il provider si riconosce dal dominio dell'indirizzo;
  prima del primo salvataggio la casella è proposta dalla PEC del profilo. La password sta
  in `secrets.json` come quelle del Sistema TS (voce `email_password`); "Verifica
  collegamento" fa un accesso reale al server senza inviare nulla.
- **PEC verso caselle normali**: il paziente riceve una busta di posta certificata con il
  messaggio originale allegato (`postacert.eml`); l'app lo dice nelle impostazioni e
  nell'invio.
- **Modello**: oggetto e testo con segnaposto tra graffe (`{paziente}`, `{nome_paziente}`,
  `{numero_fattura}`, `{data_fattura}`, `{importo}`, `{scadenza}`, `{professionista}`).
  Un nome sconosciuto viene rifiutato al salvataggio; "Ripristina" torna al modello
  predefinito. L'anteprima accanto all'editor è la resa del backend con un paziente di esempio.
- **Invio**: il dialogo propone destinatario (dalla scheda del paziente), oggetto e testo,
  tutti modificabili; il PDF si apre per controllarlo. Se il paziente non ha un indirizzo,
  quello digitato si può salvare nella sua scheda. Bozze e fatture annullate non partono.
  L'invio in blocco fa prima un accesso di prova, poi invia una fattura alla volta e salta
  chi non ha un indirizzo.
- **Registro**: tabella `invoice_emails`, una riga per tentativo (`sent` / `failed` con
  l'errore). La fattura mostra stato e storico; la lista un segno a forma di busta
  (`MailMark.vue`): tratteggiata se non inviata, inchiostrata con il sigillo se inviata,
  rossa se l'unico tentativo è fallito.
- **PDF**: generato nel backend (`service/pdf/`, libreria `krilla`) con la stessa
  impaginazione della vista di stampa: A4, piè di pagina su ogni pagina, numerazione
  delle pagine quando sono più d'una, righe che continuano sulla pagina seguente con
  l'intestazione della tabella. I caratteri (Geist e Newsreader, licenza OFL) sono in
  `src-tauri/resources/fonts/`, inclusi nel binario e ridotti ai soli glifi usati. Dalla
  fattura, "Salva PDF" scrive lo stesso file dove si sceglie.

Per rivedere l'impaginazione dopo una modifica:

```bash
cd src-tauri && cargo test preview_sample_invoice -- --ignored   # scrive target/invoice-preview.pdf
```

## Condizioni d'uso e licenza

Il codice è distribuito con licenza MIT (`LICENSE`), che esclude ogni garanzia. Le condizioni
d'uso stanno in `src/legal/terms.ts` e si accettano prima di qualsiasi altra schermata.

- **Accettazione**: `TermsView.vue` mostra il testo con due caselle separate, una per le
  condizioni e una per l'approvazione specifica delle clausole di limitazione (artt. 1341 e
  1342 c.c.). Il router non apre nessuna pagina, onboarding compreso, finché la versione
  corrente non è accettata.
- **Registro**: tabella `terms_acceptances`, una riga per versione accettata con data e
  approvazione specifica. `terms_service` rifiuta l'accettazione se una delle due caselle
  manca; accettare due volte la stessa versione tiene la prima riga.
- **Nuova versione**: se il testo cambia, aggiorna `TERMS_VERSION`. Chi aggiorna l'app vede
  di nuovo la schermata alla prima apertura, anche con un database già pieno.
- **Rilettura**: Impostazioni → *Condizioni d'uso* mostra il testo e la data di accettazione.

Il testo delle condizioni non è stato rivisto da un legale.

## Screenshot per il sito vetrina

`scripts/screenshots/capture.mjs` genera le immagini WebP di psifatture.it dal dev
server, con l'IPC di Tauri sostituito da dati demo (`mock-tauri.js`). Non pilota la
finestra nativa, quindi macOS non chiede permessi di registrazione schermo.

```bash
npm run dev -- --port 1421        # in un altro terminale
npm install --no-save playwright sharp
node scripts/screenshots/capture.mjs ../psi-fatture-brochure/public/screenshots
```

Oltre alle schermate intere lo script ritaglia quattro parti dell'app per la sezione
Sistema TS ed email del sito: `sts-coda`, `sts-fattura`, `email-invio` ed `email-casella`,
ciascuna chiara e `-dark`. `ONLY=shots` o `ONLY=excerpts` limita il giro a uno dei due gruppi.
Installa `playwright` e `sharp` nello stesso comando: un `npm install --no-save` successivo
rimuove i pacchetti installati senza salvarli.

## Struttura del progetto

```
psi-fatture-sa/
├── src/                    # Frontend Vue 3 + TypeScript
│   ├── api.ts              # Wrapper chiamate Tauri invoke
│   ├── types.ts            # Tipi condivisi frontend
│   ├── style.css           # Token del design system, campi, movimenti
│   ├── views/              # Pagine dell'applicazione (settings/: una per contesto)
│   ├── components/
│   │   ├── ui/             # Componenti base (bottoni, campi, dialoghi, sigillo…)
│   │   ├── layout/         # Barra laterale, layout, palette ⌘K
│   │   ├── dashboard/      # Grafico mensile, soglia forfettario, stima fiscale
│   │   ├── profile/        # Sezioni del profilo, navigazione impostazioni, credenziali
│   │   ├── email/          # Casella, modello, invio e storico delle email ai pazienti
│   │   ├── legal/          # Testo delle condizioni d'uso, uguale in accettazione e in Impostazioni
│   │   └── sts/            # Pannelli del Sistema Tessera Sanitaria
│   ├── composables/        # Tema, focus trap, form profilo, smooth scroll
│   ├── legal/              # Condizioni d'uso e loro versione
│   ├── stores/             # State management (Pinia), notifiche
│   └── utils/              # Formattazione, fisco, validazione, stato fatture
├── src-tauri/              # Backend Rust + Tauri
│   ├── src/
│   │   ├── lib.rs          # Avvio: stato condiviso, worker, registrazione comandi
│   │   ├── app/
│   │   │   ├── app_state.rs # Stato condiviso dai comandi (DB, credenziali, gateway)
│   │   │   ├── controller/ # Comandi Tauri (API layer); ts/ per il Sistema TS
│   │   │   ├── service/    # Logica di business; ts/ Sistema TS, email/ invii, pdf/ fattura in PDF
│   │   │   ├── repository/ # Accesso dati (SeaORM, SQL) e servizi esterni
│   │   │   │   ├── invoice/    # Fatture e proiezione delle righe lette
│   │   │   │   ├── secret/     # Archivio cifrato delle credenziali
│   │   │   │   ├── email/      # Casella, modello, registro invii e gateway SMTP (lettre)
│   │   │   │   └── ts/         # Coda STS; sistema_ts/ è il client SOAP
│   │   │   ├── scheduler/  # Worker in background (invio coda STS)
│   │   │   ├── entity/     # Entità database (una tabella per file)
│   │   │   ├── model/      # DTO e tipi condivisi, un sottopackage per dominio
│   │   │   └── db/         # Connessione e percorsi dei file dati
│   │   ├── migration/      # Migrazioni schema SQLite
│   │   └── test_support/   # Test double e fixture condivisi (solo in `cargo test`)
│   ├── tauri.conf.json     # Configurazione Tauri
│   └── Cargo.toml          # Dipendenze Rust
├── LICENSE                 # Licenza MIT
└── package.json            # Dipendenze frontend
```

### Convenzioni del backend

- **Strati**: `controller → service → repository → model/entity`. I controller
  inoltrano ai service; solo i repository scrivono SQL o chiamano servizi esterni.
- **Un tipo per file**: ogni `struct`, `enum` e `trait` ha il suo file, chiamato come
  il tipo in snake_case (`TsSubmissionStatus` → `ts_submission_status.rs`). Il
  `mod.rs` del package lo riesporta, così si importa `model::ts::TsSubmission`.
  Eccezioni: le entità SeaORM (`Model` e `Relation` devono stare nello stesso modulo)
  e le struct di riga dichiarate dentro una sola funzione di query.
- **Sottopackage per dominio** quando un concetto ha più file (`ts/`, `invoice/`,
  `secret/`). I tipi di servizio interni al package sono `pub(super)`.
- **Test separati**: i test di `foo.rs` stanno in `foo_test.rs` accanto, dichiarati
  con `#[cfg(test)] #[path = "foo_test.rs"] mod tests;`. Restano figli del modulo, quindi
  vedono anche le funzioni private. I doppi riusati in più test stanno in `src/test_support/`.

## Test

```bash
# Frontend (vitest): logica fiscale, validazione (codice fiscale, P.IVA, IBAN…),
# sigillo delle fatture, soglia forfettario, date locali, ricorrenze, tema, notifiche
npm test

# Backend (cargo): calcolo totali fattura, validazione input, Sistema TS (buste SOAP,
# risposte reali catturate, coda, invio con gateway finto), PDF della fattura, email
# (casella, modello, invio con gateway SMTP finto). I test usano un archivio
# credenziali in memoria o in un file temporaneo
cd src-tauri && cargo test

# Test contro servizi reali (rete): ambiente di test Sogei e server SMTP di psypec.it,
# che deve rifiutare una password sbagliata come login e non come errore di rete
cd src-tauri && cargo test live_ -- --ignored
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
- **Backend**: Rust, Tauri 2, SeaORM, SQLite, aes-gcm + hkdf (credenziali cifrate),
  reqwest + rustls, quick-xml, rsa (Sistema TS), lettre + rustls (SMTP), krilla + skrifa (PDF)
- **Build**: Tauri CLI, Vite, vue-tsc
- **Test**: Vitest (frontend), cargo test (backend)

I font (Geist, Geist Mono, Newsreader) sono self-hosted via `@fontsource-variable`:
l'app rende correttamente anche offline, senza dipendere da Google Fonts. Il PDF usa gli
stessi caratteri in formato TTF variabile, da `src-tauri/resources/fonts/`.
