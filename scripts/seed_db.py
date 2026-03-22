#!/usr/bin/env python3
"""
seed_db.py — Seed the PSI Fatture SA database with realistic demo data.

Usage:
    python3 scripts/seed_db.py [--db PATH] [--help]

What it creates:
  - 1 professional config (utente di default: Dott.ssa Maria Demo)
  - 50 pazienti con dati anagrafici italiani
  - 20 prestazioni psicologiche (70–120 €)
  - Appuntamenti per 2025-01-01 → 2026-12-31 (≥6 al giorno, pazienti diversi)
  - Fatture mensili per ogni paziente con appuntamenti completati

Il caricamento sovrascrive tutti i dati esistenti nel database.
"""

from __future__ import annotations

import sqlite3
import random
import sys
import os
from collections import defaultdict
from datetime import date, timedelta
from pathlib import Path
from typing import Optional

# ─── Seed per riproducibilità ────────────────────────────────────────────────
SEED = 42
random.seed(SEED)

# ─── Percorso database (macOS: ~/Library/Application Support/...) ─────────────
def default_db_path() -> Path:
    if sys.platform == "darwin":
        base = Path.home() / "Library" / "Application Support"
    elif sys.platform == "win32":
        base = Path(os.environ.get("APPDATA", Path.home()))
    else:
        base = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))
    return base / "psi-fatture-sa" / "database.db"

# ─── Costanti ─────────────────────────────────────────────────────────────────
TODAY          = date(2026, 3, 22)
START          = date(2025, 1, 1)
END            = date(2026, 12, 31)
MIN_APPTS      = 6
MAX_APPTS      = 9
N_CLIENTS      = 50
N_SERVICES     = 20
PRICE_MIN      = 70
PRICE_MAX      = 120

# ─── Dati italiani ────────────────────────────────────────────────────────────

NOMI_F = [
    "Giulia", "Sofia", "Martina", "Sara", "Laura", "Valentina",
    "Francesca", "Chiara", "Elena", "Serena", "Roberta", "Maria",
    "Anna", "Cristina", "Paola", "Lucia", "Alessia", "Federica",
    "Elisa", "Silvia", "Beatrice", "Camilla", "Alice", "Giorgia",
    "Noemi", "Aurora", "Vittoria", "Irene", "Arianna", "Claudia",
]

NOMI_M = [
    "Marco", "Luca", "Andrea", "Giovanni", "Francesco", "Antonio",
    "Matteo", "Davide", "Simone", "Lorenzo", "Riccardo", "Stefano",
    "Massimo", "Roberto", "Alessandro", "Fabio", "Giorgio", "Emanuele",
    "Nicola", "Paolo", "Filippo", "Alberto", "Enrico", "Claudio",
    "Daniele", "Michele", "Gianluca", "Mauro", "Salvatore", "Vincenzo",
]

COGNOMI = [
    "Rossi", "Ferrari", "Russo", "Bianchi", "Romano", "Gallo",
    "Costa", "Fontana", "Conti", "Esposito", "Ricci", "Bruno",
    "De Luca", "Moretti", "Barbieri", "Lombardi", "Marino", "Greco",
    "Martini", "Colombo", "Giordano", "Coppola", "Mancini", "Rizzo",
    "Caruso", "Leone", "Ferrara", "Monti", "Basile", "Testa",
    "Rinaldi", "Pellegrini", "Cattaneo", "Bianco", "Santoro",
    "Giuliani", "Fiore", "Marini", "Neri", "Valentini", "Vitale",
    "Serra", "Gatti", "Marchetti", "Parisi", "Villa", "Palumbo",
    "Fabbri", "Ruggiero", "Amato",
]

