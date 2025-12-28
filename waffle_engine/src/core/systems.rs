// Waffle Engine Core Systems
// Contains core engine systems and logic

use bevy::prelude::*;
use crate::core::resources::*;
use crate::core::events::*;

/// Core engine systems
pub fn update_engine_state(
    mut engine_state: ResMut<EngineState>,
    time: Res<Time>,
) {
    engine_state.frame_count += 1;
    engine_state.last_frame_time = time.delta();
    engine_state.running = true;
}

pub fn update_performance_metrics(
    mut metrics: ResMut<PerformanceMetrics>,
    time: Res<Time>,
    engine_state: Res<EngineState>,
) {
    metrics.frame_time = time.delta().as_secs_f32();
    metrics.fps = 1.0 / metrics.frame_time;

    // Update frame count (convert u64 to u32)
    metrics.entities = engine_state.frame_count as u32;
}

pub fn handle_engine_events(
    mut events: EventReader<EngineUpdateEvent>,
    mut engine_state: ResMut<EngineState>,
) {
    for event in events.read() {
        match event {
            EngineUpdateEvent::Initialized => {
                engine_state.initialized = true;
                info!("Engine initialized");
            }
            EngineUpdateEvent::Shutdown => {
                engine_state.running = false;
                info!("Engine shutdown");
            }
            EngineUpdateEvent::SceneLoaded(scene_name) => {
                engine_state.current_scene = Some(scene_name.clone());
                info!("Scene loaded: {}", scene_name);
            }
            EngineUpdateEvent::SceneUnloaded(scene_name) => {
                if engine_state.current_scene == Some(scene_name.clone()) {
                    engine_state.current_scene = None;
                }
                info!("Scene unloaded: {}", scene_name);
            }
            EngineUpdateEvent::AssetLoaded(asset_name) => {
                info!("Asset loaded: {}", asset_name);
            }
            EngineUpdateEvent::AssetUnloaded(asset_name) => {
                info!("Asset unloaded: {}", asset_name);
            }
            EngineUpdateEvent::Error(error) => {
                error!("Engine error: {}", error);
            }
            EngineUpdateEvent::Warning(warning) => {
                warn!("Engine warning: {}", warning);
            }
            EngineUpdateEvent::Info(info_msg) => {
                info!("Engine info: {}", info_msg);
            }
        }
    }
}
