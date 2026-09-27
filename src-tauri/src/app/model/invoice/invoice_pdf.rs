/// An invoice rendered as PDF, with the file name it travels under.
#[derive(Debug, Clone)]
pub struct InvoicePdf {
    pub file_name: String,
    pub bytes: Vec<u8>,
}
