#![warn(rust_2021_compatibility, future_incompatible, nonstandard_style)]
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
#![warn(missing_docs)]

//! # Cosmwasm Vault Token
//!
//! ## Description
//!
//! An abstraction for different ways of implementing a vault token.
//! This crate defines a set of traits that define the behavior of a vault
//! token. Two implementations are provided, one for an Osmosis native denom
//! minted through the TokenFactory module and one for Cw4626 tokenized vaults.
//! See the cosmwasm-vault-standard crate for more information about tokenized
//! vaults.

/// Error Handling
mod error;

/// CW4626 and Denom impl
mod implementations;

/// Traits functionality interface
mod traits;

pub use error::*;
pub use implementations::*;
pub use traits::*;
