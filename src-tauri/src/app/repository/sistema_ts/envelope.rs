//! SOAP 1.1 request bodies for the synchronous Sistema TS services, shaped
//! after the Sogei kit (kit730P 20240214). The server insists on
//! `tipoDocumento` and on a VAT rate or natura for every item.

use quick_xml::escape::escape;

use super::cipher::FieldCipher;
use crate::app::model::ts::{TsDocumentId, TsExpenseDocument, TsReportBasis, TsVatTreatment};

const DOCUMENT_NS: &str = "http://documentospesap730.sanita.finanze.it";
const QUERY_NS: &str = "http://interrogazionepuntuale.p730.sanita.finanze.it";
const REPORT_NS: &str = "http://reportmensile.p730.sanita.finanze.it";

/// The login and PINCODE of the professional, in clear; encrypted when written.
pub struct Credentials<'a> {
    pub username: &'a str,
    pub pincode: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentCall {
    Insert,
    Update,
}

impl DocumentCall {
    fn request_element(&self) -> &'static str {
        match self {
            DocumentCall::Insert => "inserimentoDocumentoSpesaRequest",
            DocumentCall::Update => "variazioneDocumentoSpesaRequest",
        }
    }

    fn document_element(&self) -> &'static str {
        match self {
            DocumentCall::Insert => "idInserimentoDocumentoFiscale",
            DocumentCall::Update => "idVariazioneDocumentoFiscale",
        }
    }
}

pub fn document_request(
    call: DocumentCall,
    credentials: &Credentials,
    document: &TsExpenseDocument,
    cipher: &dyn FieldCipher,
) -> Result<String, String> {
    let mut body = owner_block(credentials, cipher)?;
    body.push_str(&format!("<{}>", call.document_element()));
    body.push_str(&expense_document(document, cipher)?);
    body.push_str(&format!("</{}>", call.document_element()));
    Ok(envelope(call.request_element(), DOCUMENT_NS, &body))
}

pub fn cancel_request(
    credentials: &Credentials,
    id: &TsDocumentId,
    cipher: &dyn FieldCipher,
) -> Result<String, String> {
    let mut body = owner_block(credentials, cipher)?;
    body.push_str(&element(
        "idCancellazioneDocumentoFiscale",
        &document_id(id),
    ));
    Ok(envelope(
        "cancellazioneDocumentoSpesaRequest",
        DOCUMENT_NS,
        &body,
    ))
}

pub fn query_request(
    credentials: &Credentials,
    id: &TsDocumentId,
    cipher: &dyn FieldCipher,
) -> Result<String, String> {
    let mut body = owner_block(credentials, cipher)?;
    body.push_str(&element("idDocumentoFiscale", &document_id(id)));
    Ok(envelope("interrogazionePuntualeRequest", QUERY_NS, &body))
}

pub fn monthly_report_request(
    credentials: &Credentials,
    year: i32,
    month: u32,
    basis: TsReportBasis,
    cipher: &dyn FieldCipher,
) -> Result<String, String> {
    let mut body = owner_block(credentials, cipher)?;
    body.push_str(&text_element("annoMese", &format!("{year:04}{month:02}")));
    body.push_str(&text_element("tipoEstrazione", basis.code()));
    Ok(envelope("reportMensileRequest", REPORT_NS, &body))
}

fn envelope(request: &str, namespace: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\">\
         <soapenv:Header/><soapenv:Body>\
         <{request} xmlns=\"{namespace}\">{body}</{request}>\
         </soapenv:Body></soapenv:Envelope>"
    )
}

fn owner_block(credentials: &Credentials, cipher: &dyn FieldCipher) -> Result<String, String> {
    let pincode = cipher.encrypt(credentials.pincode)?;
    let owner = cipher.encrypt(&credentials.username.to_uppercase())?;
    Ok(format!(
        "{}{}",
        text_element("pincode", &pincode),
        element("Proprietario", &text_element("cfProprietario", &owner))
    ))
}

fn expense_document(
    document: &TsExpenseDocument,
    cipher: &dyn FieldCipher,
) -> Result<String, String> {
    let mut xml = element("idSpesa", &document_id(&document.id));
    xml.push_str(&text_element("dataPagamento", &document.payment_date));
    if document.paid_in_advance() {
        xml.push_str(&text_element("flagPagamentoAnticipato", "1"));
    }
    if let Some(citizen) = &document.citizen_fiscal_code {
        xml.push_str(&text_element(
            "cfCittadino",
            &cipher.encrypt(&citizen.to_uppercase())?,
        ));
    }
    for item in &document.items {
        let mut voce = text_element("tipoSpesa", "SP");
        voce.push_str(&text_element("importo", &format!("{:.2}", item.amount)));
        voce.push_str(&match &item.vat {
            TsVatTreatment::Rate(rate) => text_element("aliquotaIVA", &format!("{rate:.2}")),
            TsVatTreatment::Natura(code) => text_element("naturaIVA", code),
        });
        xml.push_str(&element("voceSpesa", &voce));
    }
    xml.push_str(&text_element(
        "pagamentoTracciato",
        if document.traced_payment { "SI" } else { "NO" },
    ));
    xml.push_str(&text_element("tipoDocumento", "F"));
    xml.push_str(&text_element(
        "flagOpposizione",
        if document.opposed() { "1" } else { "0" },
    ));
    Ok(xml)
}

fn document_id(id: &TsDocumentId) -> String {
    let number = format!(
        "{}{}",
        text_element("dispositivo", "1"),
        text_element("numDocumento", &id.number)
    );
    format!(
        "{}{}{}",
        text_element("pIva", &id.vat_number),
        text_element("dataEmissione", &id.issue_date),
        element("numDocumentoFiscale", &number)
    )
}

fn element(name: &str, inner_xml: &str) -> String {
    format!("<{name}>{inner_xml}</{name}>")
}

fn text_element(name: &str, text: &str) -> String {
    element(name, &escape(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::ts::TsExpenseItem;
    use crate::app::repository::sistema_ts::cipher::testing::VisibleCipher;

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
        let xml =
            document_request(DocumentCall::Insert, &CREDENTIALS, &taxed, &VisibleCipher).unwrap();
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
        let xml =
            document_request(DocumentCall::Insert, &CREDENTIALS, &early, &VisibleCipher).unwrap();
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
        let xml =
            document_request(DocumentCall::Insert, &CREDENTIALS, &odd, &VisibleCipher).unwrap();
        assert!(xml.contains("<numDocumento>1&lt;2</numDocumento>"));
    }
}
