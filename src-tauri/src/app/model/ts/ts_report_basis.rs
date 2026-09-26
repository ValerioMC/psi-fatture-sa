use serde::{Deserialize, Serialize};

/// Which date the monthly report selects documents by (`tipoEstrazione`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TsReportBasis {
    Invio,
    Pagamento,
}

impl TsReportBasis {
    pub fn code(&self) -> &'static str {
        match self {
            TsReportBasis::Invio => "I",
            TsReportBasis::Pagamento => "P",
        }
    }
}
