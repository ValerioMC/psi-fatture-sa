//! The page header: the professional's letterhead on the left, the invoice's identity on the right.

use super::identity::invoice_identity;
use super::style::{ACCENT, ACCENT_STRONG, DETAIL, INK, LEFT};
use crate::app::model::config::{Profession, ProfessionalConfig};
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::page_flow::CONTENT_WIDTH;
use crate::app::service::pdf::{Block, Face, InvoiceDocument, PdfFonts, TextStyle};

const STRONG: TextStyle = TextStyle::new(Face::SemiBold, 7.5, ACCENT_STRONG);

pub(super) fn header<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let mut block = Block::new(fonts);
    let identity_bottom = invoice_identity(&mut block, document);
    let column_width = CONTENT_WIDTH - 190.0;
    let letterhead_bottom = letterhead(&mut block, document, column_width);
    block.height = identity_bottom.max(letterhead_bottom);
    block
}

pub(super) fn professional_name(document: &InvoiceDocument) -> String {
    let config = &document.config;
    [&config.title, &config.first_name, &config.last_name]
        .iter()
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The left column; returns its bottom.
fn letterhead(block: &mut Block, document: &InvoiceDocument, width: f32) -> f32 {
    let config = &document.config;
    let mut y = name_and_profession(block, document, width);
    y = memberships(block, config, y);
    y += 6.0 - 1.5;
    let leading = 7.5 * 1.7;
    for line in &contact_lines(config) {
        block.text(LEFT, baseline(y, 7.5, leading), DETAIL, line);
        y += leading;
    }
    y
}

/// The name, wrapped to the column, and the profession in capitals under it; returns the bottom.
fn name_and_profession(block: &mut Block, document: &InvoiceDocument, width: f32) -> f32 {
    let name_style = TextStyle::new(Face::Display, 19.0, INK).tracked(-0.35);
    let name_leading = 19.0 * 1.15;
    let name = professional_name(document);
    let mut y = block.paragraph(LEFT, 0.0, width, name_style, name_leading, &name);
    y += 3.0;

    let profession = TextStyle::new(Face::SemiBold, 7.5, ACCENT).tracked(1.05);
    let title = profession_line(document).to_uppercase();
    block.text(LEFT, baseline(y, 7.5, 11.0), profession, &title);
    y += 11.0 + 3.75;
    y
}

fn profession_line(document: &InvoiceDocument) -> String {
    let config = &document.config;
    let base = match config.profession {
        Profession::Psicoterapeuta => "Psicoterapeuta",
        Profession::Psicologo => "Psicologo",
    };
    if config.is_psicoanalista {
        format!("{base} — Psicoanalista")
    } else if !config.specialization.trim().is_empty() {
        format!("{base} — {}", config.specialization.trim())
    } else {
        base.to_string()
    }
}

/// The Albo registration and the IPA membership, each on its own line when present.
fn memberships(block: &mut Block, config: &ProfessionalConfig, mut y: f32) -> f32 {
    let leading = 7.5 * 1.45;
    let albo = albo_runs(config);
    if !albo.is_empty() {
        block.runs(LEFT, baseline(y, 7.5, leading), &albo);
        y += leading + 1.5;
    }
    if config.is_psicoanalista {
        block.runs(
            LEFT,
            baseline(y, 7.5, leading),
            &[
                (DETAIL, "Membro della "),
                (STRONG, "International Psychoanalytical Association (IPA)"),
            ],
        );
        y += leading + 1.5;
    }
    y
}

/// "Iscriz. Albo n. 1234 — Regione Lombardia", either half left out when blank.
fn albo_runs(config: &ProfessionalConfig) -> Vec<(TextStyle, &str)> {
    let number = config.albo_number.trim();
    let region = config.albo_region.trim();
    let mut runs: Vec<(TextStyle, &str)> = Vec::new();
    if !number.is_empty() {
        runs.extend([(DETAIL, "Iscriz. Albo n. "), (STRONG, number)]);
    }
    if !number.is_empty() && !region.is_empty() {
        runs.push((DETAIL, " — "));
    }
    if !region.is_empty() {
        runs.extend([(DETAIL, "Regione "), (STRONG, region)]);
    }
    runs
}

fn contact_lines(config: &ProfessionalConfig) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if !config.address.trim().is_empty() {
        lines.push(config.address.trim().to_string());
    }
    lines.push(format!(
        "{} {} ({})",
        config.zip_code, config.city, config.province
    ));
    lines.push(format!(
        "P.IVA {} · C.F. {}",
        config.vat_number, config.fiscal_code
    ));
    let contacts: Vec<String> = [
        (!config.pec_email.trim().is_empty()).then(|| format!("PEC: {}", config.pec_email.trim())),
        (!config.phone.trim().is_empty()).then(|| format!("Tel: {}", config.phone.trim())),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !contacts.is_empty() {
        lines.push(contacts.join(" · "));
    }
    lines
}
