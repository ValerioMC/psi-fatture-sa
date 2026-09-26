//! Reads Sistema TS SOAP responses into domain types. Namespaces are ignored:
//! the services mix default and prefixed ones, and local names are unique.

use std::io::{Cursor, Read};

use base64::{engine::general_purpose::STANDARD, Engine};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::app::model::ts::{
    TsCallResponse, TsDocumentId, TsEsito, TsExpenseTotal, TsMessage, TsQueryResult,
    TsRemoteDocument, TsReportRow,
};

const NOT_FOUND_CODE: &str = "WS96";
const CANCELLED_CODE: &str = "W010";

#[derive(Debug, Default)]
struct Node {
    name: String,
    text: String,
    children: Vec<Node>,
}

impl Node {
    fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }

    fn text_of(&self, name: &str) -> Option<String> {
        self.child(name)
            .map(|c| c.text.trim().to_string())
            .filter(|t| !t.is_empty())
    }

    fn find(&self, name: &str) -> Option<&Node> {
        if self.name == name {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(name))
    }
}

/// The `faultstring` of a SOAP Fault, if the body is one.
pub fn fault_message(xml: &str) -> Option<String> {
    let root = parse_tree(xml).ok()?;
    root.find("Fault")
        .map(|fault| fault.text_of("faultstring").unwrap_or_default())
}

pub fn call_response(xml: &str) -> Result<TsCallResponse, String> {
    let root = parse_tree(xml)?;
    let esito = esito_of(&root)?;
    let response = parent_of(&root, "esitoChiamata").ok_or("Risposta senza esito")?;
    Ok(TsCallResponse {
        esito,
        protocol: response.text_of("protocollo"),
        messages: messages_of(response),
    })
}

pub fn query_response(xml: &str) -> Result<TsQueryResult, String> {
    let root = parse_tree(xml)?;
    let esito = esito_of(&root)?;
    let response = parent_of(&root, "esitoChiamata").ok_or("Risposta senza esito")?;
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

#[derive(Debug, PartialEq)]
pub enum ReportOutcome {
    Rows(Vec<TsReportRow>),
    Refused(Vec<TsMessage>),
}

/// The monthly report rows, empty when the month has no documents.
pub fn report_response(xml: &str) -> Result<ReportOutcome, String> {
    let root = parse_tree(xml)?;
    let esito = esito_of(&root)?;
    let response = parent_of(&root, "esitoChiamata").ok_or("Risposta senza esito")?;
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

fn remote_document(node: &Node, messages: Vec<TsMessage>) -> Result<TsRemoteDocument, String> {
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

fn totals_of(node: &Node, name: &str) -> Result<Vec<TsExpenseTotal>, String> {
    node.children_named(name)
        .map(|total| {
            Ok(TsExpenseTotal {
                expense_type: total.text_of("tipoSpesa").unwrap_or_default(),
                amount: parse_amount(&total.text_of("importo").unwrap_or_default())?,
            })
        })
        .collect()
}

fn esito_of(root: &Node) -> Result<TsEsito, String> {
    let node = root
        .find("esitoChiamata")
        .ok_or("Risposta del Sistema TS senza esitoChiamata")?;
    TsEsito::parse(&node.text)
}

fn messages_of(response: &Node) -> Vec<TsMessage> {
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

fn parent_of<'a>(node: &'a Node, child: &str) -> Option<&'a Node> {
    if node.child(child).is_some() {
        return Some(node);
    }
    node.children.iter().find_map(|c| parent_of(c, child))
}

fn parse_tree(xml: &str) -> Result<Node, String> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Node> = vec![Node::default()];
    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => stack.push(Node {
                name: local_name(start.local_name().as_ref()),
                ..Default::default()
            }),
            Ok(Event::Empty(empty)) => {
                let node = Node {
                    name: local_name(empty.local_name().as_ref()),
                    ..Default::default()
                };
                attach(&mut stack, node)?;
            }
            Ok(Event::Text(text)) => {
                let value = text.unescape().map_err(|e| e.to_string())?;
                if let Some(current) = stack.last_mut() {
                    current.text.push_str(&value);
                }
            }
            Ok(Event::End(_)) => {
                let node = stack.pop().ok_or("XML non bilanciato")?;
                attach(&mut stack, node)?;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(format!("Risposta XML non valida: {e}")),
        }
    }
    stack.pop().ok_or_else(|| "XML vuoto".to_string())
}

