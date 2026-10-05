//! The invoice page, one module per section. Only `layout` is reachable from the
//! rest of the PDF package.

mod boxes;
mod columns;
mod footer;
mod identity;
mod letterhead;
mod lines_table;
mod page;
mod parties;
mod style;
mod totals;

pub(super) use page::layout;

use columns::Columns;
