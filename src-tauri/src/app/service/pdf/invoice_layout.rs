//! The invoice page, section by section as `InvoicePrintView.vue` draws it: letterhead
//! and number, recipient and payment, the lines, the totals, then the fiscal notes,
//! the Sistema TS authorisation and the footer. CSS pixels become points at 0.75.

use crate::app::model::client::ClientType;
use crate::app::model::config::Profession;
use crate::app::model::invoice::{InvoiceLine, PaymentMethod};
use crate::app::service::italian_format::{currency, date_long};

use super::block::baseline;
use super::page_flow::{CONTENT_WIDTH, MARGIN_BOTTOM, MARGIN_SIDE, PAGE_HEIGHT};
use super::{
    invoice_legal_notes, Area, Block, DrawOp, Face, InvoiceDocument, PageFlow, PdfFonts, Rgb,
    TextStyle,
};

const INK: Rgb = Rgb::hex(0x1d1b24);
const MUTED: Rgb = Rgb::hex(0x5c5866);
const SUBTLE: Rgb = Rgb::hex(0x8a8694);
const ACCENT: Rgb = Rgb::hex(0x34388f);
const ACCENT_STRONG: Rgb = Rgb::hex(0x2c2f80);
const RULE: Rgb = Rgb::hex(0xe6e2d9);
const ROW_RULE: Rgb = Rgb::hex(0xf2efe9);
const PAPER_TINT: Rgb = Rgb::hex(0xfaf8f4);
const ACCENT_WASH: Rgb = Rgb::hex(0xecedf9);
const ACCENT_EDGE: Rgb = Rgb::hex(0xc9cbee);
const DEDUCTION: Rgb = Rgb::hex(0xa83a2c);
const HAIRLINE: f32 = 0.75;

const LEFT: f32 = MARGIN_SIDE;
const RIGHT: f32 = MARGIN_SIDE + CONTENT_WIDTH;

/// A small uppercase label that names a section ("DESTINATARIO").
fn section_label(color: Rgb) -> TextStyle {
    TextStyle::new(Face::Bold, 6.5, color).tracked(1.1)
}

fn upper(text: &str) -> String {
    text.to_uppercase()
}

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

// ─── Letterhead and invoice number ───────────────────────────────────────────

fn professional_name(document: &InvoiceDocument) -> String {
    let config = &document.config;
    [&config.title, &config.first_name, &config.last_name]
        .iter()
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
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

fn header<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let mut block = Block::new(fonts);
    let identity_bottom = invoice_identity(&mut block, document);
    let column_width = CONTENT_WIDTH - 190.0;
    let letterhead_bottom = letterhead(&mut block, document, column_width);
    block.height = identity_bottom.max(letterhead_bottom);
    block
}

/// The left column; returns its bottom.
fn letterhead(block: &mut Block, document: &InvoiceDocument, width: f32) -> f32 {
    let config = &document.config;
    let name_style = TextStyle::new(Face::Display, 19.0, INK).tracked(-0.35);
    let name_leading = 19.0 * 1.15;
    let mut y = block.paragraph(
        LEFT,
        0.0,
        width,
        name_style,
        name_leading,
        &professional_name(document),
    );
    y += 3.0;

    let profession = TextStyle::new(Face::SemiBold, 7.5, ACCENT).tracked(1.05);
    block.text(
        LEFT,
        baseline(y, 7.5, 11.0),
        profession,
        &upper(&profession_line(document)),
    );
    y += 11.0 + 3.75;

    let detail = TextStyle::new(Face::Regular, 7.5, MUTED);
    let strong = TextStyle::new(Face::SemiBold, 7.5, ACCENT_STRONG);
    let detail_leading = 7.5 * 1.45;
    let albo_number = config.albo_number.trim();
    let albo_region = config.albo_region.trim();
    if !albo_number.is_empty() || !albo_region.is_empty() {
        let mut runs: Vec<(TextStyle, &str)> = Vec::new();
        if !albo_number.is_empty() {
            runs.extend([(detail, "Iscriz. Albo n. "), (strong, albo_number)]);
        }
        if !albo_number.is_empty() && !albo_region.is_empty() {
            runs.push((detail, " — "));
        }
        if !albo_region.is_empty() {
            runs.extend([(detail, "Regione "), (strong, albo_region)]);
        }
        block.runs(LEFT, baseline(y, 7.5, detail_leading), &runs);
        y += detail_leading + 1.5;
    }
    if config.is_psicoanalista {
        block.runs(
            LEFT,
            baseline(y, 7.5, detail_leading),
            &[
                (detail, "Membro della "),
                (strong, "International Psychoanalytical Association (IPA)"),
            ],
        );
        y += detail_leading + 1.5;
    }

    y += 6.0 - 1.5;
    let leading = 7.5 * 1.7;
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
    for line in &lines {
        block.text(LEFT, baseline(y, 7.5, leading), detail, line);
        y += leading;
    }
    y
}

