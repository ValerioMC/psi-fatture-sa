use super::*;
use crate::app::model::ts::TsExpenseItem;
use crate::test_support::VisibleCipher;

const CREDENTIALS: Credentials = Credentials {
    username: "mtomra66a41g224m",
    pincode: "3489543096",
};

fn id() -> TsDocumentId {
    TsDocumentId {
        vat_number: "65498732105".to_string(),
        issue_date: "2026-09-20".to_string(),
        number: "12/A".to_string(),
    }
}

fn document(citizen: Option<&str>) -> TsExpenseDocument {
    TsExpenseDocument {
        id: id(),
        payment_date: "2026-09-21".to_string(),
        citizen_fiscal_code: citizen.map(str::to_string),
        items: vec![TsExpenseItem {
            amount: 81.6,
            vat: TsVatTreatment::Natura("N2.2".to_string()),
        }],
        traced_payment: true,
    }
}

#[test]
fn insert_carries_encrypted_owner_citizen_and_the_mandatory_fields() {
    let xml = document_request(
        DocumentCall::Insert,
        &CREDENTIALS,
        &document(Some("rssmra80a01h501u")),
        &VisibleCipher,
    )
    .unwrap();
    assert!(xml.contains(
        "<inserimentoDocumentoSpesaRequest xmlns=\"http://documentospesap730.sanita.finanze.it\">"
    ));
    assert!(xml.contains("<pincode>enc(3489543096)</pincode>"));
    assert!(xml.contains("<cfProprietario>enc(MTOMRA66A41G224M)</cfProprietario>"));
    assert!(xml.contains("<idInserimentoDocumentoFiscale><idSpesa><pIva>65498732105</pIva>"));
    assert!(xml.contains("<numDocumento>12/A</numDocumento>"));
    assert!(xml.contains("<cfCittadino>enc(RSSMRA80A01H501U)</cfCittadino>"));
    assert!(xml.contains(
        "<voceSpesa><tipoSpesa>SP</tipoSpesa><importo>81.60</importo><naturaIVA>N2.2</naturaIVA></voceSpesa>"
    ));
    assert!(xml.contains(
        "<pagamentoTracciato>SI</pagamentoTracciato><tipoDocumento>F</tipoDocumento><flagOpposizione>0</flagOpposizione>"
    ));
}

#[test]
fn opposition_omits_the_citizen_and_sets_the_flag() {
    let xml = document_request(
        DocumentCall::Update,
        &CREDENTIALS,
        &document(None),
        &VisibleCipher,
    )
    .unwrap();
    assert!(xml.contains("<variazioneDocumentoSpesaRequest"));
    assert!(xml.contains("<idVariazioneDocumentoFiscale>"));
    assert!(!xml.contains("cfCittadino"));
    assert!(xml.contains("<flagOpposizione>1</flagOpposizione>"));
}

#[test]
fn vat_rate_is_written_with_two_decimals() {
    let mut taxed = document(Some("RSSMRA80A01H501U"));
    taxed.items[0].vat = TsVatTreatment::Rate(22.0);
    taxed.traced_payment = false;
    let xml = document_request(DocumentCall::Insert, &CREDENTIALS, &taxed, &VisibleCipher).unwrap();
    assert!(xml.contains("<aliquotaIVA>22.00</aliquotaIVA>"));
    assert!(xml.contains("<pagamentoTracciato>NO</pagamentoTracciato>"));
}

#[test]
fn cancellation_and_query_reference_the_document_id_only() {
    let cancel = cancel_request(&CREDENTIALS, &id(), &VisibleCipher).unwrap();
    assert!(cancel.contains("<idCancellazioneDocumentoFiscale><pIva>65498732105</pIva><dataEmissione>2026-09-20</dataEmissione><numDocumentoFiscale><dispositivo>1</dispositivo><numDocumento>12/A</numDocumento></numDocumentoFiscale></idCancellazioneDocumentoFiscale>"));
    assert!(!cancel.contains("voceSpesa"));

    let query = query_request(&CREDENTIALS, &id(), &VisibleCipher).unwrap();
    assert!(query.contains("xmlns=\"http://interrogazionepuntuale.p730.sanita.finanze.it\""));
    assert!(query.contains("<idDocumentoFiscale><pIva>"));
}

#[test]
fn monthly_report_pads_the_period() {
    let xml = monthly_report_request(
        &CREDENTIALS,
        2026,
        3,
        TsReportBasis::Pagamento,
        &VisibleCipher,
    )
    .unwrap();
    assert!(xml.contains("<annoMese>202603</annoMese><tipoEstrazione>P</tipoEstrazione>"));
}

#[test]
fn payment_before_issue_is_flagged_as_advance() {
    let mut early = document(Some("RSSMRA80A01H501U"));
    early.payment_date = "2026-09-19".to_string();
    let xml = document_request(DocumentCall::Insert, &CREDENTIALS, &early, &VisibleCipher).unwrap();
    assert!(xml.contains("<dataPagamento>2026-09-19</dataPagamento><flagPagamentoAnticipato>1</flagPagamentoAnticipato>"));
    let on_time = document_request(
        DocumentCall::Insert,
        &CREDENTIALS,
        &document(None),
        &VisibleCipher,
    )
    .unwrap();
    assert!(!on_time.contains("flagPagamentoAnticipato"));
}

#[test]
fn escapes_text_values() {
    let mut odd = document(Some("RSSMRA80A01H501U"));
    odd.id.number = "1<2".to_string();
    let xml = document_request(DocumentCall::Insert, &CREDENTIALS, &odd, &VisibleCipher).unwrap();
    assert!(xml.contains("<numDocumento>1&lt;2</numDocumento>"));
}
