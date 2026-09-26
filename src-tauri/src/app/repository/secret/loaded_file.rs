use super::SecretFile;

/// What reading the secrets file found: a corrupt file holds nothing readable.
pub(super) enum LoadedFile {
    Missing,
    Corrupt,
    Parsed(SecretFile),
}
