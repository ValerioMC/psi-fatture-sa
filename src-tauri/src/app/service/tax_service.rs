//! Italian tax calculation logic for PSI Fatture SA.
//!
//! Handles forfettario and ordinario regimes, ENPAP, ritenuta d'acconto,
//! and marca da bollo.

/// Statutory ENPAP integrative contribution rate (percent).
pub const ENPAP_RATE: f64 = 2.0;
/// Ritenuta d'acconto rate (percent) applied in the ordinario regime.
pub const RITENUTA_ORDINARIO_RATE: f64 = 20.0;
/// Tax regime identifier for the ordinario regime.
pub const TAX_REGIME_ORDINARIO: &str = "ordinario";

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

    let contributo_enpap = round2(total_net * enpap_rate / 100.0);
    let total_gross = total_net + total_tax + contributo_enpap;
    let ritenuta_acconto = round2((total_net + contributo_enpap) * ritenuta_rate / 100.0);

    let needs_bollo = total_tax == 0.0 && total_net > MARCA_DA_BOLLO_THRESHOLD;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn line(quantity: i64, unit_price: f64, vat_rate: f64) -> InvoiceLineData {
        InvoiceLineData {
            quantity,
            unit_price,
            vat_rate,
        }
    }

    #[test]
    fn forfettario_invoice_with_enpap_and_bollo() {
        let totals = calculate_invoice_totals(&[line(4, 70.0, 0.0)], ENPAP_RATE, 0.0);
        assert_eq!(totals.total_net, 280.0);
        assert_eq!(totals.total_tax, 0.0);
        assert_eq!(totals.contributo_enpap, 5.6);
        assert_eq!(totals.ritenuta_acconto, 0.0);
        assert_eq!(totals.marca_da_bollo, 2.0);
        assert_eq!(totals.total_gross, 285.6);
        assert_eq!(totals.total_due, 287.6);
    }

    #[test]
    fn no_bollo_below_threshold() {
        let totals = calculate_invoice_totals(&[line(1, 77.47, 0.0)], ENPAP_RATE, 0.0);
        assert_eq!(totals.marca_da_bollo, 0.0);
    }

    #[test]
    fn no_bollo_when_vat_applies() {
        let totals = calculate_invoice_totals(&[line(1, 100.0, 22.0)], ENPAP_RATE, 0.0);
        assert_eq!(totals.total_tax, 22.0);
        assert_eq!(totals.marca_da_bollo, 0.0);
    }

    #[test]
    fn bollo_applies_to_exempt_invoice_in_ordinario_regime() {
        let ritenuta = ritenuta_rate_for_regime(TAX_REGIME_ORDINARIO);
        let totals = calculate_invoice_totals(&[line(1, 100.0, 0.0)], ENPAP_RATE, ritenuta);
        assert_eq!(totals.marca_da_bollo, 2.0);
    }

    #[test]
    fn ordinario_ritenuta_on_net_plus_enpap() {
        let ritenuta = ritenuta_rate_for_regime(TAX_REGIME_ORDINARIO);
        let totals = calculate_invoice_totals(&[line(1, 100.0, 0.0)], ENPAP_RATE, ritenuta);
        assert_eq!(totals.contributo_enpap, 2.0);
        assert_eq!(totals.ritenuta_acconto, 20.4);
        assert_eq!(totals.total_due, 100.0 + 2.0 - 20.4 + 2.0);
    }

    #[test]
    fn ritenuta_rate_is_zero_for_forfettario() {
        assert_eq!(ritenuta_rate_for_regime("forfettario"), 0.0);
    }

    #[test]
    fn empty_lines_produce_zero_totals() {
        let totals = calculate_invoice_totals(&[], ENPAP_RATE, 0.0);
        assert_eq!(totals.total_net, 0.0);
        assert_eq!(totals.total_due, 0.0);
        assert_eq!(totals.marca_da_bollo, 0.0);
    }

    #[test]
    fn per_line_rounding_matches_stored_line_totals() {
        // 3 × 33.335 rounds per line first (100.01), not 100.005 → 100.0
        let totals = calculate_invoice_totals(&[line(3, 33.335, 0.0)], 0.0, 0.0);
        assert_eq!(totals.total_net, 100.01);
    }
}
