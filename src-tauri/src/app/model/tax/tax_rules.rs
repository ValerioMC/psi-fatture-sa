/// The rates and bases one invoice's totals are computed with.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxRules {
    /// Percent; zero when ENPAP is not charged.
    pub enpap_rate: f64,
    /// Percent; zero outside the ordinario regime.
    pub ritenuta_rate: f64,
    /// Whether the marca da bollo charged to the client concurs to the ENPAP base.
    pub enpap_includes_bollo: bool,
}