fn attach(stack: &mut [Node], node: Node) -> Result<(), String> {
    stack
        .last_mut()
        .map(|parent| parent.children.push(node))
        .ok_or_else(|| "XML non bilanciato".to_string())
}

fn local_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).into_owned()
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
mod tests {
    use super::*;
    use std::io::Write;

    // Responses captured from the Sistema TS test environment (26/09/2026).
    const ACCEPTED_WITH_WARNING: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"><soapenv:Body><inserimentoDocumentoSpesaResponse xmlns="http://documentospesap730.sanita.finanze.it"><esitoChiamata>2</esitoChiamata><protocollo>99260926001866680</protocollo><listaMessaggi><messaggio><codice>W008</codice><descrizione>IL DOCUMENTO E' STATO TRASMESSO OLTRE I TERMINI PREVISTI</descrizione><tipo>W</tipo></messaggio><messaggio><codice>0</codice><descrizione>Operazione eseguita correttamente</descrizione><tipo/></messaggio></listaMessaggi></inserimentoDocumentoSpesaResponse></soapenv:Body></soapenv:Envelope>"#;

    const REJECTED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"><soapenv:Body><inserimentoDocumentoSpesaResponse xmlns="http://documentospesap730.sanita.finanze.it"><esitoChiamata>1</esitoChiamata><listaMessaggi><messaggio><codice>S017</codice><descrizione>IDENTIFICATIVO DOCUMENTO FISCALE GIA' PRESENTE</descrizione><tipo>E</tipo></messaggio></listaMessaggi></inserimentoDocumentoSpesaResponse></soapenv:Body></soapenv:Envelope>"#;

    const QUERY_CANCELLED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"><soapenv:Body><interrogazionePuntualeResponse xmlns="http://interrogazionepuntuale.p730.sanita.finanze.it"><esitoChiamata>2</esitoChiamata><documentoFiscale><idDocumentoFiscale><pIva>65498732105</pIva><dataEmissione>2026-09-20</dataEmissione><numDocumentoFiscale><dispositivo>1</dispositivo><numDocumento>I133406</numDocumento></numDocumentoFiscale></idDocumentoFiscale><dataPagamento>2026-09-21</dataPagamento><totaliVociSpesa><tipoSpesa>SP</tipoSpesa><importo>90.0</importo></totaliVociSpesa><protocollo>99260926001866687</protocollo><nomeFile>SERVIZIO SINCRONO</nomeFile><dataInvio>2026-09-26</dataInvio><tipoInvio>V</tipoInvio></documentoFiscale><listaMessaggi><messaggio><codice>0</codice><descrizione>Operazione eseguita correttamente</descrizione><tipo/></messaggio><messaggio><codice>W010</codice><descrizione>IL DOCUMENTO E' STATO ANNULLATO IN PRECEDENZA</descrizione><tipo>W</tipo></messaggio></listaMessaggi></interrogazionePuntualeResponse></soapenv:Body></soapenv:Envelope>"#;

    const QUERY_NOT_FOUND: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"><soapenv:Body><interrogazionePuntualeResponse xmlns="http://interrogazionepuntuale.p730.sanita.finanze.it"><esitoChiamata>1</esitoChiamata><listaMessaggi><messaggio><codice>WS96</codice><descrizione>LA RICERCA NON HA PRODOTTO RISULTATI</descrizione><tipo>E</tipo></messaggio></listaMessaggi></interrogazionePuntualeResponse></soapenv:Body></soapenv:Envelope>"#;

    const AUTH_FAULT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
<env:Body>
<env:Fault>
<faultcode>env:Client</faultcode>
<faultstring>Errore generico di autenticazione</faultstring></env:Fault></env:Body></env:Envelope>"#;

