use thiserror::Error;

/// Errors produced while parsing arguments or running a command.
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Invalid arguments or option values.
    #[error("{0}")]
    Usage(String),
    /// Unknown or unsupported structure.
    #[error("{0}")]
    Structure(String),
    /// Structure placement calculation failed.
    #[error("{0}")]
    Placement(String),
    /// World generation scanning failed.
    #[error("{0}")]
    Worldgen(String),
    /// Loot table replay failed.
    #[error("{0}")]
    Loot(String),
    /// Embedded data parsing failed.
    #[error("{0}")]
    Data(String),
}
