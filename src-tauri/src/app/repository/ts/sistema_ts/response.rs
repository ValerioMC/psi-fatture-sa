//! Reads Sistema TS SOAP responses into domain types. Namespaces are ignored:
//! the services mix default and prefixed ones, and local names are unique.

use std::io::{Cursor, Read};

use base64::{engine::general_purpose::STANDARD, Engine};

use super::{ReportOutcome, XmlNode};
use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsEsito, TsExpenseTotal, TsMessage, TsQueryResult,
    TsRemoteDocument, TsReportRow,
};

const NOT_FOUND_CODE: &str = "WS96";
const CANCELLED_CODE: &str = "W010";

/// The `faultstring` of a SOAP Fault, if the body is one.
pub fn fault_message(xml: &str) -> Option<String> {
    let root = XmlNode::parse(xml).ok()?;
    root.find("Fault")
        .map(|fault| fault.text_of("faultstring").unwrap_or_default())
}

pub fn call_response(xml: &str) -> Result<TsCallResponse, String> {
    let root = XmlNode::parse(xml)?;
    let esito = esito_of(&root)?;
    let response = root
        .parent_of("esitoChiamata")
        .ok_or("Risposta senza esito")?;
    Ok(TsCallResponse {
        esito,
        protocol: response.text_of("protocollo"),
        messages: messages_of(response),
    })
}

pub fn query_response(xml: &str) -> Result<TsQueryResult, String> {
    let root = XmlNode::parse(xml)?;
    let esito = esito_of(&root)?;
    let response = root
        .parent_of("esitoChiamata")
        .ok_or("Risposta senza esito")?;
    let messages = messages_of(response);

    if esito == TsEsito::Rejected {
        if messages.iter().any(|m| m.code == NOT_FOUND_CODE) {
            return Ok(TsQueryResult::NotFound);
        }
        return Ok(TsQueryResult::Refused { messages });
    }
    let document = response
        .child("documentoFiscale")
        .ok_or("Documento assente nella risposta")?;
    Ok(TsQueryResult::Found {
        document: Box::new(remote_document(document, messages)?),
    })
}

/// The monthly report rows, empty when the month has no documents.
pub fn report_response(xml: &str) -> Result<ReportOutcome, String> {
    let root = XmlNode::parse(xml)?;
    let esito = esito_of(&root)?;
    let response = root
        .parent_of("esitoChiamata")
        .ok_or("Risposta senza esito")?;
    let messages = messages_of(response);

    if esito == TsEsito::Rejected {
        if messages.iter().any(|m| m.code == NOT_FOUND_CODE) {
            return Ok(ReportOutcome::Rows(Vec::new()));
        }
        return Ok(ReportOutcome::Refused(messages));
    }
    let Some(encoded) = response.text_of("fileCSV") else {
        return Ok(ReportOutcome::Rows(Vec::new()));
    };
    let csv = unzip_report(&encoded)?;
    Ok(ReportOutcome::Rows(report_rows(&csv)?))
}

fn remote_document(node: &XmlNode, messages: Vec<TsMessage>) -> Result<TsRemoteDocument, String> {
    let id_node = node
        .child("idDocumentoFiscale")
        .ok_or("Identificativo assente nella risposta")?;
    let number = id_node
        .child("numDocumentoFiscale")
        .and_then(|n| n.text_of("numDocumento"))
        .unwrap_or_default();
    Ok(TsRemoteDocument {
        id: TsDocumentId {
            vat_number: id_node.text_of("pIva").unwrap_or_default(),
            issue_date: id_node.text_of("dataEmissione").unwrap_or_default(),
            number,
        },
        payment_date: node.text_of("dataPagamento"),
        totals: totals_of(node, "totaliVociSpesa")?,
        refunded_totals: totals_of(node, "totaliVociSpesaRimborsate")?,
        protocol: node.text_of("protocollo"),
        sent_date: node.text_of("dataInvio"),
        send_kind: node.text_of("tipoInvio"),
        cancelled: messages.iter().any(|m| m.code == CANCELLED_CODE),
        messages,
    })
}

