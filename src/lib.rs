//! Pure Rust core for Archive.org client functionality.
//!
//! This crate provides the Rust-side building blocks for a future PyO3 wrapper
//! and for direct Rust consumers on crates.io.

#![forbid(unsafe_code)]

pub mod config;
pub mod errors;
pub mod identifier;
pub mod item;
pub mod search;
pub mod session;
pub mod utils;

pub use config::{ConfigMap, ConfigValue, deep_update};
pub use errors::IdentifierError;
pub use identifier::validate_s3_identifier;
pub use item::{ArchiveFile, ArchiveItem, ArchiveItemKind};
pub use search::SearchQuery;
pub use session::{ArchiveSession, ArchiveSessionConfig};
pub use utils::{flatten_pipe_patterns, needs_quote, norm_filepath};
