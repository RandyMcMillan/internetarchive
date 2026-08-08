//! Pure Rust core for Archive.org client functionality.
//!
//! This crate provides the Rust-side building blocks for a future PyO3 wrapper
//! and for direct Rust consumers on crates.io.

#![forbid(unsafe_code)]

pub mod config;
pub mod auth;
pub mod errors;
pub mod identifier;
pub mod iarequest;
pub mod item;
pub mod search;
pub mod session;
pub mod utils;

pub use config::{
    ConfigError,
    ConfigFileResolution,
    ConfigMap,
    ConfigValue,
    deep_update,
    get_config,
    parse_config_file,
    parse_config_file_path,
};
pub use auth::{S3Auth, S3PostAuth};
pub use errors::IdentifierError;
pub use identifier::validate_s3_identifier;
pub use iarequest::{
    prepare_files_patch,
    prepare_metadata,
    prepare_metadata_headers,
    prepare_patch,
    prepare_target_patch,
};
pub use item::{ArchiveFile, ArchiveItem, ArchiveItemKind};
pub use search::SearchQuery;
pub use session::{ArchiveSession, ArchiveSessionConfig};
pub use utils::{flatten_pipe_patterns, needs_quote, norm_filepath};
pub use utils::{
    is_path_within_directory,
    is_valid_metadata_key,
    is_windows,
    is_dir,
    is_filelike_obj,
    merge_dictionaries,
    recursive_file_count,
    recursive_file_count_and_size,
    iter_directory,
    parse_dict_cookies,
    sanitize_windows_filename,
    sanitize_windows_relpath,
};
