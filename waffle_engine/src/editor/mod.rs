/// Waffle Engine Editor Module
/// Contains the complete editor interface with dark theme

pub mod ui;
pub mod windows;
pub mod theme;
pub mod panels;

use bevy::prelude::*;
use bevy::ecs::system::{ParamSet, SystemParam};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use bevy::log::{tracing_subscriber, BoxedLayer, Level};
use bevy::log::tracing_subscriber::Layer;
use bevy::utils::tracing::{self, Subscriber};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use bevy::render::render_resource::Extent3d;
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseButton;
use bevy::input::keyboard::KeyCode;
use bevy::render::camera::Camera;
use bevy::render::mesh::Mesh;
use bevy::render::primitives::Aabb;
use serde::{Deserialize, Serialize};
use crate::core::resources::EngineConfig;
use crate::rendering::camera::{ViewportRenderTarget, WaffleCamera, WaffleMainCamera};
use crate::core::components::EditorHidden;
use crate::rendering::scene::{EnvironmentSettings, SceneSettings, WaffleSceneRoot, WaffleSceneObject};
use crate::rendering::atmosphere::AtmosphereSettingsComponent;
use crate::rendering::lighting::WaffleLight;
use crate::rendering::materials::PbrTextureOverrides;
use walkdir::WalkDir;
use bevy::window::FileDragAndDrop;

use ui::*;
use windows::*;
use theme::*;
use panels::*;

/// Editor UI plugin
pub struct WaffleEditorPlugin;

impl Plugin for WaffleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, setup_editor)
            .add_systems(Update, update_editor_ui)
            .add_systems(Update, sync_editor_camera_focus)
            .add_systems(Update, update_selected_entity_transform)
            .add_systems(Update, update_editor_camera_orbit_focus.after(crate::rendering::camera::update_camera))
            .add_systems(Update, draw_selected_gizmos.after(crate::rendering::camera::update_camera))
            .add_systems(Update, draw_editor_grid.after(crate::rendering::camera::update_camera))
            .add_systems(Update, collect_editor_logs)
            .add_systems(Update, refresh_asset_cache)
            .add_systems(Update, apply_reparent_events)
            .add_systems(Update, apply_delete_events)
            .add_systems(Update, apply_spawn_primitive_events)
            .add_systems(Update, apply_spawn_asset_events)
            .init_resource::<EditorState>()
            .init_resource::<EditorSettings>()
            .init_resource::<EditorOutput>()
            .init_resource::<AssetBrowserCache>()
            .add_event::<HierarchyReparentEvent>()
            .add_event::<DeleteEntityEvent>()
            .add_event::<SpawnPrimitiveEvent>()
            .add_event::<SpawnAssetEvent>();
    }
}

/// Editor state resource
#[derive(Resource)]
pub struct EditorState {
    pub dock_state: DockState<EditorTab>,
    pub show_demo_window: bool,
    pub selected_entity: Option<Entity>,
    pub gizmo_mode: GizmoMode,
    pub active_axis: Option<GizmoAxis>,
    pub axis_space: AxisSpace,
    pub gizmo_overlay: Option<GizmoOverlay>,
    pub viewport_focused: bool,
    pub viewport_size: Vec2,
    pub viewport_origin: Vec2,
    pub viewport_hovered: bool,
    pub viewport_clicked: bool,
    pub viewport_click_pos: Option<Vec2>,
    pub viewport_focus_request: bool,
    pub hierarchy_filter: String,
    pub asset_filter: String,
    pub selected_asset: Option<String>,
    pub delete_confirm: Option<Entity>,
    pub layout_cache: String,
    pub layout_last_save: Instant,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![EditorTab::Viewport]);
        let tree = dock_state.main_surface_mut();

        // Create initial layout
        let [main_node, _hierarchy_node] =
            tree.split_left(NodeIndex::root(), 0.2, vec![EditorTab::Hierarchy]);
        let [main_node, _inspector_node] = tree.split_right(
            main_node,
            0.75,
            vec![EditorTab::Inspector],
        );
        let [_viewport_node, _output_node] =
            tree.split_below(main_node, 0.75, vec![EditorTab::Console, EditorTab::Profiler, EditorTab::Assets]);

        Self {
            dock_state,
            show_demo_window: false,
            selected_entity: None,
            gizmo_mode: GizmoMode::Move,
            active_axis: None,
            axis_space: AxisSpace::Global,
            gizmo_overlay: None,
            viewport_focused: false,
            viewport_size: Vec2::new(1280.0, 720.0),
            viewport_origin: Vec2::ZERO,
            viewport_hovered: false,
            viewport_clicked: false,
            viewport_click_pos: None,
            viewport_focus_request: false,
            hierarchy_filter: String::new(),
            asset_filter: String::new(),
            selected_asset: None,
            delete_confirm: None,
            layout_cache: String::new(),
            layout_last_save: Instant::now(),
        }
    }
}

/// Editor settings
#[derive(Resource)]
pub struct EditorSettings {
    pub theme: EditorTheme,
    pub show_fps: bool,
    pub show_debug_info: bool,
    pub grid_enabled: bool,
    pub grid_size: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            theme: EditorTheme::default(),
            show_fps: true,
            show_debug_info: false,
            grid_enabled: true,
            grid_size: 1.0,
        }
    }
}

/// Editor tab types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorTab {
    Viewport,
    Hierarchy,
    Inspector,
    Assets,
    Console,
    Profiler,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Move,
    Rotate,
    Scale,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AxisSpace {
    Local,
    Global,
}

#[derive(Clone)]
pub struct GizmoOverlay {
    pub origin: Vec2,
    pub x_end: Vec2,
    pub y_end: Vec2,
    pub z_end: Vec2,
    pub rotate_rings: Option<RotateOverlay>,
}

#[derive(Clone)]
pub struct RotateOverlay {
    pub x_points: Vec<Vec2>,
    pub y_points: Vec<Vec2>,
    pub z_points: Vec<Vec2>,
}

#[derive(Resource, Default)]
pub struct EditorOutput {
    pub lines: Vec<String>,
}

#[derive(Event)]
pub struct HierarchyReparentEvent {
    pub child: Entity,
    pub new_parent: Option<Entity>,
}

#[derive(Event)]
pub struct DeleteEntityEvent {
    pub entity: Entity,
}

#[derive(Event, Clone)]
pub struct SpawnPrimitiveEvent {
    pub kind: SpawnPrimitiveKind,
    pub parent: Option<Entity>,
}

#[derive(Event, Clone)]
pub struct SpawnAssetEvent {
    pub path: String,
    pub parent: Option<Entity>,
}

