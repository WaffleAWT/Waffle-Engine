// Waffle Engine Core Components
// Contains core ECS components for the engine

use bevy::prelude::*;
use bevy::reflect::Reflect;

/// Marker component for the engine root entity
#[derive(Component, Reflect, Default)]
pub struct EngineRoot;

/// Marks entities that should be hidden from the editor hierarchy/selection.
#[derive(Component, Reflect, Default)]
pub struct EditorHidden;

/// Engine camera component
#[derive(Component, Reflect)]
pub struct EngineCamera {
    pub is_main: bool,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub clear_color: Color,
}

impl Default for EngineCamera {
    fn default() -> Self {
        Self {
            is_main: false,
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            clear_color: Color::BLACK,
        }
    }
}

/// Engine light component
#[derive(Component, Reflect)]
pub struct EngineLight {
    pub light_type: LightType,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub shadows_enabled: bool,
}

impl Default for EngineLight {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: Color::WHITE,
            intensity: 1.0,
            range: 100.0,
            shadows_enabled: true,
        }
    }
}

/// Light types
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Engine transform component (extends Bevy's Transform)
#[derive(Component, Reflect)]
pub struct EngineTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub dirty: bool,
}

impl Default for EngineTransform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            dirty: false,
        }
    }
}

/// Engine identifier component
#[derive(Component, Reflect)]
pub struct EngineId {
    pub id: u64,
    pub name: String,
    pub tags: Vec<String>,
}

impl Default for EngineId {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Unnamed".to_string(),
            tags: Vec::new(),
        }
    }
}
