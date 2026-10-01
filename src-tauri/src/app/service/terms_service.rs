use sea_orm::{ActiveValue::Set, ConnectionTrait};

use crate::app::entity::terms_acceptance::{self, ActiveModel};
use crate::app::model::terms::{AcceptTermsInput, TermsAcceptance};
use crate::app::repository::terms_acceptance_repository;

const VERSION_MAX_LENGTH: usize = 40;

/// The acceptance of that version of the terms, or None while it is still to be accepted.
pub async fn find(
    db: &impl ConnectionTrait,
    version: &str,
) -> Result<Option<TermsAcceptance>, String> {
    let version = validate_version(version)?;
    Ok(terms_acceptance_repository::find_by_version(db, version)
        .await?
        .map(into_domain))
}

/// Records the acceptance only with both boxes ticked; accepting a version twice keeps the first record.
pub async fn accept(
    db: &impl ConnectionTrait,
    input: AcceptTermsInput,
) -> Result<TermsAcceptance, String> {
    let version = validate_version(&input.version)?;
    if !input.terms_accepted {
        return Err("Per usare l'app accetta le condizioni d'uso".to_string());
    }
    if !input.clauses_approved {
        return Err(
            "Per usare l'app approva le clausole indicate (artt. 1341 e 1342 c.c.)".to_string(),
        );
    }
    if let Some(existing) = terms_acceptance_repository::find_by_version(db, version).await? {
        return Ok(into_domain(existing));
    }
    let active = ActiveModel {
        version: Set(version.to_string()),
        clauses_approved: Set(true),
        accepted_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        ..Default::default()
    };
    Ok(into_domain(
        terms_acceptance_repository::insert(db, active).await?,
    ))
}

fn validate_version(version: &str) -> Result<&str, String> {
    let version = version.trim();
    if version.is_empty() || version.chars().count() > VERSION_MAX_LENGTH {
        return Err("Versione delle condizioni d'uso non valida".to_string());
    }
    Ok(version)
}

fn into_domain(row: terms_acceptance::Model) -> TermsAcceptance {
    TermsAcceptance {
        version: row.version,
        clauses_approved: row.clauses_approved,
        accepted_at: row.accepted_at,
    }
}

#[cfg(test)]
#[path = "terms_service_test.rs"]
mod tests;
