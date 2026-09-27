use super::{Block, DrawOp};

/// A4 in points.
pub const PAGE_WIDTH: f32 = 595.28;
pub const PAGE_HEIGHT: f32 = 841.89;
/// The print view's 36px × 48px padding, at 0.75pt per CSS pixel.
pub const MARGIN_TOP: f32 = 27.0;
pub const MARGIN_SIDE: f32 = 36.0;
pub const MARGIN_BOTTOM: f32 = 24.0;
pub const CONTENT_WIDTH: f32 = PAGE_WIDTH - 2.0 * MARGIN_SIDE;

/// Places blocks down the page and starts a new page when one does not fit.
/// A block never splits: long content is laid out as several blocks.
/// `reserved` keeps room at the foot of every page for a footer drawn later.
pub struct PageFlow {
    pages: Vec<Vec<DrawOp>>,
    reserved: f32,
    pub y: f32,
}

impl PageFlow {
    pub fn new(reserved: f32) -> Self {
        PageFlow {
            pages: vec![Vec::new()],
            reserved,
            y: MARGIN_TOP,
        }
    }

    pub fn fits(&self, height: f32) -> bool {
        self.y + height <= PAGE_HEIGHT - MARGIN_BOTTOM - self.reserved
    }

    pub fn break_page(&mut self) {
        self.pages.push(Vec::new());
        self.y = MARGIN_TOP;
    }

    /// Breaks the page unless the height fits, or the page is still empty and a
    /// break would only move the problem.
    pub fn keep(&mut self, height: f32) {
        if !self.fits(height) && self.y > MARGIN_TOP {
            self.break_page();
        }
    }

    pub fn gap(&mut self, height: f32) {
        self.y += height;
    }

    pub fn place(&mut self, block: &Block) {
        let ops = block.shifted(self.y);
        self.current().extend(ops);
        self.y += block.height;
    }

    pub fn place_kept(&mut self, block: &Block) {
        self.keep(block.height);
        self.place(block);
    }

    /// Where the next op of the current page will go, for inserting in front of it later.
    pub fn mark(&mut self) -> usize {
        self.current().len()
    }

    pub fn insert(&mut self, mark: usize, op: DrawOp) {
        self.current().insert(mark, op);
    }

    pub fn push(&mut self, op: DrawOp) {
        self.current().push(op);
    }

    pub fn finish(self) -> Vec<Vec<DrawOp>> {
        self.pages
    }

    fn current(&mut self) -> &mut Vec<DrawOp> {
        let last = self.pages.len() - 1;
        &mut self.pages[last]
    }
}
