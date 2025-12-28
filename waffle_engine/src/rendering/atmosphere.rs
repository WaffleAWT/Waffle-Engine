/// Atmosphere Module
/// Handles atmospheric scattering and sky rendering

use bevy::prelude::*;
use crate::rendering::scene::{SceneRootEntity, WaffleSceneObject};

#[derive(Resource, Default, Clone)]
pub struct AtmosphereSettings {
    pub atmosphere_enabled: bool,
    pub planet_radius: f32,
    pub atmosphere_height: f32,
    pub rayleigh_scattering: Vec3,
    pub mie_scattering: Vec3,
    pub mie_absorption: Vec3,
    pub sun_intensity: f32,
    pub sun_angle: f32,
}

#[derive(Component, Clone)]
pub struct AtmosphereSettingsComponent {
    pub atmosphere_enabled: bool,
    pub planet_radius: f32,
    pub atmosphere_height: f32,
    pub rayleigh_scattering: Vec3,
    pub mie_scattering: Vec3,
    pub mie_absorption: Vec3,
    pub sun_intensity: f32,
    pub sun_angle: f32,
}

pub fn setup_atmosphere(
    mut commands: Commands,
    scene_root: Option<Res<SceneRootEntity>>,
) {
    info!("Setting up atmosphere system");

    let settings = AtmosphereSettings {
        atmosphere_enabled: true,
        planet_radius: 6360000.0, // Earth radius in meters
        atmosphere_height: 60000.0, // Atmosphere height in meters
        rayleigh_scattering: Vec3::new(5.8e-6, 1.35e-5, 3.31e-5),
        mie_scattering: Vec3::new(2.0e-5, 2.0e-5, 2.0e-5),
        mie_absorption: Vec3::new(0.0, 0.0, 0.0),
        sun_intensity: 20.0,
        sun_angle: 0.0,
    };
    commands.insert_resource(settings.clone());

    let entity = commands.spawn((
        WaffleSceneObject,
        Name::new("Atmosphere"),
        AtmosphereSettingsComponent {
            atmosphere_enabled: settings.atmosphere_enabled,
            planet_radius: settings.planet_radius,
            atmosphere_height: settings.atmosphere_height,
            rayleigh_scattering: settings.rayleigh_scattering,
            mie_scattering: settings.mie_scattering,
            mie_absorption: settings.mie_absorption,
            sun_intensity: settings.sun_intensity,
            sun_angle: settings.sun_angle,
        },
        Transform::default(),
        GlobalTransform::default(),
    )).id();
    if let Some(root) = scene_root.map(|root| root.0) {
        commands.entity(entity).set_parent(root);
    }
}

pub fn update_atmosphere(
    mut settings: ResMut<AtmosphereSettings>,
) {
    // Atmosphere update placeholder
    settings.sun_angle = settings.sun_angle.rem_euclid(std::f32::consts::TAU);
}

pub fn sync_atmosphere_from_entity(
    mut settings: ResMut<AtmosphereSettings>,
    query: Query<&AtmosphereSettingsComponent>,
    scene_root: Option<Res<SceneRootEntity>>,
    mut commands: Commands,
) {
    if let Some(component) = query.iter().next() {
        settings.atmosphere_enabled = component.atmosphere_enabled;
        settings.planet_radius = component.planet_radius;
        settings.atmosphere_height = component.atmosphere_height;
        settings.rayleigh_scattering = component.rayleigh_scattering;
        settings.mie_scattering = component.mie_scattering;
        settings.mie_absorption = component.mie_absorption;
        settings.sun_intensity = component.sun_intensity;
        settings.sun_angle = component.sun_angle;
        return;
    }

    let Some(root) = scene_root.map(|root| root.0) else {
        return;
    };
    commands.spawn((
        WaffleSceneObject,
        Name::new("Atmosphere"),
        AtmosphereSettingsComponent {
            atmosphere_enabled: settings.atmosphere_enabled,
            planet_radius: settings.planet_radius,
            atmosphere_height: settings.atmosphere_height,
            rayleigh_scattering: settings.rayleigh_scattering,
            mie_scattering: settings.mie_scattering,
            mie_absorption: settings.mie_absorption,
            sun_intensity: settings.sun_intensity,
            sun_angle: settings.sun_angle,
        },
        Transform::default(),
        GlobalTransform::default(),
    )).set_parent(root);
}
