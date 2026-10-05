//! The framed boxes under the totals: fiscal notes, the Sistema TS authorisation, free notes.

use super::style::{
    section_label, ACCENT, ACCENT_STRONG, HAIRLINE, INK, LEFT, PAPER_TINT, RULE, SUBTLE,
};
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::page_flow::CONTENT_WIDTH;
use crate::app::service::pdf::{
    Area, Block, DrawOp, Face, InvoiceDocument, PdfFonts, Rgb, TextStyle,
};

const STS_GREY: Rgb = Rgb::hex(0x3d3a45);
const STS_TEXT: TextStyle = TextStyle::new(Face::Regular, 7.5, STS_GREY);
const STS_LEADING: f32 = 7.5 * 1.55;
const STS_PREAMBLE: &str = "Ai sensi dell'art. 3, commi 1 e 2, del D.Lgs. 21 luglio 2014, n. 175 (Dichiarazione dei redditi precompilata), l'intestatario della presente ricevuta sanitaria";
const STS_PURPOSE: &str = "la trasmissione dei dati relativi alla presente spesa sanitaria al Sistema Tessera Sanitaria ai fini dell'elaborazione della dichiarazione dei redditi precompilata.";

pub(super) fn legal_notes_box<'f>(notes: &[String], fonts: &'f PdfFonts) -> Block<'f> {
    framed(
        fonts,
        "NOTE FISCALI",
        Rgb::hex(0xb45309),
        Rgb::hex(0xfffbeb),
        Rgb::hex(0xfde68a),
        |block, x, top, width| {
            let style = TextStyle::new(Face::Regular, 7.0, Rgb::hex(0x78350f));
            notes.iter().fold(top, |y, note| {
                block.paragraph(x, y, width, style, 7.0 * 1.55, note) + 2.25
            }) - 2.25
        },
    )
}

/// The patient's choice on sending the expense to the Sistema TS, printed as ticked boxes.
pub(super) fn sts_box<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let authorised = document.client.sts_authorization;
    framed(
        fonts,
        "AUTORIZZAZIONE SISTEMA TESSERA SANITARIA",
        SUBTLE,
        Rgb::hex(0xfafafa),
        Rgb::hex(0xe0e0e0),
        |block, x, top, width| {
            let mut y = block.paragraph(x, top, width, STS_TEXT, STS_LEADING, STS_PREAMBLE);
            y += 1.5;
            authorisation_choice(block, x, y, authorised);
            y += STS_LEADING + 1.5;
            block.paragraph(x, y, width, STS_TEXT, STS_LEADING, STS_PURPOSE)
        },
    )
}

/// "AUTORIZZA" and "NON AUTORIZZA" side by side, the chosen one ticked and bold.
fn authorisation_choice(block: &mut Block, x: f32, y: f32, authorised: bool) {
    let mut cursor = x;
    for (label, chosen) in [("AUTORIZZA", authorised), ("NON AUTORIZZA", !authorised)] {
        let face = if chosen { Face::Bold } else { Face::Regular };
        let color = if chosen { INK } else { STS_GREY };
        let square = Area::new(cursor, y + (STS_LEADING - 8.25) / 2.0, 8.25, 8.25).rounded(1.5);
        block.rect(square, None, Some((SUBTLE, 1.1)));
        if chosen {
            block.push(DrawOp::Tick {
                origin: (square.x, square.y),
                size: 8.25,
                color: ACCENT_STRONG,
            });
        }
        cursor = block.text(
            cursor + 11.25,
            baseline(y, 7.5, STS_LEADING),
            TextStyle::new(face, 7.5, color),
            label,
        ) + 12.0;
    }
}

pub(super) fn notes_box<'f>(notes: &str, fonts: &'f PdfFonts) -> Block<'f> {
    framed(
        fonts,
        "NOTE",
        ACCENT,
        PAPER_TINT,
        RULE,
        |block, x, top, width| {
            block.paragraph(
                x,
                top,
                width,
                TextStyle::new(Face::Regular, 8.0, INK),
                8.0 * 1.65,
                notes.trim(),
            )
        },
    )
}

/// A tinted box with a label on top and whatever `fill` draws inside; `fill` gets the
/// inner left edge, the inner top and the inner width, and returns the inner bottom.
fn framed<'f>(
    fonts: &'f PdfFonts,
    title: &str,
    title_color: Rgb,
    background: Rgb,
    edge: Rgb,
    fill: impl FnOnce(&mut Block<'f>, f32, f32, f32) -> f32,
) -> Block<'f> {
    let (pad_x, pad_y) = (11.25, 7.5);
    let mut content = Block::new(fonts);
    content.text(
        LEFT + pad_x,
        baseline(pad_y, 6.5, 9.0),
        section_label(title_color),
        title,
    );
    let bottom = fill(
        &mut content,
        LEFT + pad_x,
        pad_y + 9.0 + 4.5,
        CONTENT_WIDTH - 2.0 * pad_x,
    );
    let height = bottom + pad_y;

    let mut block = Block::new(fonts);
    block.rect(
        Area::new(LEFT, 0.0, CONTENT_WIDTH, height).rounded(4.5),
        Some(background),
        Some((edge, HAIRLINE)),
    );
    block.ops.extend(content.ops);
    block.height = height;
    block
}
