//! Explicit render-window protocol shared by full and bounded render modes.
//!
//! Bounded plans use the renderer's video-time grid. They remain explicitly
//! diagnostic: the headless profile omits audio/video backgrounds and full
//! state checkpointing, while the full plan remains the reference path.

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

/// Describes how scene state is obtained for a bounded request.
///
/// `Replay` is the only state mode implemented by the current renderer: it
/// advances the real scene from time zero with fixed frame steps.  `Stateless`
/// is part of the wire protocol so callers can distinguish a future direct
/// seek implementation from replay, but it fails closed until scene state
/// dependencies (judge, particles, effects and video) have an explicit
/// contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateMode {
    #[default]
    Replay,
    Stateless,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPlan {
    pub mode: PreviewMode,
    #[serde(default)]
    pub state_mode: StateMode,
    pub start_s: Option<f64>,
    pub end_s: Option<f64>,
    pub warmup_s: f64,
}

impl Default for RenderPlan {
    fn default() -> Self {
        Self {
            mode: PreviewMode::Full,
            state_mode: StateMode::Replay,
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
                if self.state_mode != StateMode::Replay {
                    bail!("full render requires replay state mode")
                }
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
                if self.state_mode == StateMode::Stateless {
                    bail!("stateless frame mode is reserved until direct scene seeking is implemented")
                }
            }
            PreviewMode::Segment => {
                let (Some(start), Some(end)) = (self.start_s, self.end_s) else {
                    bail!("segment render requires start_s and end_s")
                };
                if !start.is_finite() || !end.is_finite() || start < 0.0 || end <= start {
                    bail!("segment window must satisfy 0 <= start_s < end_s")
                }
                if self.state_mode == StateMode::Stateless {
                    bail!("stateless segment mode is reserved until direct scene seeking is implemented")
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
                let (start, end) = self.segment_frame_range(fps)?;
                end - start
            }
        })
    }

    /// Return the half-open frame-index range that intersects the requested
    /// video-time window. This keeps the manifest count identical to the
    /// frames actually emitted by the render loop for off-grid boundaries.
    pub fn segment_frame_range(self, fps: u32) -> Result<(u64, u64)> {
        if fps == 0 {
            bail!("render fps must be positive")
        }
        let start = (self
            .start_s
            .ok_or_else(|| anyhow::anyhow!("segment start_s is missing"))?
            * fps as f64)
            .ceil() as u64;
        let end = (self
            .end_s
            .ok_or_else(|| anyhow::anyhow!("segment end_s is missing"))?
            * fps as f64)
            .ceil() as u64;
        if end <= start {
            bail!("segment window contains no frame on the requested fps grid")
        }
        Ok((start, end))
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
            state_mode: StateMode::Replay,
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
            state_mode: StateMode::Replay,
            start_s: Some(1.0),
            end_s: Some(2.01),
            warmup_s: 0.5,
        }
        .validate()
        .unwrap();
        assert_eq!(plan.output_frame_count(30).unwrap(), 31);
        assert_eq!(plan.segment_frame_range(30).unwrap(), (30, 61));
        assert_eq!(plan.warmup_start_s(), Some(0.5));
    }

    #[test]
    fn off_grid_segment_uses_emitted_frame_range() {
        let plan = RenderPlan {
            mode: PreviewMode::Segment,
            state_mode: StateMode::Replay,
            start_s: Some(0.01),
            end_s: Some(0.02),
            warmup_s: 0.0,
        }
        .validate()
        .unwrap();
        assert!(plan.segment_frame_range(30).is_err());
    }

    #[test]
    fn malformed_bounded_plans_fail_closed() {
        let plan = RenderPlan {
            mode: PreviewMode::Segment,
            state_mode: StateMode::Replay,
            start_s: Some(4.0),
            end_s: Some(4.0),
            warmup_s: 0.0,
        };
        assert!(plan.validate().is_err());
    }

    #[test]
    fn stateless_bounded_requests_fail_closed_until_seek_contract_exists() {
        let plan = RenderPlan {
            mode: PreviewMode::Frame,
            state_mode: StateMode::Stateless,
            start_s: Some(2.0),
            end_s: None,
            warmup_s: 0.0,
        };
        let error = plan.validate().unwrap_err().to_string();
        assert!(error.contains("direct scene seeking"));
    }
}
