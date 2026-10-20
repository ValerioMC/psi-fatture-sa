use super::*;

fn line(quantity: i64, unit_price: f64, vat_rate: f64) -> InvoiceLineData {
    InvoiceLineData {
        quantity,
        unit_price,
        vat_rate,
        amount_override: None,
    }
}

fn profile(tax_regime: TaxRegime, enpap_excludes_bollo: bool) -> TaxProfile {
    TaxProfile {
        tax_regime,
        enpap_excludes_bollo,
    }
}

fn forfettario() -> TaxRules {
    rules_for(&profile(TaxRegime::Forfettario, false), true)
}

fn ordinario() -> TaxRules {
    rules_for(&profile(TaxRegime::Ordinario, false), true)
}

#[test]
fn forfettario_invoice_with_enpap_and_bollo() {
    let totals = calculate_invoice_totals(&[line(4, 70.0, 0.0)], &forfettario());
    assert_eq!(totals.total_net, 280.0);
    assert_eq!(totals.total_tax, 0.0);
    assert_eq!(totals.contributo_enpap, 5.64);
    assert_eq!(totals.ritenuta_acconto, 0.0);
    assert_eq!(totals.marca_da_bollo, 2.0);
    assert_eq!(totals.total_gross, 285.64);
    assert_eq!(totals.total_due, 287.64);
}

#[test]
fn forfettario_can_leave_bollo_out_of_enpap_base() {
    let rules = rules_for(&profile(TaxRegime::Forfettario, true), true);
    let totals = calculate_invoice_totals(&[line(4, 70.0, 0.0)], &rules);
    assert_eq!(totals.contributo_enpap, 5.60);
    assert_eq!(totals.marca_da_bollo, 2.0);
    assert_eq!(totals.total_due, 287.60);
}

#[test]
fn ordinario_never_puts_bollo_in_enpap_base() {
    assert!(!ordinario().enpap_includes_bollo);
    let opted_out = rules_for(&profile(TaxRegime::Ordinario, true), true);
    assert!(!opted_out.enpap_includes_bollo);
}

#[test]
fn enpap_rate_is_zero_when_not_applied() {
    let rules = rules_for(&profile(TaxRegime::Forfettario, false), false);
    let totals = calculate_invoice_totals(&[line(1, 100.0, 0.0)], &rules);
    assert_eq!(totals.contributo_enpap, 0.0);
    assert_eq!(totals.total_due, 102.0);
}

#[test]
fn no_bollo_below_threshold() {
    let totals = calculate_invoice_totals(&[line(1, 77.47, 0.0)], &forfettario());
    assert_eq!(totals.marca_da_bollo, 0.0);
}

#[test]
fn no_bollo_when_vat_applies() {
    let totals = calculate_invoice_totals(&[line(1, 100.0, 22.0)], &forfettario());
    assert_eq!(totals.total_tax, 22.0);
    assert_eq!(totals.marca_da_bollo, 0.0);
}

#[test]
fn bollo_applies_to_exempt_invoice_in_ordinario_regime() {
    let totals = calculate_invoice_totals(&[line(1, 100.0, 0.0)], &ordinario());
    assert_eq!(totals.marca_da_bollo, 2.0);
}

#[test]
fn ordinario_enpap_on_compenso_and_ritenuta_on_net_plus_enpap() {
    let totals = calculate_invoice_totals(&[line(1, 100.0, 0.0)], &ordinario());
    assert_eq!(totals.contributo_enpap, 2.0);
    assert_eq!(totals.ritenuta_acconto, 20.4);
    assert_eq!(totals.total_due, 83.6);
}

#[test]
fn ritenuta_rate_is_zero_for_forfettario() {
    assert_eq!(ritenuta_rate_for_regime(&TaxRegime::Forfettario), 0.0);
    assert_eq!(
        ritenuta_rate_for_regime(&TaxRegime::Ordinario),
        RITENUTA_ORDINARIO_RATE
    );
}

#[test]
fn empty_lines_produce_zero_totals() {
    let totals = calculate_invoice_totals(&[], &forfettario());
    assert_eq!(totals.total_net, 0.0);
    assert_eq!(totals.total_due, 0.0);
    assert_eq!(totals.marca_da_bollo, 0.0);
}

#[test]
fn per_line_rounding_matches_stored_line_totals() {
    // 3 × 33.335 rounds per line first (100.01), not 100.005 → 100.0
    let rules = rules_for(&profile(TaxRegime::Forfettario, false), false);
    let totals = calculate_invoice_totals(&[line(3, 33.335, 0.0)], &rules);
    assert_eq!(totals.total_net, 100.01);
}

#[test]
fn amount_override_replaces_quantity_times_price() {
    let package = InvoiceLineData {
        amount_override: Some(250.0),
        ..line(4, 70.0, 0.0)
    };
    let totals = calculate_invoice_totals(&[package], &forfettario());
    assert_eq!(totals.total_net, 250.0);
    assert_eq!(totals.contributo_enpap, 5.04);
    assert_eq!(totals.total_due, 257.04);
}

#[test]
fn amount_override_of_zero_is_kept() {
    let free = InvoiceLineData {
        amount_override: Some(0.0),
        ..line(1, 70.0, 0.0)
    };
    assert_eq!(free.net_amount(), 0.0);
}
