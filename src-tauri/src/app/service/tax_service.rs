//! Italian tax calculation logic for PSI Fatture SA.
//!
//! Handles forfettario and ordinario regimes, ENPAP, ritenuta d'acconto,
//! marca da bollo, IRPEF, and INPS contributions.

const MARCA_DA_BOLLO_THRESHOLD: f64 = 77.47;
const MARCA_DA_BOLLO_AMOUNT: f64 = 2.00;

#[derive(Debug, Clone)]
pub struct InvoiceLineData {
    pub quantity: i64,
    pub unit_price: f64,
    pub vat_rate: f64,
}

#[derive(Debug, Clone)]
pub struct InvoiceTotals {
    pub total_net: f64,
    pub total_tax: f64,
    pub contributo_enpap: f64,
    pub ritenuta_acconto: f64,
    pub marca_da_bollo: f64,
    pub total_gross: f64,
    pub total_due: f64,
}

/// Calculates all invoice totals based on lines and tax regime.
pub fn calculate_invoice_totals(
    lines: &[InvoiceLineData],
    tax_regime: &str,
    enpap_rate: f64,
    ritenuta_rate: f64,
) -> InvoiceTotals {
    let (total_net, total_tax) = lines.iter().fold((0.0, 0.0), |(net, tax), line| {
        let line_net = round2(line.quantity as f64 * line.unit_price);
        let line_vat = round2(line_net * line.vat_rate / 100.0);
        (net + line_net, tax + line_vat)
    });

    let contributo_enpap = round2(total_net * enpap_rate / 100.0);
    let total_gross = total_net + total_tax + contributo_enpap;
    let ritenuta_acconto = round2((total_net + contributo_enpap) * ritenuta_rate / 100.0);

    let needs_bollo =
        tax_regime == "forfettario" && total_net > MARCA_DA_BOLLO_THRESHOLD && total_tax == 0.0;
    let marca_da_bollo = if needs_bollo {
        MARCA_DA_BOLLO_AMOUNT
    } else {
        0.0
    };

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
