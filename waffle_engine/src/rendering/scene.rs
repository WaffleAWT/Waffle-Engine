/// 3D Scene Module
/// Handles 3D scene setup, management, and rendering

use bevy::core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings};
use bevy::core_pipeline::prepass::{DeferredPrepass, DepthPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::pbr::{
    DefaultOpaqueRendererMethod, FogFalloff, FogSettings, NotShadowCaster, NotShadowReceiver,
    ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    ScreenSpaceReflectionsSettings,
};
use bevy::render::camera::Exposure;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection};
use crate::rendering::camera::WaffleMainCamera;
use crate::core::components::EditorHidden;

#[derive(Component)]
pub struct WaffleSceneRoot;

#[derive(Component)]
pub struct WaffleSceneObject;

#[derive(Component)]
pub struct WaffleSkyDome;

#[derive(Component, Clone)]
pub struct EnvironmentSettings {
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub sun_color: Color,
    pub sun_intensity: f32,
    pub sun_azimuth: f32,
    pub time_of_day: f32,
    pub sky_top_day: Color,
    pub sky_horizon_day: Color,
    pub sky_top_night: Color,
    pub sky_horizon_night: Color,
    pub sun_disk_intensity: f32,
    pub sun_disk_size: f32,
    pub exposure_ev100: f32,
    pub tonemapping: EnvironmentTonemapping,
    pub color_grading: EnvironmentColorGrading,
    pub bloom: EnvironmentBloomSettings,
    pub fog: EnvironmentFogSettings,
    pub ssao: EnvironmentSsaoSettings,
    pub ssr: EnvironmentSsrSettings,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvironmentTonemapping {
    None,
    Reinhard,
    AcesFitted,
    AgX,
    SomewhatBoringDisplayTransform,
    TonyMcMapface,
    BlenderFilmic,
}

#[derive(Clone, Copy)]
pub struct EnvironmentColorGrading {
    pub gamma: f32,
    pub pre_saturation: f32,
    pub post_saturation: f32,
}

#[derive(Clone, Copy)]
pub struct EnvironmentBloomSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub low_frequency_boost: f32,
    pub threshold: f32,
}

#[derive(Clone, Copy)]
pub struct EnvironmentSsaoSettings {
    pub enabled: bool,
    pub quality: EnvironmentSsaoQuality,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvironmentSsaoQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Clone, Copy)]
pub struct EnvironmentSsrSettings {
    pub enabled: bool,
    pub roughness_threshold: f32,
    pub thickness: f32,
    pub linear_steps: u32,
    pub bisection_steps: u32,
    pub use_secant: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvironmentFogMode {
    Linear,
    Exponential,
    ExponentialSquared,
    Atmospheric,
}

#[derive(Clone, Copy)]
pub struct EnvironmentFogSettings {
    pub enabled: bool,
    pub color: Color,
    pub mode: EnvironmentFogMode,
    pub start: f32,
    pub end: f32,
    pub density: f32,
}

#[derive(Resource, Clone, Copy)]
pub struct SceneRootEntity(pub Entity);

#[derive(Resource, Default)]
pub struct SceneSettings {
    pub ambient_light_color: Color,
    pub ambient_light_intensity: f32,
    pub background_color: Color,
    pub default_material: Handle<StandardMaterial>,
}

pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up 3D scene");

    // Create scene root entity
    let root = commands.spawn((
        WaffleSceneRoot,
        Name::new("Scene Root"),
        SpatialBundle::default(),
    )).id();
    commands.insert_resource(SceneRootEntity(root));

