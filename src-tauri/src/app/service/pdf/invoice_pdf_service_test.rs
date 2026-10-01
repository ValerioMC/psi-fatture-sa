use super::super::{invoice_layout, invoice_legal_notes, DrawOp, Face};
use super::*;
use crate::app::model::client::{Client, ClientType};
use crate::app::model::config::{Profession, ProfessionalConfig, TaxRegime};
use crate::app::model::invoice::{InvoiceLine, InvoiceStatus, PaymentMethod};

fn line(description: &str, total: f64) -> InvoiceLine {
    InvoiceLine {
        id: None,
        invoice_id: None,
        service_id: None,
        description: description.to_string(),
        quantity: 1,
        unit_price: total,
        vat_rate: 0.0,
        line_total: total,
        amount_override: None,
    }
}

fn document(lines: Vec<InvoiceLine>) -> InvoiceDocument {
    let total_net: f64 = lines.iter().map(|l| l.line_total).sum();
    InvoiceDocument {
        invoice: Invoice {
            id: 7,
            client_id: 1,
            client_name: "Anna Bianchi".into(),
            invoice_number: "12".into(),
            year: 2026,
            issue_date: "2026-03-20".into(),
            due_date: Some("2026-04-19".into()),
            status: InvoiceStatus::Issued,
            payment_method: PaymentMethod::Bonifico,
            notes: "Sedute di marzo.\nGrazie.".into(),
            apply_enpap: true,
            contributo_enpap: total_net * 0.02,
            ritenuta_acconto: 0.0,
            marca_da_bollo: true,
            total_net,
            total_tax: 0.0,
            total_gross: total_net * 1.02 + 2.0,
            total_due: total_net * 1.02 + 2.0,
            paid_date: None,
            hide_quantity: false,
            lines,
            created_at: String::new(),
            updated_at: String::new(),
        },
        client: Client {
            id: 1,
            client_type: ClientType::PersonaFisica,
            first_name: "Anna".into(),
            last_name: "Bianchi".into(),
            birth_date: None,
            gender: None,
            fiscal_code: "BNCNNA80A41H501U".into(),
            vat_number: None,
            address: "Via Roma 1".into(),
            city: "Milano".into(),
            province: "MI".into(),
            zip_code: "20100".into(),
            email: Some("anna@example.it".into()),
            phone: String::new(),
            notes: None,
            sts_authorization: true,
            hide_quantity_in_invoice: None,
            created_at: String::new(),
            updated_at: String::new(),
        },
        config: ProfessionalConfig {
            id: 1,
            title: "Dott.ssa".into(),
            first_name: "Maria".into(),
            last_name: "Demo".into(),
            vat_number: "12345678903".into(),
            fiscal_code: "DMEMRA80A41H501X".into(),
            tax_regime: TaxRegime::Forfettario,
            albo_number: "1234".into(),
            albo_region: "Lombardia".into(),
            address: "Corso Buenos Aires 10".into(),
            city: "Milano".into(),
            province: "MI".into(),
            zip_code: "20124".into(),
            country: "IT".into(),
            phone: "02 1234567".into(),
            pec_email: "maria.demo@psypec.it".into(),
            iban: "IT60X0542811101000000123456".into(),
            coefficient: 78.0,
            profession: Profession::Psicoterapeuta,
            is_psicoanalista: true,
            specialization: String::new(),
            hide_quantity_in_invoice: false,
            enpap_excludes_bollo: false,
            initial_invoice_number: 1,
            created_at: String::new(),
            updated_at: String::new(),
        },
    }
}

