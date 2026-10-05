use super::style::{LEFT, RIGHT};
use crate::app::service::pdf::page_flow::CONTENT_WIDTH;

/// Space between a cell's text and its column edge.
pub(super) const CELL_PAD: f32 = 9.0;
/// Space between the outer columns' text and the table frame.
pub(super) const EDGE_PAD: f32 = 11.25;

/// Column edges of the lines table; the quantity and unit-price columns come and go together.
pub(super) struct Columns {
    pub(super) description: (f32, f32),
    pub(super) quantity: Option<(f32, f32)>,
    pub(super) unit_price: Option<(f32, f32)>,
    pub(super) vat: (f32, f32),
    pub(super) amount: (f32, f32),
}

impl Columns {
    pub(super) fn new(show_quantity: bool) -> Self {
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