#[derive(Clone, Copy)]
pub enum SpawnPrimitiveKind {
    Empty,
    Cube,
    Sphere,
    Plane,
    DirectionalLight,
    PointLight,
    SpotLight,
}

#[derive(Clone)]
pub struct AssetEntry {
    pub path: String,
    pub kind: AssetKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Image,
    Model,
    Audio,
    Script,
    Material,
    Other,
}

#[derive(Resource)]
pub struct AssetBrowserCache {
    pub(crate) root: PathBuf,
    pub(crate) entries: Vec<AssetEntry>,
    last_scan: Option<Instant>,
}

impl Default for AssetBrowserCache {
    fn default() -> Self {
        Self {
            root: PathBuf::from("assets"),
            entries: Vec::new(),
            last_scan: None,
        }
    }
}

/// Setup the editor
fn setup_editor(
    mut commands: Commands,
    mut editor_state: ResMut<EditorState>,
) {
    info!("Setting up Waffle Engine Editor");

    // Insert editor resources
    commands.init_resource::<EditorState>();
    commands.init_resource::<EditorSettings>();
    commands.init_resource::<EditorOutput>();
    commands.init_resource::<AssetBrowserCache>();

    if let Some(loaded) = load_layout() {
        editor_state.dock_state = loaded;
    }
    editor_state.layout_cache = ron::ser::to_string(&editor_state.dock_state).unwrap_or_default();
}

#[derive(SystemParam)]
struct EditorUiWorldParams<'w, 's> {
    queries: ParamSet<'w, 's, (
        Query<'w, 's, (Entity, Option<&'static Name>, Option<&'static Parent>), Without<EditorHidden>>,
        Query<'w, 's, &'static mut Name>,
    )>,
    scene_root_query: Query<'w, 's, Entity, With<WaffleSceneRoot>>,
    transform_query: Query<'w, 's, &'static mut Transform>,
    global_transform_query: Query<'w, 's, &'static GlobalTransform>,
    material_handle_query: Query<'w, 's, &'static Handle<StandardMaterial>>,
    pbr_overrides_query: Query<'w, 's, &'static mut PbrTextureOverrides>,
    environment_query: Query<'w, 's, &'static mut EnvironmentSettings>,
    atmosphere_query: Query<'w, 's, &'static mut AtmosphereSettingsComponent>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    asset_server: Res<'w, AssetServer>,
    images: ResMut<'w, Assets<Image>>,
    meshes: Res<'w, Assets<Mesh>>,
    waffle_light_query: Query<'w, 's, &'static mut crate::rendering::lighting::WaffleLight>,
    directional_light_query: Query<'w, 's, &'static mut DirectionalLight>,
    point_light_query: Query<'w, 's, &'static mut PointLight>,
    spot_light_query: Query<'w, 's, &'static mut SpotLight>,
    diagnostics: Res<'w, bevy::diagnostic::DiagnosticsStore>,
    window_query: Query<'w, 's, (), With<bevy::window::PrimaryWindow>>,
    asset_cache: ResMut<'w, AssetBrowserCache>,
    viewport_target: ResMut<'w, ViewportRenderTarget>,
    reparent_events: EventWriter<'w, HierarchyReparentEvent>,
    delete_events: EventWriter<'w, DeleteEntityEvent>,
    spawn_primitive_events: EventWriter<'w, SpawnPrimitiveEvent>,
    spawn_asset_events: EventWriter<'w, SpawnAssetEvent>,
    keyboard_input: Res<'w, ButtonInput<KeyCode>>,
    mouse_input: Res<'w, ButtonInput<MouseButton>>,
    file_drop_events: EventReader<'w, 's, FileDragAndDrop>,
    camera_query: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<WaffleMainCamera>>,
    mesh_query: Query<'w, 's, (Entity, &'static GlobalTransform, &'static Handle<Mesh>), Without<EditorHidden>>,
}

