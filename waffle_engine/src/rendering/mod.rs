/// Waffle Engine 3D Rendering Module
/// Contains all 3D rendering functionality and systems

pub mod scene;
pub mod lighting;
pub mod materials;
pub mod camera;
pub mod post_processing;
pub mod shadows;
pub mod atmosphere;
pub mod fog;

use bevy::prelude::*;
use scene::*;
use lighting::*;
use materials::*;
use camera::*;
use post_processing::*;
use shadows::*;
use fog::*;

pub struct WaffleRenderingPlugin;

impl Plugin for WaffleRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add 3D scene systems
            .add_systems(Startup, setup_3d_scene)
            .add_systems(Update, update_3d_scene)
            .add_systems(Update, apply_environment_settings)
            .add_systems(Update, update_sky_dome)
            .add_systems(Update, sync_sky_dome_to_camera)
            .add_systems(Update, ensure_scene_root_parenting)

            // Add lighting systems
            .add_systems(Startup, setup_lighting.after(setup_3d_scene))
            .add_systems(Update, sync_sun_from_environment)
            .add_systems(Update, sync_light_components)

            // Add material systems
            .add_systems(Startup, setup_materials.after(setup_3d_scene))
            .add_systems(Update, update_materials)
            .add_systems(Update, ensure_pbr_overrides)
            .add_systems(Update, update_pbr_texture_overrides)

            // Add camera systems
            .add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera)

            // Add post-processing systems
            .add_systems(Startup, setup_post_processing)
            .add_systems(Update, update_post_processing)

            // Add shadow systems
            .add_systems(Startup, setup_shadows)
            .add_systems(Update, update_shadows)

            // Add fog systems
            .add_systems(Startup, setup_fog)
            .add_systems(Update, update_fog);
    }
}
