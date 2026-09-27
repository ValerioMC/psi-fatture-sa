use krilla::color::rgb;
use krilla::geom::{Path, PathBuilder, Point};
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineCap, LineJoin, Stroke};
use krilla::surface::Surface;
use krilla::Document;

use super::page_flow::{PAGE_HEIGHT, PAGE_WIDTH};
use super::{Area, DrawOp, PdfFonts, Rgb, TextStyle};

/// Turns laid-out pages into an A4 PDF with the fonts embedded and subset.
pub fn paint(
    pages: &[Vec<DrawOp>],
    fonts: &PdfFonts,
    metadata: Metadata,
) -> Result<Vec<u8>, String> {
    let mut document = Document::new();
    document.set_metadata(metadata);
    for ops in pages {
        let settings =
            PageSettings::from_wh(PAGE_WIDTH, PAGE_HEIGHT).ok_or("Formato pagina non valido")?;
        let mut page = document.start_page_with(settings);
        let mut surface = page.surface();
        for op in ops {
            draw(&mut surface, fonts, op);
        }
        surface.finish();
        page.finish();
    }
    document
        .finish()
        .map_err(|e| format!("Creazione del PDF non riuscita: {e:?}"))
}

fn draw(surface: &mut Surface, fonts: &PdfFonts, op: &DrawOp) {
    match op {
        DrawOp::Text { x, y, style, text } => draw_text(surface, fonts, *x, *y, style, text),
        DrawOp::Box { area, fill, stroke } => {
            if let Some(path) = rounded_rect(area) {
                surface.set_fill(fill.map(solid_fill));
                surface.set_stroke(stroke.map(|(color, width)| solid_stroke(color, width)));
                surface.draw_path(&path);
            }
        }
        DrawOp::Line {
            from,
            to,
            color,
            width,
        } => {
            let mut builder = PathBuilder::new();
            builder.move_to(from.0, from.1);
            builder.line_to(to.0, to.1);
            if let Some(path) = builder.finish() {
                surface.set_fill(None);
                surface.set_stroke(Some(solid_stroke(*color, *width)));
                surface.draw_path(&path);
            }
        }
        DrawOp::Tick {
            origin,
            size,
            color,
        } => {
            let (x, y) = *origin;
            let mut builder = PathBuilder::new();
            builder.move_to(x + size * 0.2, y + size * 0.52);
            builder.line_to(x + size * 0.42, y + size * 0.74);
            builder.line_to(x + size * 0.8, y + size * 0.28);
            if let Some(path) = builder.finish() {
                surface.set_fill(None);
                surface.set_stroke(Some(solid_stroke(*color, size * 0.16)));
                surface.draw_path(&path);
            }
        }
        DrawOp::PushClip(area) => {
            if let Some(path) = rounded_rect(area) {
                surface.push_clip_path(&path, &FillRule::NonZero);
            }
        }
        DrawOp::PopClip => surface.pop(),
    }
}

fn draw_text(
    surface: &mut Surface,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    style: &TextStyle,
    text: &str,
) {
    let face = fonts.face(style.face);
    let glyphs = face.glyphs(text, style.tracking / style.size);
    surface.set_stroke(None);
    surface.set_fill(Some(solid_fill(style.color)));
    surface.draw_glyphs(
        Point::from_xy(x, y),
        &glyphs,
        face.font(),
        text,
        style.size,
        false,
    );
}

fn solid_fill(color: Rgb) -> Fill {
    Fill {
        paint: rgb::Color::new(color.red, color.green, color.blue).into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }
}

fn solid_stroke(color: Rgb, width: f32) -> Stroke {
    Stroke {
        paint: rgb::Color::new(color.red, color.green, color.blue).into(),
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    }
}

/// A rectangle with quarter-circle corners (cubic approximation, k = 0.5523).
fn rounded_rect(area: &Area) -> Option<Path> {
    let Area {
        x,
        y,
        width,
        height,
        radius,
    } = *area;
    let r = radius.min(width / 2.0).min(height / 2.0).max(0.0);
    let k = r * 0.552_284_8;
    let (right, bottom) = (x + width, y + height);
    let mut builder = PathBuilder::new();
    builder.move_to(x + r, y);
    builder.line_to(right - r, y);
    builder.cubic_to(right - r + k, y, right, y + r - k, right, y + r);
    builder.line_to(right, bottom - r);
    builder.cubic_to(
        right,
        bottom - r + k,
        right - r + k,
        bottom,
        right - r,
        bottom,
    );
    builder.line_to(x + r, bottom);
    builder.cubic_to(x + r - k, bottom, x, bottom - r + k, x, bottom - r);
    builder.line_to(x, y + r);
    builder.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    builder.close();
    builder.finish()
}
