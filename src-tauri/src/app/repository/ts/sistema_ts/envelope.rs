//! SOAP 1.1 request bodies for the synchronous Sistema TS services, shaped
//! after the Sogei kit (kit730P 20240214). The server insists on
//! `tipoDocumento` and on a VAT rate or natura for every item.

use quick_xml::escape::escape;

use super::{Credentials, DocumentCall, FieldCipher};
use crate::app::model::ts::{TsDocumentId, TsExpenseDocument, TsReportBasis, TsVatTreatment};

const DOCUMENT_NS: &str = "http://documentospesap730.sanita.finanze.it";
const QUERY_NS: &str = "http://interrogazionepuntuale.p730.sanita.finanze.it";
const REPORT_NS: &str = "http://reportmensile.p730.sanita.finanze.it";

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
#[path = "envelope_test.rs"]
mod tests;