CITTA = [
    ("Roma",    "RM", "00100"), ("Milano",  "MI", "20100"), ("Torino",  "TO", "10100"),
    ("Napoli",  "NA", "80100"), ("Bologna", "BO", "40100"), ("Firenze", "FI", "50100"),
    ("Bari",    "BA", "70100"), ("Padova",  "PD", "35100"), ("Catania", "CT", "95100"),
    ("Verona",  "VR", "37100"), ("Venezia", "VE", "30100"), ("Palermo", "PA", "90100"),
    ("Genova",  "GE", "16100"), ("Brescia", "BS", "25100"), ("Modena",  "MO", "41100"),
    ("Pisa",    "PI", "56100"), ("Reggio Emilia", "RE", "42100"), ("Perugia", "PG", "06100"),
    ("Salerno", "SA", "84100"), ("Ancona",  "AN", "60100"),
]

VIE = [
    "Mazzini", "Garibaldi", "Dante", "Leopardi", "Verdi",
    "Roma", "Milano", "Napoli", "Cavour", "Vittorio Emanuele",
    "Matteotti", "Gramsci", "Marconi", "De Gasperi", "Togliatti",
]

PRESTAZIONI = [
    ("Colloquio clinico individuale",     "Seduta individuale di psicoterapia"),
    ("Seduta di psicoterapia",            "Sessione terapeutica settimanale"),
    ("Consulenza psicologica",            "Primo colloquio di valutazione"),
    ("Terapia cognitivo-comportamentale", "Sessione TCC strutturata"),
    ("Seduta di supporto psicologico",    "Supporto e sostegno psicologico"),
    ("Colloquio di sostegno",             "Sostegno psicologico individuale"),
    ("Psicoterapia analitica",            "Sessione di analisi psicologica"),
    ("Colloquio familiare",               "Seduta con il nucleo familiare"),
    ("Seduta di coppia",                  "Terapia di coppia"),
    ("Valutazione psicodiagnostica",      "Valutazione e diagnosi psicologica"),
    ("Test proiettivi",                   "Somministrazione test proiettivi"),
    ("EMDR",                              "Elaborazione traumi con EMDR"),
    ("Mindfulness individuale",           "Sessione di mindfulness guidata"),
    ("Seduta REBT",                       "Rational Emotive Behaviour Therapy"),
    ("Terapia sistemico-relazionale",     "Approccio sistemico relazionale"),
    ("Colloquio motivazionale",           "Colloquio di orientamento motivazionale"),
    ("Psicoterapia breve",                "Intervento terapeutico focalizzato"),
    ("Seduta narrativa",                  "Psicoterapia ad approccio narrativo"),
    ("Terapia integrata",                 "Seduta con approccio integrativo"),
    ("Supervisione clinica",              "Supervisione e consultazione clinica"),
]

MUNICIPALITY_CODES = ["H501", "F205", "L219", "F839", "A662", "D969", "C351", "B519", "M208", "G273"]
MONTH_CODES        = "ABCDEHLMPRST"
PAYMENT_METHODS    = ["contanti", "bonifico", "pos"]

# ─── Helper functions ─────────────────────────────────────────────────────────

def r2(v: float) -> float:
    """Round to 2 decimal places."""
    return round(v, 2)


def calc_invoice_totals(total_net: float, apply_enpap: bool = True) -> dict:
    """Compute forfettario invoice totals (no VAT, 2% ENPAP, marca da bollo)."""
    enpap        = r2(total_net * 0.02) if apply_enpap else 0.0
    total_gross  = r2(total_net + enpap)
    ritenuta     = 0.0
    bollo        = 2.00 if total_net > 77.47 else 0.0
    total_due    = r2(total_gross + bollo)
    return {
        "total_net":        r2(total_net),
        "total_tax":        0.0,
        "contributo_enpap": enpap,
        "ritenuta_acconto": ritenuta,
        "marca_da_bollo":   int(bollo > 0),
        "total_gross":      total_gross,
        "total_due":        total_due,
    }


