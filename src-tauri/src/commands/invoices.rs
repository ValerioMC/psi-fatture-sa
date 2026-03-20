/// Tauri commands for invoice CRUD operations with Italian tax logic.
use crate::db;
use crate::domain::models::{
    CreateInvoiceInput, Invoice, InvoiceFilters, InvoiceLine, InvoiceLineInput, InvoiceStatus,
    PaymentMethod, UpdateInvoiceInput,
};
use crate::domain::tax::{calculate_invoice_totals, InvoiceLineData};
use rusqlite::params;

#[tauri::command]
pub fn list_invoices(filters: InvoiceFilters) -> Result<Vec<Invoice>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let mut conditions = vec!["1=1".to_string()];
    if let Some(y) = filters.year {
        conditions.push(format!("i.year = {y}"));
    }
    if let Some(s) = &filters.status {
        if !s.is_empty() {
            conditions.push(format!("i.status = '{s}'"));
        }
    }
    if let Some(cid) = filters.client_id {
        conditions.push(format!("i.client_id = {cid}"));
    }
    if let Some(q) = &filters.search {
        if !q.trim().is_empty() {
            let q = q.trim().replace('\'', "''");
            conditions.push(format!(
                "(lower(c.first_name || ' ' || c.last_name) LIKE lower('%{q}%') OR i.invoice_number LIKE '%{q}%')"
            ));
        }
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT i.*, c.first_name || ' ' || c.last_name as client_name
         FROM invoices i
         JOIN clients c ON i.client_id = c.id
         WHERE {where_clause}
         ORDER BY i.issue_date DESC, i.invoice_number DESC"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let ids: Vec<i64> = stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut invoices = Vec::new();
    for id in ids {
        invoices.push(get_invoice(id)?);
    }
    Ok(invoices)
}

#[tauri::command]
pub fn get_invoice(id: i64) -> Result<Invoice, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    let invoice = conn
        .query_row(
            "SELECT i.*, c.first_name || ' ' || c.last_name as client_name
             FROM invoices i
             JOIN clients c ON i.client_id = c.id
             WHERE i.id = ?1",
            params![id],
            row_to_invoice_header,
        )
        .map_err(|e| e.to_string())?;

    let lines = load_lines(&conn, id)?;
    Ok(Invoice { lines, ..invoice })
}

#[tauri::command]
pub fn create_invoice(input: CreateInvoiceInput) -> Result<Invoice, String> {
    let mut conn = db::open().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let year = extract_year(&input.issue_date)?;
    let invoice_number = next_invoice_number(&tx, year)?;
    let totals = compute_totals(&tx, &input.lines, &input.apply_enpap)?;

    tx.execute(
        "INSERT INTO invoices
            (client_id, invoice_number, year, issue_date, due_date, status,
             payment_method, notes, apply_enpap, contributo_enpap,
             ritenuta_acconto, marca_da_bollo, total_net, total_tax,
             total_gross, total_due)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        params![
            input.client_id, invoice_number, year, input.issue_date, input.due_date,
            input.status.as_str(), input.payment_method.as_str(), input.notes,
            input.apply_enpap as i64, totals.contributo_enpap, totals.ritenuta_acconto,
            totals.marca_da_bollo > 0.0, totals.total_net, totals.total_tax,
            totals.total_gross, totals.total_due
        ],
    )
    .map_err(|e| e.to_string())?;

    let invoice_id = tx.last_insert_rowid();
    insert_lines(&tx, invoice_id, &input.lines)?;
    tx.commit().map_err(|e| e.to_string())?;

    get_invoice(invoice_id)
}

