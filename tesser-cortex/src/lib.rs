//! Cortex: zero-copy, hardware-agnostic inference primitives for Tesser.

pub mod buffer;
pub mod config;
#[cfg(not(target_env = "musl"))]
pub mod engine;
#[cfg(target_env = "musl")]
mod engine_stub;

pub use buffer::FeatureBuffer;
pub use config::{CortexConfig, CortexDevice};
#[cfg(not(target_env = "musl"))]
pub use engine::CortexEngine;
#[cfg(target_env = "musl")]
pub use engine_stub::CortexEngine;

#[cfg(not(target_env = "musl"))]
pub use ort::Error as OrtError;
