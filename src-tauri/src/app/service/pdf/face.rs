/// The typefaces an invoice is set in: Geist at four weights for text and
/// Newsreader for the names and figures the print view sets in display type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Regular,
    Medium,
    SemiBold,
    Bold,
    Display,
    DisplayLight,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Regular,
        Face::Medium,
        Face::SemiBold,
        Face::Bold,
        Face::Display,
        Face::DisplayLight,
    ];
}
