//! Credentials kept by the app itself in an encrypted file next to the
//! database: never in clear on disk, never in the OS keychain (which would
//! prompt for permission). `SecretStore` is the seam that lets tests run
//! against memory.

pub mod encrypted_file_secret_store;
pub mod machine_id_source;
pub mod os_machine_id;
pub mod secret_kind;
pub mod secret_store;
pub mod secret_store_error;

mod loaded_file;
mod sealed_secret;
mod secret_file;

pub use encrypted_file_secret_store::EncryptedFileSecretStore;
pub use machine_id_source::MachineIdSource;
pub use os_machine_id::OsMachineId;
pub use secret_kind::SecretKind;
pub use secret_store::SecretStore;
pub use secret_store_error::SecretStoreError;

use loaded_file::LoadedFile;
use sealed_secret::SealedSecret;
use secret_file::SecretFile;
