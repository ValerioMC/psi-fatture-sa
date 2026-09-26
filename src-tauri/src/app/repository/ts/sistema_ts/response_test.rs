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
