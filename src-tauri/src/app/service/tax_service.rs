//! Italian tax calculation logic for PSI Fatture SA.
//!
//! Handles forfettario and ordinario regimes, ENPAP, ritenuta d'acconto,
//! and marca da bollo.

use crate::app::model::tax::{InvoiceLineData, InvoiceTotals};

/// Statutory ENPAP integrative contribution rate (percent).
pub const ENPAP_RATE: f64 = 2.0;
/// Ritenuta d'acconto rate (percent) applied in the ordinario regime.
pub const RITENUTA_ORDINARIO_RATE: f64 = 20.0;
/// Tax regime identifier for the ordinario regime.
pub const TAX_REGIME_ORDINARIO: &str = "ordinario";

const MARCA_DA_BOLLO_THRESHOLD: f64 = 77.47;
const MARCA_DA_BOLLO_AMOUNT: f64 = 2.00;

/// Returns the ritenuta d'acconto rate (percent) for the given tax regime.
pub fn ritenuta_rate_for_regime(tax_regime: &str) -> f64 {
    if tax_regime == TAX_REGIME_ORDINARIO {
        RITENUTA_ORDINARIO_RATE
    } else {
        0.0
    }
}

/// Calculates all invoice totals based on lines and rates.
///
/// Marca da bollo (€2) applies to VAT-exempt invoices above €77.47
/// regardless of regime, as is the case for exempt healthcare services.
/// The bollo is compenso charged to the client, so it concurs to the
/// ENPAP base and is included before the ENPAP contribution is computed.
pub fn calculate_invoice_totals(
    lines: &[InvoiceLineData],
    enpap_rate: f64,
    ritenuta_rate: f64,
) -> InvoiceTotals {
    let (total_net, total_tax) = lines.iter().fold((0.0, 0.0), |(net, tax), line| {
        let line_net = round2(line.quantity as f64 * line.unit_price);
        let line_vat = round2(line_net * line.vat_rate / 100.0);
        (net + line_net, tax + line_vat)
    });

    let needs_bollo = total_tax == 0.0 && total_net > MARCA_DA_BOLLO_THRESHOLD;
    let marca_da_bollo = if needs_bollo {
        MARCA_DA_BOLLO_AMOUNT
    } else {
        0.0
    };

    // Marca da bollo charged to the client is additional compenso for a
    // forfettario professional, so it concurs to the ENPAP base too.
    let contributo_enpap = round2((total_net + marca_da_bollo) * enpap_rate / 100.0);
    let total_gross = total_net + total_tax + contributo_enpap;
    let ritenuta_acconto = round2((total_net + contributo_enpap) * ritenuta_rate / 100.0);

    let total_due = total_gross - ritenuta_acconto + marca_da_bollo;

    InvoiceTotals {
        total_net: round2(total_net),
        total_tax: round2(total_tax),
        contributo_enpap,
        ritenuta_acconto,
        marca_da_bollo,
        total_gross: round2(total_gross),
        total_due: round2(total_due),
    }
}

/// Rounds a value to 2 decimal places (ROUND_HALF_UP equivalent).
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
#[path = "tax_service_test.rs"]
mod tests;