/// Main editor UI update system
fn update_editor_ui(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut editor_settings: ResMut<EditorSettings>,
    _engine_config: Res<EngineConfig>,
    mut editor_output: ResMut<EditorOutput>,
    mut world: EditorUiWorldParams,
) {
    if world.window_query.is_empty() {
        return;
    }

    let viewport_texture_id = match contexts.image_id(&world.viewport_target.image) {
        Some(texture_id) => Some(texture_id),
        None => Some(contexts.add_image(world.viewport_target.image.clone())),
    };

    let Some(ctx) = contexts.try_ctx_mut() else {
        return;
    };

    // Apply dark theme
    editor_settings.theme.apply(ctx);

    let mut dock_state = std::mem::replace(&mut editor_state.dock_state, DockState::new(Vec::new()));

    let hierarchy = {
        let entity_query = world.queries.p0();
        let root = world.scene_root_query.get_single().ok();
        build_hierarchy_snapshot(&entity_query, root)
    };
    let mut reparent_queue: Vec<HierarchyReparentEvent> = Vec::new();
    let mut spawn_primitive_queue: Vec<SpawnPrimitiveEvent> = Vec::new();
    let mut spawn_asset_queue: Vec<SpawnAssetEvent> = Vec::new();

    let selected_entity = editor_state.selected_entity;

    let mut selected_transform = selected_entity
        .and_then(|entity| world.transform_query.get_mut(entity).ok());
    let mut name_query = world.queries.p1();
    let mut selected_name = selected_entity
        .and_then(|entity| name_query.get_mut(entity).ok());
    let selected_material_entity = selected_entity.and_then(|entity| {
        if world.material_handle_query.get(entity).is_ok() {
            return Some(entity);
        }
        find_descendant_with_material(entity, &hierarchy, &world.material_handle_query)
    });
    let selected_material_handle = selected_material_entity
        .and_then(|entity| world.material_handle_query.get(entity).ok().cloned());
    let mut selected_overrides = selected_material_entity
        .and_then(|entity| world.pbr_overrides_query.get_mut(entity).ok());
    let mut selected_environment = selected_entity
        .and_then(|entity| world.environment_query.get_mut(entity).ok());
    let mut selected_atmosphere = selected_entity
        .and_then(|entity| world.atmosphere_query.get_mut(entity).ok());
    let mut selected_waffle_light = selected_entity
        .and_then(|entity| world.waffle_light_query.get_mut(entity).ok());
    let mut selected_directional_light = selected_entity
        .and_then(|entity| world.directional_light_query.get_mut(entity).ok());
    let mut selected_point_light = selected_entity
        .and_then(|entity| world.point_light_query.get_mut(entity).ok());
    let mut selected_spot_light = selected_entity
        .and_then(|entity| world.spot_light_query.get_mut(entity).ok());

    handle_file_drops(&mut world.file_drop_events, &mut world.asset_cache);

    editor_state.gizmo_overlay = None;
    if let Some(selected) = selected_entity {
        if let Ok((camera, camera_transform)) = world.camera_query.get_single() {
            if let Ok(transform) = world.global_transform_query.get(selected) {
                let origin = transform.translation();
                let distance = camera_transform.translation().distance(origin);
                let axis_length = (distance * 0.2).clamp(0.5, 5.0);
                let basis = match editor_state.axis_space {
                    AxisSpace::Global => Mat3::IDENTITY,
                    AxisSpace::Local => Mat3::from_quat(transform.compute_transform().rotation),
                };
                let axis_x = basis * Vec3::X;
                let axis_y = basis * Vec3::Y;
                let axis_z = basis * Vec3::Z;
                if let (Some(origin_screen), Some(x_end), Some(y_end), Some(z_end)) = (
                    camera.world_to_viewport(camera_transform, origin),
                    camera.world_to_viewport(camera_transform, origin + axis_x * axis_length),
                    camera.world_to_viewport(camera_transform, origin + axis_y * axis_length),
                    camera.world_to_viewport(camera_transform, origin + axis_z * axis_length),
                ) {
                    let rotate_rings = build_rotate_overlay(
                        camera,
                        camera_transform,
                        origin,
                        axis_x,
                        axis_y,
                        axis_z,
                        axis_length,
                    );
                    editor_state.gizmo_overlay = Some(GizmoOverlay {
                        origin: origin_screen,
                        x_end,
                        y_end,
                        z_end,
                        rotate_rings,
                    });
                }
            }
        }
    }

    // Main editor window
    egui::CentralPanel::default().show(ctx, |ui| {
        if !ctx.wants_keyboard_input() && !world.mouse_input.pressed(MouseButton::Right) {
            if world.keyboard_input.just_pressed(KeyCode::KeyQ) {
                editor_state.gizmo_mode = GizmoMode::Move;
            }
            if world.keyboard_input.just_pressed(KeyCode::KeyW) {
                editor_state.gizmo_mode = GizmoMode::Rotate;
            }
            if world.keyboard_input.just_pressed(KeyCode::KeyE) {
                editor_state.gizmo_mode = GizmoMode::Scale;
            }
        }
        // Menu bar
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Scene").clicked() {
                    // TODO: New scene
                }
                if ui.button("Open Scene").clicked() {
                    // TODO: Open scene dialog
                }
                if ui.button("Save Scene").clicked() {
                    // TODO: Save scene
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    // TODO: Exit application
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // TODO: Undo
                }
                if ui.button("Redo").clicked() {
                    // TODO: Redo
                }
            });

            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut editor_settings.show_fps, "Show FPS").clicked() {
                    // TODO: Toggle FPS display
                }
                if ui.checkbox(&mut editor_settings.show_debug_info, "Debug Info").clicked() {
                    // TODO: Toggle debug info
                }
                if ui.checkbox(&mut editor_settings.grid_enabled, "Grid").clicked() {
                    // TODO: Toggle grid
                }
            });

            ui.menu_button("Tools", |ui| {
                if ui.button("Script Editor").clicked() {
                    // TODO: Open script editor
                }
                if ui.button("Asset Browser").clicked() {
                    // TODO: Open asset browser
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    // TODO: Show about dialog
                }
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            let _ = ui.button(">");
            let _ = ui.button("||");
            let _ = ui.button("[]");
            ui.separator();
            if ui
                .selectable_label(editor_state.gizmo_mode == GizmoMode::Move, "Move")
                .clicked()
            {
                editor_state.gizmo_mode = GizmoMode::Move;
            }
            if ui
                .selectable_label(editor_state.gizmo_mode == GizmoMode::Rotate, "Rotate")
                .clicked()
            {
                editor_state.gizmo_mode = GizmoMode::Rotate;
            }
            if ui
                .selectable_label(editor_state.gizmo_mode == GizmoMode::Scale, "Scale")
                .clicked()
            {
                editor_state.gizmo_mode = GizmoMode::Scale;
            }
            ui.separator();
            if ui
                .selectable_label(editor_state.axis_space == AxisSpace::Global, "Global")
                .clicked()
            {
                editor_state.axis_space = AxisSpace::Global;
            }
            if ui
                .selectable_label(editor_state.axis_space == AxisSpace::Local, "Local")
                .clicked()
            {
                editor_state.axis_space = AxisSpace::Local;
            }
        });

        ui.separator();

        // Main dock area
        let mut dock_style = Style::from_egui(ctx.style().as_ref());
        dock_style.tab_bar.bg_fill = editor_settings.theme.panel_color;
        dock_style.tab_bar.height = 24.0;

        let selected_asset = editor_state.selected_asset.clone();
        DockArea::new(&mut dock_state)
            .style(dock_style)
            .show_inside(ui, &mut EditorTabViewer {
                editor_state: &mut editor_state,
                editor_settings: &mut editor_settings,
                editor_output: &mut editor_output,
                hierarchy: &hierarchy,
                selected_transform: selected_transform.as_deref_mut(),
                selected_name: selected_name.as_deref_mut(),
                selected_material_handle,
                selected_overrides: selected_overrides.as_deref_mut(),
                selected_environment: selected_environment.as_deref_mut(),
                selected_atmosphere: selected_atmosphere.as_deref_mut(),
                material_assets: &mut world.material_assets,
                asset_server: &world.asset_server,
                selected_asset: selected_asset.as_deref(),
                selected_waffle_light: selected_waffle_light.as_deref_mut(),
                selected_directional_light: selected_directional_light.as_deref_mut(),
                selected_point_light: selected_point_light.as_deref_mut(),
                selected_spot_light: selected_spot_light.as_deref_mut(),
                diagnostics: &world.diagnostics,
                asset_cache: &world.asset_cache,
                reparent_queue: &mut reparent_queue,
                spawn_primitive_queue: &mut spawn_primitive_queue,
                spawn_asset_queue: &mut spawn_asset_queue,
                viewport_texture_id,
            });
    });
    editor_state.dock_state = dock_state;

    for event in reparent_queue {
        world.reparent_events.send(event);
    }
    for event in spawn_primitive_queue {
        world.spawn_primitive_events.send(event);
    }
    for event in spawn_asset_queue {
        world.spawn_asset_events.send(event);
    }

    resize_viewport_target(&mut world.viewport_target, &mut world.images, editor_state.viewport_size);
    handle_viewport_picking(
        &mut editor_state,
        &world.camera_query,
        &world.mesh_query,
        &world.meshes,
    );
    save_layout_if_changed(&mut editor_state);

    if editor_state.delete_confirm.is_none()
        && world.keyboard_input.just_pressed(KeyCode::Delete)
        && editor_state.selected_entity.is_some()
        && !ctx.wants_keyboard_input()
    {
        editor_state.delete_confirm = editor_state.selected_entity;
    }

    if let Some(entity) = editor_state.delete_confirm {
        let label = hierarchy
            .names
            .get(&entity)
            .map(|name| name.as_str())
            .unwrap_or("Entity");
        let mut keep_open = true;
        egui::Window::new("Delete Entity?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("Delete \"{}\"?", label));
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        world.delete_events.send(DeleteEntityEvent { entity });
                        if editor_state.selected_entity == Some(entity) {
                            editor_state.selected_entity = None;
                        }
                        keep_open = false;
                    }
                    if ui.button("Cancel").clicked() {
                        keep_open = false;
                    }
                });
            });
        if !keep_open {
            editor_state.delete_confirm = None;
        }
    }

    // Demo window for development
    let mut show_demo_window = editor_state.show_demo_window;
    if show_demo_window {
        let mut should_close = false;
        egui::Window::new("Demo Window")
            .open(&mut show_demo_window)
            .show(ctx, |ui| {
                ui.label("This is a demo window for testing EGUI components.");
                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        if should_close {
            show_demo_window = false;
        }
    }
    editor_state.show_demo_window = show_demo_window;
}

fn resize_viewport_target(
    target: &mut ViewportRenderTarget,
    images: &mut Assets<Image>,
    size: Vec2,
) {
    let width = size.x.max(1.0).round() as u32;
    let height = size.y.max(1.0).round() as u32;
    let new_size = UVec2::new(width, height);
    if new_size == target.size {
        return;
    }

    if let Some(image) = images.get_mut(&target.image) {
        image.resize(Extent3d {
            width,
            height,
            ..default()
        });
    }
    target.size = new_size;
}

fn handle_viewport_picking(
    editor_state: &mut EditorState,
    camera_query: &Query<(&Camera, &GlobalTransform), With<WaffleMainCamera>>,
    mesh_query: &Query<(Entity, &GlobalTransform, &Handle<Mesh>), Without<EditorHidden>>,
    meshes: &Assets<Mesh>,
) {
    if !editor_state.viewport_clicked {
        return;
    }
    editor_state.viewport_clicked = false;
    editor_state.active_axis = None;

    let Some(local_pos) = editor_state.viewport_click_pos.take() else {
        return;
    };

    if let Some(axis) = pick_gizmo_axis(
        editor_state.gizmo_overlay.as_ref(),
        editor_state.gizmo_mode,
        local_pos,
    ) {
        editor_state.active_axis = Some(axis);
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, local_pos) else {
        return;
    };

    let mut best_hit: Option<(Entity, f32)> = None;
    for (entity, transform, mesh_handle) in mesh_query.iter() {
        let Some(mesh) = meshes.get(mesh_handle) else {
            continue;
        };
        let Some(aabb) = mesh.compute_aabb() else {
            continue;
        };
        let matrix = transform.compute_matrix();
        let Some(distance) = ray_aabb_intersection_world(
            ray.origin,
            *ray.direction,
            &matrix,
            &aabb,
        ) else {
            continue;
        };
        if best_hit.map(|(_, best)| distance < best).unwrap_or(true) {
            best_hit = Some((entity, distance));
        }
    }

    if let Some((entity, _)) = best_hit {
        editor_state.selected_entity = Some(entity);
    } else {
        editor_state.selected_entity = None;
    }
}

fn ray_aabb_intersection_world(
    origin: Vec3,
    direction: Vec3,
    world_from_local: &Mat4,
    aabb: &Aabb,
) -> Option<f32> {
    let local_from_world = world_from_local.inverse();
    if !local_from_world.is_finite() {
        return None;
    }
    let local_origin = local_from_world.transform_point3(origin);
    let local_dir = local_from_world.transform_vector3(direction).normalize_or_zero();

    let min = Vec3::from(aabb.min());
    let max = Vec3::from(aabb.max());
    let Some(t) = ray_aabb_intersection(local_origin, local_dir, min, max) else {
        return None;
    };
    let local_hit = local_origin + local_dir * t;
    let world_hit = world_from_local.transform_point3(local_hit);
    Some(world_hit.distance(origin))
}

fn ray_aabb_intersection(
    origin: Vec3,
    direction: Vec3,
    min: Vec3,
    max: Vec3,
) -> Option<f32> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;

    for i in 0..3 {
        let origin_i = origin[i];
        let dir_i = direction[i];
        let min_i = min[i];
        let max_i = max[i];
        if dir_i.abs() < 1e-6 {
            if origin_i < min_i || origin_i > max_i {
                return None;
            }
            continue;
        }
        let inv = 1.0 / dir_i;
        let mut t1 = (min_i - origin_i) * inv;
        let mut t2 = (max_i - origin_i) * inv;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        tmin = tmin.max(t1);
        tmax = tmax.min(t2);
        if tmin > tmax {
            return None;
        }
    }

    if tmax < 0.0 {
        None
    } else if tmin >= 0.0 {
        Some(tmin)
    } else {
        Some(tmax)
    }
}

