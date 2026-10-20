use std::path::{Path, PathBuf};

use krilla::metadata::Metadata;
use sea_orm::ConnectionTrait;

use crate::app::common::AppError;
use crate::app::model::invoice::{Invoice, InvoicePdf};
use crate::app::repository::invoice::invoice_repository;
use crate::app::repository::pdf::document_file_repository;
use crate::app::repository::{client_repository, config_repository};
use crate::app::service::{client_service, config_service};

use super::{layout, pdf_painter, InvoiceDocument, PdfFonts};

/// The invoice as a PDF, built from what is saved now.
pub async fn render(db: &impl ConnectionTrait, invoice_id: i64) -> Result<InvoicePdf, AppError> {
    let document = load_document(db, invoice_id).await?;
    render_document(&document)
}

/// Writes the PDF to the app's temporary folder, replacing an earlier preview, and returns where.
pub async fn write_preview(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<PathBuf, AppError> {
    let pdf = render(db, invoice_id).await?;
    document_file_repository::write_preview(&pdf.file_name, &pdf.bytes)
}

pub async fn save(db: &impl ConnectionTrait, invoice_id: i64, path: &Path) -> Result<(), AppError> {
    let pdf = render(db, invoice_id).await?;
    document_file_repository::write(path, &pdf.bytes)
}

/// Loads the whole document, so an incomplete profile is reported before the save dialog opens.
pub async fn file_name_of(db: &impl ConnectionTrait, invoice_id: i64) -> Result<String, AppError> {
    let document = load_document(db, invoice_id).await?;
    Ok(file_name(&document.invoice))
}

pub async fn load_document(
    db: &impl ConnectionTrait,
    invoice_id: i64,
) -> Result<InvoiceDocument, AppError> {
    let invoice = invoice_repository::require(db, invoice_id).await?;
    let client = client_repository::require(db, invoice.client_id).await?;
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
    let pages = layout::layout(document, fonts);
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
