#![warn(rust_2021_compatibility, future_incompatible, nonstandard_style)]
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
#![warn(missing_docs)]
// Clippy:
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

#![doc(html_logo_url = "../../../.images/logo.jpg")]
//! # Apollo CW-TOKEN
//!
//! ## Description
//!
//! Apollo DAO offers various strategies to maximize yield across farming products.
//!
//! We need a project that defines tokens struct IBC compatible that the platform will work with.
//!
//! ## Objectives
//!
//! The main goal of the **apollo cw-token** is to:
//!   - Define cw4626 LP tokenized pools
//!   - Define for Osmosis denom token
//!
//! ## Use Cases
//!
//! ## Scenarios
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
