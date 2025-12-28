// Waffle Engine - Main Entry Point
// A complete game engine built on Bevy with Lua scripting support

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::LogPlugin;
use bevy::window::WindowMode;

// Import core engine module
mod core;
// Import rendering module
mod rendering;
// Import editor module
mod editor;

use core::*;
use rendering::*;
use editor::*;

// Main engine application
fn main() {
    App::new()
        // Core plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Waffle Engine".into(),
                resolution: (1280.0, 720.0).into(),
                mode: WindowMode::Windowed,
                resizable: true,
                ..default()
            }),
            ..default()
        }).set(LogPlugin {
            custom_layer: editor::editor_log_layer,
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        // Physics plugins temporarily disabled due to version compatibility
        // .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())

        // Engine modules
        .add_plugins(WaffleCorePlugin)
        .add_plugins(WaffleRenderingPlugin)
        .add_plugins(WaffleEditorPlugin)

        // Start the engine
        .run();
}
