//! Turns a paid invoice into the expense document the Sistema TS records.
//! The amount is what the patient paid for the service (fee plus ENPAP,
//! without the marca da bollo), one item per VAT rate.

use std::collections::BTreeMap;

use sea_orm::ConnectionTrait;

use crate::app::model::client::{Client, ClientType};
use crate::app::model::config::TaxRegime;
use crate::app::model::invoice::{Invoice, InvoiceStatus, PaymentMethod};
use crate::app::model::ts::{TsDocumentId, TsExpenseDocument, TsExpenseItem, TsVatTreatment};
use crate::app::repository::{client_repository, config_repository, invoice_repository};
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
) -> Result<TsExpenseDocument, String> {
    let invoice = invoice_repository::load_invoice(db, invoice_id).await?;
    let client = client_repository::find_by_id(db, invoice.client_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Paziente {} non trovato", invoice.client_id))?;
    let regime = config_repository::find(db)
        .await
        .map_err(|e| e.to_string())?
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
) -> Result<TsExpenseDocument, String> {
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

fn payment_date_of(invoice: &Invoice) -> Result<String, String> {
    if invoice.status != InvoiceStatus::Paid {
        return Err("Solo le fatture pagate possono essere trasmesse al Sistema TS".to_string());
    }
    let date = invoice
        .paid_date
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .ok_or("Indica la data di pagamento prima di trasmettere la fattura")?;
    validate::parse_iso_date(date, "Data pagamento")?;
    Ok(date.to_string())
}

/// The patient's codice fiscale, or None when they opposed the transmission.
fn citizen_of(client: &Client) -> Result<Option<String>, String> {
    if client.client_type != ClientType::PersonaFisica {
        return Err("Al Sistema TS vanno solo le spese di persone fisiche".to_string());
    }
    if !client.sts_authorization {
        return Ok(None);
    }
    let code = client.fiscal_code.trim().to_uppercase();
    if code.chars().count() != 16 {
        return Err(format!(
            "Serve il codice fiscale di {} {} (16 caratteri) per trasmettere la spesa",
            client.first_name, client.last_name
        ));
    }
    validate::validate_fiscal_code(&code)?;
    Ok(Some(code))
}

fn document_number_of(invoice: &Invoice) -> Result<String, String> {
    let number = invoice.invoice_number.trim();
    let allowed = |c: char| c.is_ascii_alphanumeric() || "_./\\-".contains(c);
    if number.is_empty() || number.chars().count() > 20 || !number.chars().all(allowed) {
        return Err(format!(
            "Numero fattura «{number}» non accettato dal Sistema TS (max 20 tra lettere, cifre e _ . / - )"
        ));
    }
    Ok(number.to_string())
}

/// One item per VAT rate. ENPAP follows each group's share of the net, and
/// the last group absorbs rounding so the items add up to `total_gross`.
fn items_of(invoice: &Invoice, regime: &TaxRegime) -> Result<Vec<TsExpenseItem>, String> {
    let total = round2(invoice.total_gross);
    if total <= 0.0 {
        return Err("La fattura ha importo nullo: niente da trasmettere".to_string());
    }
    if total > MAX_AMOUNT {
        return Err("Importo oltre il massimo accettato dal Sistema TS (99.999,99 €)".to_string());
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
mod tests {
    use super::*;
    use crate::app::model::invoice::InvoiceLine;

    fn line(net: f64, rate: f64) -> InvoiceLine {
        InvoiceLine {
            id: None,
            invoice_id: None,
            service_id: None,
            description: "Seduta".to_string(),
            quantity: 1,
            unit_price: net,
            vat_rate: rate,
            line_total: net,
        }
    }

    fn invoice() -> Invoice {
        Invoice {
            id: 1,
            client_id: 1,
            client_name: "Anna Bianchi".to_string(),
            invoice_number: "12".to_string(),
            year: 2026,
            issue_date: "2026-03-01".to_string(),
            due_date: None,
            status: InvoiceStatus::Paid,
            payment_method: PaymentMethod::Bonifico,
            notes: String::new(),
            apply_enpap: true,
            contributo_enpap: 1.64,
            ritenuta_acconto: 0.0,
            marca_da_bollo: true,
            total_net: 80.0,
            total_tax: 0.0,
            total_gross: 81.64,
            total_due: 83.64,
            paid_date: Some("2026-03-02".to_string()),
            lines: vec![line(80.0, 0.0)],
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn client() -> Client {
        Client {
            id: 1,
            client_type: ClientType::PersonaFisica,
            first_name: "Anna".to_string(),
            last_name: "Bianchi".to_string(),
            birth_date: None,
            gender: None,
            fiscal_code: "rssmra80a41h501y".to_string(),
            vat_number: None,
            address: String::new(),
            city: String::new(),
            province: String::new(),
            zip_code: String::new(),
            email: None,
            phone: String::new(),
            notes: None,
            sts_authorization: true,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn build(
        invoice: &Invoice,
        client: &Client,
        regime: TaxRegime,
    ) -> Result<TsExpenseDocument, String> {
        document_from(invoice, client, &regime, "12345678903", None)
    }

    #[test]
    fn forfettario_invoice_sends_fee_plus_enpap_without_bollo() {
        let document = build(&invoice(), &client(), TaxRegime::Forfettario).unwrap();
        assert_eq!(document.id.vat_number, "12345678903");
        assert_eq!(document.id.issue_date, "2026-03-01");
        assert_eq!(document.id.number, "12");
        assert_eq!(document.payment_date, "2026-03-02");
        assert_eq!(
            document.citizen_fiscal_code.as_deref(),
            Some("RSSMRA80A41H501Y")
        );
        assert_eq!(
            document.items,
            vec![TsExpenseItem {
                amount: 81.64,
                vat: TsVatTreatment::Natura("N2.2".to_string())
            }]
        );
        assert!(document.traced_payment);
    }

    #[test]
    fn ordinario_exempt_service_uses_n4() {
        let document = build(&invoice(), &client(), TaxRegime::Ordinario).unwrap();
        assert_eq!(
            document.items[0].vat,
            TsVatTreatment::Natura("N4".to_string())
        );
    }

    #[test]
    fn opposition_drops_the_citizen_code() {
        let mut opposed = client();
        opposed.sts_authorization = false;
        opposed.fiscal_code.clear();
        let document = build(&invoice(), &opposed, TaxRegime::Forfettario).unwrap();
        assert!(document.opposed());
    }

    #[test]
    fn cash_and_other_payments_are_not_traced() {
        for method in [PaymentMethod::Contanti, PaymentMethod::Altro] {
            let mut cash = invoice();
            cash.payment_method = method;
            assert!(
                !build(&cash, &client(), TaxRegime::Forfettario)
                    .unwrap()
                    .traced_payment
            );
        }
        let mut pos = invoice();
        pos.payment_method = PaymentMethod::Pos;
        assert!(
            build(&pos, &client(), TaxRegime::Forfettario)
                .unwrap()
                .traced_payment
        );
    }

    #[test]
    fn mixed_rates_split_enpap_and_add_up_to_the_gross_total() {
        let mut mixed = invoice();
        mixed.lines = vec![line(100.0, 0.0), line(50.0, 22.0)];
        mixed.contributo_enpap = 3.0;
        mixed.total_tax = 11.0;
        mixed.total_gross = 164.0;
        let items = build(&mixed, &client(), TaxRegime::Ordinario)
            .unwrap()
            .items;
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0],
            TsExpenseItem {
                amount: 102.0,
                vat: TsVatTreatment::Natura("N4".to_string())
            }
        );
        assert_eq!(
            items[1],
            TsExpenseItem {
                amount: 62.0,
                vat: TsVatTreatment::Rate(22.0)
            }
        );
    }

    #[test]
    fn refuses_what_the_sistema_ts_cannot_take() {
        let mut unpaid = invoice();
        unpaid.status = InvoiceStatus::Issued;
        assert!(build(&unpaid, &client(), TaxRegime::Forfettario)
            .unwrap_err()
            .contains("pagate"));

        let mut undated = invoice();
        undated.paid_date = None;
        assert!(build(&undated, &client(), TaxRegime::Forfettario)
            .unwrap_err()
            .contains("data di pagamento"));

        let mut company = client();
        company.client_type = ClientType::Azienda;
        assert!(build(&invoice(), &company, TaxRegime::Forfettario)
            .unwrap_err()
            .contains("persone fisiche"));

        let mut no_code = client();
        no_code.fiscal_code.clear();
        assert!(build(&invoice(), &no_code, TaxRegime::Forfettario)
            .unwrap_err()
            .contains("codice fiscale"));

        let mut odd_number = invoice();
        odd_number.invoice_number = "12 bis".to_string();
        assert!(build(&odd_number, &client(), TaxRegime::Forfettario).is_err());

        let mut empty = invoice();
        empty.total_gross = 0.0;
        assert!(build(&empty, &client(), TaxRegime::Forfettario)
            .unwrap_err()
            .contains("nullo"));
    }

    #[test]
    fn explicit_id_is_kept_for_replacements() {
        let original = TsDocumentId {
            vat_number: "12345678903".to_string(),
            issue_date: "2026-02-28".to_string(),
            number: "11".to_string(),
        };
        let document = document_from(
            &invoice(),
            &client(),
            &TaxRegime::Forfettario,
            "99999999999",
            Some(original.clone()),
        )
        .unwrap();
        assert_eq!(document.id, original);
    }
}
