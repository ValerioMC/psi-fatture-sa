use super::{Face, Rgb};

/// How a run of text is set: `tracking` is extra space after every glyph, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub face: Face,
    pub size: f32,
    pub color: Rgb,
    pub tracking: f32,
}

impl TextStyle {
    pub const fn new(face: Face, size: f32, color: Rgb) -> Self {
        TextStyle {
            face,
            size,
            color,
            tracking: 0.0,
        }
    }

    pub const fn tracked(self, tracking: f32) -> Self {
        TextStyle { tracking, ..self }
    }
}
