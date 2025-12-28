/// Lighting Module
/// Handles all types of lights and lighting effects

use bevy::prelude::*;
use std::f32::consts::PI;
use crate::rendering::scene::{EnvironmentSettings, SceneRootEntity};

#[derive(Component)]
pub struct WaffleLight {
    pub light_type: LightType,
    pub intensity: f32,
    pub color: Color,
    pub range: f32,
    pub shadows_enabled: bool,
}

#[derive(Component)]
pub struct WaffleDirectionalLight;

#[derive(Component)]
pub struct WafflePointLight;

#[derive(Component)]
pub struct WaffleSpotLight;

#[derive(Component)]
pub struct WaffleAmbientLight;

#[derive(Clone, Copy, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Ambient,
}

pub fn setup_lighting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_root: Option<Res<SceneRootEntity>>,
) {
    info!("Setting up lighting system");
    let root_entity = scene_root.map(|root| root.0);

    // Create a directional light (sun)
    let directional = commands
        .spawn((
            crate::rendering::scene::WaffleSceneObject,
            WaffleLight {
                light_type: LightType::Directional,
                intensity: 10000.0,
                color: Color::srgb(1.0, 1.0, 1.0),
                range: 100.0,
                shadows_enabled: true,
            },
            WaffleDirectionalLight,
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 10000.0,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                    ..default()
                },
                ..default()
            },
            Name::new("Directional Light (Sun)"),
        ))
        .id();
    if let Some(root) = root_entity {
        commands.entity(directional).set_parent(root);
    }

    // Create some point lights
    spawn_point_light(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::ZERO,
        Color::srgb(1.0, 0.0, 0.0),
        5000.0,
        true,
        "Point Light 1".to_string(),
        root_entity,
    );

    spawn_point_light(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::ZERO,
        Color::srgb(0.0, 0.0, 1.0),
        5000.0,
        true,
        "Point Light 2".to_string(),
        root_entity,
    );

    // Create a spot light
    spawn_spot_light(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::ZERO,
        Color::srgb(0.0, 1.0, 0.0),
        10000.0,
        true,
        "Spot Light 1".to_string(),
        root_entity,
    );
}

fn spawn_point_light(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
    intensity: f32,
    shadows_enabled: bool,
    name: String,
    parent: Option<Entity>,
) {
    let entity = commands
        .spawn((
            crate::rendering::scene::WaffleSceneObject,
            WaffleLight {
                light_type: LightType::Point,
                intensity,
                color,
                range: 10.0,
                shadows_enabled,
            },
            WafflePointLight,
            PointLightBundle {
                point_light: PointLight {
                    intensity,
                    color,
                    range: 10.0,
                    shadows_enabled,
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            Name::new(name),
        ))
        .id();
    if let Some(parent) = parent {
        commands.entity(entity).set_parent(parent);
    }
    commands.entity(entity).with_children(|parent| {
        parent.spawn(PbrBundle {
            mesh: meshes.add(Sphere::new(0.1)),
            material: materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 2.0,
                ..default()
            }),
            ..default()
        });
    });
}

fn spawn_spot_light(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
    intensity: f32,
    shadows_enabled: bool,
    name: String,
    parent: Option<Entity>,
) {
    let entity = commands
        .spawn((
            crate::rendering::scene::WaffleSceneObject,
            WaffleLight {
                light_type: LightType::Spot,
                intensity,
                color,
                range: 15.0,
                shadows_enabled,
            },
            WaffleSpotLight,
            SpotLightBundle {
                spot_light: SpotLight {
                    intensity,
                    color,
                    range: 15.0,
                    shadows_enabled,
                    inner_angle: 0.6,
                    outer_angle: 0.8,
                    ..default()
                },
                transform: Transform::from_translation(position)
                    .looking_at(Vec3::new(0.0, 0.0, -1.0), Vec3::Y),
                ..default()
            },
            Name::new(name),
        ))
        .id();
    if let Some(parent) = parent {
        commands.entity(entity).set_parent(parent);
    }
    commands.entity(entity).with_children(|parent| {
        parent.spawn(PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.1, 0.125)),
            material: materials.add(StandardMaterial {
                base_color: color,
                emissive: color.to_linear() * 2.0,
                ..default()
            }),
            transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)),
            ..default()
        });
    });
}

pub fn sync_sun_from_environment(
    env_query: Query<&EnvironmentSettings, Or<(Added<EnvironmentSettings>, Changed<EnvironmentSettings>)>>,
    mut light_query: Query<(&mut DirectionalLight, &mut Transform), With<WaffleDirectionalLight>>,
) {
    let Some(env) = env_query.iter().next() else {
        return;
    };
    for (mut light, mut transform) in &mut light_query {
        light.color = env.sun_color;
        light.illuminance = env.sun_intensity;
        let direction = sun_direction(env);
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -direction);
    }
}

pub fn sync_light_components(
    mut query: Query<(
        &mut WaffleLight,
        Option<&DirectionalLight>,
        Option<&PointLight>,
        Option<&SpotLight>,
    )>,
) {
    for (mut waffle_light, directional, point, spot) in &mut query {
        if let Some(light) = directional {
            waffle_light.intensity = light.illuminance;
            waffle_light.color = light.color;
            waffle_light.shadows_enabled = light.shadows_enabled;
        }
        if let Some(light) = point {
            waffle_light.intensity = light.intensity;
            waffle_light.color = light.color;
            waffle_light.range = light.range;
            waffle_light.shadows_enabled = light.shadows_enabled;
        }
        if let Some(light) = spot {
            waffle_light.intensity = light.intensity;
            waffle_light.color = light.color;
            waffle_light.range = light.range;
            waffle_light.shadows_enabled = light.shadows_enabled;
        }
    }
}

fn sun_direction(env: &EnvironmentSettings) -> Vec3 {
    let time = env.time_of_day.rem_euclid(24.0);
    let elevation = (time / 24.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let azimuth = env.sun_azimuth.to_radians();
    let (sin_e, cos_e) = elevation.sin_cos();
    let (sin_a, cos_a) = azimuth.sin_cos();
    Vec3::new(cos_a * cos_e, sin_e, sin_a * cos_e).normalize_or_zero()
}