/// The right column, right-aligned; returns its bottom.
fn invoice_identity(block: &mut Block, document: &InvoiceDocument) -> f32 {
    let invoice = &document.invoice;
    block.text_right(
        RIGHT,
        baseline(0.0, 7.0, 10.0),
        TextStyle::new(Face::Bold, 7.0, ACCENT).tracked(1.5),
        "FATTURA",
    );
    let mut y = 10.0 + 3.0;

    let number = TextStyle::new(Face::Display, 30.0, INK).tracked(-0.75);
    let prefix = TextStyle::new(Face::DisplayLight, 15.0, SUBTLE);
    let number_width = block.width(&number, &invoice.invoice_number);
    let prefix_width = block.width(&prefix, "N.") + 1.5;
    let number_baseline = y + 30.0 * 0.8;
    block.text(
        RIGHT - number_width - prefix_width,
        number_baseline,
        prefix,
        "N.",
    );
    block.text(
        RIGHT - number_width,
        number_baseline,
        number,
        &invoice.invoice_number,
    );
    y += 30.0 + 7.5;

    let label = TextStyle::new(Face::SemiBold, 7.0, SUBTLE).tracked(0.4);
    let value = TextStyle::new(Face::SemiBold, 8.0, INK);
    let mut rows = vec![("DATA EMISSIONE", date_long(&invoice.issue_date))];
    if let Some(due) = invoice.due_date.as_deref().filter(|due| !due.is_empty()) {
        rows.push(("SCADENZA", date_long(due)));
    }
    let row_width = rows
        .iter()
        .map(|(name, date)| block.width(&label, name) + 6.0 + block.width(&value, date))
        .fold(number_width + prefix_width, f32::max);
    block.line((RIGHT - row_width, y), (RIGHT, y), RULE, HAIRLINE);
    y += 6.0;
    for (name, date) in &rows {
        let row_baseline = baseline(y, 8.0, 12.0);
        let date_width = block.width(&value, date);
        block.text_right(RIGHT, row_baseline, value, date);
        block.text_right(RIGHT - date_width - 6.0, row_baseline, label, name);
        y += 12.0 + 2.25;
    }
    y
}

// ─── Recipient and payment ───────────────────────────────────────────────────

fn client_name(document: &InvoiceDocument) -> String {
    let client = &document.client;
    match client.client_type {
        ClientType::Azienda => client.last_name.trim().to_string(),
        ClientType::PersonaFisica => {
            format!("{} {}", client.first_name.trim(), client.last_name.trim())
                .trim()
                .to_string()
        }
    }
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

fn parties<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
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
    let client = &document.client;
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
        &client_name(document),
    );
    y += 3.75;

    let detail = TextStyle::new(Face::Regular, 7.5, MUTED);
    let tag = TextStyle::new(Face::Bold, 6.5, ACCENT).tracked(0.4);
    let leading = 7.5 * 1.65;
    let tagged = [
        ("C.F. ", Some(client.fiscal_code.as_str())),
        ("P.IVA ", client.vat_number.as_deref()),
    ];
    for (name, value) in tagged {
        if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
            block.runs(
                x,
                baseline(y, 7.5, leading),
                &[(tag, name), (detail, value)],
            );
            y += leading;
        }
    }
    let place = format!(
        "{} {}{}",
        client.zip_code.trim(),
        client.city.trim(),
        if client.province.trim().is_empty() {
            String::new()
        } else {
            format!(" ({})", client.province.trim())
        }
    );
    let lines = [
        Some(client.address.trim().to_string()),
        Some(place.trim().to_string()),
        client
            .email
            .as_deref()
            .map(|email| email.trim().to_string()),
    ];
    for line in lines.into_iter().flatten().filter(|line| !line.is_empty()) {
        y = block.paragraph(x, y, width, detail, leading, &line);
    }
    y
}

