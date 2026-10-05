//! The totals box at the right: one row per amount that applies, then the total due.

use super::style::{
    ACCENT, ACCENT_STRONG, ACCENT_WASH, DEDUCTION, HAIRLINE, INK, MUTED, RIGHT, ROW_RULE, RULE,
};
use crate::app::model::invoice::Invoice;
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::{
    Area, Block, DrawOp, Face, InvoiceDocument, PdfFonts, Rgb, TextStyle,
};
use crate::app::service::support::italian_format::currency;

const WIDTH: f32 = 221.25;
const BOX_LEFT: f32 = RIGHT - WIDTH;
const ROW_HEIGHT: f32 = 8.5 * 1.55 + 2.0 * 5.25;
const GRAND_HEIGHT: f32 = 15.0 + 2.0 * 9.0;

pub(super) fn totals<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let rows = total_rows(&document.invoice);
    let mut block = Block::new(fonts);
    let height = ROW_HEIGHT * rows.len() as f32 + 1.5 + GRAND_HEIGHT;
    let frame = Area::new(BOX_LEFT, 0.0, WIDTH, height).rounded(5.25);
    block.push(DrawOp::PushClip(frame));
    let grand_top = amount_rows(&mut block, &rows);
    total_due(&mut block, grand_top, document.invoice.total_due);
    block.push(DrawOp::PopClip);
    block.rect(frame, None, Some((RULE, HAIRLINE)));
    block.height = height;
    block
}

/// Label, amount and colour of each row; a deduction is red and printed with a minus.
fn total_rows(invoice: &Invoice) -> Vec<(String, String, Rgb)> {
    let mut rows: Vec<(String, String, Rgb)> =
        vec![("Imponibile".into(), currency(invoice.total_net), INK)];
    if invoice.total_tax > 0.0 {
        rows.push(("IVA".into(), currency(invoice.total_tax), INK));
    }
    if invoice.marca_da_bollo {
        rows.push(("Marca da bollo".into(), currency(2.0), INK));
    }
    if invoice.apply_enpap && invoice.contributo_enpap > 0.0 {
        rows.push((
            "Contributo ENPAP 2%".into(),
            currency(invoice.contributo_enpap),
            INK,
        ));
    }
    if invoice.ritenuta_acconto > 0.0 {
        rows.push((
            "Ritenuta d'acconto 20%".into(),
            format!("− {}", currency(invoice.ritenuta_acconto)),
            DEDUCTION,
        ));
    }
    rows
}

/// Draws the rows and the rule under them; returns where the total due starts.
fn amount_rows(block: &mut Block, rows: &[(String, String, Rgb)]) -> f32 {
    let mut y = 0.0;
    for (label, value, color) in rows {
        let row_baseline = baseline(y, 8.5, ROW_HEIGHT);
        let label_color = if *color == INK { MUTED } else { *color };
        block.text(
            BOX_LEFT + 12.0,
            row_baseline,
            TextStyle::new(Face::Regular, 8.5, label_color),
            label,
        );
        block.text_right(
            RIGHT - 12.0,
            row_baseline,
            TextStyle::new(Face::SemiBold, 8.5, *color),
            value,
        );
        y += ROW_HEIGHT;
        block.line((BOX_LEFT, y), (RIGHT, y), ROW_RULE, HAIRLINE);
    }
    block.line((BOX_LEFT, y + 0.75), (RIGHT, y + 0.75), RULE, 1.5);
    y + 1.5
}

fn total_due(block: &mut Block, top: f32, amount: f64) {
    block.rect(
        Area::new(BOX_LEFT, top, WIDTH, GRAND_HEIGHT),
        Some(ACCENT_WASH),
        None,
    );
    block.text(
        BOX_LEFT + 12.0,
        baseline(top, 8.0, GRAND_HEIGHT),
        TextStyle::new(Face::SemiBold, 8.0, ACCENT),
        "Totale dovuto",
    );
    block.text_right(
        RIGHT - 12.0,
        baseline(top, 15.0, GRAND_HEIGHT),
        TextStyle::new(Face::Display, 15.0, ACCENT_STRONG).tracked(-0.2),
        &currency(amount),
    );
}
