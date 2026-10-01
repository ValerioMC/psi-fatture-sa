//! Italian tax calculation logic for PSI Fatture SA.
//!
//! Handles forfettario and ordinario regimes, ENPAP, ritenuta d'acconto,
//! and marca da bollo.

use crate::app::model::config::TaxRegime;
use crate::app::model::tax::{round_cents, InvoiceLineData, InvoiceTotals, TaxProfile, TaxRules};

/// Statutory ENPAP integrative contribution rate (percent).
pub const ENPAP_RATE: f64 = 2.0;
/// Ritenuta d'acconto rate (percent) applied in the ordinario regime.
pub const RITENUTA_ORDINARIO_RATE: f64 = 20.0;

const MARCA_DA_BOLLO_THRESHOLD: f64 = 77.47;
const MARCA_DA_BOLLO_AMOUNT: f64 = 2.00;

/// Returns the ritenuta d'acconto rate (percent) for the given tax regime.
pub fn ritenuta_rate_for_regime(tax_regime: &TaxRegime) -> f64 {
    match tax_regime {
        TaxRegime::Ordinario => RITENUTA_ORDINARIO_RATE,
        TaxRegime::Forfettario => 0.0,
    }
}

/// The rules an invoice is taxed with. In forfettario the bollo is compenso
/// (Interpello AdE) and joins the ENPAP base unless the profile opts out; in
/// ordinario it never does.
pub fn rules_for(profile: &TaxProfile, apply_enpap: bool) -> TaxRules {
    TaxRules {
        enpap_rate: if apply_enpap { ENPAP_RATE } else { 0.0 },
        ritenuta_rate: ritenuta_rate_for_regime(&profile.tax_regime),
        enpap_includes_bollo: profile.tax_regime == TaxRegime::Forfettario
            && !profile.enpap_excludes_bollo,
    }
}

/// Calculates all invoice totals based on lines and rules.
///
/// Marca da bollo (€2) applies to VAT-exempt invoices above €77.47
/// regardless of regime, as is the case for exempt healthcare services.
pub fn calculate_invoice_totals(lines: &[InvoiceLineData], rules: &TaxRules) -> InvoiceTotals {
    let (total_net, total_tax) = lines.iter().fold((0.0, 0.0), |(net, tax), line| {
        (net + line.net_amount(), tax + line.vat_amount())
    });

    let needs_bollo = total_tax == 0.0 && total_net > MARCA_DA_BOLLO_THRESHOLD;
    let marca_da_bollo = if needs_bollo {
        MARCA_DA_BOLLO_AMOUNT
    } else {
        0.0
    };

    let enpap_base = if rules.enpap_includes_bollo {
        total_net + marca_da_bollo
    } else {
        total_net
    };
    let contributo_enpap = round_cents(enpap_base * rules.enpap_rate / 100.0);
    let total_gross = total_net + total_tax + contributo_enpap;
    let ritenuta_acconto =
        round_cents((total_net + contributo_enpap) * rules.ritenuta_rate / 100.0);

    let total_due = total_gross - ritenuta_acconto + marca_da_bollo;

    InvoiceTotals {
        total_net: round_cents(total_net),
        total_tax: round_cents(total_tax),
        contributo_enpap,
        ritenuta_acconto,
        marca_da_bollo,
        total_gross: round_cents(total_gross),
        total_due: round_cents(total_due),
    }
}

#[cfg(test)]
#[path = "tax_service_test.rs"]
mod tests;