fn payment(block: &mut Block, document: &InvoiceDocument, x: f32, top: f32) -> f32 {
    let invoice = &document.invoice;
    let config = &document.config;
    block.text(
        x,
        baseline(top, 6.5, 9.0),
        section_label(ACCENT),
        "PAGAMENTO",
    );
    let mut y = top + 9.0 + 6.0;

    let tag = TextStyle::new(Face::SemiBold, 8.0, ACCENT_STRONG);
    let label = payment_label(&invoice.payment_method);
    let tag_height = 8.0 * 1.55 + 4.5;
    let tag_width = block.width(&tag, label) + 15.0;
    block.rect(
        Area::new(x, y, tag_width, tag_height).rounded(3.0),
        Some(ACCENT_WASH),
        Some((ACCENT_EDGE, HAIRLINE)),
    );
    block.text(x + 7.5, baseline(y, 8.0, tag_height), tag, label);
    y += tag_height + 6.75;

    let name = TextStyle::new(Face::SemiBold, 6.5, SUBTLE).tracked(0.6);
    let value = TextStyle::new(Face::SemiBold, 8.0, INK);
    let mut fields: Vec<(&str, String, TextStyle)> = Vec::new();
    if let Some(condition) = payment_condition(&invoice.payment_method) {
        fields.push(("CONDIZIONI", condition.to_string(), value));
    }
    if invoice.payment_method == PaymentMethod::Bonifico && !config.iban.trim().is_empty() {
        fields.push((
            "IBAN",
            config.iban.trim().to_string(),
            TextStyle::new(Face::SemiBold, 7.5, INK).tracked(0.6),
        ));
        let holder = format!("{} {}", config.first_name.trim(), config.last_name.trim());
        fields.push(("INTESTATO A", holder.trim().to_string(), value));
    }
    for (field, text, style) in fields {
        block.text(x, baseline(y, 6.5, 9.0), name, field);
        y += 9.0 + 0.75;
        block.text(x, baseline(y, style.size, 12.0), style, &text);
        y += 12.0 + 4.5;
    }
    y - 4.5
}

// ─── Lines ───────────────────────────────────────────────────────────────────

/// Column edges of the lines table; the quantity and unit-price columns come and go together.
struct Columns {
    description: (f32, f32),
    quantity: Option<(f32, f32)>,
    unit_price: Option<(f32, f32)>,
    vat: (f32, f32),
    amount: (f32, f32),
}

impl Columns {
    fn new(show_quantity: bool) -> Self {
        let at = |share: f32| LEFT + CONTENT_WIDTH * share;
        if show_quantity {
            Columns {
                description: (LEFT, at(0.44)),
                quantity: Some((at(0.44), at(0.53))),
                unit_price: Some((at(0.53), at(0.71))),
                vat: (at(0.71), at(0.80)),
                amount: (at(0.80), RIGHT),
            }
        } else {
            Columns {
                description: (LEFT, at(0.73)),
                quantity: None,
                unit_price: None,
                vat: (at(0.73), at(0.82)),
                amount: (at(0.82), RIGHT),
            }
        }
    }
}

const CELL_PAD: f32 = 9.0;
const EDGE_PAD: f32 = 11.25;

