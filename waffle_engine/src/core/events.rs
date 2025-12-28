// Waffle Engine Core Events
// Contains core engine events

use bevy::prelude::*;


/// Engine initialization event
#[derive(Debug, Clone, Event)]
pub struct EngineInitializedEvent {
    pub timestamp: std::time::Instant,
    pub config: crate::core::config::EngineConfig,
}

/// Engine shutdown event
#[derive(Debug, Clone, Event)]
pub struct EngineShutdownEvent {
    pub timestamp: std::time::Instant,
    pub reason: ShutdownReason,
}

/// Shutdown reasons
#[derive(Debug, Clone, PartialEq)]
pub enum ShutdownReason {
    UserRequest,
    Error,
    Crash,
    Normal,
}

/// Engine update event
#[derive(Debug, Clone, Event)]
pub enum EngineUpdateEvent {
    Initialized,
    Shutdown,
    SceneLoaded(String),
    SceneUnloaded(String),
    AssetLoaded(String),
    AssetUnloaded(String),
    Error(String),
    Warning(String),
    Info(String),
}

/// Scene management events
#[derive(Debug, Clone, Event)]
pub enum SceneEvent {
    LoadScene(String),
    UnloadScene(String),
    SceneLoaded(String),
    SceneUnloaded(String),
    SceneError(String),
}

/// Asset management events
#[derive(Debug, Clone, Event)]
pub enum AssetEvent {
    LoadAsset(String),
    UnloadAsset(String),
    AssetLoaded(String),
    AssetUnloaded(String),
    AssetError(String),
}

/// Input events
#[derive(Debug, Clone, Event)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseMoved(Vec2),
    MouseWheel(f32),
}

/// Engine error event
#[derive(Debug, Clone, Event)]
pub struct EngineErrorEvent {
    pub error: String,
    pub severity: ErrorSeverity,
    pub timestamp: std::time::Instant,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
    Warning,
}

/// Engine performance event
#[derive(Debug, Clone, Event)]
pub struct PerformanceEvent {
    pub fps: f32,
    pub frame_time: f32,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub timestamp: std::time::Instant,
}
