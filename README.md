# WallpaperRS

A high-performance animated wallpaper engine for Windows, built with Rust + WGPU.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Windows-blue?logo=windows)
![License](https://img.shields.io/badge/License-MIT-green)

## Download

> **[Releases →](../../releases/latest)**  Download the latest `.zip`, extract, and run `wallpaper-rs.exe`. No installation required.

**Requirements:** Windows 10/11 · DirectX 12 GPU · [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (pre-installed on Windows 11; for web wallpapers only)

## Features

- **WGSL Shaders** — real-time animated shaders rendered directly to the desktop
- **Video Wallpapers** — MP4 / WebM decoding via Windows Media Foundation, with seamless looping
- **Particle Scenes** — 8 000 GPU-instanced particles with additive blending
- **Web Wallpapers** — HTML / JS / CSS pages embedded via WebView2
- **Auto-pause** — detects fullscreen games/apps and pauses rendering automatically
- **System Tray** — show, pause, or quit from the tray without opening the UI
- **Fine-grained controls** — target FPS, GPU quality, battery saving mode

## Stack

| Component | Technology |
|-----------|-----------|
| Rendering | [WGPU 22](https://github.com/gfx-rs/wgpu) (DirectX 12 / Vulkan) |
| UI | [egui 0.29](https://github.com/emilk/egui) + eframe |
| Desktop injection | Win32 WorkerW technique |
| Video decoding | Windows Media Foundation (`IMFSourceReader`) |
| Web content | WebView2 (`webview2-com`) |
| Language | Rust 1.75+ |

## Requirements

- Windows 10 / 11 (64-bit)
- DirectX 12 capable GPU
- [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (for web wallpapers)

## Building

```bash
# Install Rust (https://rustup.rs)
git clone https://github.com/YOUR_USERNAME/WallpaperRS
cd WallpaperRS
cargo build --release
```

The compiled binary will be at `target/release/wallpaper-rs.exe`.

## Workspace structure

```
crates/
  core/               — Renderer trait, WallpaperEngine, AppConfig
  platform-win/       — WorkerW injection, Win32 window, fullscreen detection
  renderer-shader/    — WGSL fullscreen quad with time/resolution uniforms
  renderer-video/     — Windows Media Foundation video decoder
  renderer-scene/     — 8 000 particle simulation (instanced rendering)
  renderer-web/       — WebView2 embedded browser
  app/                — Main binary: egui UI, tray icon, render thread
```

## How it works

1. The app creates an invisible Win32 popup window.
2. It injects that window into the Windows desktop by parenting it to the **WorkerW** layer (below desktop icons, above the desktop background) — the same technique used by Wallpaper Engine.
3. A WGPU surface is attached to the window and the selected renderer runs in a dedicated thread.
4. The egui control panel runs in the main thread and communicates with the render thread via atomics.

## License

MIT
