/// A rectangle on the page, top-left origin as the page is read, with rounded corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

impl Area {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Area {
            x,
            y,
            width,
            height,
            radius: 0.0,
        }
    }

    pub const fn rounded(self, radius: f32) -> Self {
        Area { radius, ..self }
    }
}
