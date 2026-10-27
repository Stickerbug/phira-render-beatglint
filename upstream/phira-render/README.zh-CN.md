# phira-render（中文说明）

这是 Phira 图形渲染器的原始 Tauri/Vuetify 应用源码快照。公开仓库中的 BeatGlint headless 优化路径复用其中的渲染协议和 `render.rs`，但不包含私有谱面、资源、AppImage 或用户渲染结果。

原始 Tauri 应用适合完整 GUI 渲染；BeatGlint 的 bounded frame/segment 和性能优化请使用仓库根目录的 `upstream/headless-render`。

原始构建命令：

```bash
cargo tauri dev
cargo tauri build
```

许可证和第三方归属见仓库根目录的 `LICENSE`、`LICENSES/` 和 `THIRD_PARTY_NOTICES.md`。原始 English README：[`README.md`](README.md)。