fn build_rotate_overlay(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    origin: Vec3,
    axis_x: Vec3,
    axis_y: Vec3,
    axis_z: Vec3,
    radius: f32,
) -> Option<RotateOverlay> {
    let steps = 40;
    let mut x_points = Vec::with_capacity(steps + 1);
    let mut y_points = Vec::with_capacity(steps + 1);
    let mut z_points = Vec::with_capacity(steps + 1);

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = t * std::f32::consts::TAU;
        let (s, c) = angle.sin_cos();

        let p_x = origin + (axis_y * c + axis_z * s) * radius;
        let p_y = origin + (axis_x * c + axis_z * s) * radius;
        let p_z = origin + (axis_x * c + axis_y * s) * radius;

        let Some(px) = camera.world_to_viewport(camera_transform, p_x) else {
            return None;
        };
        let Some(py) = camera.world_to_viewport(camera_transform, p_y) else {
            return None;
        };
        let Some(pz) = camera.world_to_viewport(camera_transform, p_z) else {
            return None;
        };
        x_points.push(px);
        y_points.push(py);
        z_points.push(pz);
    }

    Some(RotateOverlay {
        x_points,
        y_points,
        z_points,
    })
}

fn pick_gizmo_axis(
    overlay: Option<&GizmoOverlay>,
    mode: GizmoMode,
    click_pos: Vec2,
) -> Option<GizmoAxis> {
    let overlay = overlay?;
    let origin_screen = overlay.origin;
    let x_screen = overlay.x_end;
    let y_screen = overlay.y_end;
    let z_screen = overlay.z_end;

    let threshold = 10.0;
    let mut best: Option<(GizmoAxis, f32)> = None;

    match mode {
        GizmoMode::Rotate => {
            if let Some(rings) = overlay.rotate_rings.as_ref() {
                for (axis, points) in [
                    (GizmoAxis::X, &rings.x_points),
                    (GizmoAxis::Y, &rings.y_points),
                    (GizmoAxis::Z, &rings.z_points),
                ] {
                    if points.len() >= 2 {
                        let distance = distance_to_polyline(click_pos, points);
                        if distance <= threshold {
                            if best.map(|(_, best_dist)| distance < best_dist).unwrap_or(true) {
                                best = Some((axis, distance));
                            }
                        }
                    }
                }
            }
        }
        _ => {
            for (axis, end) in [
                (GizmoAxis::X, x_screen),
                (GizmoAxis::Y, y_screen),
                (GizmoAxis::Z, z_screen),
            ] {
                let distance = distance_to_segment(click_pos, origin_screen, end);
                if distance <= threshold {
                    if best.map(|(_, best_dist)| distance < best_dist).unwrap_or(true) {
                        best = Some((axis, distance));
                    }
                }
            }
        }
    }

    best.map(|(axis, _)| axis)
}

