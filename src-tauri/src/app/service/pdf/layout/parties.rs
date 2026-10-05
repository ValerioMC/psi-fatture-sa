//! The card under the header: who the invoice is for on the left, how to pay on the right.

use super::style::{
    section_label, ACCENT, ACCENT_EDGE, ACCENT_STRONG, ACCENT_WASH, DETAIL, HAIRLINE, INK, LEFT,
    PAPER_TINT, RULE, SUBTLE,
};
use crate::app::model::client::{Client, ClientType};
use crate::app::model::invoice::PaymentMethod;
use crate::app::service::pdf::block::baseline;
use crate::app::service::pdf::page_flow::CONTENT_WIDTH;
use crate::app::service::pdf::{Area, Block, DrawOp, Face, InvoiceDocument, PdfFonts, TextStyle};

const DETAIL_LEADING: f32 = 7.5 * 1.65;
const FIELD_VALUE: TextStyle = TextStyle::new(Face::SemiBold, 8.0, INK);

pub(super) fn parties<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let pad_x = 13.5;
    let pad_y = 10.5;
    let left_width = CONTENT_WIDTH * 0.48;
    let mut content = Block::new(fonts);
    let recipient_bottom = recipient(
        &mut content,
        document,
        LEFT + pad_x,
        pad_y,
        left_width - 2.0 * pad_x,
    );
    let payment_bottom = payment(&mut content, document, LEFT + left_width + pad_x, pad_y);
    let height = recipient_bottom.max(payment_bottom) + pad_y;
    two_column_card(fonts, content, left_width, height)
}

/// A rounded card, its left column tinted, holding `content` clipped to its corners.
fn two_column_card<'f>(
    fonts: &'f PdfFonts,
    content: Block<'f>,
    left_width: f32,
    height: f32,
) -> Block<'f> {
    let mut block = Block::new(fonts);
    let card = Area::new(LEFT, 0.0, CONTENT_WIDTH, height).rounded(6.0);
    block.push(DrawOp::PushClip(card));
    block.rect(
        Area::new(LEFT, 0.0, left_width, height),
        Some(PAPER_TINT),
        None,
    );
    block.line(
        (LEFT + left_width, 0.0),
        (LEFT + left_width, height),
        RULE,
        HAIRLINE,
    );
    block.ops.extend(content.ops);
    block.push(DrawOp::PopClip);
    block.rect(card, None, Some((RULE, HAIRLINE)));
    block.height = height;
    block
}

fn recipient(block: &mut Block, document: &InvoiceDocument, x: f32, top: f32, width: f32) -> f32 {
    block.text(
        x,
        baseline(top, 6.5, 9.0),
        section_label(ACCENT),
        "DESTINATARIO",
    );
    let mut y = top + 9.0 + 6.0;
    y = block.paragraph(
        x,
        y,
        width,
        TextStyle::new(Face::Display, 13.0, INK).tracked(-0.15),
        13.0 * 1.2,
        &client_name(&document.client),
    );
    y += 3.75;
    y = tax_ids(block, &document.client, x, y);
    for line in address_lines(&document.client) {
        y = block.paragraph(x, y, width, DETAIL, DETAIL_LEADING, &line);
    }
    y
}

fn client_name(client: &Client) -> String {
    match client.client_type {
        ClientType::Azienda => client.last_name.trim().to_string(),
        ClientType::PersonaFisica => {
            format!("{} {}", client.first_name.trim(), client.last_name.trim())
                .trim()
                .to_string()
        }
    }
}

/// "C.F." and "P.IVA" with a coloured tag, each only when the client has one.
fn tax_ids(block: &mut Block, client: &Client, x: f32, mut y: f32) -> f32 {
    let tag = TextStyle::new(Face::Bold, 6.5, ACCENT).tracked(0.4);
    let tagged = [
        ("C.F. ", Some(client.fiscal_code.as_str())),
        ("P.IVA ", client.vat_number.as_deref()),
    ];
    for (name, value) in tagged {
        if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
            block.runs(
                x,
                baseline(y, 7.5, DETAIL_LEADING),
                &[(tag, name), (DETAIL, value)],
            );
            y += DETAIL_LEADING;
        }
    }
    y
}

