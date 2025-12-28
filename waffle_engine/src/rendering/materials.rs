/// Materials Module
/// Handles material creation, management, and rendering

use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, TextureUsages};

#[derive(Component)]
pub struct WaffleMaterial {
    pub material_type: MaterialType,
    pub base_color: Color,
    pub roughness: f32,
    pub metallic: f32,
    pub emissive: Color,
    pub normal_map_strength: f32,
    pub occlusion_strength: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MaterialType {
    Standard,
    Pbr,
    Unlit,
    Transparent,
    Custom,
}

#[derive(Resource, Default)]
pub struct MaterialLibrary {
    pub materials: Vec<Handle<StandardMaterial>>,
    pub material_names: Vec<String>,
}

#[derive(Component, Default)]
pub struct PbrTextureOverrides {
    pub metallic_map: Option<Handle<Image>>,
    pub roughness_map: Option<Handle<Image>>,
    pub combined_map: Option<Handle<Image>>,
    pub last_metallic: Option<Handle<Image>>,
    pub last_roughness: Option<Handle<Image>>,
}

pub fn setup_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_root: Option<Res<crate::rendering::scene::SceneRootEntity>>,
) {
    info!("Setting up materials system");
    let root = scene_root.map(|root| root.0);

    // Create a material library resource
    let mut library = MaterialLibrary::default();

    // Add some default materials
    let white_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        metallic: 0.1,
        ..default()
    });

    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        perceptual_roughness: 0.5,
        metallic: 0.5,
        ..default()
    });

    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        perceptual_roughness: 0.3,
        metallic: 0.8,
        ..default()
    });

    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        perceptual_roughness: 0.7,
        metallic: 0.2,
        ..default()
    });

    let metal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        perceptual_roughness: 0.1,
        metallic: 1.0,
        ..default()
    });

    let emissive_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.0, 0.5),
        emissive: Color::srgb(1.0, 0.5, 1.0).to_linear() * 2.0,
        perceptual_roughness: 0.6,
        metallic: 0.3,
        ..default()
    });

    // Add materials to library
    library.materials.push(white_material.clone());
    library.material_names.push("White Material".to_string());

    library.materials.push(red_material.clone());
    library.material_names.push("Red Material".to_string());

    library.materials.push(blue_material.clone());
    library.material_names.push("Blue Material".to_string());

    library.materials.push(green_material.clone());
    library.material_names.push("Green Material".to_string());

    library.materials.push(metal_material.clone());
    library.material_names.push("Metal Material".to_string());

    library.materials.push(emissive_material.clone());
    library.material_names.push("Emissive Material".to_string());

    commands.insert_resource(library);

    // Create some example objects with different materials
    spawn_material_example(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-2.0, 1.0, 0.0),
        white_material,
        "White Object".to_string(),
        root,
    );

    spawn_material_example(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-1.0, 1.0, 0.0),
        red_material,
        "Red Object".to_string(),
        root,
    );

    spawn_material_example(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 1.0, 0.0),
        blue_material,
        "Blue Object".to_string(),
        root,
    );

    spawn_material_example(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(1.0, 1.0, 0.0),
        green_material,
        "Green Object".to_string(),
        root,
    );

    spawn_material_example(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(2.0, 1.0, 0.0),
        metal_material,
        "Metal Object".to_string(),
        root,
    );
}

fn spawn_material_example(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    material: Handle<StandardMaterial>,
    name: String,
    parent: Option<Entity>,
) {
    let entity = commands.spawn((
        crate::rendering::scene::WaffleSceneObject,
        WaffleMaterial {
            material_type: MaterialType::Standard,
            base_color: Color::srgb(1.0, 1.0, 1.0),
            roughness: 0.8,
            metallic: 0.1,
            emissive: Color::srgb(0.0, 0.0, 0.0),
            normal_map_strength: 1.0,
            occlusion_strength: 1.0,
        },
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.8, 0.8, 0.8)),
            material,
            transform: Transform::from_translation(position),
            ..default()
        },
        Name::new(name),
    )).id();
    if let Some(parent) = parent {
        commands.entity(entity).set_parent(parent);
    }
}

pub fn update_materials() {
    // Material update logic will go here
}

pub fn ensure_pbr_overrides(
    mut commands: Commands,
    query: Query<Entity, (With<Handle<StandardMaterial>>, Without<PbrTextureOverrides>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(PbrTextureOverrides::default());
    }
}

pub fn update_pbr_texture_overrides(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&Handle<StandardMaterial>, &mut PbrTextureOverrides)>,
) {
    for (material_handle, mut overrides) in &mut query {
        if overrides.metallic_map == overrides.last_metallic
            && overrides.roughness_map == overrides.last_roughness
            && overrides.combined_map.is_some()
        {
            continue;
        }

        let metallic_image = overrides
            .metallic_map
            .as_ref()
            .and_then(|handle| images.get(handle));
        let roughness_image = overrides
            .roughness_map
            .as_ref()
            .and_then(|handle| images.get(handle));

        let source = metallic_image.or(roughness_image);
        let Some(source_image) = source else {
            if let Some(material) = materials.get_mut(material_handle) {
                material.metallic_roughness_texture = None;
            }
            overrides.combined_map = None;
            overrides.last_metallic = overrides.metallic_map.clone();
            overrides.last_roughness = overrides.roughness_map.clone();
            continue;
        };

        let size = source_image.texture_descriptor.size;
        if !matches!(
            source_image.texture_descriptor.format,
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb
        ) {
            continue;
        }
        let len = (size.width * size.height * 4) as usize;
        if source_image.data.len() < len {
            continue;
        }

        let mut combined = vec![0u8; len];
        for i in 0..(size.width * size.height) as usize {
            let idx = i * 4;
            let metallic = metallic_image
                .and_then(|img| img.data.get(idx).copied())
                .unwrap_or(0);
            let roughness = roughness_image
                .and_then(|img| img.data.get(idx).copied())
                .unwrap_or(255);
            combined[idx] = 0;
            combined[idx + 1] = roughness;
            combined[idx + 2] = metallic;
            combined[idx + 3] = 255;
        }

        let mut image = Image::default();
        image.texture_descriptor.size = size;
        image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
        image.texture_descriptor.usage =
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
        image.data = combined;
        let combined_handle = images.add(image);
        if let Some(material) = materials.get_mut(material_handle) {
            material.metallic_roughness_texture = Some(combined_handle.clone());
        }
        overrides.combined_map = Some(combined_handle);
        overrides.last_metallic = overrides.metallic_map.clone();
        overrides.last_roughness = overrides.roughness_map.clone();
    }
}
