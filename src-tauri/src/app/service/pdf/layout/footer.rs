//! The footer of every page, with "Pagina 1 di 2" under it when there is more than one.

use super::letterhead::professional_name;
use super::style::{ACCENT_STRONG, HAIRLINE, LEFT, RIGHT, RULE, SUBTLE};
use crate::app::model::config::ProfessionalConfig;
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::page_flow::CONTENT_WIDTH;
use crate::app::service::pdf::{Block, Face, InvoiceDocument, PdfFonts, Rgb, TextStyle};

const TEXT: TextStyle = TextStyle::new(Face::Regular, 7.0, SUBTLE);
const LEADING: f32 = 7.0 * 1.7;
const CENTER: f32 = LEFT + CONTENT_WIDTH / 2.0;

pub(super) fn bottom_bar<'f>(
    document: &InvoiceDocument,
    fonts: &'f PdfFonts,
    numbering: Option<&str>,
) -> Block<'f> {
    let mut block = Block::new(fonts);
    block.line((LEFT, 0.0), (RIGHT, 0.0), RULE, HAIRLINE);
    let first = baseline(9.0, 7.0, LEADING);
    identity_line(&mut block, document, first);
    block.text_center(CENTER, first + LEADING, TEXT, &address(&document.config));
    block.height = 9.0 + 2.0 * LEADING;
    if let Some(numbering) = numbering {
        block.text_center(CENTER, first + 2.0 * LEADING, TEXT, numbering);
        block.height += LEADING;
    }
    block
}

/// Name, VAT number, fiscal code and PEC on one centred line, separated by dots.
fn identity_line(block: &mut Block, document: &InvoiceDocument, baseline_y: f32) {
    let config = &document.config;
    let dot = TextStyle::new(Face::Regular, 7.0, Rgb::hex(0xd4cfc4));
    let name = professional_name(document);
    let vat = format!("P.IVA {}", config.vat_number);
    let fiscal = format!("C.F. {}", config.fiscal_code);
    let pec = format!("PEC: {}", config.pec_email.trim());
    let mut runs: Vec<(TextStyle, &str)> = vec![
        (TextStyle::new(Face::SemiBold, 7.0, ACCENT_STRONG), &name),
        (dot, "  ·  "),
        (TEXT, &vat),
        (dot, "  ·  "),
        (TEXT, &fiscal),
    ];
    if !config.pec_email.trim().is_empty() {
        runs.extend([(dot, "  ·  "), (TEXT, pec.as_str())]);
    }
    let width = block.runs_width(&runs);
    block.runs(LEFT + (CONTENT_WIDTH - width) / 2.0, baseline_y, &runs);
}

fn address(config: &ProfessionalConfig) -> String {
    if config.address.trim().is_empty() {
        format!("{} {} ({})", config.zip_code, config.city, config.province)
    } else {
        format!(
            "{}, {} {} ({})",
            config.address.trim(),
            config.zip_code,
            config.city,
            config.province
        )
    }
}
