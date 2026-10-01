//! Storage crate — Neon Object Storage (S3-compatible) file handling.
//!
//! This crate depends on `core` only.

pub mod client;
pub mod download;
pub mod presign;
pub mod upload;

pub use client::{StorageClient, StorageConfig};
