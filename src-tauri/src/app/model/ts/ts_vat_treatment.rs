/// VAT treatment of one expense item: a rate, or the natura code of an exemption.
#[derive(Debug, Clone, PartialEq)]
pub enum TsVatTreatment {
    Rate(f64),
    Natura(String),
}
