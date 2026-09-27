use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EmailConnectionCheck {
    pub ok: bool,
    pub message: String,
}
