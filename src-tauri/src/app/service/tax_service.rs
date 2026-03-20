/// Italian tax calculation logic for PSI Fatture SA.
///
/// Handles forfettario and ordinario regimes, ENPAP, ritenuta d'acconto,
/// marca da bollo, IRPEF, and INPS contributions.

const MARCA_DA_BOLLO_THRESHOLD: f64 = 77.47;
const MARCA_DA_BOLLO_AMOUNT: f64 = 2.00;
const INPS_GESTIONE_SEPARATA_RATE: f64 = 0.2607;

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

    let needs_bollo = tax_regime == "forfettario"
        && total_net > MARCA_DA_BOLLO_THRESHOLD
        && total_tax == 0.0;
    let marca_da_bollo = if needs_bollo { MARCA_DA_BOLLO_AMOUNT } else { 0.0 };

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForfettarioTaxEstimate {
    pub annual_revenue: f64,
    pub taxable_income: f64,
    pub substitute_tax_rate: f64,
    pub inps_contribution: f64,
    pub taxable_base_after_inps: f64,
    pub substitute_tax: f64,
    pub total_tax: f64,
    pub net_income: f64,
}

/// Estimates annual taxes for the forfettario regime.
pub fn calculate_forfettario_tax(
    annual_revenue: f64,
    coefficient: f64,
    first_five_years: bool,
) -> ForfettarioTaxEstimate {
    let taxable_income = round2(annual_revenue * coefficient / 100.0);
    let tax_rate = if first_five_years { 5.0 } else { 15.0 };
    let inps = round2(taxable_income * INPS_GESTIONE_SEPARATA_RATE);
    let taxable_base_after_inps = taxable_income - inps;
    let substitute_tax = round2(taxable_base_after_inps * tax_rate / 100.0);
    let total_tax = inps + substitute_tax;
    let net_income = round2(annual_revenue - total_tax);

    ForfettarioTaxEstimate {
        annual_revenue,
        taxable_income,
        substitute_tax_rate: tax_rate,
        inps_contribution: inps,
        taxable_base_after_inps,
        substitute_tax,
        total_tax,
        net_income,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrdinarioTaxEstimate {
    pub annual_revenue: f64,
    pub estimated_costs: f64,
    pub taxable_income: f64,
    pub inps_contribution: f64,
    pub taxable_base_after_inps: f64,
    pub irpef: f64,
    pub addizionale_regionale: f64,
    pub addizionale_comunale: f64,
    pub total_tax: f64,
    pub net_income: f64,
}

/// Estimates annual taxes for the ordinario regime.
pub fn calculate_ordinario_tax(
    annual_revenue: f64,
    profitability_coefficient: f64,
) -> OrdinarioTaxEstimate {
    let estimated_costs = round2(annual_revenue * (100.0 - profitability_coefficient) / 100.0);
    let taxable_income = round2(annual_revenue * profitability_coefficient / 100.0);
    let inps = round2(taxable_income * INPS_GESTIONE_SEPARATA_RATE);
    let taxable_base_after_inps = taxable_income - inps;
    let irpef = calculate_irpef(taxable_base_after_inps);
    let addizionale_regionale = round2(taxable_base_after_inps * 0.0173);
    let addizionale_comunale = round2(taxable_base_after_inps * 0.008);
    let total_tax = inps + irpef + addizionale_regionale + addizionale_comunale;
    let net_income = round2(annual_revenue - total_tax);

    OrdinarioTaxEstimate {
        annual_revenue,
        estimated_costs,
        taxable_income,
        inps_contribution: inps,
        taxable_base_after_inps,
        irpef,
        addizionale_regionale,
        addizionale_comunale,
        total_tax,
        net_income,
    }
}

/// IRPEF 2024 brackets: progressive tax calculation.
fn calculate_irpef(taxable_income: f64) -> f64 {
    const BRACKETS: &[(f64, f64, f64)] = &[
        (0.0, 28_000.0, 0.23),
        (28_000.0, 50_000.0, 0.35),
        (50_000.0, f64::MAX, 0.43),
    ];

    let mut tax = 0.0;
    let mut remaining = taxable_income;

    for &(lower, upper, rate) in BRACKETS {
        if remaining <= 0.0 {
            break;
        }
        let bracket_size = (upper - lower).min(remaining);
        tax += round2(bracket_size * rate);
        remaining -= bracket_size;
    }

    round2(tax)
}

/// Rounds a value to 2 decimal places (ROUND_HALF_UP equivalent).
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
