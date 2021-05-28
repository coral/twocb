pub mod colorchord;
pub mod input;
pub mod processing;

pub use colorchord::Colorchord;
pub use input::Input;
pub use processing::Processing;
pub use processing::TempoResult;

#[derive(Debug, Copy, Clone)]
pub struct StreamSetting {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: u16,
}
