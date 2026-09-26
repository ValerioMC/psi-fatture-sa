/// The two document calls that carry a whole expense document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentCall {
    Insert,
    Update,
}

impl DocumentCall {
    pub(super) fn request_element(&self) -> &'static str {
        match self {
            DocumentCall::Insert => "inserimentoDocumentoSpesaRequest",
            DocumentCall::Update => "variazioneDocumentoSpesaRequest",
        }
    }

    pub(super) fn document_element(&self) -> &'static str {
        match self {
            DocumentCall::Insert => "idInserimentoDocumentoFiscale",
            DocumentCall::Update => "idVariazioneDocumentoFiscale",
        }
    }
}
