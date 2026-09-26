use crate::app::repository::ts::sistema_ts::FieldCipher;

/// Marks values instead of encrypting them, so envelopes are readable in tests.
pub struct VisibleCipher;

impl FieldCipher for VisibleCipher {
    fn encrypt(&self, plain: &str) -> Result<String, String> {
        Ok(format!("enc({plain})"))
    }
}
