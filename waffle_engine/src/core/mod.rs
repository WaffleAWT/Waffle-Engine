// Waffle Engine Core Module
// Contains fundamental engine functionality and systems

pub mod config;
pub mod systems;
pub mod components;
pub mod resources;
pub mod events;

use bevy::prelude::*;

use config::*;
use systems::*;
use components::*;
use resources::*;
use events::*;

// Core plugin group
pub struct WaffleCorePlugin;

impl Plugin for WaffleCorePlugin {
    fn build(&self, app: &mut App) {
        // Add core systems
        app.add_systems(Startup, setup_core_systems)
            .add_systems(Update, update_core_systems)
            .add_systems(PostUpdate, post_update_core_systems)

            // Add core resources
            .init_resource::<EngineConfig>()
            .init_resource::<EngineState>()
            .init_resource::<PerformanceMetrics>()

            // Add core events
            .add_event::<EngineInitializedEvent>()
            .add_event::<EngineShutdownEvent>()
            .add_event::<EngineUpdateEvent>()
            .add_event::<SceneEvent>()
            .add_event::<InputEvent>()
            .add_event::<EngineErrorEvent>()
            .add_event::<PerformanceEvent>();

        // Register core components
        app.register_type::<EngineRoot>()
            .register_type::<EngineCamera>()
            .register_type::<EngineLight>()
            .register_type::<EngineTransform>();
    }
}

// Core systems
pub fn setup_core_systems(mut commands: Commands) {
    info!("Setting up Waffle Engine core systems");

    // Create engine root entity
    commands.spawn((
        EngineRoot,
        EditorHidden,
        Name::new("Engine Root"),
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Initialize engine configuration
    let config = EngineConfig::default();
    commands.insert_resource(config);

    // Initialize engine state
    let state = EngineState::default();
    commands.insert_resource(state);

    // Initialize performance metrics
    let metrics = PerformanceMetrics::default();
    commands.insert_resource(metrics);
}

pub fn update_core_systems() {
    // Core engine update logic
}

pub fn post_update_core_systems() {
    // Post-update core engine logic
}
