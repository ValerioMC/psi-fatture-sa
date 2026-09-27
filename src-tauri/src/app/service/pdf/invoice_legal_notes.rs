use crate::app::model::config::TaxRegime;
use crate::app::service::italian_format;

use super::InvoiceDocument;

const FORFETTARIO_NOTE: &str = "Operazione senza applicazione dell'IVA effettuata ai sensi dell'art. 1, commi da 54 a 89, L. n. 190 del 2014 e modificato dalla L.n. 208 del 2015 e dalla L.n. 145 del 2018 – Regime Forfettario. Imposta non dovuta.";
const EXEMPT_NOTE: &str = "Operazione esente da IVA ai sensi dell'art. 10, n. 18, DPR 633/72.";

/// The fiscal wording the regime and the totals call for, as the print view writes it:
/// the forfettario and ENPAP sentences share a paragraph when both apply.
pub fn legal_notes(document: &InvoiceDocument) -> Vec<String> {
    let invoice = &document.invoice;
    let forfettario = document.config.tax_regime == TaxRegime::Forfettario;
    let enpap = (invoice.apply_enpap && invoice.contributo_enpap > 0.0).then(|| {
        format!(
            "Contributo integrativo ENPAP 2% ({}) addebitato al cliente ai sensi dell'art. 8, L. 21/86.",
            italian_format::currency(invoice.contributo_enpap)
        )
    });

    let mut notes = Vec::new();
    match (forfettario, enpap) {
        (true, Some(enpap)) => notes.push(format!("{FORFETTARIO_NOTE} {enpap}")),
        (true, None) => notes.push(FORFETTARIO_NOTE.to_string()),
        (false, Some(enpap)) => notes.push(enpap),
        (false, None) => {}
    }
    if !forfettario && invoice.total_tax <= 0.0 {
        notes.push(EXEMPT_NOTE.to_string());
    }
    if invoice.ritenuta_acconto > 0.0 {
        notes.push(format!(
            "Si richiede di operare una ritenuta d'acconto del 20% pari a {}.",
            italian_format::currency(invoice.ritenuta_acconto)
        ));
    }
    notes
}
