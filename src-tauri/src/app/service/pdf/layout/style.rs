//! The page's palette and the edges every section aligns to, as in the print components
//! (`src/features/invoice/components/print/`).

use crate::app::service::pdf::page_flow::{CONTENT_WIDTH, MARGIN_SIDE};
use crate::app::service::pdf::{Face, Rgb, TextStyle};

pub(super) const INK: Rgb = Rgb::hex(0x1d1b24);
pub(super) const MUTED: Rgb = Rgb::hex(0x5c5866);
pub(super) const SUBTLE: Rgb = Rgb::hex(0x8a8694);
pub(super) const ACCENT: Rgb = Rgb::hex(0x34388f);
pub(super) const ACCENT_STRONG: Rgb = Rgb::hex(0x2c2f80);
pub(super) const RULE: Rgb = Rgb::hex(0xe6e2d9);
pub(super) const ROW_RULE: Rgb = Rgb::hex(0xf2efe9);
pub(super) const PAPER_TINT: Rgb = Rgb::hex(0xfaf8f4);
pub(super) const ACCENT_WASH: Rgb = Rgb::hex(0xecedf9);
pub(super) const ACCENT_EDGE: Rgb = Rgb::hex(0xc9cbee);
pub(super) const DEDUCTION: Rgb = Rgb::hex(0xa83a2c);
pub(super) const HAIRLINE: f32 = 0.75;

pub(super) const LEFT: f32 = MARGIN_SIDE;
pub(super) const RIGHT: f32 = MARGIN_SIDE + CONTENT_WIDTH;

/// The small grey text of addresses and registrations, in the letterhead and the recipient.
pub(super) const DETAIL: TextStyle = TextStyle::new(Face::Regular, 7.5, MUTED);

/// A small uppercase label that names a section ("DESTINATARIO").
pub(super) fn section_label(color: Rgb) -> TextStyle {
    TextStyle::new(Face::Bold, 6.5, color).tracked(1.1)
}
