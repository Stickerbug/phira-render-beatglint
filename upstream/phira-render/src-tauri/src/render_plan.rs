//! Explicit render-window protocol shared by full and bounded render modes.
//!
//! The current renderer still executes only the `Full` plan. Bounded modes
//! are rejected until state warmup and output paths are implemented; this
//! prevents a caller from believing it received a segment while the old
//! renderer silently produced a full-song video.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewMode {
    #[default]
    Full,
    Frame,
    Segment,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPlan {
    pub mode: PreviewMode,
    pub start_s: Option<f64>,
    pub end_s: Option<f64>,
    pub warmup_s: f64,
}

impl Default for RenderPlan {
    fn default() -> Self {
        Self {
            mode: PreviewMode::Full,
            start_s: None,
            end_s: None,
            warmup_s: 0.0,
        }
    }
}

impl RenderPlan {
    pub fn validate(self) -> Result<Self> {
        if !self.warmup_s.is_finite() || self.warmup_s < 0.0 {
            bail!("render warmup_s must be finite and non-negative")
        }
        match self.mode {
            PreviewMode::Full => {
                if self.start_s.is_some() || self.end_s.is_some() || self.warmup_s != 0.0 {
                    bail!("full render cannot include a bounded time window")
                }
            }
            PreviewMode::Frame => {
                let Some(start) = self.start_s else {
                    bail!("frame render requires start_s")
                };
                if !start.is_finite() || start < 0.0 {
                    bail!("frame start_s must be finite and non-negative")
                }
                if self.end_s.is_some() {
                    bail!("frame render accepts start_s and not end_s")
                }
            }
            PreviewMode::Segment => {
                let (Some(start), Some(end)) = (self.start_s, self.end_s) else {
                    bail!("segment render requires start_s and end_s")
                };
                if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start {
                    bail!("segment window must satisfy 0 <= start_s < end_s")
                }
            }
        }
        Ok(self)
    }

    pub fn output_frame_count(self, fps: u32) -> Result<u64> {
        if fps == 0 {
            bail!("render fps must be positive")
        }
        Ok(match self.mode {
            PreviewMode::Full => 0,
            PreviewMode::Frame => 1,
            PreviewMode::Segment => {
                ((self.end_s.unwrap() - self.start_s.unwrap()) * fps as f64).ceil() as u64
            }
        })
    }

    pub fn warmup_start_s(self) -> Option<f64> {
        self.start_s.map(|start| (start - self.warmup_s).max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::{PreviewMode, RenderPlan};

    #[test]
    fn default_is_full() {
        let plan = RenderPlan::default().validate().unwrap();
        assert_eq!(plan.mode, PreviewMode::Full);
        assert_eq!(plan.output_frame_count(30).unwrap(), 0);
    }

    #[test]
    fn frame_requires_one_time() {
        let plan = RenderPlan {
            mode: PreviewMode::Frame,
            start_s: Some(12.5),
            end_s: None,
            warmup_s: 1.0,
        }
        .validate()
        .unwrap();
        assert_eq!(plan.output_frame_count(30).unwrap(), 1);
        assert_eq!(plan.warmup_start_s(), Some(11.5));
    }

    #[test]
    fn segment_rounds_up_to_frame_grid() {
        let plan = RenderPlan {
            mode: PreviewMode::Segment,
            start_s: Some(1.0),
            end_s: Some(2.01),
            warmup_s: 0.5,
        }
        .validate()
        .unwrap();
        assert_eq!(plan.output_frame_count(30).unwrap(), 31);
        assert_eq!(plan.warmup_start_s(), Some(0.5));
    }

    #[test]
    fn malformed_bounded_plans_fail_closed() {
        let plan = RenderPlan {
            mode: PreviewMode::Segment,
            start_s: Some(4.0),
            end_s: Some(4.0),
            warmup_s: 0.0,
        };
        assert!(plan.validate().is_err());
    }
}