fn totals_of(node: &XmlNode, name: &str) -> Result<Vec<TsExpenseTotal>, String> {
    node.children_named(name)
        .map(|total| {
            Ok(TsExpenseTotal {
                expense_type: total.text_of("tipoSpesa").unwrap_or_default(),
                amount: parse_amount(&total.text_of("importo").unwrap_or_default())?,
            })
        })
        .collect()
}

fn esito_of(root: &XmlNode) -> Result<TsEsito, String> {
    let node = root
        .find("esitoChiamata")
        .ok_or("Risposta del Sistema TS senza esitoChiamata")?;
    TsEsito::parse(&node.text)
}

fn messages_of(response: &XmlNode) -> Vec<TsMessage> {
    response
        .child("listaMessaggi")
        .map(|list| {
            list.children_named("messaggio")
                .map(|m| TsMessage {
                    code: m.text_of("codice").unwrap_or_default(),
                    description: m.text_of("descrizione").unwrap_or_default(),
                    kind: m.text_of("tipo").unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn unzip_report(encoded: &str) -> Result<String, String> {
    let compact: String = encoded.split_whitespace().collect();
    let bytes = STANDARD
        .decode(compact)
        .map_err(|e| format!("Report non decodificabile: {e}"))?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("Report non è uno ZIP: {e}"))?;
    let mut file = archive
        .by_index(0)
        .map_err(|e| format!("Report vuoto: {e}"))?;
    let mut raw = Vec::new();
    file.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&raw).into_owned())
}

/// Parses `report.csv`: `;`-separated, header first, dates as dd/mm/yyyy,
/// one `TOTALE xx` / `TOTALE xx RIMBORSATO` column pair per expense type.
fn report_rows(csv: &str) -> Result<Vec<TsReportRow>, String> {
    let mut lines = csv.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<String> = lines
        .next()
        .map(|h| h.split(';').map(|c| c.trim().to_uppercase()).collect())
        .unwrap_or_default();
    let column = |name: &str| header.iter().position(|h| h == name);
    let required = |name: &str| column(name).ok_or(format!("Colonna {name} assente nel report"));

    let vat = required("PARTITA IVA")?;
    let issue = required("DATA EMISSIONE")?;
    let number = required("DOCUMENTO FISCALE NUMERO")?;
    let payment = required("DATA PAGAMENTO")?;
    let protocol = required("PROTOCOLLO")?;
    let sent = required("DATA INVIO")?;
    let kind = required("TIPO INVIO")?;
    let totals: Vec<usize> = header
        .iter()
        .enumerate()
        .filter(|(_, h)| h.starts_with("TOTALE ") && !h.ends_with("RIMBORSATO"))
        .map(|(i, _)| i)
        .collect();
    let refunds: Vec<usize> = header
        .iter()
        .enumerate()
        .filter(|(_, h)| h.starts_with("TOTALE ") && h.ends_with("RIMBORSATO"))
        .map(|(i, _)| i)
        .collect();

    lines
        .map(|line| {
            let cells: Vec<&str> = line.split(';').map(str::trim).collect();
            let cell = |i: usize| cells.get(i).copied().unwrap_or_default().to_string();
            let sum = |columns: &[usize]| -> Result<f64, String> {
                columns.iter().map(|i| parse_amount(&cell(*i))).sum()
            };
            Ok(TsReportRow {
                vat_number: cell(vat),
                issue_date: italian_to_iso(&cell(issue)),
                document_number: cell(number),
                payment_date: italian_to_iso(&cell(payment)),
                protocol: cell(protocol),
                sent_date: italian_to_iso(&cell(sent)),
                send_kind: cell(kind),
                amount: sum(&totals)?,
                refunded_amount: sum(&refunds)?,
                invoice_id: None,
            })
        })
        .collect()
}

fn parse_amount(value: &str) -> Result<f64, String> {
    if value.is_empty() {
        return Ok(0.0);
    }
    value
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| format!("Importo non valido nel report: {value}"))
}

fn italian_to_iso(value: &str) -> String {
    match value.split('/').collect::<Vec<_>>().as_slice() {
        [day, month, year] => format!("{year}-{month}-{day}"),
        _ => value.to_string(),
    }
}

#[cfg(test)]
#[path = "response_test.rs"]
mod tests;
