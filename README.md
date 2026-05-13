# WallpaperRS

A high-performance animated wallpaper engine for Windows, built with Rust + WGPU.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11-blue?logo=windows)
![License](https://img.shields.io/badge/License-MIT-green)
[![Release](https://img.shields.io/github/v/release/otimtt/WallpaperRS?label=latest%20release)](https://github.com/otimtt/WallpaperRS/releases/latest)

---

## Download

**[⬇ Download latest release](https://github.com/otimtt/WallpaperRS/releases/latest)**

1. Download `WallpaperRS-vX.X.X-windows-x64.zip`
2. Extract anywhere
3. Run `wallpaper-rs.exe`

No installation required.

**Requirements**
- Windows 10 / 11 (64-bit)
- GPU with DirectX 12 support
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) — only needed for web wallpapers, already included in Windows 11

---

## Features

| | |
|---|---|
| **WGSL Shaders** | Real-time animated shaders rendered directly on the desktop |
| **Video** | MP4 / WebM playback via Windows Media Foundation, seamless looping |
| **Particles** | 8 000 GPU-instanced particles with additive blending |
| **Web** | HTML / JS / CSS pages embedded via WebView2 |
| **Auto-pause** | Detects fullscreen games and pauses automatically |
| **System Tray** | Control the wallpaper without opening the main window |
| **Performance** | Target FPS, GPU quality level, battery-saving mode |

---

## Usage

### Adding wallpapers

| Type | How to add |
|------|-----------|
| **Shader** | Point to a `.wgsl` file. The built-in *Nebula Crimson* and *Dark Pulse* shaders are bundled. |
| **Video** | Point to any `.mp4` or `.webm` file on your machine. |
| **Scene** | Select *Particle Storm* — no file needed, fully procedural. |
| **Web** | Enter any URL (e.g. `https://example.com/animation`). Requires WebView2. |

### Keyboard / Tray shortcuts

| Action | How |
|--------|-----|
| Pause / Resume | Tray icon → *Pausar* or the button in the UI |
| Stop wallpaper | *Parar* button in the UI |
| Quit | Tray icon → *Sair* |

---

## Building from source

```bash
# 1. Install Rust  →  https://rustup.rs
# 2. Clone
git clone https://github.com/otimtt/WallpaperRS.git
cd WallpaperRS

# 3. Build (requires Windows — cross-compilation not supported)
cargo build --release
# Output: target/release/wallpaper-rs.exe
```

> **Note:** the project checks cleanly on Linux (`cargo check`) but must be compiled on Windows to produce a working binary, since it depends on DirectX 12, Windows Media Foundation, and WebView2.

---

## How it works

1. A Win32 popup window is created and injected into the Windows desktop via the **WorkerW** technique — it sits below the desktop icons but above the wallpaper background. This is the same approach used by Wallpaper Engine.
2. A **WGPU** surface is attached to that window and the selected renderer runs in a dedicated background thread.
3. The **egui** control panel runs in the main thread and communicates with the render thread via atomics (pause, stop, FPS target).
4. A fullscreen check runs every second; if a fullscreen window is detected the wallpaper pauses automatically to save GPU resources.

---

## Workspace structure

```
crates/
  core/               — Renderer trait, WallpaperEngine, AppConfig
  platform-win/       — WorkerW injection, Win32 window, fullscreen detection
  renderer-shader/    — WGSL fullscreen quad (time + resolution uniforms)
  renderer-video/     — Windows Media Foundation IMFSourceReader decoder
  renderer-scene/     — 8 000-particle CPU→GPU simulation, instanced draw
  renderer-web/       — WebView2 child-window embedding
  app/                — Binary: egui UI, system tray, render thread orchestration
assets/
  shaders/            — default.wgsl, particles.wgsl, textured.wgsl
  icon.png            — Application icon (embedded in the .exe at build time)
```

---

## Tech stack

| Component | Library |
|-----------|---------|
| Rendering | [wgpu 22](https://github.com/gfx-rs/wgpu) (DX12 / Vulkan backend) |
| UI | [egui 0.29](https://github.com/emilk/egui) + eframe |
| Win32 / COM | [windows-rs 0.58](https://github.com/microsoft/windows-rs) |
| Video | Windows Media Foundation (`IMFSourceReader`) |
| Web | [webview2-com 0.35](https://github.com/wravery/webview2-rs) |
| Tray | [tray-icon 0.19](https://github.com/tauri-apps/tray-icon) |

---

## License

MIT — see [LICENSE](LICENSE) for details.
