use super::{Area, DrawOp, PdfFonts, Rgb, TextStyle};

/// A piece of the page laid out on its own, top at 0, so its height is known before
/// the flow decides which page it goes on.
pub struct Block<'f> {
    fonts: &'f PdfFonts,
    pub ops: Vec<DrawOp>,
    pub height: f32,
}

impl<'f> Block<'f> {
    pub fn new(fonts: &'f PdfFonts) -> Self {
        Block {
            fonts,
            ops: Vec::new(),
            height: 0.0,
        }
    }

    pub fn width(&self, style: &TextStyle, text: &str) -> f32 {
        self.fonts.width(style, text)
    }

    pub fn push(&mut self, op: DrawOp) {
        self.ops.push(op);
    }

    /// Returns where the text ends.
    pub fn text(&mut self, x: f32, baseline: f32, style: TextStyle, text: &str) -> f32 {
        let width = self.width(&style, text);
        if !text.is_empty() {
            self.ops.push(DrawOp::Text {
                x,
                y: baseline,
                style,
                text: text.to_string(),
            });
        }
        x + width
    }

    pub fn text_right(&mut self, right: f32, baseline: f32, style: TextStyle, text: &str) {
        let width = self.width(&style, text);
        self.text(right - width, baseline, style, text);
    }

    pub fn text_center(&mut self, center: f32, baseline: f32, style: TextStyle, text: &str) {
        let width = self.width(&style, text);
        self.text(center - width / 2.0, baseline, style, text);
    }

    /// Several styles on one line, one after the other; returns where the last ends.
    pub fn runs(&mut self, x: f32, baseline: f32, runs: &[(TextStyle, &str)]) -> f32 {
        runs.iter().fold(x, |cursor, (style, text)| {
            self.text(cursor, baseline, *style, text)
        })
    }

    pub fn runs_width(&self, runs: &[(TextStyle, &str)]) -> f32 {
        runs.iter()
            .map(|(style, text)| self.width(style, text))
            .sum()
    }

    pub fn rect(&mut self, area: Area, fill: Option<Rgb>, stroke: Option<(Rgb, f32)>) {
        self.ops.push(DrawOp::Box { area, fill, stroke });
    }

    pub fn line(&mut self, from: (f32, f32), to: (f32, f32), color: Rgb, width: f32) {
        self.ops.push(DrawOp::Line {
            from,
            to,
            color,
            width,
        });
    }

    /// Greedy word wrap; a word longer than the width gets a line of its own.
    pub fn wrap(&self, style: &TextStyle, text: &str, width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let mut line = String::new();
            for word in paragraph.split_whitespace() {
                let candidate = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{line} {word}")
                };
                if line.is_empty() || self.width(style, &candidate) <= width {
                    line = candidate;
                } else {
                    lines.push(std::mem::replace(&mut line, word.to_string()));
                }
            }
            lines.push(line);
        }
        lines
    }

    /// Draws wrapped text from `top`, one line every `leading` points; returns the bottom.
    pub fn paragraph(
        &mut self,
        x: f32,
        top: f32,
        width: f32,
        style: TextStyle,
        leading: f32,
        text: &str,
    ) -> f32 {
        let lines = self.wrap(&style, text, width);
        for (index, line) in lines.iter().enumerate() {
            let line_top = top + leading * index as f32;
            self.text(x, baseline(line_top, style.size, leading), style, line);
        }
        top + leading * lines.len() as f32
    }

    /// Everything shifted down by `dy`, for placing the block on a page.
    pub fn shifted(&self, dy: f32) -> Vec<DrawOp> {
        self.ops.iter().map(|op| op.shifted(dy)).collect()
    }
}

/// The baseline of a line box starting at `top`, text centred in it as CSS centres it.
pub fn baseline(top: f32, size: f32, leading: f32) -> f32 {
    top + (leading - size) / 2.0 + size * 0.8
}
