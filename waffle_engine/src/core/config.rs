// Waffle Engine Core Configuration
// Contains engine configuration structures and settings

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Main engine configuration
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct EngineConfig {
    pub engine_name: String,
    pub version: String,
    pub max_fps: u32,
    pub vsync: bool,
    pub show_fps: bool,
    pub show_debug_info: bool,
    pub asset_root: PathBuf,
    pub script_root: PathBuf,
    pub default_scene: Option<PathBuf>,
    pub physics_enabled: bool,
    pub audio_enabled: bool,
    pub rendering_quality: RenderingQuality,
    pub logging_level: LogLevel,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            engine_name: "Waffle Engine".to_string(),
            version: "0.1.0".to_string(),
            max_fps: 144,
            vsync: true,
            show_fps: true,
            show_debug_info: false,
            asset_root: PathBuf::from("assets"),
            script_root: PathBuf::from("scripts"),
            default_scene: None,
            physics_enabled: true,
            audio_enabled: true,
            rendering_quality: RenderingQuality::High,
            logging_level: LogLevel::Info,
        }
    }
}

/// Rendering quality settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RenderingQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// Logging level settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Engine state tracking
#[derive(Debug, Clone, Resource)]
pub struct EngineState {
    pub initialized: bool,
    pub running: bool,
    pub frame_count: u64,
    pub startup_time: std::time::Instant,
    pub last_frame_time: std::time::Duration,
    pub current_scene: Option<String>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            initialized: false,
            running: false,
            frame_count: 0,
            startup_time: std::time::Instant::now(),
            last_frame_time: std::time::Duration::default(),
            current_scene: None,
        }
    }
}

/// Performance metrics tracking
#[derive(Debug, Clone, Resource, Default)]
pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time: f32,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub draw_calls: u32,
    pub triangles: u32,
    pub entities: u32,
    pub systems_runtime: std::collections::HashMap<String, f32>,
}
