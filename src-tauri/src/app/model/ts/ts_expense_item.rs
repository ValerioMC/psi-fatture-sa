use super::TsVatTreatment;

#[derive(Debug, Clone, PartialEq)]
pub struct TsExpenseItem {
    pub amount: f64,
    pub vat: TsVatTreatment,
}
