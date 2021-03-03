pub mod pattern;
pub mod rs_engine;
use std::sync::Arc;

pub use rs_engine::RSEngine;

pub use pattern::Pattern;

use anyhow::Result;

pub trait Engine {
    fn bootstrap(&mut self) -> anyhow::Result<()>;
    fn list(&mut self) -> Vec<Arc<dyn pattern::Pattern>>;
}
