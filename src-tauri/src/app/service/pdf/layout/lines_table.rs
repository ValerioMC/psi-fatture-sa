//! The lines table. It breaks across pages row by row, repeating its head, and each
//! page's part gets its own rounded frame.

use super::columns::{CELL_PAD, EDGE_PAD};
use super::style::{
    section_label, ACCENT, ACCENT_STRONG, HAIRLINE, INK, LEFT, MUTED, PAPER_TINT, RIGHT, ROW_RULE,
    RULE,
};
use super::Columns;
use crate::app::model::invoice::InvoiceLine;
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::page_flow::CONTENT_WIDTH;
use crate::app::service::pdf::{
    Area, Block, DrawOp, Face, InvoiceDocument, PageFlow, PdfFonts, TextStyle,
};
use crate::app::service::support::italian_format::currency;

const DESCRIPTION: TextStyle = TextStyle::new(Face::Medium, 8.5, INK);
const SECONDARY: TextStyle = TextStyle::new(Face::Regular, 8.0, MUTED);
const ROW_LEADING: f32 = 8.5 * 1.45;
const ROW_PAD_Y: f32 = 6.75;

/// The label, head and first row stay together; later rows start a new page when they do not fit.
pub(super) fn lines_table(flow: &mut PageFlow, document: &InvoiceDocument, fonts: &PdfFonts) {
    let columns = Columns::new(!document.invoice.hide_quantity);
    let label = table_label(fonts);
    let head = table_head(&columns, fonts);
    let rows: Vec<Block> = document
        .invoice
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| table_row(&columns, line, index, fonts))
        .collect();

    let first_row = rows.first().map_or(0.0, |row| row.height);
    flow.keep(label.height + head.height + first_row);
    flow.place(&label);
    let mut segment = open_segment(flow, &head);
    for row in &rows {
        if !flow.fits(row.height) {
            close_segment(flow, segment);
            flow.break_page();
            segment = open_segment(flow, &head);
        }
        flow.place(row);
    }
    close_segment(flow, segment);
}

fn table_label(fonts: &PdfFonts) -> Block<'_> {
    let mut label = Block::new(fonts);
    label.text(
        LEFT,
        baseline(0.0, 6.5, 9.0),
        section_label(ACCENT),
        "DETTAGLIO PRESTAZIONI",
    );
    label.height = 9.0 + 5.25;
    label
}

/// Where a page's part of the table starts: its top and the op index to clip from.
fn open_segment(flow: &mut PageFlow, head: &Block) -> (f32, usize) {
    let segment = (flow.y, flow.mark());
    flow.place(head);
    segment
}

fn close_segment(flow: &mut PageFlow, (top, mark): (f32, usize)) {
    let frame = Area::new(LEFT, top, CONTENT_WIDTH, flow.y - top).rounded(5.25);
    flow.insert(mark, DrawOp::PushClip(frame));
    flow.push(DrawOp::PopClip);
    flow.push(DrawOp::Box {
        area: frame,
        fill: None,
        stroke: Some((RULE, HAIRLINE)),
    });
}

fn table_head<'f>(columns: &Columns, fonts: &'f PdfFonts) -> Block<'f> {
    let mut block = Block::new(fonts);
    let height = 6.5 + 2.0 * 6.75 + 2.0;
    block.rect(
        Area::new(LEFT, 0.0, CONTENT_WIDTH, height),
        Some(ROW_RULE),
        None,
    );
    block.line((LEFT, height - 0.6), (RIGHT, height - 0.6), RULE, 1.1);
    let style = TextStyle::new(Face::Bold, 6.5, ACCENT_STRONG).tracked(0.75);
    let y = baseline(0.0, 6.5, height);
    block.text(columns.description.0 + EDGE_PAD, y, style, "DESCRIZIONE");
    if let Some((start, end)) = columns.quantity {
        block.text_center((start + end) / 2.0, y, style, "QTÀ");
    }
    if let Some((_, end)) = columns.unit_price {
        block.text_right(end - CELL_PAD, y, style, "PREZZO UNIT.");
    }
    block.text_center((columns.vat.0 + columns.vat.1) / 2.0, y, style, "IVA");
    block.text_right(columns.amount.1 - EDGE_PAD, y, style, "IMPORTO");
    block.height = height;
    block
}

/// Odd rows are tinted; the description wraps and sets the row's height.
fn table_row<'f>(
    columns: &Columns,
    line: &InvoiceLine,
    index: usize,
    fonts: &'f PdfFonts,
) -> Block<'f> {
    let mut block = Block::new(fonts);
    let description_width = columns.description.1 - columns.description.0 - EDGE_PAD - CELL_PAD;
    let lines = block.wrap(&DESCRIPTION, &line.description, description_width);
    let height = ROW_LEADING * lines.len() as f32 + 2.0 * ROW_PAD_Y;

    if index % 2 == 1 {
        block.rect(
            Area::new(LEFT, 0.0, CONTENT_WIDTH, height),
            Some(PAPER_TINT),
            None,
        );
    }
    block.line((LEFT, height), (RIGHT, height), ROW_RULE, HAIRLINE);
    block.paragraph(
        columns.description.0 + EDGE_PAD,
        ROW_PAD_Y,
        description_width,
        DESCRIPTION,
        ROW_LEADING,
        &line.description,
    );
    row_figures(&mut block, columns, line);
    block.height = height;
    block
}

/// Quantity, unit price, VAT and amount, aligned with the description's first line.
fn row_figures(block: &mut Block, columns: &Columns, line: &InvoiceLine) {
    let first = baseline(ROW_PAD_Y, 8.5, ROW_LEADING);
    if let Some((start, end)) = columns.quantity {
        block.text_center(
            (start + end) / 2.0,
            first,
            SECONDARY,
            &line.quantity.to_string(),
        );
    }
    // A hand-typed amount is not quantity × unit price, so no unit price is printed beside it.
    if let (Some((_, end)), None) = (columns.unit_price, line.amount_override) {
        block.text_right(end - CELL_PAD, first, SECONDARY, &currency(line.unit_price));
    }
    vat_cell(block, columns.vat, first, line.vat_rate);
    block.text_right(
        columns.amount.1 - EDGE_PAD,
        first,
        TextStyle::new(Face::Bold, 8.5, INK),
        &currency(line.line_total),
    );
}

fn vat_cell(block: &mut Block, (start, end): (f32, f32), y: f32, rate: f64) {
    let center = (start + end) / 2.0;
    if rate > 0.0 {
        let text = format!("{}%", format_rate(rate));
        block.text_center(center, y, SECONDARY, &text);
        return;
    }
    let pill = TextStyle::new(Face::SemiBold, 7.0, ACCENT);
    let width = block.width(&pill, "Esente") + 7.5;
    block.rect(
        Area::new(center - width / 2.0, y - 7.0, width, 9.5).rounded(2.25),
        Some(ROW_RULE),
        None,
    );
    block.text_center(center, y, pill, "Esente");
}

fn format_rate(rate: f64) -> String {
    if rate.fract() == 0.0 {
        format!("{rate:.0}")
    } else {
        format!("{rate}").replace('.', ",")
    }
}
