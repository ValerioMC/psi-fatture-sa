# PSI Fatture

Gestionale fatture per psicologi — applicazione desktop per macOS e Windows costruita con Tauri 2, Vue 3 e SQLite.

## Prerequisiti

| Strumento | Versione minima | Installazione |
|-----------|----------------|---------------|
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org/) oppure `brew install node` |
| **Rust** | 1.99 (fissato in `rust-toolchain.toml`, `rustup` lo scarica da solo) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
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

Il sistema visivo si chiama "Indaco e lavanda": ogni colore viene dall'icona dell'app.
L'indaco della piastra (`#4540b8`, `#2c238c` in pressione) è l'unico accento per selezione,
azione primaria e focus; il bianco-lavanda del Ψ dà le superfici chiare; testo, bordi e
ombre portano una traccia dello stesso indaco. Il tema scuro parte dall'indaco più profondo
e usa la lavanda come accento. Tutti i testi stanno sopra 4,5:1 di contrasto.

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
  `/data`, `/appearance`, con una navigazione laterale che segnala con un punto ciò che manca
  (casella da configurare, credenziali TS). Il modulo del profilo è condiviso da Profilo e
  Fatturazione (`useSettingsProfile.ts`) e si salva da un'unica barra.
- **Sigillo della fattura**: `InvoiceSeal.vue` disegna lo stato di una fattura
  (bozza, emessa con l'arco verso la scadenza, pagata, scaduta, annullata); la logica
  sta in `src/features/invoice/lib/invoiceSeal.ts`.
- **Layout adattivo**: le colonne affiancate (riepilogo, anteprima, giorno in agenda)
  dipendono dalla larghezza dell'area di contenuto, non della finestra: container query
  su `@container/pane` (`AppLayout.vue`), `@container/settings` (Impostazioni) e
  `@container/section` (`FormSection.vue`). La colonna laterale scende sotto prima che i
  campi principali si restringano; le righe prestazione della fattura vanno su due righe
  quando la card è stretta.
- **Tema**: chiaro, scuro o come il sistema, da Impostazioni → Aspetto. La scelta è salvata in
  `localStorage` (chiave `psi-fatture.theme`).
- **Scorciatoie**: ⌘K (Ctrl K su Windows) apre la ricerca di pazienti, azioni e sezioni.
- **Finestra macOS**: `titleBarStyle: "Overlay"` in `tauri.conf.json`; i semafori stanno
  sopra la barra laterale e intestazioni e barra laterale trascinano la finestra
  (permesso `core:window:allow-start-dragging`).

## Calcolo della fattura

Solo il backend (`src-tauri/src/app/model/tax/tax_calculator.rs`) calcola i totali: il
riepilogo del modulo li chiede mentre scrivi al comando `preview_invoice_totals`
(`src/features/invoice/composables/useInvoiceTotals.ts`), lo stesso codice che poi salva la fattura.

- **Base ENPAP 2%**: nel forfettario la marca da bollo addebitata al paziente è compenso
  (Interpello AdE) e concorre alla base; nell'ordinario la base è il solo imponibile.
  Impostazioni → Fatturazione → *Escludi la marca da bollo dal calcolo ENPAP* toglie il
  bollo dalla base anche nel forfettario.
- **Importo a mano**: ogni riga ha un campo *Importo* che vale quantità × prezzo finché non
  lo si scrive a mano; da lì l'importo scritto (`invoice_lines.amount_override`) sostituisce
  il prodotto in tutti i calcoli, e in fattura la riga non mostra il prezzo unitario.
  *Ricalcola* torna al prodotto.
- **Importo minimo**: una fattura con totale zero non si salva. Nella fatturazione
  mensile le sedute senza prestazione o con prezzo zero sono segnalate e non si
  possono fatturare finché non si assegna una prestazione in agenda.
- **Numerazione e date**: i numeri seguono l'ordine delle date nell'anno. Una fattura
  nuova non può avere una data precedente all'ultima emessa, e una rinumerata deve
  restare tra le date dei numeri vicini. La fatturazione mensile propone come data
  l'ultimo giorno del mese, modificabile.
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
- **Avanzamento**: ogni giro, manuale ("Invia ora", "Trasmetti al Sistema TS") o del
  worker, emette l'evento Tauri `ts-dispatch-progress` (`TsDispatchProgress`: totale,
  risposte ricevute, esiti, `finished` sull'ultimo) dopo ogni risposta. Un invio di più
  fatture avviato dall'utente apre `StsSendDialog` (anello che si riempie, "N di M"
  fatture inviate, esiti); chiuderla non ferma l'invio. A fine invio resta aperta
  sull'esito (anello verde, ambra o rosso, cosa fare) finché l'utente preme OK; se era
  stata chiusa prima, l'esito arriva in un avviso. I giri del worker non aprono nulla: a
  fine giro la pagina ricarica l'elenco, così non mostra mai uno stato già superato.
