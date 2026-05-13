use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_wallpaper: Option<WallpaperEntry>,
    pub library_path: PathBuf,
    pub performance: PerformanceConfig,
    pub display: DisplayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_wallpaper: None,
            library_path: dirs_path(),
            performance: PerformanceConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

fn dirs_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    return PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
        .join("WallpaperRS")
        .join("library");

    #[cfg(not(target_os = "windows"))]
    PathBuf::from("wallpapers")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub target_fps: u32,
    pub pause_on_fullscreen: bool,
    pub reduce_on_battery: bool,
    pub gpu_quality: GpuQuality,
    pub idle_fps: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            pause_on_fullscreen: true,
            reduce_on_battery: true,
            gpu_quality: GpuQuality::High,
            idle_fps: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GpuQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl GpuQuality {
    pub fn label(&self) -> &str {
        match self {
            Self::Low => "Baixa",
            Self::Medium => "Média",
            Self::High => "Alta",
            Self::Ultra => "Ultra",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub monitor_index: usize,
    pub stretch_mode: StretchMode,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            monitor_index: 0,
            stretch_mode: StretchMode::Fill,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StretchMode {
    Fill,
    Fit,
    Stretch,
    Tile,
    Center,
}

impl StretchMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Fill => "Preencher",
            Self::Fit => "Ajustar",
            Self::Stretch => "Esticar",
            Self::Tile => "Ladrilhar",
            Self::Center => "Centralizar",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperEntry {
    pub id: String,
    pub name: String,
    pub author: String,
    pub path: PathBuf,
    pub kind: WallpaperKind,
    pub thumbnail: Option<PathBuf>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WallpaperKind {
    Shader,
    Video,
    Scene,
    Web,
}

impl WallpaperKind {
    pub fn label(&self) -> &str {
        match self {
            Self::Shader => "Shader",
            Self::Video => "Vídeo",
            Self::Scene => "Cena 2D",
            Self::Web => "Web",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Shader => "⬡",
            Self::Video => "▶",
            Self::Scene => "✦",
            Self::Web => "◈",
        }
    }
}
