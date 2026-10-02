use krilla::metadata::Metadata;
use sea_orm::ConnectionTrait;

use crate::app::common::AppError;
use crate::app::model::invoice::{Invoice, InvoicePdf};
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::{client_repository, config_repository};
use crate::app::service::{client_service, config_service};

use super::{invoice_layout, pdf_painter, InvoiceDocument, PdfFonts};

/// The invoice as a PDF, built from what is saved now.
pub async fn render(db: &impl ConnectionTrait, invoice_id: i64) -> Result<InvoicePdf, AppError> {
    let document = load_document(db, invoice_id).await?;
    render_document(&document)
}

pub async fn load_document(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<InvoiceDocument, AppError> {
    let invoice = invoice_repository::load_invoice(db, invoice_id).await?;
    let client = client_repository::find_by_id(db, invoice.client_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Paziente {} non trovato", invoice.client_id)))?;
    let config = config_repository::find(db).await?.ok_or_else(|| {
        AppError::Invalid(
            "Completa il profilo professionale prima di preparare la fattura".to_string(),
        )
    })?;
    Ok(InvoiceDocument {
        invoice,
        client: client_service::into_domain(client),
        config: config_service::into_domain(config),
    })
}

pub fn render_document(document: &InvoiceDocument) -> Result<InvoicePdf, AppError> {
    let fonts = PdfFonts::shared().map_err(AppError::External)?;
    let pages = invoice_layout::layout(document, fonts);
    let bytes =
        pdf_painter::paint(&pages, fonts, metadata(document)).map_err(AppError::External)?;
    Ok(InvoicePdf {
        file_name: file_name(&document.invoice),
        bytes,
    })
}

/// "Fattura_12_2026.pdf": only letters, digits and dashes from the number survive.
pub fn file_name(invoice: &Invoice) -> String {
    let number: String = invoice
        .invoice_number
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("Fattura_{number}_{}.pdf", invoice.year)
}

fn metadata(document: &InvoiceDocument) -> Metadata {
    let config = &document.config;
    let author = format!("{} {}", config.first_name.trim(), config.last_name.trim());
    Metadata::new()
        .title(format!(
            "Fattura N. {}/{}",
            document.invoice.invoice_number, document.invoice.year
        ))
        .authors(vec![author.trim().to_string()])
        .language("it".to_string())
        .creator("PSI Fatture".to_string())
}

#[cfg(test)]
#[path = "invoice_pdf_service_test.rs"]
mod tests;
