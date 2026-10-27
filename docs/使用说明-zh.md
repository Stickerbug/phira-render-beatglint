# BeatGlint Phira 优化渲染器中文使用说明

本文说明当前公开候选中的 **Tauri-free headless 诊断渲染器**。它复用 Phira 的谱面、判定线和几何渲染代码，适合生成单帧 PNG 或短片段 MP4。当前实现从时间零开始固定帧回放，不能随机跳转，也不能替代完整 Phira AppImage 的最终视觉验收。

## 公开仓库边界

仓库只包含源码、构建文件、测试工具和不含私有输入的验证记录。请不要把谱面、音频、曲绘、皮肤、AppImage、迁移包、模型权重或渲染视频提交到仓库。运行时输入应由使用者自行准备，并在自己的 manifest 中记录 SHA-256。

当前 headless profile 不包含视频解码和实时音频设备；视频背景、粒子、命中特效、完整游戏音频和状态快照仍以完整 Phira 渲染器为参考。

## 环境要求

- Linux，能启动 Mesa/OpenGL 的 Xorg 或其他图形会话；
- Rust 1.99.1 或兼容的稳定版 Rust；
- FFmpeg，且 `ffmpeg`、`ffprobe` 在 `PATH` 中；
- Python 3 仅用于 manifest 验证工具；
- 一份合法的 Phira 资源目录，其中至少包含 `font.ttf`、`ending.mp3`、音效文件和皮肤资源。

如果使用无显示器服务器，需要自行准备 dummy Xorg。仓库不会携带私有的 Xorg 配置、ALSA 配置或用户资源。示例命令可以是：

```bash
export DISPLAY=:99
export ALSA_CONFIG_PATH=/path/to/alsa-null.conf
```

## 获取源码和构建

```bash
git clone https://github.com/Stickerbug/phira-render-beatglint.git
cd phira-render-beatglint
git checkout work/headless-frame-segment

export RUSTUP_HOME="$PWD/.rustup"
export CARGO_HOME="$PWD/.cargo"
export PATH="$CARGO_HOME/bin:$PATH"

cargo check --manifest-path upstream/headless-render/Cargo.toml
cargo test --manifest-path upstream/headless-render/Cargo.toml
cargo build --release --manifest-path upstream/headless-render/Cargo.toml
```

默认二进制为：

```text
upstream/headless-render/target/release/phira-render-headless
```

性能对照构建使用独立 profile，不会自动替换默认构建：

```bash
cargo build --profile release-fast \
  --manifest-path upstream/headless-render/Cargo.toml
```

## 运行单帧或短片段

渲染器通过标准输入接收两行 JSON：第一行是 `RenderParams`，第二行是输出路径。命令最后一个参数必须是资源目录：

```bash
printf '%s\n%s\n' \
  "$(cat params.json)" \
  '"/tmp/preview.mp4"' \
| DISPLAY=:99 \
  upstream/headless-render/target/release/phira-render-headless \
  render /path/to/phira-render/assets
```

bounded 配置字段示例：

```json
{
  "config": {
    "resolution": [640, 360],
    "fps": 30,
    "bitrate": "3500k",
    "renderMode": "segment",
    "stateMode": "replay",
    "startS": 40.3333333333,
    "endS": 41.3333333333,
    "warmupS": 0.0,
    "audioMode": "none"
  }
}
```

`startS` 和 `endS` 是 **renderer video/scene time**，不是谱面 nominal chart time。将官谱标注时间转换为 renderer 时间时使用：

```bash
python tools/convert_chart_window.py \
  --start-chart-s 36.7 --end-chart-s 37.7 --fps 30 \
  --readback-lag-frames 2
```

转换结果应随 artifact manifest 保存。不要在调用代码中直接写死 `+3.6`；完整 AppImage 和 bounded direct-readback 使用不同的 readback profile。

单帧输出时将第二行输出路径改成 `.png`，例如 `/tmp/frame.png`。`audioMode=music-only` 只适用于 segment，且只包含源音乐，不包含判定音效和 ending 音频。

## 验证结果

验证已有 smoke manifest，不需要重新渲染：

```bash
python tools/verify_headless_smoke.py \
  fixtures/manifests/headless-smoke-20261027.json \
  --renderer-binary \
  upstream/headless-render/target/release/phira-render-headless
```

比较两份已经生成的 artifact：

```bash
python tools/compare_render_artifacts.py \
  --left /path/to/left.mp4 \
  --right /path/to/right.mp4 \
  --manifest fixtures/manifests/headless-smoke-20261027.json \
  --case-index 3
```

该工具比较帧网格、视频流、逐帧 `framemd5` 和可用时的解码 PCM。两个 bounded 输出相同，只能说明两个 bounded 输出相同，不能证明与完整 Phira 或官谱视觉 parity。

## 公开发布前检查

1. 确认所有 vendored 源码的许可证和版权声明已经保留，并补齐第三方许可证目录。
2. 删除或替换只对私有工作区有效的路径、私有输入和私有 artifact 引用。
3. 重新生成不含私有资源的 smoke manifest，并检查二进制、源码和输入 SHA。
4. 在干净环境执行 `cargo check`、`cargo test`、release build 和 manifest verifier。
5. 明确标注 headless profile 的限制，不把它描述成完整 Phira parity 或随机 seek 实现。
6. 公开仓库前再确认主仓库的用户资源和网站部署目录没有被复制进来。

许可证应以源码审计结果为准。若仓库最终采用 GPL-3.0，应在根目录放置完整 GPL-3.0 文本并保留各上游组件原有许可证和版权声明；不能用一个新的许可证文件覆盖第三方原有声明。