    // Create default materials
    let default_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        perceptual_roughness: 0.8,
        metallic: 0.1,
        ..default()
    });

    // Store scene settings
    commands.insert_resource(SceneSettings {
        ambient_light_color: Color::srgb(1.0, 1.0, 1.0),
        ambient_light_intensity: 0.1,
        background_color: Color::srgb(0.0, 0.0, 0.0),
        default_material: default_material.clone(),
    });

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(1.0, 1.0, 1.0),
        brightness: 100.0,
    });

    // Create a basic ground plane using Plane3d
    commands.spawn((
        WaffleSceneObject,
        Name::new("Ground Plane"),
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
            material: default_material.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
    )).set_parent(root);

    commands.spawn((
        WaffleSceneObject,
        Name::new("Environment"),
        EnvironmentSettings {
            ambient_color: Color::srgb(0.85, 0.9, 1.0),
            ambient_intensity: 200.0,
            sun_color: Color::srgb(1.0, 0.98, 0.92),
            sun_intensity: 12000.0,
            sun_azimuth: 35.0,
            time_of_day: 10.5,
            sky_top_day: Color::srgb(0.26, 0.49, 0.93),
            sky_horizon_day: Color::srgb(0.85, 0.93, 1.0),
            sky_top_night: Color::srgb(0.02, 0.04, 0.08),
            sky_horizon_night: Color::srgb(0.12, 0.14, 0.2),
            sun_disk_intensity: 3.5,
            sun_disk_size: 0.025,
            exposure_ev100: Exposure::EV100_BLENDER,
            tonemapping: EnvironmentTonemapping::AcesFitted,
            color_grading: EnvironmentColorGrading {
                gamma: 1.0,
                pre_saturation: 1.0,
                post_saturation: 1.0,
            },
            bloom: EnvironmentBloomSettings {
                enabled: true,
                intensity: 0.2,
                low_frequency_boost: 0.7,
                threshold: 0.8,
            },
            fog: EnvironmentFogSettings {
                enabled: false,
                color: Color::srgb(0.6, 0.7, 0.8),
                mode: EnvironmentFogMode::Linear,
                start: 20.0,
                end: 60.0,
                density: 0.02,
            },
            ssao: EnvironmentSsaoSettings {
                enabled: true,
                quality: EnvironmentSsaoQuality::High,
            },
            ssr: EnvironmentSsrSettings {
                enabled: false,
                roughness_threshold: 0.8,
                thickness: 0.2,
                linear_steps: 32,
                bisection_steps: 8,
                use_secant: true,
            },
        },
        Transform::default(),
        GlobalTransform::default(),
    )).set_parent(root);

    let sky_mesh = Mesh::from(Sphere::new(1.0).mesh().uv(48, 24));
    let sky_mesh_handle = meshes.add(sky_mesh);
    let sky_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    commands.spawn((
        WaffleSkyDome,
        EditorHidden,
        NotShadowCaster,
        NotShadowReceiver,
        PbrBundle {
            mesh: sky_mesh_handle,
            material: sky_material,
            transform: Transform::from_scale(Vec3::splat(200.0)),
            ..default()
        },
        Name::new("Sky Dome"),
    )).set_parent(root);
}

pub fn update_3d_scene() {
}