    const REPORT_CSV: &str = "CODICE REGIONE; CODICE ASL; CODICE SSA; CF PROPRIETARIO; PARTITA IVA; DATA EMISSIONE; DOCUMENTO FISCALE NUMERO; DOCUMENTO FISCALE DISPOSITIVO; DATA PAGAMENTO; PROTOCOLLO; NOME FILE; DATA INVIO; TIPO INVIO; TOTALE TK; TOTALE TK RIMBORSATO; TOTALE SP; TOTALE SP RIMBORSATO; \n;;;MTOMRA66A41G224M;65498732105;03/09/2026;FT0001/2026;1;03/09/2026;99260903001861939;SERVIZIO SINCRONO;03/09/2026;I;0.0;0.0;402.0;0.0;\n;;;MTOMRA66A41G224M;65498732105;01/12/2025;181;1;01/12/2025;99260903001862008;SERVIZIO SINCRONO;03/09/2026;V;0.0;0.0;40.0;5.5;\n";

    fn zipped_report(csv: &str) -> String {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut buffer);
            writer
                .start_file("report.csv", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(csv.as_bytes()).unwrap();
            writer.finish().unwrap();
        }
        let encoded = STANDARD.encode(buffer.into_inner());
        format!(
            r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"><soapenv:Body><reportMensileResponse xmlns="http://reportmensile.p730.sanita.finanze.it"><esitoChiamata>0</esitoChiamata><fileCSV>{encoded}</fileCSV><listaMessaggi><messaggio><codice>0</codice><descrizione>ok</descrizione><tipo/></messaggio></listaMessaggi></reportMensileResponse></soapenv:Body></soapenv:Envelope>"#
        )
    }

    #[test]
    fn reads_an_accepted_call_with_protocol_and_warnings() {
        let response = call_response(ACCEPTED_WITH_WARNING).unwrap();
        assert_eq!(response.esito, TsEsito::AcceptedWithWarnings);
        assert_eq!(response.protocol.as_deref(), Some("99260926001866680"));
        assert_eq!(response.messages.len(), 2);
        assert_eq!(response.messages[0].code, "W008");
        assert_eq!(response.messages[1].kind, "");
    }

    #[test]
    fn reads_a_rejection_without_protocol() {
        let response = call_response(REJECTED).unwrap();
        assert_eq!(response.esito, TsEsito::Rejected);
        assert!(response.protocol.is_none());
        assert!(response.has_code("S017"));
    }

    #[test]
    fn reads_a_cancelled_document_from_the_point_query() {
        let TsQueryResult::Found { document } = query_response(QUERY_CANCELLED).unwrap() else {
            panic!("expected a document");
        };
        assert_eq!(document.id.number, "I133406");
        assert_eq!(document.id.issue_date, "2026-09-20");
        assert_eq!(document.payment_date.as_deref(), Some("2026-09-21"));
        assert_eq!(document.totals[0].amount, 90.0);
        assert_eq!(document.send_kind.as_deref(), Some("V"));
        assert!(document.cancelled);
    }

    #[test]
    fn missing_document_is_not_found_rather_than_an_error() {
        assert_eq!(
            query_response(QUERY_NOT_FOUND).unwrap(),
            TsQueryResult::NotFound
        );
    }

    #[test]
    fn detects_soap_faults() {
        assert_eq!(
            fault_message(AUTH_FAULT).as_deref(),
            Some("Errore generico di autenticazione")
        );
        assert_eq!(fault_message(REJECTED), None);
        assert!(call_response(AUTH_FAULT).is_err());
    }

    #[test]
    fn unpacks_the_zipped_monthly_report() {
        let ReportOutcome::Rows(rows) = report_response(&zipped_report(REPORT_CSV)).unwrap() else {
            panic!("expected rows");
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].document_number, "FT0001/2026");
        assert_eq!(rows[0].issue_date, "2026-09-03");
        assert_eq!(rows[0].amount, 402.0);
        assert_eq!(rows[1].send_kind, "V");
        assert_eq!(rows[1].refunded_amount, 5.5);
        assert_eq!(rows[1].sent_date, "2026-09-03");
    }

    #[test]
    fn empty_month_is_an_empty_report() {
        let empty = QUERY_NOT_FOUND.replace("interrogazionePuntuale", "reportMensile");
        assert_eq!(
            report_response(&empty).unwrap(),
            ReportOutcome::Rows(Vec::new())
        );
    }

    #[test]
    fn report_refusal_keeps_the_messages() {
        let refused = REJECTED.replace("S017", "WS46");
        let ReportOutcome::Refused(messages) = report_response(&refused).unwrap() else {
            panic!("expected a refusal");
        };
        assert_eq!(messages[0].code, "WS46");
    }
}