def make_fiscal_code(last: str, first: str, gender: str, birth_year: int) -> str:
    """Generate a plausible (not validated) Italian fiscal code."""
    def consonants(s):
        return [c for c in s.upper() if c not in "AEIOU "]
    def vowels(s):
        return [c for c in s.upper() if c in "AEIOU"]

    def surname_part(s):
        c, v = consonants(s), vowels(s)
        chars = (c + v)[:3]
        return "".join(chars).ljust(3, "X")[:3]

    def name_part(s):
        c, v = consonants(s), vowels(s)
        if len(c) >= 4:
            chars = [c[0], c[2], c[3]]
        else:
            chars = (c + v)[:3]
        return "".join(chars).ljust(3, "X")[:3]

    yy  = str(birth_year)[-2:]
    mc  = MONTH_CODES[random.randint(0, 11)]
    day = random.randint(1, 28)
    dd  = f"{day + (40 if gender == 'F' else 0):02d}"
    mun = random.choice(MUNICIPALITY_CODES)
    raw = f"{surname_part(last)}{name_part(first)}{yy}{mc}{dd}{mun}"
    chk = chr(ord("A") + (sum(ord(c) for c in raw) % 26))
    return raw + chk


def last_day_of_month(y: int, m: int) -> date:
    if m == 12:
        return date(y + 1, 1, 1) - timedelta(days=1)
    return date(y, m + 1, 1) - timedelta(days=1)


def invoice_status_for_month(y: int, m: int) -> tuple[str, str | None]:
    """Return (status, paid_date) for a monthly invoice."""
    if y == 2025:
        paid = (last_day_of_month(y, m) + timedelta(days=random.randint(5, 30))).isoformat()
        return "paid", paid
    if y == 2026 and m == 1:
        paid = (last_day_of_month(y, m) + timedelta(days=random.randint(5, 20))).isoformat()
        return "paid", paid
    if y == 2026 and m == 2:
        if random.random() < 0.6:
            paid = (last_day_of_month(y, m) + timedelta(days=random.randint(1, 15))).isoformat()
            return "paid", paid
        return "issued", None
    # March 2026 onwards — draft
    return "draft", None

# ─── Database operations ──────────────────────────────────────────────────────

def clear_all(conn: sqlite3.Connection) -> None:
    """Delete all seeded data in FK-safe order and reset autoincrement sequences."""
    tables = [
        "invoice_lines", "appointments", "invoices",
        "recurrence_groups", "services", "clients", "professional_config",
    ]
    for t in tables:
        conn.execute(f"DELETE FROM {t}")
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name IN "
        "('clients','services','invoices','invoice_lines','appointments','recurrence_groups')"
    )
    conn.commit()
    print("  ✓ Dati esistenti rimossi")


def insert_config(conn: sqlite3.Connection) -> None:
    conn.execute("""
        INSERT INTO professional_config
            (id, title, first_name, last_name, vat_number, fiscal_code,
             tax_regime, albo_number, albo_region, address, city, province,
             zip_code, country, phone, pec_email, iban, coefficient,
             is_psicoanalista, initial_invoice_number)
        VALUES
            (1, 'Dott.ssa', 'Maria', 'Demo',
             '12345678901', 'DMRMRA80A41H501Z',
             'forfettario', '12345', 'Lazio',
             'Via Roma 1', 'Roma', 'RM', '00100', 'Italia',
             '+39 06 12345678', 'maria.demo@pec.it',
             'IT60X0542811101000000123456',
             78.0, 0, 1)
        ON CONFLICT(id) DO UPDATE SET
            title              = excluded.title,
            first_name         = excluded.first_name,
            last_name          = excluded.last_name,
            vat_number         = excluded.vat_number,
            fiscal_code        = excluded.fiscal_code,
            tax_regime         = excluded.tax_regime,
            albo_number        = excluded.albo_number,
            albo_region        = excluded.albo_region,
            address            = excluded.address,
            city               = excluded.city,
            province           = excluded.province,
            zip_code           = excluded.zip_code,
            phone              = excluded.phone,
            pec_email          = excluded.pec_email,
            iban               = excluded.iban,
            coefficient        = excluded.coefficient,
            is_psicoanalista   = excluded.is_psicoanalista,
            initial_invoice_number = excluded.initial_invoice_number,
            updated_at         = datetime('now')
    """)
    print("  ✓ Configurazione professionale inserita")


