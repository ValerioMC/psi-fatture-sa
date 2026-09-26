use serde::Serialize;

/// `esitoChiamata` of a document call: 0 accepted, 2 accepted with warnings, 1 rejected.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TsEsito {
    Accepted,
    AcceptedWithWarnings,
    Rejected,
}

impl TsEsito {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim() {
            "0" => Ok(TsEsito::Accepted),
            "2" => Ok(TsEsito::AcceptedWithWarnings),
            "1" => Ok(TsEsito::Rejected),
            other => Err(format!("esitoChiamata sconosciuto: {other}")),
        }
    }
}

#[cfg(test)]
#[path = "ts_esito_test.rs"]
mod tests;
