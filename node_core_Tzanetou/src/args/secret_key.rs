//! Module for managing loading and generating a [`SecretKey`].
//!
//! This module provides functionality to load a [`SecretKey`] from a specified path or generate a new one
//! if it doesn't exist at the given path. It handles errors related to file system operations and
//! decoding the secret key, returning a [`SecretKeyError`] in case of failures.
//!
//! The `get_secret_key` function attempts to:
//! - Load a [`SecretKey`] from the provided `secret_key_path`.
//! - Generate a new [`SecretKey`] if no file exists at `secret_key_path`.
//! - Handle errors such as decoding errors, file system path errors, and IO errors during file operations.
//!
//! Errors returned by `get_secret_key` are encapsulated in the [`SecretKeyError`] enum, which includes:
//! - `SecretKeyDecodeError`: Error encountered during decoding of the secret key.
//! - `SecretKeyFsPathError`: Error related to file system path operations.
//! - `FailedToAccessKeyFile`: Error indicating failure to access the key file due to an IO error.
use reth_fs_util::{self as fs, FsPathError};
use reth_network::config::rng_secret_key;
use reth_primitives::hex::encode as hex_encode;
use secp256k1::{Error as SecretKeyBaseError, SecretKey};
use std::{
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Errors returned by loading a [`SecretKey`], including IO errors.
#[derive(Error, Debug)]
pub enum SecretKeyError {
    /// Error encountered during decoding of the secret key.
    #[error(transparent)]
    SecretKeyDecodeError(#[from] SecretKeyBaseError),

    /// Error related to file system path operations.
    #[error(transparent)]
    SecretKeyFsPathError(#[from] FsPathError),

    /// Represents an error when failed to access the key file.
    #[error("failed to access key file {secret_file:?}: {error}")]
    FailedToAccessKeyFile {
        /// The encountered IO error.
        error: io::Error,
        /// Path to the secret key file.
        secret_file: PathBuf,
    },
}

/// Attempts to load a [`SecretKey`] from a specified path. If no file exists there, then it
/// generates a secret key and stores it in the provided path. I/O errors might occur during write
/// operations in the form of a [`SecretKeyError`]
pub fn get_secret_key(secret_key_path: &Path) -> Result<SecretKey, SecretKeyError> {
    let exists = secret_key_path.try_exists();

    match exists {
        Ok(true) => {
            let contents = fs::read_to_string(secret_key_path)?;
            Ok(contents
                .as_str()
                .parse::<SecretKey>()
                .map_err(SecretKeyError::SecretKeyDecodeError)?)
        }
        Ok(false) => {
            if let Some(dir) = secret_key_path.parent() {
                // Create parent directory
                fs::create_dir_all(dir)?;
            }

            let secret = rng_secret_key();
            let hex = hex_encode(secret.as_ref());
            fs::write(secret_key_path, hex)?;
            Ok(secret)
        }
        Err(error) => Err(SecretKeyError::FailedToAccessKeyFile {
            error,
            secret_file: secret_key_path.to_path_buf(),
        }),
    }
}
