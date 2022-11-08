#![warn(rust_2021_compatibility, future_incompatible, nonstandard_style)]
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
#![warn(missing_docs)]
// Clippy:
#![warn(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
//! # CW-VAULT-TOKEN
//!
//! ## Description
//!
//! The main goal of the **cw-vault- token** is to:
//!   - Define cw4626 LP tokenized pools
//!   - Define for Osmosis denom token
//!

/// Error Handling
mod error;

/// CW4626 and Denom impl
mod implementations;

/// Traits functionality interface
mod traits;

pub use error::*;
pub use implementations::*;
pub use traits::*;