pub fn apply_environment_settings(
    mut commands: Commands,
    env_query: Query<&EnvironmentSettings, Or<(Added<EnvironmentSettings>, Changed<EnvironmentSettings>)>>,
    camera_query: Query<Entity, With<WaffleMainCamera>>,
    mut ambient_light: ResMut<AmbientLight>,
    mut default_opaque_method: ResMut<DefaultOpaqueRendererMethod>,
    mut msaa: ResMut<Msaa>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        return;
    };

    let Some(env) = env_query.iter().next() else {
        return;
    };

    let tonemapping = match env.tonemapping {
        EnvironmentTonemapping::None => Tonemapping::None,
        EnvironmentTonemapping::Reinhard => Tonemapping::Reinhard,
        EnvironmentTonemapping::AcesFitted => Tonemapping::AcesFitted,
        EnvironmentTonemapping::AgX => Tonemapping::AgX,
        EnvironmentTonemapping::SomewhatBoringDisplayTransform => {
            Tonemapping::SomewhatBoringDisplayTransform
        }
        EnvironmentTonemapping::TonyMcMapface => Tonemapping::TonyMcMapface,
        EnvironmentTonemapping::BlenderFilmic => Tonemapping::BlenderFilmic,
    };
    let global = ColorGradingGlobal {
        post_saturation: env.color_grading.post_saturation,
        ..Default::default()
    };
    let section = ColorGradingSection {
        saturation: env.color_grading.pre_saturation,
        gamma: env.color_grading.gamma,
        ..Default::default()
    };
    let color_grading = ColorGrading::with_identical_sections(global, section);
    let exposure = Exposure {
        ev100: env.exposure_ev100,
    };

    commands.entity(camera_entity).insert((
        tonemapping,
        color_grading,
        exposure,
    ));

    if env.bloom.enabled {
        commands.entity(camera_entity).insert(BloomSettings {
            intensity: env.bloom.intensity,
            low_frequency_boost: env.bloom.low_frequency_boost,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
            prefilter_settings: BloomPrefilterSettings {
                threshold: env.bloom.threshold,
                threshold_softness: 0.0,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
        });
    } else {
        commands.entity(camera_entity).remove::<BloomSettings>();
    }

    if env.fog.enabled {
        let falloff = match env.fog.mode {
            EnvironmentFogMode::Linear => FogFalloff::Linear {
                start: env.fog.start,
                end: env.fog.end,
            },
            EnvironmentFogMode::Exponential => FogFalloff::Exponential {
                density: env.fog.density,
            },
            EnvironmentFogMode::ExponentialSquared => FogFalloff::ExponentialSquared {
                density: env.fog.density,
            },
            EnvironmentFogMode::Atmospheric => FogFalloff::Atmospheric {
                extinction: Vec3::splat(env.fog.density),
                inscattering: Vec3::splat(env.fog.density * 0.4),
            },
        };
        commands.entity(camera_entity).insert(FogSettings {
            color: env.fog.color,
            directional_light_color: Color::NONE,
            directional_light_exponent: 0.0,
            falloff,
        });
    } else {
        commands.entity(camera_entity).remove::<FogSettings>();
    }

    if env.ssao.enabled {
        let quality = match env.ssao.quality {
            EnvironmentSsaoQuality::Low => ScreenSpaceAmbientOcclusionQualityLevel::Low,
            EnvironmentSsaoQuality::Medium => ScreenSpaceAmbientOcclusionQualityLevel::Medium,
            EnvironmentSsaoQuality::High => ScreenSpaceAmbientOcclusionQualityLevel::High,
            EnvironmentSsaoQuality::Ultra => ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
        };
        commands.entity(camera_entity).insert(ScreenSpaceAmbientOcclusionSettings {
            quality_level: quality,
        });
    } else {
        commands
            .entity(camera_entity)
            .remove::<ScreenSpaceAmbientOcclusionSettings>();
    }

    if env.ssr.enabled {
        default_opaque_method.set_to_deferred();
        *msaa = Msaa::Off;
        commands.entity(camera_entity).insert(ScreenSpaceReflectionsSettings {
            perceptual_roughness_threshold: env.ssr.roughness_threshold,
            thickness: env.ssr.thickness,
            linear_steps: env.ssr.linear_steps.max(1),
            linear_march_exponent: 1.0,
            bisection_steps: env.ssr.bisection_steps,
            use_secant: env.ssr.use_secant,
        });
        commands.entity(camera_entity).insert(DepthPrepass);
        commands.entity(camera_entity).insert(NormalPrepass);
        commands.entity(camera_entity).insert(DeferredPrepass);
    } else {
        default_opaque_method.set_to_forward();
        commands
            .entity(camera_entity)
            .remove::<ScreenSpaceReflectionsSettings>();
        *msaa = Msaa::default();
        commands.entity(camera_entity).remove::<DepthPrepass>();
        commands.entity(camera_entity).remove::<NormalPrepass>();
        commands.entity(camera_entity).remove::<DeferredPrepass>();
    }

    ambient_light.color = env.ambient_color;
    ambient_light.brightness = env.ambient_intensity;
}

