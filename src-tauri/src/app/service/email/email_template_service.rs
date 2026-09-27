use sea_orm::{ActiveValue::Set, ConnectionTrait};

use super::email_template_renderer;
use super::EmailValues;
use crate::app::entity::email_template::ActiveModel;
use crate::app::model::email::{
    EmailPlaceholder, EmailPlaceholderInfo, EmailPreview, EmailTemplate,
};
use crate::app::repository::config_repository;
use crate::app::repository::email::email_template_repository;
use crate::app::service::italian_format;

const SUBJECT_MAX_LENGTH: usize = 200;
const BODY_MAX_LENGTH: usize = 10_000;

pub const DEFAULT_SUBJECT: &str = "Fattura n. {numero_fattura} – {professionista}";
pub const DEFAULT_BODY: &str = "Gentile {paziente},

in allegato trova la fattura n. {numero_fattura} del {data_fattura}, per un importo di {importo}.

Resto a disposizione per qualsiasi chiarimento.

Cordiali saluti,
{professionista}";

pub fn default_template() -> EmailTemplate {
    EmailTemplate {
        subject: DEFAULT_SUBJECT.to_string(),
        body: DEFAULT_BODY.to_string(),
    }
}

pub fn placeholders() -> Vec<EmailPlaceholderInfo> {
    EmailPlaceholder::ALL
        .iter()
        .map(EmailPlaceholder::describe)
        .collect()
}

/// The saved template, or the built-in one while none is saved.
pub async fn get(db: &impl ConnectionTrait) -> Result<EmailTemplate, String> {
    Ok(email_template_repository::find(db)
        .await?
        .map(|row| EmailTemplate {
            subject: row.subject,
            body: row.body,
        })
        .unwrap_or_else(default_template))
}

pub async fn update(
    db: &impl ConnectionTrait,
    template: EmailTemplate,
) -> Result<EmailTemplate, String> {
    let template = EmailTemplate {
        subject: template.subject.trim().to_string(),
        body: template.body.trim().to_string(),
    };
    validate_template(&template)?;
    let active = ActiveModel {
        id: Set(1),
        subject: Set(template.subject.clone()),
        body: Set(template.body.clone()),
        updated_at: Set(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    };
    email_template_repository::save(db, active).await?;
    Ok(template)
}

/// Back to the built-in template.
pub async fn reset(db: &impl ConnectionTrait) -> Result<EmailTemplate, String> {
    email_template_repository::delete(db).await?;
    Ok(default_template())
}

/// The template filled with an example patient and the professional's own name.
pub async fn preview(
    db: &impl ConnectionTrait,
    template: EmailTemplate,
) -> Result<EmailPreview, String> {
    let profile = config_repository::find(db)
        .await
        .map_err(|e| e.to_string())?;
    let today = chrono::Local::now().date_naive();
    let due = today + chrono::Duration::days(30);
    let professional = profile
        .map(|p| format!("{} {} {}", p.title, p.first_name, p.last_name))
        .map(|name| name.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Dott.ssa Maria Rossi".to_string());
    let values = EmailValues {
        patient: "Anna Bianchi".to_string(),
        patient_first_name: "Anna".to_string(),
        invoice_number: format!("12/{}", today.format("%Y")),
        invoice_date: italian_format::date_long(&today.format("%Y-%m-%d").to_string()),
        amount: italian_format::currency(81.6),
        due_date: italian_format::date_long(&due.format("%Y-%m-%d").to_string()),
        professional,
    };
    Ok(fill(&template, &values))
}

pub fn fill(template: &EmailTemplate, values: &EmailValues) -> EmailPreview {
    EmailPreview {
        subject: email_template_renderer::render(&template.subject, |p| values.get(p)),
        body: email_template_renderer::render(&template.body, |p| values.get(p)),
    }
}

fn validate_template(template: &EmailTemplate) -> Result<(), String> {
    if template.subject.is_empty() {
        return Err("L'oggetto dell'email non può essere vuoto".to_string());
    }
    if template.subject.contains('\n') {
        return Err("L'oggetto dell'email sta su una sola riga".to_string());
    }
    if template.subject.chars().count() > SUBJECT_MAX_LENGTH {
        return Err(format!("Oggetto: massimo {SUBJECT_MAX_LENGTH} caratteri"));
    }
    if template.body.is_empty() {
        return Err("Il testo dell'email non può essere vuoto".to_string());
    }
    if template.body.chars().count() > BODY_MAX_LENGTH {
        return Err(format!("Testo: massimo {BODY_MAX_LENGTH} caratteri"));
    }
    let mut unknown = email_template_renderer::unknown_placeholders(&template.subject);
    for name in email_template_renderer::unknown_placeholders(&template.body) {
        if !unknown.contains(&name) {
            unknown.push(name);
        }
    }
    if !unknown.is_empty() {
        let names: Vec<String> = unknown.iter().map(|name| format!("{{{name}}}")).collect();
        return Err(format!("Segnaposto sconosciuti: {}", names.join(", ")));
    }
    Ok(())
}

#[cfg(test)]
#[path = "email_template_service_test.rs"]
mod tests;
