//! Test doubles and fixtures shared by the unit tests of several packages.

pub mod clock;
pub mod email_database;
pub mod fake_gateway;
pub mod fake_mail_gateway;
pub mod in_memory_secret_store;
pub mod recorded_call;
pub mod ts_call_responses;
pub mod ts_database;
pub mod unavailable_secret_store;
pub mod unreadable_secret_store;
pub mod visible_cipher;

pub use fake_gateway::FakeGateway;
pub use fake_mail_gateway::FakeMailGateway;
pub use in_memory_secret_store::InMemorySecretStore;
pub use recorded_call::RecordedCall;
pub use unavailable_secret_store::UnavailableSecretStore;
pub use unreadable_secret_store::UnreadableSecretStore;
pub use visible_cipher::VisibleCipher;
