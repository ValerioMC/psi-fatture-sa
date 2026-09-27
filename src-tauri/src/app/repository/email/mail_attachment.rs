#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailAttachment {
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}