- **Annullamento**: è una trasmissione a sé, con il proprio protocollo. Accettata, porta
  l'invio originale ad `annullata` e nell'elenco compare come *Cancellata dal TS* sotto
  il filtro *Annullate*, non tra le *Accettate*. "Annulla" si offre solo quando per
  quella fattura non c'è già un'altra trasmissione in coda. Una fattura annullata si
  può trasmettere di nuovo: conta come già sul Sistema TS solo un invio o una
  sostituzione accettati (`invoice_holds_ts_data`).
- **Operazioni Sogei**: il web service prevede Inserimento, Variazione (la nostra
  *sostituzione*: stesso identificativo, dati nuovi), Cancellazione e Rimborso
  (documento che rettifica gli importi di uno già inviato, es. nota di credito). Il
  Rimborso non è implementato: l'app non emette note di credito. Numero e data di
  emissione formano l'identificativo e la Variazione non li cambia: per correggerli
  si annulla e si ritrasmette.
- **Numero del documento**: al Sistema TS va il numero della fattura così com'è
  (`numDocumento`) con la data di emissione; il `/anno` che l'app mostra accanto al
  numero è solo un'etichetta locale.
- **Lista e verifica**: report mensile (`ReportMensile730`, per data di invio o di
  pagamento) e interrogazione puntuale del singolo documento.
- **Scadenze**: invio entro il 31 gennaio dell'anno dopo il pagamento, correzioni
  gratuite nei 5 giorni successivi (spostati al lunedì se cadono nel weekend).
- **Fatture già registrate**: non si elimina una fattura i cui dati sono sul Sistema TS
  di produzione; prima va annullato l'invio. Numero, data e stato *pagata* restano
  quelli inviati (anche dal cambio di stato in blocco). Importi e paziente si possono
  correggere: ogni invio salva un'impronta SHA-256 dei dati spediti
  (`ts_submissions.document_fingerprint`) e, se la fattura non coincide più, il
  pannello chiede di inviare i dati aggiornati (sostituzione).

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

## Backup

Tutto l'archivio (pazienti, fatture, agenda, impostazioni) è un solo file SQLite,
`database.db` nella cartella dati dell'app (`~/Library/Application Support/psi-fatture-sa/`
su macOS). Ogni copia è coerente (`VACUUM INTO`, include le modifiche ancora nel WAL).
Password del Sistema TS e della casella email restano fuori: sono cifrate per questo
computer.

- **Backup automatici**: `RotatingBackups` (`src-tauri/src/app/service/backup/rotating_backup_service.rs`)
  scrive le copie in `backups/` accanto a `database.db`, con nomi
  `psi-fatture-AAAAMMGG-HHMMSS[-N]-(auto|manual).db`. Il worker `scheduler/backup_worker.rs`
  controlla 5 secondi dopo l'avvio e poi ogni ora: se nessuna copia porta la data di oggi ne
  scrive una. *Backup ora* aggiunge una copia manuale. Si tengono le **10** più recenti,
  automatiche e manuali insieme; le altre vengono eliminate. I file con altri nomi nella
  cartella vengono ignorati. Un errore resta visibile nella pagina finché una copia riesce.
- **Copia altrove**: Impostazioni → *Dati e backup* → *Salva un backup…* scrive una copia
  dove scegli, per esempio su un disco esterno.
- La pagina *Dati e backup* mostra il percorso dell'archivio, la cartella dei backup (*Apri*
  la mostra nel Finder o in Esplora file) e l'elenco delle copie.

Per ripristinare: chiudi l'app, sostituisci `database.db` con la copia (rinominandola)
e cancella `database.db-wal` e `database.db-shm` se presenti, poi riapri l'app.

## Aggiornamenti

All'avvio l'app chiede a `https://api.github.com/repos/ValerioMC/psi-fatture-sa/releases/latest`
l'ultima release pubblicata e la confronta con la propria versione (quella del tag, vedi
*Build per Windows*). Se è più recente apre un popup con un solo pulsante, *Scarica*.
Senza rete il controllo fallisce in silenzio e riprova al prossimo avvio.
Impostazioni → *Versione* mostra la versione installata e permette di controllare a mano.

