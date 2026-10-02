//! Turns a paid invoice into the expense document the Sistema TS records.
//! The amount is what the patient paid for the service (fee plus ENPAP,
//! without the marca da bollo), one item per VAT rate.

use std::collections::BTreeMap;

use sea_orm::ConnectionTrait;

use crate::app::common::AppError;
use crate::app::model::client::{Client, ClientType};
use crate::app::model::config::TaxRegime;
use crate::app::model::invoice::{Invoice, InvoiceStatus, PaymentMethod};
use crate::app::model::ts::{TsDocumentId, TsExpenseDocument, TsExpenseItem, TsVatTreatment};
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::{client_repository, config_repository};
use crate::app::service::{client_service, validation_service as validate};

const FORFETTARIO_NATURA: &str = "N2.2";
const EXEMPT_NATURA: &str = "N4";
const MAX_AMOUNT: f64 = 99_999.99;

/// Builds the document from the invoice as it is now. `id` overrides the
/// identifier, so a replacement keeps the one the original was sent under.
pub async fn build(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    vat_number: &str,
    id: Option<TsDocumentId>,
) -> Result<TsExpenseDocument, AppError> {
    let invoice = invoice_repository::load_invoice(db, invoice_id).await?;
    let client = client_repository::find_by_id(db, invoice.client_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Paziente {} non trovato", invoice.client_id)))?;
    let regime = config_repository::find(db)
        .await?
        .map(|c| TaxRegime::from(c.tax_regime))
        .unwrap_or(TaxRegime::Forfettario);
    document_from(
        &invoice,
        &client_service::into_domain(client),
        &regime,
        vat_number,
        id,
    )
}

pub fn document_from(
    invoice: &Invoice,
    client: &Client,
    regime: &TaxRegime,
    vat_number: &str,
    id: Option<TsDocumentId>,
) -> Result<TsExpenseDocument, AppError> {
    let payment_date = payment_date_of(invoice)?;
    let citizen_fiscal_code = citizen_of(client)?;
    let id = match id {
        Some(id) => id,
        None => TsDocumentId {
            vat_number: vat_number.trim().to_string(),
            issue_date: invoice.issue_date.clone(),
            number: document_number_of(invoice)?,
        },
    };
    Ok(TsExpenseDocument {
        id,
        payment_date,
        citizen_fiscal_code,
        items: items_of(invoice, regime)?,
        traced_payment: !matches!(
            invoice.payment_method,
            PaymentMethod::Contanti | PaymentMethod::Altro
        ),
    })
}

fn payment_date_of(invoice: &Invoice) -> Result<String, AppError> {
    if invoice.status != InvoiceStatus::Paid {
        return Err(AppError::Conflict(
            "Solo le fatture pagate possono essere trasmesse al Sistema TS".to_string(),
        ));
    }
    let date = invoice
        .paid_date
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .ok_or_else(|| {
            AppError::Invalid(
                "Indica la data di pagamento prima di trasmettere la fattura".to_string(),
            )
        })?;
    validate::parse_iso_date(date, "Data pagamento")?;
    Ok(date.to_string())
}

/// The patient's codice fiscale, or None when they opposed the transmission.
fn citizen_of(client: &Client) -> Result<Option<String>, AppError> {
    if client.client_type != ClientType::PersonaFisica {
        return Err(AppError::Invalid(
            "Al Sistema TS vanno solo le spese di persone fisiche".to_string(),
        ));
    }
    if !client.sts_authorization {
        return Ok(None);
    }
    let code = client.fiscal_code.trim().to_uppercase();
    if code.chars().count() != 16 {
        return Err(AppError::Invalid(format!(
            "Serve il codice fiscale di {} {} (16 caratteri) per trasmettere la spesa",
            client.first_name, client.last_name
        )));
    }
    validate::validate_fiscal_code(&code)?;
    Ok(Some(code))
}

fn document_number_of(invoice: &Invoice) -> Result<String, AppError> {
    let number = invoice.invoice_number.trim();
    let allowed = |c: char| c.is_ascii_alphanumeric() || "_./\\-".contains(c);
    if number.is_empty() || number.chars().count() > 20 || !number.chars().all(allowed) {
        return Err(AppError::Conflict(format!(
            "Numero fattura «{number}» non accettato dal Sistema TS (max 20 tra lettere, cifre e _ . / - )"
        )));
    }
    Ok(number.to_string())
}

/// One item per VAT rate. ENPAP follows each group's share of the net, and
/// the last group absorbs rounding so the items add up to `total_gross`.
fn items_of(invoice: &Invoice, regime: &TaxRegime) -> Result<Vec<TsExpenseItem>, AppError> {
    let total = round2(invoice.total_gross);
    if total <= 0.0 {
        return Err(AppError::Invalid(
            "La fattura ha importo nullo: niente da trasmettere".to_string(),
        ));
    }
    if total > MAX_AMOUNT {
        return Err(AppError::Invalid(
            "Importo oltre il massimo accettato dal Sistema TS (99.999,99 €)".to_string(),
        ));
    }

    let mut net_by_rate: BTreeMap<i64, f64> = BTreeMap::new();
    for line in &invoice.lines {
        *net_by_rate.entry(rate_key(line.vat_rate)).or_default() += line.line_total;
    }
    let net_total: f64 = net_by_rate.values().sum();
    if net_by_rate.len() <= 1 || net_total <= 0.0 {
        let rate = net_by_rate.keys().next().copied().unwrap_or(0);
        return Ok(vec![item(total, rate, regime)]);
    }

    let mut items = Vec::with_capacity(net_by_rate.len());
    let mut allotted = 0.0;
    let last = net_by_rate.len() - 1;
    for (index, (rate, net)) in net_by_rate.into_iter().enumerate() {
        let amount = if index == last {
            round2(total - allotted)
        } else {
            let gross = net * (1.0 + rate as f64 / 10_000.0);
            round2(gross + invoice.contributo_enpap * net / net_total)
        };
        allotted += amount;
        items.push(item(amount, rate, regime));
    }
    Ok(items)
}

fn item(amount: f64, rate_key: i64, regime: &TaxRegime) -> TsExpenseItem {
    let vat = if rate_key > 0 {
        TsVatTreatment::Rate(rate_key as f64 / 100.0)
    } else {
        TsVatTreatment::Natura(
            match regime {
                TaxRegime::Forfettario => FORFETTARIO_NATURA,
                TaxRegime::Ordinario => EXEMPT_NATURA,
            }
            .to_string(),
        )
    };
    TsExpenseItem { amount, vat }
}

/// VAT rate in hundredths, so rates group exactly.
fn rate_key(rate: f64) -> i64 {
    (rate * 100.0).round() as i64
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
#[path = "ts_document_service_test.rs"]
mod tests;