def insert_clients(conn: sqlite3.Connection) -> dict[int, dict]:
    """Insert N_CLIENTS and return {id: {first_name, last_name}} map."""
    rows = []
    used = set()

    for i in range(N_CLIENTS):
        gender  = "F" if (i % 3 != 0) else "M"
        names   = NOMI_F if gender == "F" else NOMI_M

        first, last = random.choice(names), random.choice(COGNOMI)
        attempts = 0
        while (first, last) in used and attempts < 30:
            first, last = random.choice(names), random.choice(COGNOMI)
            attempts += 1
        used.add((first, last))

        birth_year  = random.randint(1960, 2002)
        birth_month = random.randint(1, 12)
        birth_day   = random.randint(1, 28)
        city, prov, zipcode = random.choice(CITTA)
        fc = make_fiscal_code(last, first, gender, birth_year)

        rows.append((
            "persona_fisica",
            first, last,
            f"{birth_year}-{birth_month:02d}-{birth_day:02d}",
            gender,
            fc,
            None,  # vat_number
            f"Via {random.choice(VIE)} {random.randint(1, 120)}",
            city, prov, zipcode,
            f"{first.lower()}.{last.lower().replace(' ', '')}@email.com",
            f"+39 3{random.randint(10, 99)} {random.randint(1000000, 9999999)}",
            None,  # notes
            0,     # sts_authorization
        ))

    conn.executemany("""
        INSERT INTO clients
            (client_type, first_name, last_name, birth_date, gender, fiscal_code,
             vat_number, address, city, province, zip_code, email, phone,
             notes, sts_authorization)
        VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
    """, rows)

    cur = conn.cursor()
    cur.execute("SELECT id, first_name, last_name FROM clients ORDER BY id")
    result = {row[0]: {"first_name": row[1], "last_name": row[2]} for row in cur.fetchall()}
    print(f"  ✓ {len(result)} pazienti inseriti")
    return result


def insert_services(conn: sqlite3.Connection) -> dict[int, float]:
    """Insert N_SERVICES and return {id: default_price} map."""
    rows = []
    prices = []
    for (name, desc) in PRESTAZIONI[:N_SERVICES]:
        price = random.randint(PRICE_MIN, PRICE_MAX)
        prices.append(price)
        rows.append((name, desc, float(price), 0.0, 1))

    conn.executemany("""
        INSERT INTO services (name, description, default_price, vat_rate, is_active)
        VALUES (?,?,?,?,?)
    """, rows)

    cur = conn.cursor()
    cur.execute("SELECT id, name, default_price FROM services ORDER BY id")
    result = {}
    for row in cur.fetchall():
        result[row[0]] = {"price": row[2], "name": row[1]}
    print(f"  ✓ {len(result)} prestazioni inserite (€{PRICE_MIN}–€{PRICE_MAX})")
    return result