/// Street, "20100 Milano (MI)" and email, skipping the blank ones.
fn address_lines(client: &Client) -> Vec<String> {
    let province = client.province.trim();
    let place = format!(
        "{} {}{}",
        client.zip_code.trim(),
        client.city.trim(),
        if province.is_empty() {
            String::new()
        } else {
            format!(" ({province})")
        }
    );
    [
        Some(client.address.trim().to_string()),
        Some(place.trim().to_string()),
        client
            .email
            .as_deref()
            .map(|email| email.trim().to_string()),
    ]
    .into_iter()
    .flatten()
    .filter(|line| !line.is_empty())
    .collect()
}

fn payment(block: &mut Block, document: &InvoiceDocument, x: f32, top: f32) -> f32 {
    block.text(
        x,
        baseline(top, 6.5, 9.0),
        section_label(ACCENT),
        "PAGAMENTO",
    );
    let mut y = top + 9.0 + 6.0;
    y = method_tag(block, &document.invoice.payment_method, x, y);

    let name = TextStyle::new(Face::SemiBold, 6.5, SUBTLE).tracked(0.6);
    for (field, text, style) in payment_fields(document) {
        block.text(x, baseline(y, 6.5, 9.0), name, field);
        y += 9.0 + 0.75;
        block.text(x, baseline(y, style.size, 12.0), style, &text);
        y += 12.0 + 4.5;
    }
    y - 4.5
}

/// The payment method in a tinted pill; returns the bottom.
fn method_tag(block: &mut Block, method: &PaymentMethod, x: f32, mut y: f32) -> f32 {
    let tag = TextStyle::new(Face::SemiBold, 8.0, ACCENT_STRONG);
    let label = payment_label(method);
    let tag_height = 8.0 * 1.55 + 4.5;
    let tag_width = block.width(&tag, label) + 15.0;
    block.rect(
        Area::new(x, y, tag_width, tag_height).rounded(3.0),
        Some(ACCENT_WASH),
        Some((ACCENT_EDGE, HAIRLINE)),
    );
    block.text(x + 7.5, baseline(y, 8.0, tag_height), tag, label);
    y += tag_height + 6.75;
    y
}

/// Conditions, and for a bank transfer the IBAN and its holder.
fn payment_fields(document: &InvoiceDocument) -> Vec<(&'static str, String, TextStyle)> {
    let invoice = &document.invoice;
    let config = &document.config;
    let mut fields: Vec<(&str, String, TextStyle)> = Vec::new();
    if let Some(condition) = payment_condition(&invoice.payment_method) {
        fields.push(("CONDIZIONI", condition.to_string(), FIELD_VALUE));
    }
    if invoice.payment_method == PaymentMethod::Bonifico && !config.iban.trim().is_empty() {
        fields.push((
            "IBAN",
            config.iban.trim().to_string(),
            TextStyle::new(Face::SemiBold, 7.5, INK).tracked(0.6),
        ));
        let holder = format!("{} {}", config.first_name.trim(), config.last_name.trim());
        fields.push(("INTESTATO A", holder.trim().to_string(), FIELD_VALUE));
    }
    fields
}

fn payment_label(method: &PaymentMethod) -> &'static str {
    match method {
        PaymentMethod::Bonifico => "Bonifico bancario",
        PaymentMethod::Contanti => "Contanti",
        PaymentMethod::Pos => "POS / Carta di credito",
        PaymentMethod::Altro => "Altro",
    }
}

fn payment_condition(method: &PaymentMethod) -> Option<&'static str> {
    match method {
        PaymentMethod::Bonifico => Some("Pagamento a vista"),
        PaymentMethod::Contanti | PaymentMethod::Pos => Some("Contestuale alla prestazione"),
        PaymentMethod::Altro => None,
    }
}