- **Mac**: `update_install_service` scarica il `.dmg` per l'architettura in uso
  (solo da `github.com/ValerioMC/psi-fatture-sa/releases/download/`), lo monta con
  `hdiutil`, copia `PSI Fatture.app` con `ditto` accanto a quella in uso, scambia i nomi
  (la vecchia resta da parte finché la nuova è al suo posto, e torna indietro se lo scambio
  fallisce) e riavvia l'app. Il file è scaricato dall'app e non dal browser, quindi non
  riceve l'attributo `com.apple.quarantine`: la nuova versione si apre senza
  `xattr -dr com.apple.quarantine`. Se l'installazione fallisce (per esempio l'app gira
  dall'immagine disco o la cartella non è scrivibile) il pulsante apre il download nel
  browser.
- **Windows** e altri sistemi: il pulsante apre nel browser l'installer per il sistema in
  uso (`-setup.exe` su Windows, la pagina della release altrimenti).
- **Prima installazione su Mac**: l'app non è firmata con un Developer ID, quindi il `.dmg`
  scaricato dal sito è in quarantena e la prima apertura richiede ancora una volta
  `xattr -dr com.apple.quarantine "/Applications/PSI Fatture.app"`. Nessun codice dell'app
  può farlo, perché è la quarantena a impedirne l'avvio: solo la firma con notarizzazione
  (vedi *Build firmata*) la elimina.

Test reale dell'installazione (rete, solo Mac, installa in una cartella temporanea):
`cd src-tauri && cargo test live_the_published -- --ignored`.

In `npm run tauri dev` il controllo all'avvio è disattivato: la versione committata è un
segnaposto (`0.1.0`) e risulterebbe sempre superata. Una build locale (`make build`) porta
la stessa versione segnaposto, quindi all'avvio propone sempre l'ultima release.

## Condizioni d'uso e licenza

Il codice è pubblicato solo per essere consultato, con licenza **PolyForm Strict 1.0.0**
(`LICENSE`): nessuno può copiarlo, modificarlo, ridistribuirlo o rivenderlo, e la licenza
copre solo usi non commerciali. L'uso dell'app per la propria attività professionale è
concesso agli utenti dalle condizioni d'uso, che stanno in `src/legal/terms.ts` (clausola 1)
e si accettano prima di qualsiasi altra schermata.

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

Oltre alle schermate intere lo script ritaglia parti dell'app, ciascuna chiara e `-dark`:
`sts-coda`, `sts-fattura`, `email-invio` ed `email-casella` per la sezione Sistema TS ed
email; `fattura-riepilogo`, `agenda-giorno`, `mensile-totale`, `soglia`, `stima` e
`paziente-scheda` per la sezione Funzioni; `onboarding-fisco` per Come si inizia.
`ONLY=shots` o `ONLY=excerpts` limita il giro a uno dei due gruppi, `NAMES=soglia,stima`
ai ritagli indicati. I ritagli che leggono la data di oggi (agenda, mensile, soglia, stima)
cambiano a ogni cattura, perché i dati demo partono dalla data corrente.
Installa `playwright` e `sharp` nello stesso comando: un `npm install --no-save` successivo
rimuove i pacchetti installati senza salvarli.

## Struttura del progetto

