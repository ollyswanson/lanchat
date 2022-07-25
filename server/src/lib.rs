mod connection;
mod internal_message;
mod run;
mod server;

pub use run::run;

// TODO: Remove usage of boxed errors where possible once API has settled.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
