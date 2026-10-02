use sea_orm::FromQueryResult;

/// An invoice's number and issue date, as the numbering order sees it.
#[derive(Debug, FromQueryResult)]
pub struct NumberedDate {
    pub invoice_number: String,
    pub issue_date: String,
}