def generate_and_insert_appointments(
    conn: sqlite3.Connection,
    client_ids: list[int],
    service_map: dict[int, dict],
) -> list[dict]:
    """Generate ≥MIN_APPTS per day for every day in [START, END] and insert in bulk."""
    svc_ids = list(service_map.keys())
    rows    = []

    current = START
    while current <= END:
        n           = random.randint(MIN_APPTS, MAX_APPTS)
        day_clients = random.sample(client_ids, min(n, len(client_ids)))
        status      = "completed" if current < TODAY else "scheduled"

        for idx, client_id in enumerate(day_clients):
            hour      = 8 + idx  # slot orario: 08:00, 09:00, ...
            start_t   = f"{hour:02d}:00"
            end_t     = f"{hour:02d}:50"
            svc_id    = random.choice(svc_ids)
            rows.append((client_id, svc_id, current.isoformat(), start_t, end_t, status, ""))

        current += timedelta(days=1)

    conn.executemany("""
        INSERT INTO appointments
            (client_id, service_id, date, start_time, end_time, status, notes)
        VALUES (?,?,?,?,?,?,?)
    """, rows)

    cur = conn.cursor()
    cur.execute("""
        SELECT id, client_id, service_id, date, status
        FROM appointments
        ORDER BY date, start_time
    """)
    result = [
        {"id": r[0], "client_id": r[1], "service_id": r[2], "date": r[3], "status": r[4]}
        for r in cur.fetchall()
    ]
    print(f"  ✓ {len(result)} appuntamenti inseriti ({START} → {END})")
    return result


def insert_invoices(
    conn: sqlite3.Connection,
    appointments: list[dict],
    service_map: dict[int, dict],
) -> None:
    """Generate one invoice per (client, year, month) for completed appointments."""
    # Group completed appointments by (client_id, year, month)
    groups: dict[tuple, list[dict]] = defaultdict(list)
    for appt in appointments:
        if appt["status"] != "completed":
            continue
        d = date.fromisoformat(appt["date"])
        groups[(appt["client_id"], d.year, d.month)].append(appt)

    # Sequential invoice counters per year
    inv_counters: dict[int, int] = defaultdict(int)

    # Sort by (year, month, client_id) for stable sequential numbering
    sorted_keys = sorted(groups.keys(), key=lambda k: (k[1], k[2], k[0]))

    cur          = conn.cursor()
    n_invoices   = 0
    n_lines      = 0

    for (client_id, year, month) in sorted_keys:
        appts_for_inv = groups[(client_id, year, month)]

        # Group lines by service
        by_service: dict[int, list[dict]] = defaultdict(list)
        for appt in appts_for_inv:
            by_service[appt["service_id"]].append(appt)

        lines      = []
        total_net  = 0.0

        for svc_id, svc_appts in sorted(by_service.items()):
            svc      = service_map[svc_id]
            price    = svc["price"]
            qty      = len(svc_appts)
            dates_str = ", ".join(
                date.fromisoformat(a["date"]).strftime("%d/%m")
                for a in sorted(svc_appts, key=lambda x: x["date"])
            )
            desc = (
                f"{svc['name']} del {dates_str}"
                if qty == 1
                else f"{svc['name']} — {qty} sedute ({dates_str})"
            )
            line_net   = r2(qty * price)
            total_net += line_net
            lines.append({
                "service_id":  svc_id,
                "description": desc,
                "quantity":    qty,
                "unit_price":  price,
                "vat_rate":    0.0,
                "line_total":  line_net,
            })

        total_net  = r2(total_net)
        totals     = calc_invoice_totals(total_net, apply_enpap=True)
        issue_date = last_day_of_month(year, month).isoformat()
        status, paid_date = invoice_status_for_month(year, month)
        payment    = random.choice(PAYMENT_METHODS)

        inv_counters[year] += 1
        inv_number = f"{inv_counters[year]:03d}"

        cur.execute("""
            INSERT INTO invoices
                (client_id, invoice_number, year, issue_date, due_date, status,
                 payment_method, notes, apply_enpap,
                 contributo_enpap, ritenuta_acconto, marca_da_bollo,
                 total_net, total_tax, total_gross, total_due, paid_date)
            VALUES (?,?,?,?,NULL,?,?,?,1,?,?,?,?,?,?,?,?)
        """, (
            client_id, inv_number, year, issue_date, status,
            payment, "",
            totals["contributo_enpap"], totals["ritenuta_acconto"],
            totals["marca_da_bollo"],   totals["total_net"],
            totals["total_tax"],        totals["total_gross"],
            totals["total_due"],        paid_date,
        ))
        invoice_id = cur.lastrowid
        n_invoices += 1

        # Insert lines
        for line in lines:
            cur.execute("""
                INSERT INTO invoice_lines
                    (invoice_id, service_id, description, quantity, unit_price, vat_rate, line_total)
                VALUES (?,?,?,?,?,?,?)
            """, (
                invoice_id,       line["service_id"],  line["description"],
                line["quantity"], line["unit_price"],  line["vat_rate"],
                line["line_total"],
            ))
            n_lines += 1

        # Link appointments to this invoice
        appt_ids = [a["id"] for a in appts_for_inv]
        placeholders = ",".join("?" * len(appt_ids))
        conn.execute(
            f"UPDATE appointments SET invoice_id = ? WHERE id IN ({placeholders})",
            [invoice_id] + appt_ids,
        )

    conn.commit()
    print(f"  ✓ {n_invoices} fatture inserite ({n_lines} righe)")

