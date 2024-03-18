#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

pub use startup::*;

pub mod configuration;
mod handler;
mod macros;
pub mod startup;
pub mod telemetry;
