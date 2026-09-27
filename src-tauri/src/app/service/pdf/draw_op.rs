use super::{Area, Rgb, TextStyle};

/// One mark on a page. Layout produces these and the painter turns them into PDF,
/// so where things land is testable without reading a PDF back.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawOp {
    /// `y` is the baseline.
    Text {
        x: f32,
        y: f32,
        style: TextStyle,
        text: String,
    },
    Box {
        area: Area,
        fill: Option<Rgb>,
        stroke: Option<(Rgb, f32)>,
    },
    Line {
        from: (f32, f32),
        to: (f32, f32),
        color: Rgb,
        width: f32,
    },
    /// A tick inside a `size`-point square whose top-left corner is `origin`.
    Tick {
        origin: (f32, f32),
        size: f32,
        color: Rgb,
    },
    /// Clips what follows to the area until the matching `PopClip`, like CSS `overflow: hidden`.
    PushClip(Area),
    PopClip,
}

impl DrawOp {
    pub fn shifted(&self, dy: f32) -> DrawOp {
        let down = |area: &Area| Area {
            y: area.y + dy,
            ..*area
        };
        match self {
            DrawOp::Text { x, y, style, text } => DrawOp::Text {
                x: *x,
                y: y + dy,
                style: *style,
                text: text.clone(),
            },
            DrawOp::Box { area, fill, stroke } => DrawOp::Box {
                area: down(area),
                fill: *fill,
                stroke: *stroke,
            },
            DrawOp::Line {
                from,
                to,
                color,
                width,
            } => DrawOp::Line {
                from: (from.0, from.1 + dy),
                to: (to.0, to.1 + dy),
                color: *color,
                width: *width,
            },
            DrawOp::Tick {
                origin,
                size,
                color,
            } => DrawOp::Tick {
                origin: (origin.0, origin.1 + dy),
                size: *size,
                color: *color,
            },
            DrawOp::PushClip(area) => DrawOp::PushClip(down(area)),
            DrawOp::PopClip => DrawOp::PopClip,
        }
    }
}
