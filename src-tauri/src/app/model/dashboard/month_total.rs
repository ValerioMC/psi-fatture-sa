/// Paid revenue and invoice count of one month, as the database sums them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonthTotal {
    pub month: i64,
    pub revenue: f64,
    pub invoice_count: i64,
}