fn texts(pages: &[Vec<DrawOp>]) -> Vec<String> {
    pages
        .iter()
        .flatten()
        .filter_map(|op| match op {
            DrawOp::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn renders_a_one_page_pdf_named_after_the_invoice() {
    let pdf = render_document(&document(vec![line("Colloquio psicologico", 80.0)])).unwrap();
    assert_eq!(pdf.file_name, "Fattura_12_2026.pdf");
    assert!(pdf.bytes.starts_with(b"%PDF-"));
    assert!(
        pdf.bytes.len() < 200_000,
        "fonts are subset: {} bytes",
        pdf.bytes.len()
    );
}

#[test]
fn prints_the_figures_the_print_view_prints() {
    let fonts = PdfFonts::shared().unwrap();
    let pages = invoice_layout::layout(&document(vec![line("Colloquio psicologico", 80.0)]), fonts);
    assert_eq!(pages.len(), 1);
    let texts = texts(&pages);
    for expected in [
        "Dott.ssa Maria Demo",
        "12",
        "20 marzo 2026",
        "Anna Bianchi",
        "Colloquio psicologico",
        "80,00 €",
        "83,60 €",
        "IT60X0542811101000000123456",
        "AUTORIZZA",
    ] {
        assert!(
            texts.iter().any(|t| t == expected),
            "missing {expected:?} in {texts:?}"
        );
    }
}

#[test]
fn long_invoices_continue_on_a_new_page_with_the_table_head_again() {
    let fonts = PdfFonts::shared().unwrap();
    let lines = (1..=45)
        .map(|n| line(&format!("Seduta del {n} marzo"), 80.0))
        .collect();
    let pages = invoice_layout::layout(&document(lines), fonts);
    assert!(pages.len() >= 2);
    let heads = texts(&pages).iter().filter(|t| *t == "DESCRIZIONE").count();
    assert!(heads >= 2, "the head repeats on the next page");
    for page in &pages {
        let pushes = page
            .iter()
            .filter(|op| matches!(op, DrawOp::PushClip(_)))
            .count();
        let pops = page
            .iter()
            .filter(|op| matches!(op, DrawOp::PopClip))
            .count();
        assert_eq!(pushes, pops, "every clip is closed on its own page");
    }
}

#[test]
fn hides_quantity_columns_when_the_invoice_asks() {
    let fonts = PdfFonts::shared().unwrap();
    let mut doc = document(vec![line("Colloquio", 80.0)]);
    doc.invoice.hide_quantity = true;
    let texts = texts(&invoice_layout::layout(&doc, fonts));
    assert!(!texts.iter().any(|t| t == "QTÀ" || t == "PREZZO UNIT."));
}

#[test]
fn hand_typed_amount_prints_without_a_unit_price() {
    let fonts = PdfFonts::shared().unwrap();
    let mut package = line("Pacchetto 4 sedute", 250.0);
    package.quantity = 4;
    package.unit_price = 70.0;
    package.amount_override = Some(250.0);
    let texts = texts(&invoice_layout::layout(&document(vec![package]), fonts));
    assert!(texts.iter().any(|t| t == "QTÀ"));
    assert!(!texts.iter().any(|t| t.contains("70,00")));
    assert!(texts.iter().any(|t| t.contains("250,00")));
}

#[test]
fn every_face_draws_italian_text_and_symbols() {
    let fonts = PdfFonts::shared().unwrap();
    for face in Face::ALL {
        assert!(
            fonts.face(face).covers("àèéìòù ÀÈÉ’'€·—–−%(),.:;/"),
            "{face:?} lacks a glyph"
        );
    }
}

#[test]
fn forfettario_and_enpap_share_one_paragraph() {
    let doc = document(vec![line("Colloquio", 100.0)]);
    let notes = invoice_legal_notes::legal_notes(&doc);
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("Regime Forfettario") && notes[0].contains("ENPAP 2% (2,00 €)"));
}

#[test]
fn ordinario_without_vat_is_exempt_and_asks_for_the_withholding() {
    let mut doc = document(vec![line("Colloquio", 100.0)]);
    doc.config.tax_regime = TaxRegime::Ordinario;
    doc.invoice.apply_enpap = false;
    doc.invoice.ritenuta_acconto = 20.0;
    let notes = invoice_legal_notes::legal_notes(&doc);
    assert_eq!(notes.len(), 2);
    assert!(notes[0].contains("esente da IVA"));
    assert!(notes[1].contains("20,00 €"));
}

#[test]
fn file_names_keep_only_safe_characters() {
    let mut doc = document(Vec::new());
    doc.invoice.invoice_number = "12/A bis".into();
    assert_eq!(file_name(&doc.invoice), "Fattura_12-A-bis_2026.pdf");
}

/// Writes a sample to `target/invoice-preview.pdf`, to look at after a layout change.
#[test]
#[ignore]
fn preview_sample_invoice() {
    let lines = vec![
        line("Colloquio psicologico individuale", 80.0),
        line("Psicoterapia individuale – seduta di 50 minuti con restituzione scritta e somministrazione di test", 95.0),
        line("Colloquio di coppia", 110.0),
    ];
    let pdf = render_document(&document(lines)).unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/invoice-preview.pdf");
    std::fs::write(path, pdf.bytes).unwrap();
}
