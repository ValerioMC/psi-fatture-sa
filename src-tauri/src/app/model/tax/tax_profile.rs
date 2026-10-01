use crate::app::model::config::TaxRegime;

/// The professional's settings that decide how an invoice is taxed.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxProfile {
    pub tax_regime: TaxRegime,
    pub enpap_excludes_bollo: bool,
}