pub fn ensure_scene_root_parenting(
    mut commands: Commands,
    scene_root: Option<Res<SceneRootEntity>>,
    query: Query<Entity, (With<WaffleSceneObject>, Without<Parent>, Without<crate::core::components::EditorHidden>)>,
) {
    let Some(root) = scene_root.map(|root| root.0) else {
        return;
    };
    for entity in &query {
        if entity != root {
            commands.entity(entity).set_parent(root);
        }
    }
}

pub fn update_sky_dome(
    env_query: Query<&EnvironmentSettings, Or<(Added<EnvironmentSettings>, Changed<EnvironmentSettings>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    sky_query: Query<&Handle<Mesh>, With<WaffleSkyDome>>,
) {
    let Some(env) = env_query.iter().next() else {
        return;
    };
    let Ok(mesh_handle) = sky_query.get_single() else {
        return;
    };
    let Some(mesh) = meshes.get_mut(mesh_handle) else {
        return;
    };
    apply_sky_gradient(mesh, env);
}

pub fn sync_sky_dome_to_camera(
    camera_query: Query<&Transform, (With<WaffleMainCamera>, Without<WaffleSkyDome>)>,
    mut sky_query: Query<&mut Transform, (With<WaffleSkyDome>, Without<WaffleMainCamera>)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };
    for mut transform in &mut sky_query {
        transform.translation = camera_transform.translation;
    }
}

fn apply_sky_gradient(mesh: &mut Mesh, env: &EnvironmentSettings) {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };

    let sun_dir = sun_direction(env);
    let day_factor = sun_day_factor(env.time_of_day);

    let sky_top = lerp_color(env.sky_top_night, env.sky_top_day, day_factor);
    let sky_horizon = lerp_color(env.sky_horizon_night, env.sky_horizon_day, day_factor);
    let sun_color = env.sun_color.to_linear().to_f32_array();

    let mut colors = Vec::with_capacity(positions.len());
    for position in positions {
        let dir = Vec3::from(*position).normalize_or_zero();
        let height = dir.y.clamp(-1.0, 1.0);
        let horizon_t = smoothstep(0.0, 0.9, (height + 1.0) * 0.5);
        let mut color = lerp_linear(sky_horizon, sky_top, horizon_t);

        let sun_dot = dir.dot(sun_dir).max(0.0);
        let sharpness = (1.0 / env.sun_disk_size.max(0.001)).clamp(1.0, 256.0);
        let sun = sun_dot.powf(sharpness) * env.sun_disk_intensity;
        color[0] += sun_color[0] * sun;
        color[1] += sun_color[1] * sun;
        color[2] += sun_color[2] * sun;
        color[3] = 1.0;

        colors.push(color);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

fn sun_direction(env: &EnvironmentSettings) -> Vec3 {
    let elevation = time_to_elevation(env.time_of_day);
    let azimuth = env.sun_azimuth.to_radians();
    let (sin_e, cos_e) = elevation.sin_cos();
    let (sin_a, cos_a) = azimuth.sin_cos();
    Vec3::new(cos_a * cos_e, sin_e, sin_a * cos_e).normalize_or_zero()
}

fn time_to_elevation(time_of_day: f32) -> f32 {
    let clamped = time_of_day.rem_euclid(24.0);
    (clamped / 24.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2
}

fn sun_day_factor(time_of_day: f32) -> f32 {
    let elevation = time_to_elevation(time_of_day);
    (elevation.sin().clamp(-1.0, 1.0) * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn lerp_color(a: Color, b: Color, t: f32) -> [f32; 4] {
    let a = a.to_linear().to_f32_array();
    let b = b.to_linear().to_f32_array();
    lerp_linear(a, b, t)
}

fn lerp_linear(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
