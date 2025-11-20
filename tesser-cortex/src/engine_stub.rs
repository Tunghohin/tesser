use crate::{buffer::FeatureBuffer, config::CortexConfig};
use anyhow::{anyhow, Result};

/// Placeholder engine used when building on unsupported targets (e.g., musl).
pub struct CortexEngine;

impl CortexEngine {
    /// Always fails because ONNX Runtime does not provide musl binaries.
    pub fn new(_config: CortexConfig) -> Result<Self> {
        Err(anyhow!(
            "tesser-cortex is unavailable on musl targets; build with glibc or disable Cortex-based strategies"
        ))
    }

    /// Always returns an error on unsupported targets.
    pub fn predict(&mut self, _buffer: &FeatureBuffer) -> Result<Option<Vec<f32>>> {
        Err(anyhow!("Cortex inference is not supported on musl targets"))
    }
}