fn lines_table(flow: &mut PageFlow, document: &InvoiceDocument, fonts: &PdfFonts) {
    let columns = Columns::new(!document.invoice.hide_quantity);
    let mut label = Block::new(fonts);
    label.text(
        LEFT,
        baseline(0.0, 6.5, 9.0),
        section_label(ACCENT),
        "DETTAGLIO PRESTAZIONI",
    );
    label.height = 9.0 + 5.25;

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

fn table_row<'f>(
    columns: &Columns,
    line: &InvoiceLine,
    index: usize,
    fonts: &'f PdfFonts,
) -> Block<'f> {
    let mut block = Block::new(fonts);
    let description = TextStyle::new(Face::Medium, 8.5, INK);
    let secondary = TextStyle::new(Face::Regular, 8.0, MUTED);
    let amount = TextStyle::new(Face::Bold, 8.5, INK);
    let leading = 8.5 * 1.45;
    let pad_y = 6.75;
    let description_width = columns.description.1 - columns.description.0 - EDGE_PAD - CELL_PAD;
    let lines = block.wrap(&description, &line.description, description_width);
    let height = leading * lines.len() as f32 + 2.0 * pad_y;

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
        pad_y,
        description_width,
        description,
        leading,
        &line.description,
    );

    let first = baseline(pad_y, 8.5, leading);
    if let Some((start, end)) = columns.quantity {
        block.text_center(
            (start + end) / 2.0,
            first,
            secondary,
            &line.quantity.to_string(),
        );
    }
    // A hand-typed amount is not quantity × unit price, so no unit price is printed beside it.
    if let (Some((_, end)), None) = (columns.unit_price, line.amount_override) {
        block.text_right(end - CELL_PAD, first, secondary, &currency(line.unit_price));
    }
    vat_cell(&mut block, columns.vat, first, line.vat_rate);
    block.text_right(
        columns.amount.1 - EDGE_PAD,
        first,
        amount,
        &currency(line.line_total),
    );
    block.height = height;
    block
}

fn vat_cell(block: &mut Block, (start, end): (f32, f32), y: f32, rate: f64) {
    let center = (start + end) / 2.0;
    if rate > 0.0 {
        let text = format!("{}%", format_rate(rate));
        block.text_center(center, y, TextStyle::new(Face::Regular, 8.0, MUTED), &text);
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

// ─── Totals ──────────────────────────────────────────────────────────────────

fn totals<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let invoice = &document.invoice;
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

    let mut block = Block::new(fonts);
    let width = 221.25;
    let left = RIGHT - width;
    let row_height = 8.5 * 1.55 + 2.0 * 5.25;
    let grand_height = 15.0 + 2.0 * 9.0;
    let height = row_height * rows.len() as f32 + 1.5 + grand_height;
    let frame = Area::new(left, 0.0, width, height).rounded(5.25);
    block.push(DrawOp::PushClip(frame));

    let mut y = 0.0;
    for (label, value, color) in &rows {
        let row_baseline = baseline(y, 8.5, row_height);
        let label_color = if *color == INK { MUTED } else { *color };
        block.text(
            left + 12.0,
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
        y += row_height;
        block.line((left, y), (RIGHT, y), ROW_RULE, HAIRLINE);
    }
    block.line((left, y + 0.75), (RIGHT, y + 0.75), RULE, 1.5);
    y += 1.5;

    block.rect(
        Area::new(left, y, width, grand_height),
        Some(ACCENT_WASH),
        None,
    );
    block.text(
        left + 12.0,
        baseline(y, 8.0, grand_height),
        TextStyle::new(Face::SemiBold, 8.0, ACCENT),
        "Totale dovuto",
    );
    block.text_right(
        RIGHT - 12.0,
        baseline(y, 15.0, grand_height),
        TextStyle::new(Face::Display, 15.0, ACCENT_STRONG).tracked(-0.2),
        &currency(invoice.total_due),
    );
    block.push(DrawOp::PopClip);
    block.rect(frame, None, Some((RULE, HAIRLINE)));
    block.height = height;
    block
}

// ─── Footer ──────────────────────────────────────────────────────────────────

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

fn legal_notes_box<'f>(notes: &[String], fonts: &'f PdfFonts) -> Block<'f> {
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

fn sts_box<'f>(document: &InvoiceDocument, fonts: &'f PdfFonts) -> Block<'f> {
    let authorised = document.client.sts_authorization;
    framed(
        fonts,
        "AUTORIZZAZIONE SISTEMA TESSERA SANITARIA",
        SUBTLE,
        Rgb::hex(0xfafafa),
        Rgb::hex(0xe0e0e0),
        |block, x, top, width| {
            let text = TextStyle::new(Face::Regular, 7.5, Rgb::hex(0x3d3a45));
            let leading = 7.5 * 1.55;
            let mut y = block.paragraph(
                x,
                top,
                width,
                text,
                leading,
                "Ai sensi dell'art. 3, commi 1 e 2, del D.Lgs. 21 luglio 2014, n. 175 (Dichiarazione dei redditi precompilata), l'intestatario della presente ricevuta sanitaria",
            );
            y += 1.5;
            let mut cursor = x;
            for (label, chosen) in [("AUTORIZZA", authorised), ("NON AUTORIZZA", !authorised)] {
                let face = if chosen { Face::Bold } else { Face::Regular };
                let color = if chosen { INK } else { Rgb::hex(0x3d3a45) };
                let square = Area::new(cursor, y + (leading - 8.25) / 2.0, 8.25, 8.25).rounded(1.5);
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
                    baseline(y, 7.5, leading),
                    TextStyle::new(face, 7.5, color),
                    label,
                ) + 12.0;
            }
            y += leading + 1.5;
            block.paragraph(
                x,
                y,
                width,
                text,
                leading,
                "la trasmissione dei dati relativi alla presente spesa sanitaria al Sistema Tessera Sanitaria ai fini dell'elaborazione della dichiarazione dei redditi precompilata.",
            )
        },
    )
}