```
psi-fatture-sa/
├── src/                      # Frontend Vue 3 + TypeScript
│   ├── features/<feature>/   # Una cartella per funzione, con gli stessi nomi dei domini del backend
│   │   ├── api/              # Chiamate al backend del dominio (unico punto che usa `invoke`)
│   │   ├── model/            # Tipi che rispecchiano `model/` del backend
│   │   ├── store/            # Stato condiviso (Pinia)
│   │   ├── composables/      # Stato e azioni delle viste
│   │   ├── lib/              # Logica pura, con il suo `*.test.ts` accanto
│   │   ├── components/       # Componenti della funzione
│   │   └── views/            # Pagine raggiunte dal router
│   ├── components/
│   │   ├── ui/               # Componenti base (bottoni, campi, dialoghi, sigillo, credenziali…)
│   │   └── layout/           # Barra laterale, layout, palette ⌘K
│   ├── common/               # Ciò che nessuna funzione possiede: formati, etichette, validazione,
│   │                         # tema, notifiche (lib/, store/, composables/)
│   ├── router/               # Rotte, guardie di onboarding e condizioni d'uso
│   └── style.css             # Token del design system, campi, movimenti
├── src-tauri/                # Backend Rust + Tauri
│   ├── src/
│   │   ├── lib.rs            # Avvio: stato condiviso, orologio, worker, registrazione comandi
│   │   ├── app/
│   │   │   ├── app_state.rs  # Stato condiviso dai comandi (DB, orologio, credenziali, gateway)
│   │   │   ├── controller/   # Comandi Tauri, uno per dominio; ts/ per il Sistema TS
│   │   │   ├── service/      # Logica di business, un service per entità
│   │   │   │   ├── invoice/      # Fatture: invoice_service più regole e tipi privati
│   │   │   │   ├── invoice_change_service.rs # Modifiche che il Sistema TS può rifiutare
│   │   │   │   ├── backup/       # Copia su richiesta (backup_service) e rotazione (RotatingBackups)
│   │   │   │   ├── update/       # Controllo della release (update_service) e installazione su Mac
│   │   │   │   ├── ts/           # Sistema TS: coda, invio, credenziali, impostazioni, vincoli sulle fatture
│   │   │   │   ├── email/        # Invio fatture e modello dell'email
│   │   │   │   ├── pdf/          # PDF della fattura; layout/ disegna la pagina, un modulo per sezione
│   │   │   │   └── support/      # Funzioni pure condivise: validazione, formati italiani
│   │   │   ├── repository/   # Accesso dati (SeaORM, SQL) e servizi esterni, una cartella o un file per dominio
│   │   │   │   ├── invoice/      # Fatture e proiezione delle righe lette
│   │   │   │   ├── secret/       # Archivio cifrato delle credenziali
│   │   │   │   ├── email/        # Casella, modello, registro invii e gateway SMTP (lettre)
│   │   │   │   ├── update/       # Release su GitHub e installazione dell'immagine disco
│   │   │   │   └── ts/           # Coda STS; sistema_ts/ è il client SOAP
│   │   │   ├── scheduler/    # Worker in background (invio coda STS, backup giornaliero)
│   │   │   ├── entity/       # Entità database (una tabella per file)
│   │   │   ├── model/        # DTO e tipi condivisi, un sottopackage per dominio; tax/ calcola i totali
│   │   │   ├── common/       # AppError, Clock e il formato con cui il database salva gli istanti
│   │   │   └── db/           # Connessione e percorsi dei file dati
│   │   ├── migration/        # Migrazioni schema SQLite
│   │   └── test_support/     # Test double e fixture condivisi (solo in `cargo test`)
│   ├── tests/architecture/   # Direzione degli strati e dipendenze fra domini
│   ├── tauri.conf.json       # Configurazione Tauri
│   └── Cargo.toml            # Dipendenze Rust
├── CLAUDE.md                 # Regole di progetto e di qualità del codice
├── LICENSE                   # Licenza PolyForm Strict 1.0.0
└── package.json              # Dipendenze frontend
```

### Convenzioni del backend

- **Strati**: `controller → service → repository → model/entity`. I controller
  inoltrano ai service; solo i repository scrivono SQL o chiamano servizi esterni. Il test
  `tests/architecture` lo verifica: un repository non nomina service o controller, un
  controller non nomina repository o entità, `model` nomina solo `model` e `common`, `common`
  non nomina nulla dell'app. `app_state` e `scheduler` sono il cablaggio.
- **Domini**: ogni strato è diviso per gli stessi domini. Un dominio con una sola entità
  che porta il suo nome resta un file (`client_service.rs`), altrimenti è una cartella
  (`service/ts/`). Un dominio usa solo quelli della tabella sotto; il test fallisce su una
  dipendenza non elencata, su una voce che il codice non usa e su ogni ciclo. Un nuovo arco
  è una decisione di progetto: si cambiano insieme questa tabella e `ALLOWED` in
  `tests/architecture/domain_edges.rs`, oppure si cambia il codice.

  | Dominio | Può usare |
  |---|---|
  | `appointment` | `support` |
  | `backup` | |
  | `client` | `support` |
  | `config` | `support` |
  | `dashboard` | `invoice` |
  | `email` | `client`, `config`, `invoice`, `pdf`, `secret`, `support` |
  | `invoice` | `appointment`, `config`, `service`, `support`, `tax` |
  | `invoice_change` | `invoice`, `ts` |
  | `pdf` | `client`, `config`, `invoice`, `support` |
  | `secret` | |
  | `service` | |
  | `support` | |
  | `tax` | `config` |
  | `terms` | |
  | `ts` | `client`, `config`, `invoice`, `secret`, `support` |
  | `update` | |

  Il Sistema TS invia le fatture, quindi `ts` dipende da `invoice` e mai il contrario: ciò che
  una trasmissione accettata vieta (cambiare numero, data o stato pagato, eliminare la
  fattura) sta in `service/ts/ts_invoice_guard.rs`. `invoice_change_service` chiede prima a
  `invoice_service::check_update` di validare i dati (così un input non valido risponde
  `Invalid` anche su una fattura trasmessa), poi controlla le regole del Sistema TS e infine
  applica la modifica con il `CheckedUpdate` ottenuto, che solo `check_update` costruisce.
