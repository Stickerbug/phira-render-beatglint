//! Deterministic no-device audio types used by the headless renderer profile.
//!
//! The full profile continues to use sasa. This module keeps chart timing and
//! audio-clip decoding available without opening ALSA/CPAL devices.

use anyhow::{anyhow, bail, Result};
use std::{io::Cursor, sync::Arc};
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    io::MediaSourceStream,
};

#[derive(Clone, Copy, Default)]
pub struct Frame(pub f32, pub f32);

impl Frame {
    pub fn avg(self) -> f32 {
        (self.0 + self.1) / 2.
    }

    pub fn interpolate(self, other: Self, factor: f32) -> Self {
        Self(
            self.0 + (other.0 - self.0) * factor,
            self.1 + (other.1 - self.1) * factor,
        )
    }
}

#[derive(Clone)]
pub struct AudioClip {
    frames: Arc<Vec<Frame>>,
    sample_rate: u32,
}

impl AudioClip {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let (frames, sample_rate) = decode(data)?;
        Ok(Self { frames: Arc::new(frames), sample_rate })
    }

    pub fn sample(&self, position: f32) -> Option<Frame> {
        let position = position * self.sample_rate as f32;
        let index = position as usize;
        let frame = self.frames.get(index).copied()?;
        let next = self.frames.get(index + 1).copied().unwrap_or(frame);
        Some(frame.interpolate(next, position - index as f32))
    }

    pub fn frames(&self) -> &[Frame] {
        self.frames.as_slice()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn length(&self) -> f32 {
        self.frame_count() as f32 / self.sample_rate() as f32
    }
}

#[derive(Debug, Clone)]
pub struct PlaySfxParams {
    pub amplifier: f32,
}

impl Default for PlaySfxParams {
    fn default() -> Self { Self { amplifier: 1.0 } }
}

#[derive(Clone, Default)]
pub struct Sfx;

impl Sfx {
    pub fn play(&mut self, _params: PlaySfxParams) -> Result<()> { Ok(()) }
}

#[derive(Debug, Clone)]
pub struct MusicParams {
    pub loop_mix_time: f32,
    pub amplifier: f32,
    pub playback_rate: f32,
}

impl Default for MusicParams {
    fn default() -> Self {
        Self { loop_mix_time: -1.0, amplifier: 1.0, playback_rate: 1.0 }
    }
}

pub struct Music {
    position: f32,
    paused: bool,
    _clip: AudioClip,
    _params: MusicParams,
}

impl Music {
    pub fn pause(&mut self) -> Result<()> { self.paused = true; Ok(()) }
    pub fn play(&mut self) -> Result<()> { self.paused = false; Ok(()) }
    pub fn seek_to(&mut self, position: f32) -> Result<()> { self.position = position; Ok(()) }
    pub fn position(&self) -> f32 { self.position }
    pub fn paused(&self) -> bool { self.paused }
}

#[derive(Default)]
pub struct AudioManager;

impl AudioManager {
    pub fn new() -> Result<Self> { Ok(Self) }
    pub fn create_sfx(&mut self, _clip: AudioClip, _buffer_size: Option<usize>) -> Result<Sfx> { Ok(Sfx) }
    pub fn create_music(&mut self, clip: AudioClip, params: MusicParams) -> Result<Music> {
        Ok(Music { position: 0.0, paused: true, _clip: clip, _params: params })
    }
    pub fn recover_if_needed(&mut self) -> Result<()> { Ok(()) }
}

fn decode(data: Vec<u8>) -> Result<(Vec<Frame>, u32)> {
    fn append_buffer(frames: &mut Vec<Frame>, buffer: &symphonia::core::audio::AudioBuffer<f32>) {
        match buffer.spec().channels.count() {
            1 => frames.extend(buffer.chan(0).iter().copied().map(|value| Frame(value, value))),
            _ => frames.extend(buffer.chan(0).iter().zip(buffer.chan(1)).map(|(left, right)| Frame(*left, *right))),
        }
    }
    fn append_ref(frames: &mut Vec<Frame>, buffer: &AudioBufferRef<'_>) -> Result<()> {
        macro_rules! convert { ($buffer:ident) => {{
            let mut dest = symphonia::core::audio::AudioBuffer::new($buffer.capacity() as u64, $buffer.spec().clone());
            $buffer.convert(&mut dest);
            append_buffer(frames, &dest);
        }}; }
        use AudioBufferRef::*;
        match buffer {
            F32(buffer) => append_buffer(frames, buffer), U8(buffer) => convert!(buffer),
            U16(buffer) => convert!(buffer), U24(buffer) => convert!(buffer), U32(buffer) => convert!(buffer),
            S8(buffer) => convert!(buffer), S16(buffer) => convert!(buffer), S24(buffer) => convert!(buffer),
            S32(buffer) => convert!(buffer), F64(buffer) => convert!(buffer),
        }
        Ok(())
    }
    let mut reader = symphonia::default::get_probe()
        .format(&Default::default(), MediaSourceStream::new(Box::new(Cursor::new(data)), Default::default()), &Default::default(), &Default::default())?
        .format;
    let track = reader.default_track().ok_or_else(|| anyhow!("default track not found"))?;
    let sample_rate = track.codec_params.sample_rate.ok_or_else(|| anyhow!("unknown sample rate"))?;
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &Default::default())?;
    let mut frames = Vec::new();
    loop {
        match reader.next_packet() {
            Ok(packet) => append_ref(&mut frames, &decoder.decode(&packet)?)?,
            Err(symphonia::core::errors::Error::IoError(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(error) => bail!(error),
        }
    }
    Ok((frames, sample_rate))
}
