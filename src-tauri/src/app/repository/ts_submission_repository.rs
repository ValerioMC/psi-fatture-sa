use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, FromQueryResult, Statement, Value};

use crate::app::entity::ts_submission::{self, ActiveModel};
use crate::app::model::ts::{
    TsDocumentId, TsEnvironment, TsOperation, TsSubmission, TsSubmissionFilters, TsSubmissionStatus,
};

const SELECT_SQL: &str = "
    SELECT s.id, s.invoice_id, i.invoice_number, i.year AS invoice_year,
           trim(c.first_name || ' ' || c.last_name) AS client_name,
           s.operation, s.status, s.target_submission_id, s.environment,
           s.document_vat_number, s.document_issue_date, s.document_number, s.protocol,
           s.outcome_code, s.outcome_message, s.attempt_count, s.last_error,
           s.next_attempt_at, s.last_attempt_at, s.sent_at, s.resolved_at,
           s.created_at, s.updated_at
    FROM ts_submissions s
    JOIN invoices i ON s.invoice_id = i.id
    JOIN clients c ON i.client_id = c.id";

#[derive(FromQueryResult)]
struct SubmissionRow {
    id: i64,
    invoice_id: i64,
    invoice_number: String,
    invoice_year: i64,
    client_name: String,
    operation: String,
    status: String,
    target_submission_id: Option<i64>,
    environment: String,
    document_vat_number: Option<String>,
    document_issue_date: Option<String>,
    document_number: Option<String>,
    protocol: Option<String>,
    outcome_code: Option<String>,
    outcome_message: Option<String>,
    attempt_count: i64,
    last_error: Option<String>,
    next_attempt_at: String,
    last_attempt_at: Option<String>,
    sent_at: Option<String>,
    resolved_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl SubmissionRow {
    fn into_submission(self) -> Result<TsSubmission, String> {
        let document = match (
            self.document_vat_number,
            self.document_issue_date,
            self.document_number,
        ) {
            (Some(vat_number), Some(issue_date), Some(number)) => Some(TsDocumentId {
                vat_number,
                issue_date,
                number,
            }),
            _ => None,
        };
        Ok(TsSubmission {
            id: self.id,
            invoice_id: self.invoice_id,
            invoice_number: self.invoice_number,
            invoice_year: self.invoice_year,
            client_name: self.client_name,
            operation: TsOperation::parse(&self.operation)?,
            status: TsSubmissionStatus::parse(&self.status)?,
            target_submission_id: self.target_submission_id,
            environment: TsEnvironment::parse(&self.environment)?,
            document,
            protocol: self.protocol,
            outcome_code: self.outcome_code,
            outcome_message: self.outcome_message,
            attempt_count: self.attempt_count,
            last_error: self.last_error,
            next_attempt_at: self.next_attempt_at,
            last_attempt_at: self.last_attempt_at,
            sent_at: self.sent_at,
            resolved_at: self.resolved_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Returns submissions matching the filters, newest first.
pub async fn find(
    db: &impl ConnectionTrait,
    filters: &TsSubmissionFilters,
) -> Result<Vec<TsSubmission>, String> {
    let mut conditions = vec!["1=1"];
    let mut values: Vec<Value> = Vec::new();

    if let Some(invoice_id) = filters.invoice_id {
        conditions.push("s.invoice_id = ?");
        values.push(invoice_id.into());
    }
    if let Some(year) = filters.year {
        conditions.push("i.year = ?");
        values.push(year.into());
    }
    if let Some(status) = filters.status {
        conditions.push("s.status = ?");
        values.push(status.as_str().into());
    }

    let sql = format!(
        "{SELECT_SQL} WHERE {} ORDER BY s.created_at DESC, s.id DESC",
        conditions.join(" AND ")
    );
    query(db, &sql, values).await
}

/// Returns one submission with its invoice details.
pub async fn load(db: &impl ConnectionTrait, id: i64) -> Result<TsSubmission, String> {
    let sql = format!("{SELECT_SQL} WHERE s.id = ?");
    query(db, &sql, vec![id.into()])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| format!("Trasmissione STS {id} non trovata"))
}

/// Returns queued submissions whose next attempt is due at `now`, oldest first.
pub async fn find_due(db: &impl ConnectionTrait, now: &str) -> Result<Vec<TsSubmission>, String> {
    let sql = format!(
        "{SELECT_SQL} WHERE s.status = ? AND s.next_attempt_at <= ?
         ORDER BY s.next_attempt_at, s.id"
    );
    let values = vec![TsSubmissionStatus::NonInviata.as_str().into(), now.into()];
    query(db, &sql, values).await
}

pub async fn find_row(db: &impl ConnectionTrait, id: i64) -> Result<ts_submission::Model, String> {
    ts_submission::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Trasmissione STS {id} non trovata"))
}

/// Whether the Sistema TS of `environment` holds, or may be about to hold,
/// expense data for the invoice: a call in flight, or an accepted send or replacement.
pub async fn invoice_holds_ts_data(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    environment: TsEnvironment,
) -> Result<bool, String> {
    let sql = "SELECT COUNT(*) AS count FROM ts_submissions
               WHERE invoice_id = ? AND environment = ?
                 AND (status = ? OR (status = ? AND operation <> ?))";
    let values: Vec<Value> = vec![
        invoice_id.into(),
        environment.as_str().into(),
        TsSubmissionStatus::Inviata.as_str().into(),
        TsSubmissionStatus::Accettata.as_str().into(),
        TsOperation::Annullamento.as_str().into(),
    ];
    count(db, sql, values).await.map(|n| n > 0)
}

/// Whether the invoice has a submission in any of the given statuses,
/// optionally only in one environment.
pub async fn invoice_has_status(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    statuses: &[TsSubmissionStatus],
    environment: Option<TsEnvironment>,
) -> Result<bool, String> {
    if statuses.is_empty() {
        return Ok(false);
    }
    let placeholders = vec!["?"; statuses.len()].join(", ");
    let mut sql = format!(
        "SELECT COUNT(*) AS count FROM ts_submissions
         WHERE invoice_id = ? AND status IN ({placeholders})"
    );
    let mut values: Vec<Value> = vec![invoice_id.into()];
    values.extend(statuses.iter().map(|s| Value::from(s.as_str())));
    if let Some(environment) = environment {
        sql.push_str(" AND environment = ?");
        values.push(environment.as_str().into());
    }
    count(db, &sql, values).await.map(|n| n > 0)
}

/// Moves a queued submission to `inviata` only if it is still queued, so two
/// dispatchers never send the same one. Returns whether this caller won.
pub async fn claim(
    db: &impl ConnectionTrait,
    id: i64,
    document: &TsDocumentId,
    now: &str,
) -> Result<bool, String> {
    let sql = "UPDATE ts_submissions
               SET status = ?, document_vat_number = ?, document_issue_date = ?,
                   document_number = ?, attempt_count = attempt_count + 1,
                   last_attempt_at = ?, updated_at = ?
               WHERE id = ? AND status = ?";
    let values: Vec<Value> = vec![
        TsSubmissionStatus::Inviata.as_str().into(),
        document.vat_number.as_str().into(),
        document.issue_date.as_str().into(),
        document.number.as_str().into(),
        now.into(),
        now.into(),
        id.into(),
        TsSubmissionStatus::NonInviata.as_str().into(),
    ];
    let result = db
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            values,
        ))
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.rows_affected() == 1)
}

/// Puts back in the queue every call left in flight by a previous run.
pub async fn requeue_in_flight(db: &impl ConnectionTrait, now: &str) -> Result<u64, String> {
    let sql = "UPDATE ts_submissions SET status = ?, next_attempt_at = ?, updated_at = ?
               WHERE status = ?";
    let values: Vec<Value> = vec![
        TsSubmissionStatus::NonInviata.as_str().into(),
        now.into(),
        now.into(),
        TsSubmissionStatus::Inviata.as_str().into(),
    ];
    let result = db
        .execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            values,
        ))
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.rows_affected())
}

pub async fn insert(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<ts_submission::Model, String> {
    active.insert(db).await.map_err(|e| e.to_string())
}

pub async fn update(
    db: &impl ConnectionTrait,
    active: ActiveModel,
) -> Result<ts_submission::Model, String> {
    active.update(db).await.map_err(|e| e.to_string())
}

pub async fn delete(db: &impl ConnectionTrait, id: i64) -> Result<(), String> {
    ts_submission::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

async fn query(
    db: &impl ConnectionTrait,
    sql: &str,
    values: Vec<Value>,
) -> Result<Vec<TsSubmission>, String> {
    SubmissionRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
        values,
    ))
    .all(db)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .map(SubmissionRow::into_submission)
    .collect()
}

async fn count(db: &impl ConnectionTrait, sql: &str, values: Vec<Value>) -> Result<i64, String> {
    #[derive(FromQueryResult)]
    struct CountRow {
        count: i64,
    }

    let row = CountRow::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        sql,
        values,
    ))
    .one(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(row.map_or(0, |r| r.count))
}
