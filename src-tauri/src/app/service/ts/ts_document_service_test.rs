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
