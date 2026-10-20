//! The invoice page, section by section as `InvoicePrintView.vue` and its `components/print/`
//! draw it: letterhead and number, recipient and payment, the lines, the totals, then the
//! fiscal notes, the Sistema TS authorisation and the footer. CSS pixels become points at 0.75.

use super::boxes::{legal_notes_box, notes_box, sts_box};
use super::footer::bottom_bar;
use super::letterhead::header;
use super::lines_table::lines_table;
use super::parties::parties;
use super::totals::totals;
use crate::app::service::pdf::page_flow::{MARGIN_BOTTOM, PAGE_HEIGHT};
use crate::app::service::pdf::{invoice_legal_notes, DrawOp, InvoiceDocument, PageFlow, PdfFonts};

/// Lays the body out around a two-line footer; only when that takes more than one page
/// does it lay out again around the three-line footer that carries the page numbers.
pub fn layout(document: &InvoiceDocument, fonts: &PdfFonts) -> Vec<Vec<DrawOp>> {
    let mut pages = body(document, fonts, bottom_bar(document, fonts, None).height);
    if pages.len() > 1 {
        pages = body(
            document,
            fonts,
            bottom_bar(document, fonts, Some("Pagina")).height,
        );
    }
    let count = pages.len();
    for (index, page) in pages.iter_mut().enumerate() {
        let numbering = (count > 1).then(|| format!("Pagina {} di {count}", index + 1));
        let footer = bottom_bar(document, fonts, numbering.as_deref());
        page.extend(footer.shifted(PAGE_HEIGHT - MARGIN_BOTTOM - footer.height));
    }
    pages
}

fn body(document: &InvoiceDocument, fonts: &PdfFonts, footer_height: f32) -> Vec<Vec<DrawOp>> {
    let mut flow = PageFlow::new(footer_height + 8.0);
    flow.place(&header(document, fonts));
    flow.gap(14.0);
    flow.place_kept(&parties(document, fonts));
    flow.gap(12.0);
    lines_table(&mut flow, document, fonts);
    flow.gap(9.0);
    flow.place_kept(&totals(document, fonts));
    flow.gap(13.5);
    let notes = invoice_legal_notes::legal_notes(document);
    if !notes.is_empty() {
        flow.place_kept(&legal_notes_box(&notes, fonts));
        flow.gap(7.5);
    }
    flow.place_kept(&sts_box(document, fonts));
    flow.gap(7.5);
    if !document.invoice.notes.trim().is_empty() {
        flow.place_kept(&notes_box(&document.invoice.notes, fonts));
        flow.gap(7.5);
    }
    flow.finish()
}