fn distance_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 0.0 {
        (point - a).dot(ab) / ab.length_squared()
    } else {
        0.0
    };
    let t = t.clamp(0.0, 1.0);
    let closest = a + ab * t;
    point.distance(closest)
}

fn distance_to_polyline(point: Vec2, points: &[Vec2]) -> f32 {
    let mut best = f32::INFINITY;
    for window in points.windows(2) {
        let d = distance_to_segment(point, window[0], window[1]);
        if d < best {
            best = d;
        }
    }
    best
}

fn save_layout_if_changed(editor_state: &mut EditorState) {
    let now = Instant::now();
    if now.duration_since(editor_state.layout_last_save) < Duration::from_secs(1) {
        return;
    }
    let Ok(layout_string) = ron::ser::to_string(&editor_state.dock_state) else {
        return;
    };
    if layout_string == editor_state.layout_cache {
        return;
    }
    if std::fs::write("editor_layout.ron", layout_string.as_bytes()).is_ok() {
        editor_state.layout_cache = layout_string;
        editor_state.layout_last_save = now;
    }
}

fn load_layout() -> Option<DockState<EditorTab>> {
    let data = std::fs::read_to_string("editor_layout.ron").ok()?;
    ron::de::from_str(&data).ok()
}

pub(crate) struct HierarchySnapshot {
    pub(crate) roots: Vec<Entity>,
    pub(crate) children: HashMap<Entity, Vec<Entity>>,
    pub(crate) names: HashMap<Entity, String>,
}

fn build_hierarchy_snapshot(
    query: &Query<(Entity, Option<&Name>, Option<&Parent>), Without<EditorHidden>>,
    root: Option<Entity>,
) -> HierarchySnapshot {
    let mut children: HashMap<Entity, Vec<Entity>> = HashMap::new();
    let mut names: HashMap<Entity, String> = HashMap::new();
    let mut all_entities: Vec<Entity> = Vec::new();
    let mut has_parent: HashMap<Entity, Entity> = HashMap::new();

    for (entity, name, parent) in query.iter() {
        let label = name
            .map(|name| name.as_str().to_string())
            .unwrap_or_else(|| format!("Entity {}", entity.index()));
        names.insert(entity, label);
        all_entities.push(entity);

        if let Some(parent) = parent {
            children.entry(parent.get()).or_default().push(entity);
            has_parent.insert(entity, parent.get());
        }
    }

    let mut roots: Vec<Entity> = if let Some(root_entity) = root {
        vec![root_entity]
    } else {
        all_entities
            .iter()
            .copied()
            .filter(|entity| !has_parent.contains_key(entity))
            .collect()
    };

    let name_lookup = |entity: &Entity| names.get(entity).map(|s| s.as_str()).unwrap_or("");
    roots.sort_by(|a, b| name_lookup(a).cmp(name_lookup(b)));

    for child_list in children.values_mut() {
        child_list.sort_by(|a, b| name_lookup(a).cmp(name_lookup(b)));
    }

    if let Some(root_entity) = root {
        let mut filtered_children: HashMap<Entity, Vec<Entity>> = HashMap::new();
        let mut filtered_names: HashMap<Entity, String> = HashMap::new();

        let mut stack = vec![root_entity];
        let mut visited: std::collections::HashSet<Entity> = std::collections::HashSet::new();
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(name) = names.get(&current) {
                filtered_names.insert(current, name.clone());
            }
            if let Some(children_list) = children.get(&current) {
                filtered_children.insert(current, children_list.clone());
                for child in children_list {
                    stack.push(*child);
                }
            }
        }

        HierarchySnapshot {
            roots,
            children: filtered_children,
            names: filtered_names,
        }
    } else {
        HierarchySnapshot {
            roots,
            children,
            names,
        }
    }
}

fn find_descendant_with_material(
    entity: Entity,
    hierarchy: &HierarchySnapshot,
    material_query: &Query<&Handle<StandardMaterial>>,
) -> Option<Entity> {
    let mut stack: Vec<Entity> = hierarchy
        .children
        .get(&entity)
        .map(|children| children.clone())
        .unwrap_or_default();
    let mut visited: std::collections::HashSet<Entity> = std::collections::HashSet::new();

    while let Some(current) = stack.pop() {
        if !visited.insert(current) {
            continue;
        }
        if material_query.get(current).is_ok() {
            return Some(current);
        }
        if let Some(children) = hierarchy.children.get(&current) {
            for child in children {
                stack.push(*child);
            }
        }
    }

    None
}

fn sync_editor_camera_focus(
    editor_state: Res<EditorState>,
    mut cameras: Query<&mut WaffleCamera, With<WaffleMainCamera>>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for mut camera in &mut cameras {
        camera.is_active = editor_state.viewport_focused;
    }
}

