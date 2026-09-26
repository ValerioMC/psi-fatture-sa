use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TsExpenseTotal {
    pub expense_type: String,
    pub amount: f64,
}
