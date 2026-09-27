//! The invoice as a PDF, drawn in the backend so it can travel as an email
//! attachment. Layout produces `DrawOp`s page by page; the painter turns them
//! into PDF with krilla, fonts bundled and subset.

pub mod invoice_pdf_service;

mod area;
mod block;
mod draw_op;
mod face;
mod invoice_document;
mod invoice_layout;
mod invoice_legal_notes;
mod loaded_face;
mod page_flow;
mod pdf_fonts;
mod pdf_painter;
mod rgb;
mod text_style;

pub use invoice_document::InvoiceDocument;

use area::Area;
use block::Block;
use draw_op::DrawOp;
use face::Face;
use loaded_face::LoadedFace;
use page_flow::PageFlow;
use pdf_fonts::PdfFonts;
use rgb::Rgb;
use text_style::TextStyle;