fn update_selected_entity_transform(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut transforms: Query<&mut Transform, Without<WaffleMainCamera>>,
    camera_query: Query<&Transform, With<WaffleMainCamera>>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else {
        mouse_motion.clear();
        return;
    };
    if ctx.wants_pointer_input() && !editor_state.viewport_hovered {
        mouse_motion.clear();
        return;
    }
    if !editor_state.viewport_focused {
        mouse_motion.clear();
        return;
    }

    if !mouse_input.pressed(MouseButton::Left) {
        editor_state.active_axis = None;
        mouse_motion.clear();
        return;
    }

    let Some(selected) = editor_state.selected_entity else {
        mouse_motion.clear();
        return;
    };

    let Ok(camera) = camera_query.get_single() else {
        mouse_motion.clear();
        return;
    };

    if mouse_input.pressed(MouseButton::Right)
        || mouse_input.pressed(MouseButton::Middle)
    {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        delta += motion.delta;
    }

    if delta.length_squared() == 0.0 {
        return;
    }

    let Ok(mut transform) = transforms.get_mut(selected) else {
        return;
    };

    let distance = camera.translation.distance(transform.translation).max(0.1);
    let drag_speed = 0.002 * distance;
    let right = camera.right();
    let up = camera.up();
    let world_delta = (right * delta.x + up * -delta.y) * drag_speed;

    match editor_state.gizmo_mode {
        GizmoMode::Move => {
            if let Some(axis) = editor_state.active_axis {
                let axis_dir = match (editor_state.axis_space, axis) {
                    (AxisSpace::Local, GizmoAxis::X) => transform.rotation * Vec3::X,
                    (AxisSpace::Local, GizmoAxis::Y) => transform.rotation * Vec3::Y,
                    (AxisSpace::Local, GizmoAxis::Z) => transform.rotation * Vec3::Z,
                    (_, GizmoAxis::X) => Vec3::X,
                    (_, GizmoAxis::Y) => Vec3::Y,
                    (_, GizmoAxis::Z) => Vec3::Z,
                };
                let amount = world_delta.dot(axis_dir);
                transform.translation += axis_dir * amount;
            } else {
                transform.translation += world_delta;
            }
        }
        GizmoMode::Rotate => {
            let Some(axis) = editor_state.active_axis else {
                return;
            };
            let axis_dir = match (editor_state.axis_space, axis) {
                (AxisSpace::Local, GizmoAxis::X) => transform.rotation * Vec3::X,
                (AxisSpace::Local, GizmoAxis::Y) => transform.rotation * Vec3::Y,
                (AxisSpace::Local, GizmoAxis::Z) => transform.rotation * Vec3::Z,
                (_, GizmoAxis::X) => Vec3::X,
                (_, GizmoAxis::Y) => Vec3::Y,
                (_, GizmoAxis::Z) => Vec3::Z,
            };
            let angle = (delta.x + delta.y) * 0.004;
            transform.rotate(Quat::from_axis_angle(axis_dir, angle));
        }
        GizmoMode::Scale => {
            let Some(axis) = editor_state.active_axis else {
                return;
            };
            let amount = 1.0 + (delta.x + delta.y) * 0.005;
            let clamped = amount.clamp(0.1, 10.0);
            match axis {
                GizmoAxis::X => transform.scale.x = (transform.scale.x * clamped).max(0.01),
                GizmoAxis::Y => transform.scale.y = (transform.scale.y * clamped).max(0.01),
                GizmoAxis::Z => transform.scale.z = (transform.scale.z * clamped).max(0.01),
            }
        }
    }
}

fn update_editor_camera_orbit_focus(
    mut editor_state: ResMut<EditorState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<WaffleMainCamera>>,
    target_query: Query<&GlobalTransform>,
) {
    if !editor_state.viewport_focused {
        mouse_motion.clear();
        return;
    }

    let Some(selected) = editor_state.selected_entity else {
        mouse_motion.clear();
        return;
    };

    let Ok(target) = target_query.get(selected) else {
        mouse_motion.clear();
        return;
    };

    let Ok(mut camera) = camera_query.get_single_mut() else {
        mouse_motion.clear();
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyF) || editor_state.viewport_focus_request {
        let target_pos = target.translation();
        let direction = (camera.translation - target_pos).normalize_or_zero();
        let distance = camera.translation.distance(target_pos).max(2.0);
        camera.translation = target_pos + direction * distance;
        camera.look_at(target_pos, Vec3::Y);
        editor_state.viewport_focus_request = false;
    }

    if mouse_input.pressed(MouseButton::Middle) {
        let mut delta = Vec2::ZERO;
        for motion in mouse_motion.read() {
            delta += motion.delta;
        }
        if delta.length_squared() == 0.0 {
            return;
        }
        let target_pos = target.translation();
        let mut offset = camera.translation - target_pos;
        let yaw = -delta.x * 0.004;
        let pitch = delta.y * 0.004;
        offset = Quat::from_axis_angle(Vec3::Y, yaw) * offset;
        let right = offset.cross(Vec3::Y).normalize_or_zero();
        offset = Quat::from_axis_angle(right, pitch) * offset;
        camera.translation = target_pos + offset;
        camera.look_at(target_pos, Vec3::Y);
    } else {
        mouse_motion.clear();
    }
}

fn draw_selected_gizmos(
    editor_state: Res<EditorState>,
    mut gizmos: Gizmos,
    transform_query: Query<&GlobalTransform>,
    mesh_query: Query<&Handle<Mesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    let Some(selected) = editor_state.selected_entity else {
        return;
    };

    let Ok(transform) = transform_query.get(selected) else {
        return;
    };

    if let Ok(mesh_handle) = mesh_query.get(selected) {
        if let Some(mesh) = meshes.get(mesh_handle) {
            if let Some(aabb) = mesh.compute_aabb() {
                draw_aabb_gizmo(&mut gizmos, transform.compute_matrix(), &aabb, Color::srgb(1.0, 0.9, 0.1));
            }
        }
    }
}

fn draw_editor_grid(
    editor_settings: Res<EditorSettings>,
    mut gizmos: Gizmos,
) {
    if !editor_settings.grid_enabled {
        return;
    }
    let size = 50.0;
    let step = editor_settings.grid_size.max(0.1);
    let color = Color::srgb(0.18, 0.18, 0.18);
    let axis_color = Color::srgb(0.25, 0.25, 0.25);

    let mut x: f32 = -size;
    while x <= size {
        let line_color = if x.abs() < 0.001 { axis_color } else { color };
        gizmos.line(Vec3::new(x, 0.0, -size), Vec3::new(x, 0.0, size), line_color);
        x += step;
    }

    let mut z: f32 = -size;
    while z <= size {
        let line_color = if z.abs() < 0.001 { axis_color } else { color };
        gizmos.line(Vec3::new(-size, 0.0, z), Vec3::new(size, 0.0, z), line_color);
        z += step;
    }
}

