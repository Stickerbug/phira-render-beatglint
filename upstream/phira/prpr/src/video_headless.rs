use super::{Anim, Resource};
use crate::ext::ScaleType;
use anyhow::Result;

/// Video placeholder for the headless profile. The full profile keeps the
/// prpr-avc decoder; bounded chart geometry tests intentionally omit video
/// decoding and record that fact in their render manifest.
pub struct Video;

impl Video {
    pub fn new(
        _data: Vec<u8>,
        _start_time: f32,
        _scale_type: ScaleType,
        _alpha: Anim<f32>,
        _dim: Anim<f32>,
    ) -> Result<Self> {
        Ok(Self)
    }

    pub fn update(&mut self, _time: f32) -> Result<()> {
        Ok(())
    }

    pub fn render(&self, _res: &mut Resource) {}
}
