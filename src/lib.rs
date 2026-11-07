//! Prometheus exporter for Supermicro BMC metrics, built on [`smbmc`].
//!
//! This crate is primarily the `smbmc-exporter` binary; the library exists
//! so the binary can be tested and makes no stability promises.

pub mod collector;
pub mod config;
pub mod server;

pub use collector::Snapshot;
pub use config::Args;
pub use server::{Exporter, router};
