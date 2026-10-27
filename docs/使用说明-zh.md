# BeatGlint Phira 优化渲染器中文使用说明

本文说明当前公开候选中的 **Tauri-free headless 诊断渲染器**。它复用 Phira 的谱面、判定线和几何渲染代码，适合生成单帧 PNG 或短片段 MP4。当前实现从时间零开始固定帧回放，不能随机跳转，也不能替代完整 Phira AppImage 的最终视觉验收。

## 公开仓库边界

仓库只包含源码、构建文件、测试工具和不含私有输入的验证记录。请不要把谱面、音频、曲绘、皮肤、AppImage、迁移包、模型权重或渲染视频提交到仓库。运行时输入应由使用者自行准备，并在自己的 manifest 中记录 SHA-256。

当前 headless profile 不包含视频解码和实时音频设备；视频背景、粒子、命中特效、完整游戏音频和状态快照仍以完整 Phira 渲染器为参考。

性能实测数据和后续优化边界见 [`docs/性能对比-20261027.md`](性能对比-20261027.md)。

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
git checkout main

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

## Resident 预览主模式（顺序窗口）

需要在一次场景回放中导出多个预览窗口时，将 `residentWindows` 作为预览请求的主输出列表。它复用一个已加载的场景和 GL context，按时间顺序把每个窗口写入独立的 FFmpeg 输出；因此适合批量预览和审查窗口，避免为每个窗口重复启动 renderer。

请求仍通过标准输入的两行协议发送：第一行是完整 JSON 请求，第二行是兼容旧调用方的占位输出路径。设置 `residentWindows` 后，第二行会被忽略；每个窗口的 `output` 才是实际产物路径。

```json
{
  "renderMode": "full",
  "stateMode": "replay",
  "audioMode": "none",
  "residentWindows": [
    {"startS": 40.0, "endS": 41.0, "output": "/tmp/window-a.mp4"},
    {"startS": 41.0, "endS": 42.0, "output": "/tmp/window-b.mp4"}
  ]
}
```

字段和约束如下：

| 字段 | 类型 | 要求 |
| --- | --- | --- |
| `renderMode` | 字符串 | 必须为 `full`；resident 自己决定要输出哪些窗口 |
| `stateMode` | 字符串 | 必须为 `replay`；从时间零按固定帧步长推进 |
| `audioMode` | 字符串 | 必须为 `none`；每个窗口为静音 MP4 |
| `residentWindows` | 数组 | 至少一个窗口，按 `startS` 递增且不能重叠 |
| `startS` / `endS` | 有限非负数 | 必须满足 `0 <= startS < endS`，单位为 renderer scene 秒 |
| `output` | 路径字符串 | 每个窗口唯一的 MP4 路径；父目录需预先存在 |

渲染器只回放一次到最大 `endS`，窗口采用半开区间 `[ceil(startS*fps), ceil(endS*fps))` 分配帧。相邻窗口可以使用相同边界（例如 `40..41` 和 `41..42`），不会重复写入边界帧。窗口不是随机跳转或 checkpoint；视频背景、完整粒子/命中特效和完整音频仍需用完整 Phira/AppImage 验收。窗口的时间、顺序、重叠和路径字段会在启动渲染前校验；输出父目录请预先创建，不应把部分输出当作成功结果。

`startS` 和 `endS` 是 **renderer video/scene time**，不是谱面 nominal chart time。将官谱标注时间转换为 renderer 时间时使用：

```bash
python tools/convert_chart_window.py \
  --start-chart-s 36.7 --end-chart-s 37.7 --fps 30 \
  --readback-lag-frames 2
```

转换结果应随 artifact manifest 保存。不要在调用代码中直接写死 `+3.6`；完整 AppImage 和 bounded direct-readback 使用不同的 readback profile。

单帧输出时将第二行输出路径改成 `.png`，例如 `/tmp/frame.png`。`audioMode=music-only` 只适用于 segment，且只包含源音乐，不包含判定音效和 ending 音频。

## 验证结果

验证已有 smoke manifest 不需要重新渲染。公开仓库不附带私有输入和 smoke manifest，运行时请把你自己的 manifest 路径传给工具：

```bash
python tools/verify_headless_smoke.py \
  /path/to/your/headless-smoke.json \
  --renderer-binary \
  upstream/headless-render/target/release/phira-render-headless
```

比较两份已经生成的 artifact：

```bash
python tools/compare_render_artifacts.py \
  --left /path/to/left.mp4 \
  --right /path/to/right.mp4 \
  --manifest /path/to/your/headless-smoke.json \
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
