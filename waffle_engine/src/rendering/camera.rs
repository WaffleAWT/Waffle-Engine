/// Camera Module
/// Handles camera setup, control, and rendering

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::input::mouse::MouseMotion;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseWheel;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::CursorGrabMode;
use crate::core::components::EditorHidden;

#[derive(Component)]
pub struct WaffleCamera {
    pub camera_type: CameraType,
    pub movement_speed: f32,
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub is_active: bool,
}

#[derive(Component)]
pub struct WaffleMainCamera;

#[derive(Component)]
pub struct WaffleEditorCamera;

#[derive(Clone, Copy, PartialEq)]
pub enum CameraType {
    Perspective,
    Orthographic,
    FirstPerson,
    ThirdPerson,
    Editor,
}

#[derive(Resource, Default)]
pub struct CameraSettings {
    pub main_camera_entity: Option<Entity>,
    pub active_camera_entity: Option<Entity>,
    pub camera_movement_enabled: bool,
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,
}

#[derive(Resource, Clone)]
pub struct ViewportRenderTarget {
    pub image: Handle<Image>,
    pub size: UVec2,
}

pub fn setup_camera(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("Setting up camera system");

    let _window = windows.single();

    let size = Extent3d {
        width: 1280,
        height: 720,
        ..default()
    };
    let mut image = Image::default();
    image.texture_descriptor.size = size;
    image.texture_descriptor.dimension = TextureDimension::D2;
    image.texture_descriptor.format = TextureFormat::Bgra8UnormSrgb;
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image.resize(size);

    let image_handle = images.add(image);
    commands.insert_resource(ViewportRenderTarget {
        image: image_handle.clone(),
        size: UVec2::new(size.width, size.height),
    });

    // Create main perspective camera
    let camera_entity = commands.spawn((
        WaffleCamera {
            camera_type: CameraType::Perspective,
            movement_speed: 5.0,
            rotation_speed: 0.7,
            zoom_speed: 1.0,
            is_active: true,
        },
        WaffleMainCamera,
        EditorHidden,
        Camera3dBundle {
            camera: Camera {
                target: RenderTarget::Image(image_handle.into()),
                order: -1,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Name::new("Main Camera"),
    )).id();

    // Store camera settings
    commands.insert_resource(CameraSettings {
        main_camera_entity: Some(camera_entity),
        active_camera_entity: Some(camera_entity),
        camera_movement_enabled: true,
        mouse_sensitivity: 0.5,
        movement_speed: 5.0,
    });
}

pub fn update_camera(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut WaffleCamera), With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut window_query_mut: Query<&mut Window, With<PrimaryWindow>>,
) {
    let rmb_down = mouse_input.pressed(MouseButton::Right);
    let any_active = camera_query.iter().any(|(_, camera)| camera.is_active);

    if let Ok(mut window) = window_query_mut.get_single_mut() {
        if rmb_down && any_active {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
        } else {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }

    let mut move_input = Vec3::ZERO;
    if rmb_down {
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            move_input.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            move_input.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            move_input.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            move_input.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            move_input.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            move_input.y -= 1.0;
        }
    }

    let mut rotation_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        rotation_delta += motion.delta;
    }

    let mut wheel_delta = 0.0;
    for wheel in mouse_wheel.read() {
        wheel_delta += wheel.y;
    }

    let speed_multiplier = if rmb_down
        && (keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight))
    {
        2.0
    } else {
        1.0
    };

    // Camera movement controls
    for (mut transform, mut camera) in &mut camera_query {
        if !camera.is_active {
            continue;
        }

        if move_input.length_squared() > 0.0 {
            let forward = transform.forward();
            let right = transform.right();
            let up = transform.up();
            let local_dir =
                (right * move_input.x) + (up * move_input.y) + (forward * -move_input.z);
            let movement =
                local_dir.normalize() * camera.movement_speed * speed_multiplier * time.delta_seconds();
            transform.translation += movement;
        }

        if rmb_down {
            let yaw = -rotation_delta.x * camera.rotation_speed * 0.004;
            let pitch = -rotation_delta.y * camera.rotation_speed * 0.004;
            transform.rotate_y(yaw);
            transform.rotate_local_x(pitch);

            if wheel_delta.abs() > 0.0 {
                let min_speed = 0.5;
                let max_speed = 50.0;
                camera.movement_speed = (camera.movement_speed + wheel_delta * 0.5)
                    .clamp(min_speed, max_speed);
            }
        } else if wheel_delta.abs() > 0.0 {
            let zoom_amount = wheel_delta * camera.zoom_speed;
            let forward = transform.forward();
            transform.translation += forward * zoom_amount;
        }
    }
}