fn notes_box<'f>(notes: &str, fonts: &'f PdfFonts) -> Block<'f> {
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

/// The footer of every page, with "Pagina 1 di 2" under it when there is more than one.
fn bottom_bar<'f>(
    document: &InvoiceDocument,
    fonts: &'f PdfFonts,
    numbering: Option<&str>,
) -> Block<'f> {
    let config = &document.config;
    let mut block = Block::new(fonts);
    block.line((LEFT, 0.0), (RIGHT, 0.0), RULE, HAIRLINE);
    let leading = 7.0 * 1.7;
    let text = TextStyle::new(Face::Regular, 7.0, SUBTLE);
    let dot = TextStyle::new(Face::Regular, 7.0, Rgb::hex(0xd4cfc4));
    let name = professional_name(document);
    let vat = format!("P.IVA {}", config.vat_number);
    let fiscal = format!("C.F. {}", config.fiscal_code);
    let pec = format!("PEC: {}", config.pec_email.trim());
    let mut runs: Vec<(TextStyle, &str)> = vec![
        (TextStyle::new(Face::SemiBold, 7.0, ACCENT_STRONG), &name),
        (dot, "  ·  "),
        (text, &vat),
        (dot, "  ·  "),
        (text, &fiscal),
    ];
    if !config.pec_email.trim().is_empty() {
        runs.extend([(dot, "  ·  "), (text, pec.as_str())]);
    }
    let first = baseline(9.0, 7.0, leading);
    let width = block.runs_width(&runs);
    block.runs(LEFT + (CONTENT_WIDTH - width) / 2.0, first, &runs);

    let address = if config.address.trim().is_empty() {
        format!("{} {} ({})", config.zip_code, config.city, config.province)
    } else {
        format!(
            "{}, {} {} ({})",
            config.address.trim(),
            config.zip_code,
            config.city,
            config.province
        )
    };
    block.text_center(LEFT + CONTENT_WIDTH / 2.0, first + leading, text, &address);
    block.height = 9.0 + 2.0 * leading;
    if let Some(numbering) = numbering {
        block.text_center(
            LEFT + CONTENT_WIDTH / 2.0,
            first + 2.0 * leading,
            text,
            numbering,
        );
        block.height += leading;
    }
    block
}
