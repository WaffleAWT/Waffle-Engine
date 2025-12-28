// Waffle Engine Core Resources
// Contains core engine resources

use bevy::prelude::*;
use std::collections::HashMap;

// Re-export config resources for convenience
pub use crate::core::config::{
    EngineConfig, EngineState, PerformanceMetrics
};

/// Asset registry resource
#[derive(Debug, Clone, Resource, Default)]
pub struct AssetRegistry {
    pub assets: HashMap<String, AssetInfo>,
    pub loaded_assets: usize,
    pub total_assets: usize,
}

/// Asset information
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub asset_id: String,
    pub asset_type: AssetType,
    pub path: String,
    pub loaded: bool,
    pub size: u64,
    pub dependencies: Vec<String>,
}

/// Asset types
#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Shader,
    Audio,
    Scene,
    Script,
    Font,
    Prefab,
    Unknown,
}

/// Scene registry resource
#[derive(Debug, Clone, Resource, Default)]
pub struct SceneRegistry {
    pub scenes: HashMap<String, SceneInfo>,
    pub current_scene: Option<String>,
}

/// Scene information
#[derive(Debug, Clone)]
pub struct SceneInfo {
    pub scene_id: String,
    pub name: String,
    pub path: String,
    pub entities: usize,
    pub loaded: bool,
}

/// Input state resource
#[derive(Debug, Clone, Resource, Default)]
pub struct InputState {
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
    pub keyboard_state: HashMap<KeyCode, bool>,
    pub mouse_button_state: HashMap<MouseButton, bool>,
}

/// Engine time resource
#[derive(Debug, Clone, Resource)]
pub struct EngineTime {
    pub startup_time: std::time::Instant,
    pub frame_start_time: std::time::Instant,
    pub last_frame_time: std::time::Duration,
    pub total_time: std::time::Duration,
    pub frame_count: u64,
    pub fps: f32,
}

impl Default for EngineTime {
    fn default() -> Self {
        let now = std::time::Instant::now();
        Self {
            startup_time: now,
            frame_start_time: now,
            last_frame_time: std::time::Duration::default(),
            total_time: std::time::Duration::default(),
            frame_count: 0,
            fps: 0.0,
        }
    }
}
