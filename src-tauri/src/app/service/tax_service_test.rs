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
    assert_eq!(totals.contributo_enpap, 5.64);
    assert_eq!(totals.ritenuta_acconto, 0.0);
    assert_eq!(totals.marca_da_bollo, 2.0);
    assert_eq!(totals.total_gross, 285.64);
    assert_eq!(totals.total_due, 287.64);
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
    assert_eq!(totals.contributo_enpap, 2.04);
    assert_eq!(totals.ritenuta_acconto, 20.41);
    assert_eq!(totals.total_due, 83.63);
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