fn draw_aabb_gizmo(
    gizmos: &mut Gizmos,
    world_from_local: Mat4,
    aabb: &Aabb,
    color: Color,
) {
    let min = Vec3::from(aabb.min());
    let max = Vec3::from(aabb.max());
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ];
    let mut world = [Vec3::ZERO; 8];
    for (i, corner) in corners.iter().enumerate() {
        world[i] = world_from_local.transform_point3(*corner);
    }
    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];
    for (a, b) in edges {
        gizmos.line(world[a], world[b], color);
    }
}

#[derive(Debug, Event)]
struct EditorLogEvent {
    message: String,
    level: Level,
}

#[derive(Deref, DerefMut)]
struct CapturedEditorLogs(mpsc::Receiver<EditorLogEvent>);

fn transfer_editor_logs(
    receiver: NonSend<CapturedEditorLogs>,
    mut event_writer: EventWriter<EditorLogEvent>,
) {
    event_writer.send_batch(receiver.try_iter());
}

struct EditorLogLayer {
    sender: mpsc::Sender<EditorLogEvent>,
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for EditorLogLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        if !is_game_log_target(metadata.target()) {
            return;
        }

        let mut message = None;
        event.record(&mut EditorLogVisitor(&mut message));
        if let Some(message) = message {
            let _ = self.sender.send(EditorLogEvent {
                message,
                level: *metadata.level(),
            });
        }
    }
}

struct EditorLogVisitor<'a>(&'a mut Option<String>);
impl tracing::field::Visit for EditorLogVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let mut text = format!("{value:?}");
            if text.starts_with('\"') && text.ends_with('\"') && text.len() >= 2 {
                text = text[1..text.len() - 1].to_string();
            }
            *self.0 = Some(text);
        }
    }
}

pub fn editor_log_layer(app: &mut App) -> Option<BoxedLayer> {
    let (sender, receiver) = mpsc::channel();

    app.insert_non_send_resource(CapturedEditorLogs(receiver));
    app.add_event::<EditorLogEvent>();
    app.add_systems(Update, transfer_editor_logs);

    Some(EditorLogLayer { sender }.boxed())
}

fn collect_editor_logs(
    mut log_reader: EventReader<EditorLogEvent>,
    mut editor_output: ResMut<EditorOutput>,
) {
    for message in log_reader.read() {
        editor_output
            .lines
            .push(format!("[{}] {}", message.level, message.message));
    }

    const MAX_LINES: usize = 500;
    if editor_output.lines.len() > MAX_LINES {
        let excess = editor_output.lines.len() - MAX_LINES;
        editor_output.lines.drain(0..excess);
    }
}

fn is_game_log_target(target: &str) -> bool {
    matches!(target, "waffle_game" | "game" | "gameplay")
}

fn refresh_asset_cache(mut cache: ResMut<AssetBrowserCache>) {
    let needs_scan = cache
        .last_scan
        .map(|last| last.elapsed() >= Duration::from_secs(1))
        .unwrap_or(true);
    if !needs_scan {
        return;
    }

    let mut entries = Vec::new();
    if cache.root.exists() {
        for entry in WalkDir::new(&cache.root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let rel = path.strip_prefix(&cache.root).unwrap_or(path);
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let kind = classify_asset(path.extension().and_then(|ext| ext.to_str()));
            entries.push(AssetEntry {
                path: rel_str,
                kind,
            });
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    cache.entries = entries;
    cache.last_scan = Some(Instant::now());
}

fn handle_file_drops(
    file_drop_events: &mut EventReader<FileDragAndDrop>,
    cache: &mut ResMut<AssetBrowserCache>,
) {
    for event in file_drop_events.read() {
        let FileDragAndDrop::DroppedFile { path_buf, .. } = event else {
            continue;
        };

        let assets_root = cache.root.clone();
        if !assets_root.exists() {
            if let Err(err) = std::fs::create_dir_all(&assets_root) {
                error!("Failed to create assets directory: {err}");
                continue;
            }
        }

        if path_buf.is_dir() {
            if let Err(err) = copy_dir_recursively(path_buf, &assets_root) {
                error!("Failed to import folder {}: {err}", path_buf.display());
                continue;
            }
        } else if path_buf.is_file() {
            if let Err(err) = copy_file_unique(path_buf, &assets_root) {
                error!("Failed to import file {}: {err}", path_buf.display());
                continue;
            }
            if path_buf.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("gltf")).unwrap_or(false) {
                if let Some(stem) = path_buf.file_stem().and_then(|s| s.to_str()) {
                    let bin_path = path_buf.with_file_name(format!("{stem}.bin"));
                    if bin_path.exists() {
                        let _ = copy_file_unique(&bin_path, &assets_root);
                    }
                }
            }
        }

        cache.last_scan = None;
    }
}

fn copy_file_unique(src: &std::path::Path, dest_root: &std::path::Path) -> std::io::Result<()> {
    let file_name = src.file_name().ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "missing file name",
    ))?;
    let mut dest = dest_root.join(file_name);
    if dest.exists() {
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("");
        for i in 1..=999 {
            let candidate = if ext.is_empty() {
                format!("{stem}_{i}")
            } else {
                format!("{stem}_{i}.{ext}")
            };
            let path = dest_root.join(candidate);
            if !path.exists() {
                dest = path;
                break;
            }
        }
    }
    std::fs::copy(src, &dest)?;
    Ok(())
}

fn copy_dir_recursively(src: &std::path::Path, dest_root: &std::path::Path) -> std::io::Result<()> {
    let folder_name = src.file_name().ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "missing folder name",
    ))?;
    let dest_dir = dest_root.join(folder_name);
    for entry in WalkDir::new(src).into_iter().filter_map(Result::ok) {
        let rel = entry.path().strip_prefix(src).unwrap_or(entry.path());
        let target = dest_dir.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn classify_asset(extension: Option<&str>) -> AssetKind {
    match extension.map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "tga" | "hdr") => {
            AssetKind::Image
        }
        Some(ext) if matches!(ext.as_str(), "gltf" | "glb" | "obj") => AssetKind::Model,
        Some(ext) if matches!(ext.as_str(), "wav" | "ogg" | "mp3") => AssetKind::Audio,
        Some(ext) if matches!(ext.as_str(), "lua") => AssetKind::Script,
        Some(ext) if matches!(ext.as_str(), "ron" | "json") => AssetKind::Material,
        _ => AssetKind::Other,
    }
}

