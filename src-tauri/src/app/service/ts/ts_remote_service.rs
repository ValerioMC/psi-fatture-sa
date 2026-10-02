//! Reads what the Sistema TS actually holds: one invoice through the point
//! query, a month through the report, and a harmless call to check the
//! credentials. Nothing here changes the local queue.

use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::app::common::AppError;
use crate::app::model::ts::{
    TsConnectionCheck, TsDocumentId, TsOperation, TsQueryResult, TsReportBasis, TsReportRow,
    TsSettings, TsSubmissionFilters, TsSubmissionStatus,
};
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::secret::SecretStore;
use crate::app::repository::ts::sistema_ts::{ReportOutcome, SistemaTsGateway, TsGatewayError};
use crate::app::repository::ts::ts_submission_repository;
use crate::app::service::ts::{ts_credential_service, ts_settings_service};
use crate::app::service::validation_service as validate;

/// Number used by the connection check; no real invoice carries it.
const PROBE_DOCUMENT_NUMBER: &str = "PSIFATTURE-CHECK";

/// Looks the invoice up on the Sistema TS, under the id it was last
/// accepted with, or its own id if it was never sent.
pub async fn query_invoice(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
    invoice_id: i64,
) -> Result<TsQueryResult, AppError> {
    validate::validate_id(invoice_id, "Fattura")?;
    let settings = ts_settings_service::get(db).await?;
    let session = ts_credential_service::session(store, &settings)?;
    let id = match last_sent_id(db, invoice_id, &settings).await? {
        Some(id) => id,
        None => own_id(db, invoice_id, &settings).await?,
    };
    gateway.query(&session, &id).await.map_err(AppError::from)
}

/// The documents the Sistema TS holds for a month, by send or payment date,
/// each matched to the local invoice it came from when there is one.
pub async fn monthly_report(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
    year: i32,
    month: u32,
    basis: TsReportBasis,
) -> Result<Vec<TsReportRow>, AppError> {
    validate::validate_year(i64::from(year))?;
    validate::validate_month(i64::from(month))?;
    let settings = ts_settings_service::get(db).await?;
    let session = ts_credential_service::session(store, &settings)?;

    let outcome = gateway.monthly_report(&session, year, month, basis).await?;
    let mut rows = match outcome {
        ReportOutcome::Rows(rows) => rows,
        ReportOutcome::Refused(messages) => {
            let reasons: Vec<String> = messages
                .iter()
                .filter(|m| m.is_error())
                .map(|m| format!("{} {}", m.code, m.description))
                .collect();
            return Err(AppError::Invalid(format!(
                "Report non disponibile: {}",
                reasons.join(" · ")
            )));
        }
    };
    for row in &mut rows {
        row.invoice_id =
            invoice_repository::find_id_by_issue(db, &row.issue_date, &row.document_number).await?;
    }
    Ok(rows)
}

/// Checks username, password and PINCODE with a query for a document that
/// does not exist: "not found" means every credential was accepted.
pub async fn check_connection(
    db: &DatabaseConnection,
    store: &dyn SecretStore,
    gateway: &dyn SistemaTsGateway,
) -> Result<TsConnectionCheck, AppError> {
    let settings = ts_settings_service::get(db).await?;
    let session = match ts_credential_service::session(store, &settings) {
        Ok(session) => session,
        Err(reason) => return Ok(failed(reason.to_string())),
    };
    let probe = TsDocumentId {
        vat_number: settings.vat_number.clone(),
        issue_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        number: PROBE_DOCUMENT_NUMBER.to_string(),
    };
    Ok(match gateway.query(&session, &probe).await {
        Ok(TsQueryResult::NotFound | TsQueryResult::Found { .. }) => TsConnectionCheck {
            ok: true,
            message: format!(
                "Credenziali accettate dal Sistema TS ({})",
                settings.environment.as_str()
            ),
        },
        Ok(TsQueryResult::Refused { messages }) => failed(
            messages
                .iter()
                .map(|m| format!("{} {}", m.code, m.description))
                .collect::<Vec<_>>()
                .join(" · "),
        ),
        Err(TsGatewayError::Authentication) => {
            failed("Utente o password del Sistema TS non validi".to_string())
        }
        Err(TsGatewayError::Network(detail)) => failed(format!(
            "Nessuna risposta dal Sistema TS ({detail}). Controlla la connessione: con una password errata il servizio a volte non risponde affatto"
        )),
        Err(error) => failed(error.to_string()),
    })
}

async fn last_sent_id(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    settings: &TsSettings,
) -> Result<Option<TsDocumentId>, AppError> {
    let filters = TsSubmissionFilters {
        invoice_id: Some(invoice_id),
        ..Default::default()
    };
    Ok(ts_submission_repository::find(db, &filters)
        .await?
        .into_iter()
        .filter(|s| s.environment == settings.environment)
        .find(|s| {
            s.operation != TsOperation::Annullamento
                && matches!(
                    s.status,
                    TsSubmissionStatus::Accettata
                        | TsSubmissionStatus::Sostituita
                        | TsSubmissionStatus::Annullata
                )
        })
        .and_then(|s| s.document))
}

async fn own_id(
    db: &impl ConnectionTrait,
    invoice_id: i64,
    settings: &TsSettings,
) -> Result<TsDocumentId, AppError> {
    let invoice = invoice_repository::load_invoice(db, invoice_id).await?;
    Ok(TsDocumentId {
        vat_number: settings.vat_number.clone(),
        issue_date: invoice.issue_date,
        number: invoice.invoice_number,
    })
}

fn failed(message: String) -> TsConnectionCheck {
    TsConnectionCheck { ok: false, message }
}

#[cfg(test)]
#[path = "ts_remote_service_test.rs"]
mod tests;
