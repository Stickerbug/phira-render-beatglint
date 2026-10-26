// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{internal_id, MSRenderTarget, NoteKind},
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, GameMode, GameScene, LoadingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
#[cfg(feature = "headless-render")]
use prpr::audio::AudioClip;
#[cfg(not(feature = "headless-render"))]
use sasa::AudioClip;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use std::{ffi::OsStr, fmt::Write as _};
use tempfile::NamedTempFile;

use crate::render_plan::{PreviewMode, RenderPlan, StateMode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BoundedAudioMode {
    #[default]
    None,
    MusicOnly,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderConfig {
    resolution: (u32, u32),
    ending_length: f64,
    fps: u32,
    hardware_accel: bool,
    bitrate: String,

    aggressive: bool,
    challenge_color: ChallengeModeColor,
    challenge_rank: u32,
    disable_effect: bool,
    double_hint: bool,
    fxaa: bool,
    note_scale: f32,
    particle: bool,
    player_avatar: Option<String>,
    player_name: String,
    player_rks: f32,
    sample_count: u32,
    res_pack_path: Option<String>,
    speed: f32,
    volume_music: f32,
    volume_sfx: f32,

    /// Bounded values select the diagnostic headless frame/segment path. They
    /// remain separate from full-faithful output and require explicit mode
    /// metadata at the caller boundary.
    #[serde(default)]
    render_mode: PreviewMode,
    /// Scene-state strategy for bounded output. Replay is currently the only
    /// executable strategy; stateless direct seeking fails closed in
    /// `RenderPlan::validate` until all history-dependent systems have a
    /// defined seek contract.
    #[serde(default)]
    state_mode: StateMode,
    #[serde(default)]
    start_s: Option<f64>,
    #[serde(default)]
    end_s: Option<f64>,
    #[serde(default)]
    warmup_s: f64,
    /// Optional audio for bounded segments. The first supported mode is
    /// music-only; gameplay SFX remain reserved for the full-faithful path.
    #[serde(default)]
    audio_mode: BoundedAudioMode,
}

impl RenderConfig {
    pub fn to_config(&self) -> Config {
        Config {
            aggressive: self.aggressive,
            challenge_color: self.challenge_color.clone(),
            challenge_rank: self.challenge_rank,
            disable_effect: self.disable_effect,
            double_hint: self.double_hint,
            fxaa: self.fxaa,
            note_scale: self.note_scale,
            particle: self.particle,
            player_name: self.player_name.clone(),
            player_rks: self.player_rks,
            sample_count: self.sample_count,
            res_pack_path: self.res_pack_path.clone(),
            speed: self.speed,
            volume_music: self.volume_music,
            volume_sfx: self.volume_sfx,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParams {
    pub path: PathBuf,
    pub info: ChartInfo,
    pub config: RenderConfig,
}

#[derive(Serialize, Deserialize)]
pub enum IPCEvent {
    StartMixing,
    StartRender(u64),
    Frame,
    Done(f64),
}

pub async fn build_player(config: &RenderConfig) -> Result<BasicPlayer> {
    Ok(BasicPlayer {
        avatar: if let Some(path) = &config.player_avatar {
            Some(
                Texture2D::from_file_with_format(
                    &tokio::fs::read(path)
                        .await
                        .with_context(|| tl!("load-avatar-failed"))?,
                    None,
                )
                .into(),
            )
        } else {
            None
        },
        id: 0,
        rks: config.player_rks,
    })
}

fn cmd_hidden(program: impl AsRef<OsStr>) -> Command {
    let cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = cmd;
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    cmd
}

pub fn find_ffmpeg() -> Result<Option<String>> {
    fn test(path: impl AsRef<OsStr>) -> bool {
        matches!(cmd_hidden(path).arg("-version").output(), Ok(_))
    }
    if test("ffmpeg") {
        return Ok(Some("ffmpeg".to_owned()));
    }
    eprintln!("Failed to find global ffmpeg. Using bundled ffmpeg");
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let ffmpeg = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let ffmpeg = exe_dir.join(ffmpeg);
    Ok(if test(&ffmpeg) {
        Some(ffmpeg.display().to_string())
    } else {
        None
    })
}

pub async fn main() -> Result<()> {
    use crate::ipc::client::*;

    let assets_path = std::env::args().last().context("renderer assets path is required")?;
    set_pc_assets_folder(&assets_path);

    let mut stdin = std::io::stdin().lock();
    let stdin = &mut stdin;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let params: RenderParams = serde_json::from_str(line.trim())?;
    let plan = RenderPlan {
        mode: params.config.render_mode,
        state_mode: params.config.state_mode,
        start_s: params.config.start_s,
        end_s: params.config.end_s,
        warmup_s: params.config.warmup_s,
    }
    .validate()?;
    let bounded = plan.mode != PreviewMode::Full;
    if bounded && plan.warmup_s != 0.0 {
        bail!("non-zero warmup is reserved until state checkpoints are implemented")
    }
    if params.config.audio_mode != BoundedAudioMode::None
        && (plan.mode != PreviewMode::Segment || plan.state_mode != StateMode::Replay)
    {
        bail!("bounded audio requires segment mode with replay state")
    }
    let path = params.path;

    line.clear();
    stdin.read_line(&mut line)?;
    let output_path: PathBuf = serde_json::from_str(line.trim())?;

    let mut fs = fs::fs_from_file(&path)?;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;

    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };
    dbg!(&ffmpeg);

    let mut painter = TextPainter::new(font);

    let mut config = params.config.to_config();
    config.mods = Mods::AUTOPLAY;

    let info = params.info;

    let (chart, ..) = GameScene::load_chart(fs.deref_mut(), &info)
        .await
        .with_context(|| tl!("load-chart-failed"))?;
    macro_rules! ld {
            ($path:literal) => {
                AudioClip::new(load_file($path).await?)
                    .with_context(|| tl!("load-sfx-failed", "name" => $path))?
            };
        }
    let music: Result<_> = async { AudioClip::new(fs.load_file(&info.music).await?) }.await;
    let music = music.with_context(|| tl!("load-music-failed"))?;
    let track_length = music.length() as f64;

    let mut gl = unsafe { get_internal_gl() };

    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);

    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = if bounded {
        plan.end_s.or(plan.start_s).unwrap()
    } else {
        O + length + A + params.config.ending_length
    };
    let offset = chart.offset.max(0.);

    let render_start_time = Instant::now();

    let mixing_output = if bounded { None } else { Some(NamedTempFile::new()?) };
    if !bounded {
        let ending = ld!("ending.mp3");
        let sfx_click = ld!("click.ogg");
        let sfx_drag = ld!("drag.ogg");
        let sfx_flick = ld!("flick.ogg");
        let sample_rate = 44100;
        assert_eq!(sample_rate, ending.sample_rate());
        assert_eq!(sample_rate, sfx_click.sample_rate());
        assert_eq!(sample_rate, sfx_drag.sample_rate());
        assert_eq!(sample_rate, sfx_flick.sample_rate());
        let mut mix_proc = cmd_hidden(&ffmpeg)
            .args("-y -f f32le -ar 44100 -ac 2 -i - -c:a mp3 -f mp3".split_whitespace())
            .arg(mixing_output.as_ref().unwrap().path())
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?;
        let input = mix_proc.stdin.as_mut().unwrap();
        let mut writer = BufWriter::new(input);
        // Keep only one bounded mix chunk resident. The old implementation
        // materialized the whole song as interleaved f32 samples (about 84 MiB
        // for a four-minute stereo track) before starting FFmpeg.
        const MIX_CHUNK_FRAMES: usize = 8192;
        const MIX_WRITE_CHUNK_BYTES: usize = 64 * 1024;
        let total_frames = (video_length * sample_rate as f64).ceil() as usize;
        let music_start = (O - chart.offset.min(0.) as f64) * sample_rate as f64;
        let music_start_frame = music_start.round() as i64;
        let music_frames = (music.length() as f64 * sample_rate as f64) as i64;
        let ending_start_frame = ((O + length + A) * sample_rate as f64).round() as i64;
        let ending_step = ending.frame_count() as i64;

        for chunk_start in (0..total_frames).step_by(MIX_CHUNK_FRAMES) {
            let chunk_frames = (total_frames - chunk_start).min(MIX_CHUNK_FRAMES);
            let mut chunk = vec![0.0_f32; chunk_frames * 2];
            for local in 0..chunk_frames {
                let absolute = chunk_start as i64 + local as i64;
                let music_index = absolute - music_start_frame;
                if (0..music_frames).contains(&music_index) {
                    let frame = music.sample(music_index as f32 / sample_rate as f32).unwrap_or_default();
                    chunk[local * 2] += frame.0 * volume_music;
                    chunk[local * 2 + 1] += frame.1 * volume_music;
                }
            }
            let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
                let event_start = (pos * sample_rate as f64).round() as i64;
                if event_start < 0 {
                    return;
                }
                let source_start = (chunk_start as i64 - event_start).max(0) as usize;
                let destination_start = (event_start - chunk_start as i64).max(0) as usize;
                if source_start >= clip.frame_count() || destination_start >= chunk_frames {
                    return;
                }
                let count = (chunk_frames - destination_start).min(clip.frame_count() - source_start);
                for index in 0..count {
                    let frame = clip.frames()[source_start + index];
                    let destination = (destination_start + index) * 2;
                    chunk[destination] += frame.0 * volume;
                    chunk[destination + 1] += frame.1 * volume;
                }
            };
            for note in chart.lines.iter().flat_map(|it| it.notes.iter()).filter(|it| !it.fake) {
                place(
                    O + note.time as f64 + offset as f64,
                    match note.kind {
                        NoteKind::Click | NoteKind::Hold { .. } => &sfx_click,
                        NoteKind::Drag => &sfx_drag,
                        NoteKind::Flick => &sfx_flick,
                    },
                    volume_sfx,
                );
            }
            if ending_step > 0 {
                let mut start = ending_start_frame;
                while start < total_frames as i64 {
                    place(start as f64 / sample_rate as f64, &ending, volume_music);
                    start += ending_step;
                }
            }
            let mut encoded = Vec::with_capacity(MIX_WRITE_CHUNK_BYTES);
            for sample in chunk {
                encoded.extend_from_slice(&sample.to_le_bytes());
                if encoded.len() >= MIX_WRITE_CHUNK_BYTES {
                    writer.write_all(&encoded)?;
                    encoded.clear();
                }
            }
            if !encoded.is_empty() {
                writer.write_all(&encoded)?;
            }
        }
        drop(writer);
        mix_proc.wait()?;
    }

    let (vw, vh) = params.config.resolution;
    let mst = Rc::new(MSRenderTarget::new((vw, vh), config.sample_count));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    static MSAA: AtomicBool = AtomicBool::new(false);
    let player = build_player(&params.config).await?;
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(
                GameMode::Normal,
                info,
                config,
                fs,
                Some(player),
                None,
                None,
            )
            .await?,
        ),
        tm,
        {
            let mut cnt = 0;
            let mst = Rc::clone(&mst);
            move || {
                cnt += 1;
                if cnt == 1 || cnt == 3 {
                    MSAA.store(true, Ordering::SeqCst);
                    Some(mst.input())
                } else {
                    MSAA.store(false, Ordering::SeqCst);
                    Some(mst.output())
                }
            }
        },
    )
    .await?;
    main.top_level = false;
    main.viewport = Some((0, 0, vw as _, vh as _));

    const O: f64 = LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_TIME as f64;
    const A: f64 = 0.7 + 0.3 + 0.4;

    let fps = params.config.fps;
    let frame_delta = 1. / fps as f32;

    // A bounded music-only segment uses the same absolute scene clock as the
    // full mixer, but writes only the emitted frame-grid window. Keep the raw
    // temporary alive until FFmpeg exits; frame output stays silent.
    let mut bounded_audio_file = None;
    if bounded && params.config.audio_mode == BoundedAudioMode::MusicOnly {
        let (start_frame, end_frame) = plan.segment_frame_range(fps)?;
        let scene_start = start_frame as f64 / fps as f64;
        let duration = (end_frame - start_frame) as f64 / fps as f64;
        let sample_rate = 44_100_u32;
        let music_start_scene = O - chart.offset.min(0.) as f64;
        let sample_count = (duration * sample_rate as f64).round() as usize;
        let audio = NamedTempFile::new()?;
        let mut writer = BufWriter::new(audio.as_file());
        for sample_index in 0..sample_count {
            let scene_time = scene_start + sample_index as f64 / sample_rate as f64;
            let position = scene_time - music_start_scene;
            let frame = if position >= 0.0 && position < music.length() as f64 {
                music.sample(position as f32).unwrap_or_default()
            } else {
                Default::default()
            };
            writer.write_all(&(frame.0 * volume_music).to_le_bytes())?;
            writer.write_all(&(frame.1 * volume_music).to_le_bytes())?;
        }
        writer.flush()?;
        drop(writer);
        bounded_audio_file = Some(audio);
    }

    let mut proc = if bounded {
        let frame_png = plan.mode == PreviewMode::Frame
            && output_path.extension().and_then(|it| it.to_str()).is_some_and(|it| it.eq_ignore_ascii_case("png"));
        let mut command = cmd_hidden(&ffmpeg);
        command.args([
            "-y", "-f", "rawvideo", "-c:v", "rawvideo", "-pix_fmt", "rgba",
            "-s", &format!("{vw}x{vh}"), "-r", &fps.to_string(), "-i", "-",
        ]);
        if frame_png {
            command.args(["-frames:v", "1", "-vf", "vflip", "-f", "image2"]);
        } else {
            if let Some(audio) = bounded_audio_file.as_ref() {
                command.args([
                    "-f", "f32le", "-ar", "44100", "-ac", "2", "-i",
                    audio.path().to_str().context("audio path is not UTF-8")?,
                    "-map", "0:v:0", "-map", "1:a:0", "-c:v", "libx264", "-pix_fmt",
                    "yuv420p", "-b:v", &params.config.bitrate, "-c:a", "aac", "-shortest",
                    "-muxdelay", "0", "-muxpreload", "0", "-vf", "vflip", "-f", "mp4",
                ]);
            } else {
                command.args([
                    "-c:v", "libx264", "-pix_fmt", "yuv420p", "-b:v", &params.config.bitrate,
                    "-vf", "vflip", "-an", "-f", "mp4",
                ]);
            }
        }
        command
            .arg(output_path)
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?
    } else {
        let codecs = String::from_utf8(
            cmd_hidden(&ffmpeg)
                .arg("-codecs")
                .output()
                .with_context(|| tl!("run-ffmpeg-failed"))?
                .stdout,
        )?;
        let use_cuda = params.config.hardware_accel && codecs.contains("h264_nvenc");
        let has_qsv = params.config.hardware_accel && codecs.contains("h264_qsv");
        let mut args = "-y -f rawvideo -c:v rawvideo".to_owned();
        if use_cuda { args += " -hwaccel_output_format cuda"; }
        write!(&mut args, " -s {vw}x{vh} -r {fps} -pix_fmt rgba -i - -i")?;
        let args2 = format!(
            "-c:a copy -c:v {} -pix_fmt yuv420p -b:v {} -map 0:v:0 -map 1:a:0 -vf vflip -f mp4",
            if use_cuda { "h264_nvenc" } else if has_qsv { "h264_qsv" } else if params.config.hardware_accel { bail!(tl!("no-hw-acc")); } else { "libx264" },
            params.config.bitrate,
        );
        cmd_hidden(&ffmpeg)
            .args(args.split_whitespace())
            .arg(mixing_output.as_ref().unwrap().path())
            .args(args2.split_whitespace())
            .arg(output_path)
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?
    };
    let mut input = proc.stdin.take().unwrap();

    let byte_size = vw as usize * vh as usize * 4;

    let frames = if bounded {
        (video_length / frame_delta as f64).ceil() as u64 + 1
    } else {
        (video_length / frame_delta as f64).ceil() as u64
    };
    let output_frames = match plan.mode {
        PreviewMode::Full => frames,
        PreviewMode::Frame => 1,
        PreviewMode::Segment => plan.output_frame_count(fps)?,
    };

    const N: usize = 3;
    let mut pbos: [GLuint; N] = [0; N];
    if !bounded { unsafe {
        use miniquad::gl::*;
        glGenBuffers(N as _, pbos.as_mut_ptr());
        for pbo in pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo);
            glBufferData(
                GL_PIXEL_PACK_BUFFER,
                (vw as u64 * vh as u64 * 4) as _,
                std::ptr::null(),
                GL_STREAM_READ,
            );
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    }}

    send(IPCEvent::StartRender(output_frames));

    let emit_pbo = |pbo: GLuint, input: &mut dyn Write| -> Result<()> {
        unsafe {
            use miniquad::gl::*;
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo);
            let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            if src.is_null() {
                bail!("failed to map completed pixel-readback buffer")
            }
            input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
            glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
        }
        send(IPCEvent::Frame);
        Ok(())
    };

    for frame in 0..frames {
        *my_time.borrow_mut() = (frame as f32 * frame_delta).max(0.) as f64;
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        clear_background(BLACK);
        main.viewport = Some((0, 0, vw as _, vh as _));
        main.update()?;
        main.render(&mut painter)?;
        // TODO magic. can't remove this line.
        draw_rectangle(0., 0., 0., 0., Color::default());
        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }
        if !bounded {
            unsafe {
                use miniquad::gl::*;
                let tex = mst.output().texture.raw_miniquad_texture_handle();
                glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));
                glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame as usize % N]);
                glReadPixels(0, 0, tex.width as _, tex.height as _, GL_RGBA, GL_UNSIGNED_BYTE, std::ptr::null_mut());
            }
            if frame + 1 >= N as u64 {
                emit_pbo(pbos[(frame + 1) as usize % N], &mut input)?;
            }
        } else {
            let emit = match plan.mode {
                PreviewMode::Frame => {
                    let target_frame = (plan.start_s.unwrap() * fps as f64).round() as u64;
                    frame == target_frame
                }
                PreviewMode::Segment => {
                    let (start_frame, end_frame) = plan.segment_frame_range(fps)?;
                    frame >= start_frame && frame < end_frame
                }
                PreviewMode::Full => false,
            };
            if emit {
                let mut rgba = vec![0_u8; byte_size];
                unsafe {
                    use miniquad::gl::*;
                    let tex = mst.output().texture.raw_miniquad_texture_handle();
                    glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));
                    glReadPixels(0, 0, tex.width as _, tex.height as _, GL_RGBA, GL_UNSIGNED_BYTE, rgba.as_mut_ptr() as *mut _);
                }
                input.write_all(&rgba)?;
                send(IPCEvent::Frame);
            }
        }
    }
    if !bounded {
        // The final N-1 readbacks are still resident in their PBOs. Drain them
        // in frame order so the first and last frames are not silently lost.
        let pending_start = frames.saturating_sub((N - 1) as u64);
        for pending in pending_start..frames {
            emit_pbo(pbos[pending as usize % N], &mut input)?;
        }
    }
    drop(input);
    proc.wait()?;

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}