fn apply_reparent_events(
    mut commands: Commands,
    mut events: EventReader<HierarchyReparentEvent>,
    children_query: Query<&Children>,
) {
    for event in events.read() {
        if let Some(parent) = event.new_parent {
            if event.child == parent {
                continue;
            }
            if is_descendant(event.child, parent, &children_query) {
                continue;
            }
            commands.entity(event.child).set_parent(parent);
        } else {
            commands.entity(event.child).remove_parent();
        }
    }
}

fn apply_delete_events(
    mut commands: Commands,
    mut events: EventReader<DeleteEntityEvent>,
) {
    for event in events.read() {
        if let Some(entity) = commands.get_entity(event.entity) {
            entity.despawn_recursive();
        }
    }
}

fn apply_spawn_primitive_events(
    mut commands: Commands,
    mut events: EventReader<SpawnPrimitiveEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_settings: Option<Res<SceneSettings>>,
    scene_root_query: Query<Entity, With<WaffleSceneRoot>>,
) {
    let default_material = scene_settings
        .as_ref()
        .map(|settings| settings.default_material.clone())
        .unwrap_or_else(|| {
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.8),
                perceptual_roughness: 0.7,
                metallic: 0.0,
                ..default()
            })
        });

    let root = scene_root_query.get_single().ok();

    for event in events.read() {
        let parent = event.parent.or(root);
        let mut entity_commands = match event.kind {
            SpawnPrimitiveKind::Empty => {
                commands.spawn((
                    WaffleSceneObject,
                    Name::new("Entity"),
                    Transform::default(),
                    GlobalTransform::default(),
                ))
            }
            SpawnPrimitiveKind::Cube => commands.spawn((
                WaffleSceneObject,
                Name::new("Cube"),
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: default_material.clone(),
                    ..default()
                },
            )),
            SpawnPrimitiveKind::Sphere => commands.spawn((
                WaffleSceneObject,
                Name::new("Sphere"),
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.5)),
                    material: default_material.clone(),
                    ..default()
                },
            )),
            SpawnPrimitiveKind::Plane => commands.spawn((
                WaffleSceneObject,
                Name::new("Plane"),
                PbrBundle {
                    mesh: meshes.add(Plane3d::default().mesh().size(2.0, 2.0)),
                    material: default_material.clone(),
                    ..default()
                },
            )),
            SpawnPrimitiveKind::DirectionalLight => commands.spawn((
                WaffleLight {
                    light_type: crate::rendering::lighting::LightType::Directional,
                    intensity: 10000.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    range: 100.0,
                    shadows_enabled: true,
                },
                crate::rendering::lighting::WaffleDirectionalLight,
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance: 10000.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_rotation(Quat::from_rotation_x(
                        -std::f32::consts::FRAC_PI_4,
                    )),
                    ..default()
                },
                Name::new("Directional Light"),
            )),
            SpawnPrimitiveKind::PointLight => commands.spawn((
                WaffleLight {
                    light_type: crate::rendering::lighting::LightType::Point,
                    intensity: 5000.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    range: 10.0,
                    shadows_enabled: true,
                },
                crate::rendering::lighting::WafflePointLight,
                PointLightBundle {
                    point_light: PointLight {
                        intensity: 5000.0,
                        color: Color::srgb(1.0, 1.0, 1.0),
                        range: 10.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::default(),
                    ..default()
                },
                Name::new("Point Light"),
            )),
            SpawnPrimitiveKind::SpotLight => commands.spawn((
                WaffleLight {
                    light_type: crate::rendering::lighting::LightType::Spot,
                    intensity: 8000.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    range: 15.0,
                    shadows_enabled: true,
                },
                crate::rendering::lighting::WaffleSpotLight,
                SpotLightBundle {
                    spot_light: SpotLight {
                        intensity: 8000.0,
                        color: Color::srgb(1.0, 1.0, 1.0),
                        range: 15.0,
                        shadows_enabled: true,
                        inner_angle: 0.6,
                        outer_angle: 0.9,
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::ZERO)
                        .looking_at(Vec3::new(0.0, 0.0, -1.0), Vec3::Y),
                    ..default()
                },
                Name::new("Spot Light"),
            )),
        };

        if let Some(parent_entity) = parent {
            entity_commands.set_parent(parent_entity);
        }
    }
}

fn apply_spawn_asset_events(
    mut commands: Commands,
    mut events: EventReader<SpawnAssetEvent>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_settings: Option<Res<SceneSettings>>,
    scene_root_query: Query<Entity, With<WaffleSceneRoot>>,
) {
    let default_material = scene_settings
        .as_ref()
        .map(|settings| settings.default_material.clone())
        .unwrap_or_else(|| {
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.8),
                perceptual_roughness: 0.7,
                metallic: 0.0,
                ..default()
            })
        });
    let root = scene_root_query.get_single().ok();

    for event in events.read() {
        let parent = event.parent.or(root);
        let path = event.path.clone();
        let extension = std::path::Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Asset")
            .to_string();

        let mut entity_commands = if matches!(extension.as_str(), "gltf" | "glb") {
            let scene_path = format!("{path}#Scene0");
            commands.spawn((
                WaffleSceneObject,
                Name::new(name),
                SceneBundle {
                    scene: asset_server.load(scene_path),
                    ..default()
                },
            ))
        } else if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "tga") {
            let texture = asset_server.load(path.clone());
            let material = materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                perceptual_roughness: 0.8,
                metallic: 0.0,
                ..default()
            });
            commands.spawn((
                WaffleSceneObject,
                Name::new(name),
                PbrBundle {
                    mesh: meshes.add(Plane3d::default().mesh().size(2.0, 2.0)),
                    material,
                    ..default()
                },
            ))
        } else if extension == "obj" {
            let mesh_handle: Handle<Mesh> = asset_server.load(path.clone());
            commands.spawn((
                WaffleSceneObject,
                Name::new(name),
                PbrBundle {
                    mesh: mesh_handle,
                    material: default_material.clone(),
                    ..default()
                },
            ))
        } else {
            commands.spawn((
                WaffleSceneObject,
                Name::new(format!("Asset {}", name)),
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: default_material.clone(),
                    ..default()
                },
            ))
        };

        if let Some(parent_entity) = parent {
            entity_commands.set_parent(parent_entity);
        }
    }
}

fn is_descendant(
    ancestor: Entity,
    node: Entity,
    children_query: &Query<&Children>,
) -> bool {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if current == ancestor {
            return true;
        }
        if let Ok(children) = children_query.get(current) {
            for child in children.iter().copied() {
                stack.push(child);
            }
        }
    }
    false
}