# ─── Entry point ──────────────────────────────────────────────────────────────

def main() -> None:
    args = sys.argv[1:]
    if "--help" in args or "-h" in args:
        print(__doc__)
        sys.exit(0)

    db_path = Path(args[args.index("--db") + 1]) if "--db" in args else default_db_path()

    if not db_path.exists():
        print(f"❌  Database non trovato: {db_path}")
        print("    Avvia l'applicazione almeno una volta per inizializzare il database.")
        sys.exit(1)

    print(f"Database: {db_path}")
    print("=" * 60)

    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA foreign_keys = OFF")
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")

    try:
        print("\n[1/6] Pulizia dati esistenti...")
        clear_all(conn)

        print("\n[2/6] Configurazione professionale...")
        insert_config(conn)

        print("\n[3/6] Inserimento pazienti...")
        clients = insert_clients(conn)

        print("\n[4/6] Inserimento prestazioni...")
        services = insert_services(conn)

        print("\n[5/6] Generazione appuntamenti...")
        appointments = generate_and_insert_appointments(conn, list(clients.keys()), services)

        print("\n[6/6] Generazione fatture mensili...")
        insert_invoices(conn, appointments, services)

        # Riepilogo finale
        cur = conn.cursor()
        cur.execute("SELECT COUNT(*) FROM appointments WHERE status='completed'")
        n_completed = cur.fetchone()[0]
        cur.execute("SELECT COUNT(*) FROM appointments WHERE status='scheduled'")
        n_scheduled = cur.fetchone()[0]
        cur.execute("SELECT COUNT(*) FROM invoices WHERE status='paid'")
        n_paid = cur.fetchone()[0]
        cur.execute("SELECT COUNT(*) FROM invoices WHERE status='issued'")
        n_issued = cur.fetchone()[0]
        cur.execute("SELECT COUNT(*) FROM invoices WHERE status='draft'")
        n_draft = cur.fetchone()[0]
        cur.execute("SELECT COALESCE(SUM(total_due),0) FROM invoices WHERE status='paid'")
        total_paid = cur.fetchone()[0]

        print("\n" + "=" * 60)
        print("✅  Seed completato con successo!")
        print("=" * 60)
        print(f"  Pazienti:              {len(clients)}")
        print(f"  Prestazioni:           {len(services)}")
        print(f"  Appuntamenti totali:   {len(appointments)}")
        print(f"    └─ Completati:       {n_completed}")
        print(f"    └─ Pianificati:      {n_scheduled}")
        print(f"  Fatture pagate:        {n_paid}")
        print(f"  Fatture emesse:        {n_issued}")
        print(f"  Fatture in bozza:      {n_draft}")
        print(f"  Incassato (2025–2026): €{total_paid:,.2f}")
        print("=" * 60)

    except Exception as e:
        conn.rollback()
        print(f"\n❌  Errore durante il seed: {e}")
        raise
    finally:
        conn.execute("PRAGMA foreign_keys = ON")
        conn.close()


if __name__ == "__main__":
    main()