- **Un tipo per file**: ogni `struct`, `enum` e `trait` ha il suo file, chiamato come
  il tipo in snake_case (`TsSubmissionStatus` → `ts_submission_status.rs`). Il
  `mod.rs` del package lo riesporta, così si importa `model::ts::TsSubmission`.
  Eccezioni: le entità SeaORM (`Model` e `Relation` devono stare nello stesso modulo)
  e le struct di riga dichiarate dentro una sola funzione di query.
- **Un service per entità**: in `service/invoice/` solo `invoice_service` è pubblico; le
  regole (`invoice_guard`, `invoice_validation`) e i tipi di appoggio (`ClientSessions`,
  `InvoiceContent`) sono moduli privati del package; ne esce solo `CheckedUpdate`, la
  modifica già validata che `invoice_change_service` passa a `update`. Allo stesso modo `backup/` espone
  `backup_service` e `RotatingBackups`, `update/` solo `update_service` (che riesporta
  l'installazione) e in `ts/` `ts_document_service` e `ts_invoice_guard` sono privati
  (`ts_submission_service` riesporta le regole del secondo). Le regole di qualità del
  codice sono in `CLAUDE.md`.
- **Repository**: ogni funzione riceve `&impl ConnectionTrait`, così gira dentro la
  transazione del chiamante. `require(db, id)` è l'unica ricerca che esige che il record
  esista e restituisce `NotFound` con il messaggio in italiano. Un repository non legge
  l'orologio: riceve l'istante come argomento.
- **Orologio**: solo `SystemClock` (`app/common/system_clock.rs`) legge l'ora di sistema;
  `lib.rs` lo mette in `AppState` come `Arc<dyn Clock>` e i service lo ricevono dal
  controller o dal worker. Nei test `FixedClock` lo ferma su un istante.
  Ogni insert valorizza le sue colonne temporali (`created_at`, `updated_at`, `sent_at`,
  `accepted_at`, `next_attempt_at`) con l'orologio: i `DEFAULT (datetime('now'))` delle
  migrazioni restano solo per compatibilità e non vanno mai usati.
- **Errori tipizzati**: controller, service e repository restituiscono
  `Result<_, AppError>` (`app/common/app_error.rs`): `Invalid` (dato da correggere),
  `NotFound`, `Conflict` (contrasta con lo stato salvato: numero già usato, fattura
  trasmessa), `Database`, `External` (Sistema TS, SMTP, file). Al frontend arriva solo
  il messaggio in italiano, come stringa. La conversione da un errore di un gateway sta
  accanto a quell'errore (`SecretStoreError`, `TsGatewayError`).
- **Test separati**: i test di `foo.rs` stanno in `foo_test.rs` accanto, dichiarati
  con `#[cfg(test)] #[path = "foo_test.rs"] mod tests;`. Restano figli del modulo, quindi
  vedono anche le funzioni private. I doppi riusati in più test stanno in `src/test_support/`.
  Le suite che leggono il crate da fuori stanno in `src-tauri/tests/`, dichiarate come
  `[[test]]` in `Cargo.toml`.

### Convenzioni del frontend

- **Funzioni**: `src/features/<feature>/` con le cartelle `api`, `model`, `store`,
  `composables`, `lib`, `components`, `views` che servono. Le funzioni hanno i nomi dei
  domini del backend: `appointment` (agenda), `client`, `dashboard`, `email`, `invoice`
  (anche PDF e stampa), `service`, `settings` (profilo, backup, onboarding), `sts`,
  `terms`, `update`. Ciò che nessuna funzione possiede sta in `src/common/` e in
  `src/components/{ui,layout}`.
- **API**: ogni chiamata al backend passa da un modulo `features/<feature>/api/`, uno per
  dominio del backend (`invoiceApi.ts`, `pdfApi.ts`, `tsApi.ts`…); viste e componenti non
  usano mai `invoke` direttamente.
- **Test**: i test di `foo.ts` stanno in `foo.test.ts` accanto, un file di test per modulo.

## Test

```bash
# Frontend (vitest): logica fiscale, validazione (codice fiscale, P.IVA, IBAN…),
# sigillo delle fatture, soglia forfettario, date locali, ricorrenze, tema, notifiche,
# filtri e riepiloghi della lista fatture, form di fattura e paziente, agenda e dashboard,
# testo della fattura stampata, stato e scadenze del Sistema TS, chiave di ricerca
npm test

# Backend (cargo): calcolo totali fattura, validazione input, Sistema TS (buste SOAP,
# risposte reali catturate, coda, invio con gateway finto), PDF della fattura, email
# (casella, modello, invio con gateway SMTP finto), rotazione dei backup, confronto
# delle versioni e scelta del download, più la suite `architecture` (strati e domini).
# I test usano un orologio fermo e un archivio credenziali in memoria o in un file temporaneo
cd src-tauri && cargo test

# Test contro servizi reali (rete): ambiente di test Sogei e server SMTP di psypec.it,
# che deve rifiutare una password sbagliata come login e non come errore di rete
cd src-tauri && cargo test live_ -- --ignored
```

## Qualità del codice Rust

Il progetto integra **Rustfmt** (formatter) e **Clippy** (linter), entrambi strumenti ufficiali del toolchain Rust. Non richiedono installazione separata: sono già inclusi se hai Rust via `rustup`.

Il workflow `.github/workflows/lint.yml` esegue entrambi i check automaticamente ad ogni push e pull request su `main`. Un warning Clippy non risolto blocca il check CI.

La versione di Rust è fissata in `rust-toolchain.toml` (radice del repo) e letta sia da `rustup` in locale sia dai workflow `lint.yml` e `release.yml`: CI e macchina locale usano lo stesso compilatore, quindi un nuovo lint di Clippy non può rompere la CI a sorpresa. Per aggiornarla, cambia `channel`, esegui `make fmt lint test-backend`, correggi i nuovi warning e committa tutto insieme. Prima di ogni push basta `make fmt lint`, gli stessi comandi della CI (Clippy ed ESLint).

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

### Clippy — linting

```bash
cd src-tauri

# Analisi con tutti i warning (stessa modalità del CI)
cargo clippy --all-targets -- -D warnings

# Analisi permissiva (solo output, non blocca)
cargo clippy
```

### VS Code

Il file `.vscode/settings.json` già presente nel repo configura:

- **Format-on-save** automatico per i file `.rs`
- **Clippy come checker** in tempo reale al posto del semplice `cargo check` — i warning appaiono direttamente nell'editor mentre scrivi

## Qualità del codice frontend

**ESLint** controlla `src/` e `scripts/` con le regole raccomandate di JavaScript,
`typescript-eslint` e `eslint-plugin-vue` (`eslint.config.js`). Lo stesso workflow
`.github/workflows/lint.yml` lo esegue in un job a parte, e un errore blocca il check CI.

```bash
npm run lint          # ESLint su tutto il frontend
make lint-frontend    # lo stesso, da make
make lint             # Clippy ed ESLint insieme, come la CI
```

Le eccezioni alle regole raccomandate sono in fondo a `eslint.config.js`, ognuna con il
suo motivo. Le regole di impaginazione dei template sono spente, perché i template usano
uno stile compatto su una riga. `vue/no-mutating-props` controlla solo la sostituzione di
una prop: `ProfileSections.vue` modifica il form reattivo che il genitore le passa.

## Stack tecnologico

- **Frontend**: Vue 3, TypeScript, Tailwind CSS 4, Pinia, Vite
- **Backend**: Rust, Tauri 2, SeaORM, SQLite, aes-gcm + hkdf (credenziali cifrate),
  reqwest + rustls, quick-xml, rsa (Sistema TS), lettre + rustls (SMTP), krilla + skrifa (PDF)
- **Build**: Tauri CLI, Vite, vue-tsc
- **Test**: Vitest (frontend), cargo test (backend)

I font (Geist, Geist Mono, Newsreader) sono self-hosted via `@fontsource-variable`:
l'app rende correttamente anche offline, senza dipendere da Google Fonts. Il PDF usa gli
stessi caratteri in formato TTF variabile, da `src-tauri/resources/fonts/`.
