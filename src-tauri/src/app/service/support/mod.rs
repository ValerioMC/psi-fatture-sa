//! Pure, stateless helpers shared by several services: input validation, tax
//! arithmetic and Italian formatting. None of them is a service of an entity.

pub mod italian_format;
pub mod tax_calculator;
pub mod validation;