#[tauri::command]
pub fn update_invoice(input: UpdateInvoiceInput) -> Result<Invoice, String> {
    let mut conn = db::open().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let totals = compute_totals(&tx, &input.lines, &input.apply_enpap)?;

    tx.execute(
        "UPDATE invoices SET
            client_id=?1, issue_date=?2, due_date=?3, status=?4,
            payment_method=?5, notes=?6, apply_enpap=?7,
            contributo_enpap=?8, ritenuta_acconto=?9, marca_da_bollo=?10,
            total_net=?11, total_tax=?12, total_gross=?13, total_due=?14,
            paid_date=?15, updated_at=datetime('now')
         WHERE id=?16",
        params![
            input.client_id, input.issue_date, input.due_date,
            input.status.as_str(), input.payment_method.as_str(), input.notes,
            input.apply_enpap as i64, totals.contributo_enpap, totals.ritenuta_acconto,
            totals.marca_da_bollo > 0.0, totals.total_net, totals.total_tax,
            totals.total_gross, totals.total_due, input.paid_date, input.id
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM invoice_lines WHERE invoice_id = ?1",
        params![input.id],
    )
    .map_err(|e| e.to_string())?;
    insert_lines(&tx, input.id, &input.lines)?;

    tx.commit().map_err(|e| e.to_string())?;
    get_invoice(input.id)
}

#[tauri::command]
pub fn delete_invoice(id: i64) -> Result<(), String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM invoices WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_next_invoice_number(year: i64) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    next_invoice_number(&conn, year)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn extract_year(date_str: &str) -> Result<i64, String> {
    date_str
        .split('-')
        .next()
        .and_then(|y| y.parse::<i64>().ok())
        .ok_or_else(|| format!("Invalid date format: {date_str}"))
}

fn next_invoice_number(conn: &rusqlite::Connection, year: i64) -> Result<String, String> {
    let max_num: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(CAST(invoice_number AS INTEGER)), 0) FROM invoices WHERE year = ?1",
            params![year],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Ok(format!("{:03}", max_num + 1))
}

fn compute_totals(
    conn: &rusqlite::Connection,
    lines: &[InvoiceLineInput],
    apply_enpap: &bool,
) -> Result<crate::domain::tax::InvoiceTotals, String> {
    let regime: String = conn
        .query_row(
            "SELECT tax_regime FROM professional_config WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "forfettario".to_string());

    let enpap_rate = if *apply_enpap { 2.0 } else { 0.0 };
    let ritenuta_rate = if regime == "ordinario" { 20.0 } else { 0.0 };

    let line_data: Vec<InvoiceLineData> = lines
        .iter()
        .map(|l| InvoiceLineData {
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
        })
        .collect();

    Ok(calculate_invoice_totals(&line_data, &regime, enpap_rate, ritenuta_rate))
}

fn insert_lines(
    conn: &rusqlite::Connection,
    invoice_id: i64,
    lines: &[InvoiceLineInput],
) -> Result<(), String> {
    for line in lines {
        let line_net = (line.quantity as f64 * line.unit_price * 100.0).round() / 100.0;
        let line_vat = (line_net * line.vat_rate / 100.0 * 100.0).round() / 100.0;
        let line_total = line_net + line_vat;

        conn.execute(
            "INSERT INTO invoice_lines
                (invoice_id, service_id, description, quantity, unit_price, vat_rate, line_total)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                invoice_id, line.service_id, line.description,
                line.quantity, line.unit_price, line.vat_rate, line_total
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn load_lines(conn: &rusqlite::Connection, invoice_id: i64) -> Result<Vec<InvoiceLine>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM invoice_lines WHERE invoice_id = ?1 ORDER BY id")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![invoice_id], |row| {
            Ok(InvoiceLine {
                id: Some(row.get(0)?),
                invoice_id: Some(row.get(1)?),
                service_id: row.get(2)?,
                description: row.get(3)?,
                quantity: row.get(4)?,
                unit_price: row.get(5)?,
                vat_rate: row.get(6)?,
                line_total: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

fn row_to_invoice_header(row: &rusqlite::Row) -> rusqlite::Result<Invoice> {
    Ok(Invoice {
        id: row.get(0)?,
        client_id: row.get(1)?,
        invoice_number: row.get(2)?,
        year: row.get(3)?,
        issue_date: row.get(4)?,
        due_date: row.get(5)?,
        status: InvoiceStatus::from(row.get::<_, String>(6)?),
        payment_method: PaymentMethod::from(row.get::<_, String>(7)?),
        notes: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
        apply_enpap: row.get::<_, i64>(9)? != 0,
        contributo_enpap: row.get(10)?,
        ritenuta_acconto: row.get(11)?,
        marca_da_bollo: row.get::<_, i64>(12)? != 0,
        total_net: row.get(13)?,
        total_tax: row.get(14)?,
        total_gross: row.get(15)?,
        total_due: row.get(16)?,
        paid_date: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
        client_name: row.get(20)?,
        lines: vec![],
    })
}
