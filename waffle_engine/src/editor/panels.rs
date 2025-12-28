/// Editor Panels Module
/// Individual panel implementations for the editor UI

use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::BTreeMap;

use super::{
    AssetBrowserCache, AssetEntry, AssetKind, EditorOutput, EditorState, EditorSettings,
    HierarchyReparentEvent, HierarchySnapshot, SpawnAssetEvent, SpawnPrimitiveEvent,
    SpawnPrimitiveKind,
};

#[derive(Clone)]
enum DragPayload {
    Entity(Entity),
    Asset(String),
}

/// Draw the viewport panel
pub fn draw_viewport_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    _editor_settings: &mut EditorSettings,
    viewport_texture_id: Option<egui::TextureId>,
) {
    ui.vertical_centered(|ui| {
        ui.heading("3D Viewport");

        ui.separator();

        // Viewport placeholder
        let available_size = ui.available_size();
        let viewport_size = egui::vec2(
            available_size.x.max(0.0),
            (available_size.y - 50.0).max(0.0),
        );
        let (viewport_rect, viewport_response) = if let Some(texture_id) = viewport_texture_id {
            let image = egui::Image::new(egui::load::SizedTexture::new(
                texture_id,
                viewport_size,
            ))
            .fit_to_exact_size(viewport_size)
            .sense(egui::Sense::click_and_drag());
            let response = ui.add(image);
            (response.rect, response)
        } else {
            let (rect, response) = ui.allocate_exact_size(viewport_size, egui::Sense::click_and_drag());
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Viewport render target not ready",
                egui::TextStyle::Body.resolve(ui.style()),
                egui::Color32::from_rgb(180, 180, 180),
            );
            (rect, response)
        };

        let pixels_per_point = ui.ctx().pixels_per_point();
        editor_state.viewport_size = Vec2::new(
            viewport_rect.width() * pixels_per_point,
            viewport_rect.height() * pixels_per_point,
        );
        editor_state.viewport_origin = Vec2::new(viewport_rect.min.x, viewport_rect.min.y);
        editor_state.viewport_hovered = viewport_response.hovered();
        let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
        editor_state.viewport_clicked = primary_pressed && viewport_response.hovered();
        editor_state.viewport_click_pos = None;
        if editor_state.viewport_clicked {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                let local = pointer_pos - viewport_rect.min;
                let local_pixels = local * pixels_per_point;
                editor_state.viewport_click_pos = Some(Vec2::new(local_pixels.x, local_pixels.y));
            }
        }

        // Draw border
        ui.painter().rect_stroke(
            viewport_rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80))
        );

        // Draw label without blocking the scene
        ui.painter().text(
            viewport_rect.left_top() + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            "3D Scene Viewport",
            egui::TextStyle::Body.resolve(ui.style()),
            egui::Color32::from_rgb(180, 180, 180),
        );

        if let Some(overlay) = editor_state.gizmo_overlay.as_ref() {
            let painter = ui.painter();
            let ppp = ui.ctx().pixels_per_point();
            let to_points = |p: Vec2| egui::pos2(
                viewport_rect.min.x + p.x / ppp,
                viewport_rect.min.y + p.y / ppp,
            );
            let origin = to_points(overlay.origin);
            let x_end = to_points(overlay.x_end);
            let y_end = to_points(overlay.y_end);
            let z_end = to_points(overlay.z_end);

            let axis_color = |axis: super::GizmoAxis, active: Option<super::GizmoAxis>| {
                let is_active = active == Some(axis);
                match axis {
                    super::GizmoAxis::X => if is_active {
                        egui::Color32::from_rgb(255, 140, 140)
                    } else {
                        egui::Color32::from_rgb(230, 70, 70)
                    },
                    super::GizmoAxis::Y => if is_active {
                        egui::Color32::from_rgb(140, 255, 140)
                    } else {
                        egui::Color32::from_rgb(70, 230, 70)
                    },
                    super::GizmoAxis::Z => if is_active {
                        egui::Color32::from_rgb(140, 200, 255)
                    } else {
                        egui::Color32::from_rgb(70, 140, 230)
                    },
                }
            };

            match editor_state.gizmo_mode {
                super::GizmoMode::Move => {
                    draw_axis_arrow(
                        painter,
                        origin,
                        x_end,
                        axis_color(super::GizmoAxis::X, editor_state.active_axis),
                    );
                    draw_axis_arrow(
                        painter,
                        origin,
                        y_end,
                        axis_color(super::GizmoAxis::Y, editor_state.active_axis),
                    );
                    draw_axis_arrow(
                        painter,
                        origin,
                        z_end,
                        axis_color(super::GizmoAxis::Z, editor_state.active_axis),
                    );
                }
                super::GizmoMode::Rotate => {
                    if let Some(rings) = overlay.rotate_rings.as_ref() {
                        let x_points: Vec<egui::Pos2> = rings
                            .x_points
                            .iter()
                            .map(|p| to_points(*p))
                            .collect();
                        let y_points: Vec<egui::Pos2> = rings
                            .y_points
                            .iter()
                            .map(|p| to_points(*p))
                            .collect();
                        let z_points: Vec<egui::Pos2> = rings
                            .z_points
                            .iter()
                            .map(|p| to_points(*p))
                            .collect();
                        draw_axis_polyline(
                            painter,
                            &x_points,
                            axis_color(super::GizmoAxis::X, editor_state.active_axis),
                        );
                        draw_axis_polyline(
                            painter,
                            &y_points,
                            axis_color(super::GizmoAxis::Y, editor_state.active_axis),
                        );
                        draw_axis_polyline(
                            painter,
                            &z_points,
                            axis_color(super::GizmoAxis::Z, editor_state.active_axis),
                        );
                    }
                }
                super::GizmoMode::Scale => {
                    draw_axis_scale(
                        painter,
                        origin,
                        x_end,
                        axis_color(super::GizmoAxis::X, editor_state.active_axis),
                    );
                    draw_axis_scale(
                        painter,
                        origin,
                        y_end,
                        axis_color(super::GizmoAxis::Y, editor_state.active_axis),
                    );
                    draw_axis_scale(
                        painter,
                        origin,
                        z_end,
                        axis_color(super::GizmoAxis::Z, editor_state.active_axis),
                    );
                }
            }
        }

        // Handle viewport focus
        let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
        let right_down = ui.input(|i| i.pointer.secondary_down());
        let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if viewport_response.clicked() || (right_down && viewport_response.hovered()) {
            editor_state.viewport_focused = true;
        } else if escape_pressed {
            editor_state.viewport_focused = false;
        } else if primary_clicked && !viewport_response.hovered() {
            editor_state.viewport_focused = false;
        }

        if editor_state.viewport_focused {
            ui.label("Viewport Focused - RMB look, Esc release, LMB drag to move");
        }
    });
}

fn draw_axis_arrow(
    painter: &egui::Painter,
    origin: egui::Pos2,
    end: egui::Pos2,
    color: egui::Color32,
) {
    painter.line_segment([origin, end], egui::Stroke::new(2.0, color));
    let dir = (end - origin).normalized();
    let head = 8.0;
    let left = egui::vec2(-dir.y, dir.x);
    let p1 = end - dir * head + left * (head * 0.6);
    let p2 = end - dir * head - left * (head * 0.6);
    painter.add(egui::Shape::convex_polygon(vec![end, p1, p2], color, egui::Stroke::NONE));
}

fn draw_axis_polyline(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    color: egui::Color32,
) {
    if points.len() < 2 {
        return;
    }
    painter.add(egui::Shape::line(
        points.to_vec(),
        egui::Stroke::new(2.0, color),
    ));
}

fn draw_axis_scale(
    painter: &egui::Painter,
    origin: egui::Pos2,
    end: egui::Pos2,
    color: egui::Color32,
) {
    painter.line_segment([origin, end], egui::Stroke::new(2.0, color));
    let size = 6.0;
    let rect = egui::Rect::from_center_size(end, egui::vec2(size * 1.6, size * 1.6));
    painter.rect_filled(rect, 1.0, color);
}

/// Draw the hierarchy panel
pub fn draw_hierarchy_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    _editor_settings: &mut EditorSettings,
    hierarchy: &HierarchySnapshot,
    reparent_events: &mut Vec<HierarchyReparentEvent>,
    spawn_primitive_queue: &mut Vec<SpawnPrimitiveEvent>,
    spawn_asset_queue: &mut Vec<SpawnAssetEvent>,
) {
    ui.vertical(|ui| {
        ui.heading("Scene Hierarchy");

        ui.separator();

        ui.horizontal(|ui| {
            ui.menu_button("+", |ui| {
                if ui.button("Empty").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::Empty,
                        parent: None,
                    });
                    ui.close_menu();
                }
                if ui.button("Cube").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::Cube,
                        parent: None,
                    });
                    ui.close_menu();
                }
                if ui.button("Sphere").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::Sphere,
                        parent: None,
                    });
                    ui.close_menu();
                }
                if ui.button("Plane").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::Plane,
                        parent: None,
                    });
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Directional Light").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::DirectionalLight,
                        parent: None,
                    });
                    ui.close_menu();
                }
                if ui.button("Point Light").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::PointLight,
                        parent: None,
                    });
                    ui.close_menu();
                }
                if ui.button("Spot Light").clicked() {
                    spawn_primitive_queue.push(SpawnPrimitiveEvent {
                        kind: SpawnPrimitiveKind::SpotLight,
                        parent: None,
                    });
                    ui.close_menu();
                }
            });
            if ui.button("X").on_hover_text("Delete").clicked() {
                if let Some(entity) = editor_state.selected_entity {
                    editor_state.delete_confirm = Some(entity);
                }
            }
            if ui.button("D").clicked() {
                // TODO: Duplicate selected entity
            }
        });

        ui.separator();

        // Search bar
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut editor_state.hierarchy_filter);
        });

        ui.separator();

        let mut clicked_entity = false;
        let scroll_response = egui::ScrollArea::vertical().show(ui, |ui| {
            let filter = editor_state.hierarchy_filter.trim().to_lowercase();
            if filter.is_empty() {
                let root_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgb(35, 35, 35))
                    .inner_margin(egui::Margin::symmetric(4.0, 2.0));
                let (root_response, dropped_root) = ui.dnd_drop_zone(root_frame, |ui| {
                    ui.label("Scene Root");
                });
                if root_response.response.clicked() {
                    editor_state.selected_entity = None;
                    clicked_entity = true;
                }
                if let Some(payload) = dropped_root.map(|payload: std::sync::Arc<DragPayload>| (*payload).clone()) {
                    match payload {
                        DragPayload::Entity(entity) => reparent_events.push(HierarchyReparentEvent {
                            child: entity,
                            new_parent: None,
                        }),
                        DragPayload::Asset(path) => spawn_asset_queue.push(SpawnAssetEvent {
                            path,
                            parent: None,
                        }),
                    }
                }

                if hierarchy.roots.is_empty() {
                    ui.label("No entities found");
                } else {
                    for root in &hierarchy.roots {
                        draw_hierarchy_node(
                            ui,
                            *root,
                            hierarchy,
                            editor_state,
                            reparent_events,
                            spawn_asset_queue,
                            &mut clicked_entity,
                        );
                    }
                }
            } else {
                let mut matches: Vec<(Entity, &str)> = hierarchy
                    .names
                    .iter()
                    .filter_map(|(entity, name)| {
                        let name_lower = name.to_lowercase();
                        if name_lower.contains(&filter) {
                            Some((*entity, name.as_str()))
                        } else {
                            None
                        }
                    })
                    .collect();
                matches.sort_by(|a, b| a.1.cmp(b.1));

                if matches.is_empty() {
                    ui.label("No matching entities");
                } else {
                    for (entity, name) in matches {
                        let selected = editor_state.selected_entity == Some(entity);
                        if ui.selectable_label(selected, name).clicked() {
                            editor_state.selected_entity = Some(entity);
                            clicked_entity = true;
                        }
                    }
                }
            }
        });
        let pointer_clicked = ui.input(|i| i.pointer.primary_clicked());
        let pointer_pos = ui.input(|i| i.pointer.latest_pos());
        if pointer_clicked && !clicked_entity {
            if let Some(pos) = pointer_pos {
                if scroll_response.inner_rect.contains(pos) {
                    editor_state.selected_entity = None;
                }
            }
        }

    });
}

fn draw_hierarchy_node(
    ui: &mut egui::Ui,
    entity: Entity,
    hierarchy: &HierarchySnapshot,
    editor_state: &mut EditorState,
    reparent_events: &mut Vec<HierarchyReparentEvent>,
    spawn_asset_queue: &mut Vec<SpawnAssetEvent>,
    clicked_entity: &mut bool,
) {
    let name = hierarchy
        .names
        .get(&entity)
        .map(|s| s.as_str())
        .unwrap_or("Entity");
    let selected = editor_state.selected_entity == Some(entity);

    let drag_payload = DragPayload::Entity(entity);
    let frame = egui::Frame::none()
        .fill(egui::Color32::from_rgb(32, 32, 32))
        .inner_margin(egui::Margin::symmetric(4.0, 1.0));

    ui.push_id(entity, |ui| {
        if let Some(children) = hierarchy.children.get(&entity) {
            let id = ui.make_persistent_id(("hierarchy_collapse", entity));
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                true,
            );
            let (_toggle, header, _body) = state
                .show_header(ui, |ui| {
                    let mut label_clicked = false;
                    let (inner, dropped) = ui.dnd_drop_zone(frame, |ui| {
                        let label = ui.selectable_label(selected, name);
                        let drag_id = ui.make_persistent_id(("hierarchy_drag", entity));
                        let drag_response =
                            ui.interact(label.rect, drag_id, egui::Sense::click_and_drag());
                        drag_response.dnd_set_drag_payload(drag_payload.clone());
                        if drag_response.clicked() {
                            label_clicked = true;
                        }
                    });
                    if let Some(payload) = dropped.map(|payload: std::sync::Arc<DragPayload>| (*payload).clone()) {
                        match payload {
                            DragPayload::Entity(child) => {
                                if child != entity {
                                    reparent_events.push(HierarchyReparentEvent {
                                        child,
                                        new_parent: Some(entity),
                                    });
                                }
                            }
                            DragPayload::Asset(path) => {
                                spawn_asset_queue.push(SpawnAssetEvent {
                                    path,
                                    parent: Some(entity),
                                });
                            }
                        }
                    }
                      if label_clicked {
                          editor_state.selected_entity = Some(entity);
                          *clicked_entity = true;
                      }
                      inner.inner
                  })
                  .body(|ui| {
                      for child in children {
                          draw_hierarchy_node(
                              ui,
                              *child,
                              hierarchy,
                              editor_state,
                              reparent_events,
                              spawn_asset_queue,
                              clicked_entity,
                          );
                      }
                  });
              if header.response.clicked() {
                  editor_state.selected_entity = Some(entity);
                  *clicked_entity = true;
              }
          } else {
            let mut label_clicked = false;
            let (inner, dropped) = ui.dnd_drop_zone(frame, |ui| {
                let label = ui.selectable_label(selected, name);
                let drag_id = ui.make_persistent_id(("hierarchy_drag", entity));
                let drag_response =
                    ui.interact(label.rect, drag_id, egui::Sense::click_and_drag());
                drag_response.dnd_set_drag_payload(drag_payload.clone());
                if drag_response.clicked() {
                    label_clicked = true;
                }
            });
              if label_clicked || inner.response.clicked() {
                  editor_state.selected_entity = Some(entity);
                  *clicked_entity = true;
              }
            if let Some(payload) = dropped.map(|payload: std::sync::Arc<DragPayload>| (*payload).clone()) {
                match payload {
                    DragPayload::Entity(child) => {
                        if child != entity {
                            reparent_events.push(HierarchyReparentEvent {
                                child,
                                new_parent: Some(entity),
                            });
                        }
                    }
                    DragPayload::Asset(path) => {
                        spawn_asset_queue.push(SpawnAssetEvent {
                            path,
                            parent: Some(entity),
                        });
                    }
                }
            }
        }
    });
}

/// Draw the inspector panel
pub fn draw_inspector_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    _editor_settings: &mut EditorSettings,
    selected_transform: Option<&mut Transform>,
    mut selected_name: Option<&mut Name>,
    selected_material_handle: Option<&Handle<StandardMaterial>>,
    mut selected_overrides: Option<&mut crate::rendering::materials::PbrTextureOverrides>,
    mut selected_environment: Option<&mut crate::rendering::scene::EnvironmentSettings>,
    mut selected_atmosphere: Option<&mut crate::rendering::atmosphere::AtmosphereSettingsComponent>,
    material_assets: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    selected_asset: Option<&str>,
    selected_waffle_light: Option<&mut crate::rendering::lighting::WaffleLight>,
    selected_directional_light: Option<&mut DirectionalLight>,
    selected_point_light: Option<&mut PointLight>,
    selected_spot_light: Option<&mut SpotLight>,
) {
    ui.vertical(|ui| {
        ui.heading("Inspector");

        ui.separator();

        if let Some(entity) = editor_state.selected_entity {
            ui.label(format!("Selected Entity: {}", entity.index()));

            ui.separator();

            ui.collapsing("Name", |ui| {
                if let Some(name) = selected_name.as_deref_mut() {
                    let mut value = name.as_str().to_string();
                    if ui.text_edit_singleline(&mut value).changed() {
                        *name = Name::new(value);
                    }
                } else {
                    ui.label("No name component");
                }
            });

            // Transform component
            ui.collapsing("Transform", |ui| {
                if let Some(transform) = selected_transform {
                    let mut translation = transform.translation;
                    let rotation = transform.rotation.to_euler(EulerRot::YXZ);
                    let mut rotation_deg = Vec3::new(
                        rotation.1.to_degrees(),
                        rotation.0.to_degrees(),
                        rotation.2.to_degrees(),
                    );
                    let mut scale = transform.scale;

                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut translation.x).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut translation.y).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut translation.z).prefix("Z: "));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.add(egui::DragValue::new(&mut rotation_deg.x).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut rotation_deg.y).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut rotation_deg.z).prefix("Z: "));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(egui::DragValue::new(&mut scale.x).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut scale.y).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut scale.z).prefix("Z: "));
                    });

                    transform.translation = translation;
                    transform.rotation = Quat::from_euler(
                        EulerRot::YXZ,
                        rotation_deg.y.to_radians(),
                        rotation_deg.x.to_radians(),
                        rotation_deg.z.to_radians(),
                    );
                    transform.scale = scale;
                } else {
                    ui.label("No transform component");
                }
            });

            if let Some(handle) = selected_material_handle {
                if let Some(material) = material_assets.get_mut(handle) {
                    ui.collapsing("Material", |ui| {
                        let selected_texture = selected_asset
                            .filter(|path| is_image_path(path))
                            .map(|path| asset_server.load(path.to_string()));
                        ui.label(format!("Source: {}", material_handle_label(&handle)));

                        ui.horizontal(|ui| {
                            ui.label("Base Color:");
                            let mut base_color = color_to_egui(material.base_color);
                            if ui.color_edit_button_srgba(&mut base_color).changed() {
                                material.base_color = egui_to_color(base_color);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Albedo Map:");
                            ui.label(image_handle_label(&material.base_color_texture));
                            if let Some(handle) = selected_texture.as_ref() {
                                if ui.button("Use Selected").clicked() {
                                    material.base_color_texture = Some(handle.clone());
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Emissive:");
                            let mut emissive = color_to_egui(Color::LinearRgba(material.emissive));
                            if ui.color_edit_button_srgba(&mut emissive).changed() {
                                material.emissive = egui_to_color(emissive).to_linear();
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Emissive Map:");
                            ui.label(image_handle_label(&material.emissive_texture));
                            if let Some(handle) = selected_texture.as_ref() {
                                if ui.button("Use Selected").clicked() {
                                    material.emissive_texture = Some(handle.clone());
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Roughness:");
                            ui.add(egui::Slider::new(&mut material.perceptual_roughness, 0.0..=1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Metallic:");
                            ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Metal/Rough Map:");
                            ui.label(image_handle_label(&material.metallic_roughness_texture));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Normal Map:");
                            ui.label(image_handle_label(&material.normal_map_texture));
                            if let Some(handle) = selected_texture.as_ref() {
                                if ui.button("Use Selected").clicked() {
                                    material.normal_map_texture = Some(handle.clone());
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("AO Map:");
                            ui.label(image_handle_label(&material.occlusion_texture));
                            if let Some(handle) = selected_texture.as_ref() {
                                if ui.button("Use Selected").clicked() {
                                    material.occlusion_texture = Some(handle.clone());
                                }
                            }
                        });

                        ui.separator();
                        ui.label("Separate Metallic/Roughness:");
                            if let Some(overrides) = selected_overrides.as_deref_mut() {
                                ui.horizontal(|ui| {
                                    ui.label("Metallic Map:");
                                    ui.label(image_handle_label(&overrides.metallic_map));
                                    if let Some(handle) = selected_texture.as_ref() {
                                        if ui.button("Use Selected").clicked() {
                                            overrides.metallic_map = Some(handle.clone());
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Roughness Map:");
                                    ui.label(image_handle_label(&overrides.roughness_map));
                                    if let Some(handle) = selected_texture.as_ref() {
                                        if ui.button("Use Selected").clicked() {
                                            overrides.roughness_map = Some(handle.clone());
                                        }
                                    }
                            });
                        } else {
                            ui.label("No PBR override component");
                        }
                    });
                }
            }

            if let Some(env) = selected_environment.as_deref_mut() {
                ui.collapsing("Environment", |ui| {
                    ui.label("Sky & Sun");
                    ui.horizontal(|ui| {
                        ui.label("Time of Day:");
                        ui.add(egui::Slider::new(&mut env.time_of_day, 0.0..=24.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Azimuth:");
                        ui.add(egui::Slider::new(&mut env.sun_azimuth, 0.0..=360.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Color:");
                        let mut color = color_to_egui(env.sun_color);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            env.sun_color = egui_to_color(color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Intensity:");
                        ui.add(egui::DragValue::new(&mut env.sun_intensity).speed(100.0).range(0.0..=200000.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Disk Intensity:");
                        ui.add(egui::Slider::new(&mut env.sun_disk_intensity, 0.0..=10.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Disk Size:");
                        ui.add(egui::Slider::new(&mut env.sun_disk_size, 0.001..=0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sky Top (Day):");
                        let mut color = color_to_egui(env.sky_top_day);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            env.sky_top_day = egui_to_color(color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sky Horizon (Day):");
                        let mut color = color_to_egui(env.sky_horizon_day);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            env.sky_horizon_day = egui_to_color(color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sky Top (Night):");
                        let mut color = color_to_egui(env.sky_top_night);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            env.sky_top_night = egui_to_color(color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sky Horizon (Night):");
                        let mut color = color_to_egui(env.sky_horizon_night);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            env.sky_horizon_night = egui_to_color(color);
                        }
                    });

                    ui.separator();
                    ui.label("Ambient");
                    ui.horizontal(|ui| {
                        ui.label("Ambient Color:");
                        let mut color = color_to_egui(env.ambient_color);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            env.ambient_color = egui_to_color(color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ambient Intensity:");
                        ui.add(egui::DragValue::new(&mut env.ambient_intensity).speed(10.0).range(0.0..=10000.0));
                    });

                    ui.separator();
                    ui.label("Post Processing");

                    ui.horizontal(|ui| {
                        ui.label("Exposure (EV100):");
                        ui.add(egui::DragValue::new(&mut env.exposure_ev100));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Tonemapping:");
                        egui::ComboBox::from_id_source("tonemapping_mode")
                            .selected_text(format!("{:?}", env.tonemapping))
                            .show_ui(ui, |ui| {
                                for mode in [
                                    crate::rendering::scene::EnvironmentTonemapping::None,
                                    crate::rendering::scene::EnvironmentTonemapping::Reinhard,
                                    crate::rendering::scene::EnvironmentTonemapping::AcesFitted,
                                    crate::rendering::scene::EnvironmentTonemapping::AgX,
                                    crate::rendering::scene::EnvironmentTonemapping::SomewhatBoringDisplayTransform,
                                    crate::rendering::scene::EnvironmentTonemapping::TonyMcMapface,
                                    crate::rendering::scene::EnvironmentTonemapping::BlenderFilmic,
                                ] {
                                    ui.selectable_value(&mut env.tonemapping, mode, format!("{:?}", mode));
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Gamma:");
                        ui.add(egui::Slider::new(&mut env.color_grading.gamma, 0.2..=2.5));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Pre Saturation:");
                        ui.add(egui::Slider::new(&mut env.color_grading.pre_saturation, 0.0..=2.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Post Saturation:");
                        ui.add(egui::Slider::new(&mut env.color_grading.post_saturation, 0.0..=2.0));
                    });

                    ui.separator();
                    ui.label("Bloom");
                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        ui.checkbox(&mut env.bloom.enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Intensity:");
                        ui.add(egui::Slider::new(&mut env.bloom.intensity, 0.0..=2.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Low Freq Boost:");
                        ui.add(egui::Slider::new(&mut env.bloom.low_frequency_boost, 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Threshold:");
                        ui.add(egui::Slider::new(&mut env.bloom.threshold, 0.0..=2.0));
                    });

                    ui.separator();
                    ui.label("Fog");
                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        ui.checkbox(&mut env.fog.enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut fog_color = color_to_egui(env.fog.color);
                        if ui.color_edit_button_srgba(&mut fog_color).changed() {
                            env.fog.color = egui_to_color(fog_color);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        egui::ComboBox::from_id_source("fog_mode")
                            .selected_text(format!("{:?}", env.fog.mode))
                            .show_ui(ui, |ui| {
                                for mode in [
                                    crate::rendering::scene::EnvironmentFogMode::Linear,
                                    crate::rendering::scene::EnvironmentFogMode::Exponential,
                                    crate::rendering::scene::EnvironmentFogMode::ExponentialSquared,
                                    crate::rendering::scene::EnvironmentFogMode::Atmospheric,
                                ] {
                                    ui.selectable_value(&mut env.fog.mode, mode, format!("{:?}", mode));
                                }
                            });
                    });
                    match env.fog.mode {
                        crate::rendering::scene::EnvironmentFogMode::Linear => {
                            ui.horizontal(|ui| {
                                ui.label("Start:");
                                ui.add(egui::DragValue::new(&mut env.fog.start));
                            });
                            ui.horizontal(|ui| {
                                ui.label("End:");
                                ui.add(egui::DragValue::new(&mut env.fog.end));
                            });
                        }
                        _ => {
                            ui.horizontal(|ui| {
                                ui.label("Density:");
                                ui.add(egui::DragValue::new(&mut env.fog.density));
                            });
                        }
                    }

                    ui.separator();
                    ui.label("Ambient Occlusion");
                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        ui.checkbox(&mut env.ssao.enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Quality:");
                        egui::ComboBox::from_id_source("ssao_quality")
                            .selected_text(format!("{:?}", env.ssao.quality))
                            .show_ui(ui, |ui| {
                                for quality in [
                                    crate::rendering::scene::EnvironmentSsaoQuality::Low,
                                    crate::rendering::scene::EnvironmentSsaoQuality::Medium,
                                    crate::rendering::scene::EnvironmentSsaoQuality::High,
                                    crate::rendering::scene::EnvironmentSsaoQuality::Ultra,
                                ] {
                                    ui.selectable_value(&mut env.ssao.quality, quality, format!("{:?}", quality));
                                }
                            });
                    });

                    ui.separator();
                    ui.label("Screen Space Reflections");
                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        ui.checkbox(&mut env.ssr.enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Roughness Threshold:");
                        ui.add(egui::Slider::new(&mut env.ssr.roughness_threshold, 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Thickness:");
                        ui.add(egui::Slider::new(&mut env.ssr.thickness, 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Linear Steps:");
                        ui.add(egui::DragValue::new(&mut env.ssr.linear_steps).speed(1.0).range(1..=128));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bisection Steps:");
                        ui.add(egui::DragValue::new(&mut env.ssr.bisection_steps).speed(1.0).range(0..=32));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Use Secant:");
                        ui.checkbox(&mut env.ssr.use_secant, "");
                    });
                });
            }

            if let Some(atmosphere) = selected_atmosphere.as_deref_mut() {
                ui.collapsing("Atmosphere", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Enabled:");
                        ui.checkbox(&mut atmosphere.atmosphere_enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Planet Radius:");
                        ui.add(egui::DragValue::new(&mut atmosphere.planet_radius));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Atmosphere Height:");
                        ui.add(egui::DragValue::new(&mut atmosphere.atmosphere_height));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rayleigh:");
                        ui.add(egui::DragValue::new(&mut atmosphere.rayleigh_scattering.x));
                        ui.add(egui::DragValue::new(&mut atmosphere.rayleigh_scattering.y));
                        ui.add(egui::DragValue::new(&mut atmosphere.rayleigh_scattering.z));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mie:");
                        ui.add(egui::DragValue::new(&mut atmosphere.mie_scattering.x));
                        ui.add(egui::DragValue::new(&mut atmosphere.mie_scattering.y));
                        ui.add(egui::DragValue::new(&mut atmosphere.mie_scattering.z));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Absorption:");
                        ui.add(egui::DragValue::new(&mut atmosphere.mie_absorption.x));
                        ui.add(egui::DragValue::new(&mut atmosphere.mie_absorption.y));
                        ui.add(egui::DragValue::new(&mut atmosphere.mie_absorption.z));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Intensity:");
                        ui.add(egui::DragValue::new(&mut atmosphere.sun_intensity));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sun Angle:");
                        ui.add(egui::DragValue::new(&mut atmosphere.sun_angle));
                    });
                });
            }

            if selected_directional_light.is_some()
                || selected_point_light.is_some()
                || selected_spot_light.is_some()
            {
                ui.collapsing("Light", |ui| {
                    if let Some(light) = selected_directional_light {
                        ui.label("Type: Directional");
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut color = color_to_egui(light.color);
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                light.color = egui_to_color(color);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Illuminance:");
                            ui.add(egui::DragValue::new(&mut light.illuminance).speed(100.0).range(0.0..=200000.0));
                        });
                        ui.checkbox(&mut light.shadows_enabled, "Shadows");
                    }

                    if let Some(light) = selected_point_light {
                        ui.separator();
                        ui.label("Type: Point");
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut color = color_to_egui(light.color);
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                light.color = egui_to_color(color);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::DragValue::new(&mut light.intensity).speed(100.0).range(0.0..=200000.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Range:");
                            ui.add(egui::DragValue::new(&mut light.range).speed(0.5).range(0.0..=200.0));
                        });
                        ui.checkbox(&mut light.shadows_enabled, "Shadows");
                    }

                    if let Some(light) = selected_spot_light {
                        ui.separator();
                        ui.label("Type: Spot");
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            let mut color = color_to_egui(light.color);
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                light.color = egui_to_color(color);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::DragValue::new(&mut light.intensity).speed(100.0).range(0.0..=200000.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Range:");
                            ui.add(egui::DragValue::new(&mut light.range).speed(0.5).range(0.0..=200.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Inner Angle:");
                            ui.add(egui::DragValue::new(&mut light.inner_angle).speed(0.01).range(0.0..=1.57));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Outer Angle:");
                            ui.add(egui::DragValue::new(&mut light.outer_angle).speed(0.01).range(0.0..=1.57));
                        });
                        ui.checkbox(&mut light.shadows_enabled, "Shadows");
                    }
                });
            }

            if let Some(light) = selected_waffle_light {
                ui.collapsing("Waffle Light", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Intensity:");
                        ui.add(egui::DragValue::new(&mut light.intensity).speed(100.0).range(0.0..=200000.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Range:");
                        ui.add(egui::DragValue::new(&mut light.range).speed(0.5).range(0.0..=200.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = color_to_egui(light.color);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            light.color = egui_to_color(color);
                        }
                    });
                    ui.checkbox(&mut light.shadows_enabled, "Shadows");
                });
            }

        } else {
            ui.vertical_centered(|ui| {
                ui.label("No entity selected");
                ui.label("Select an entity from the Hierarchy panel");
            });
        }
    });
}

/// Draw the assets panel
pub fn draw_assets_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    _editor_settings: &mut EditorSettings,
    asset_cache: &AssetBrowserCache,
    spawn_asset_queue: &mut Vec<SpawnAssetEvent>,
) {
    ui.vertical(|ui| {
        ui.heading("Assets");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut editor_state.asset_filter);
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let filter = editor_state.asset_filter.trim().to_lowercase();
            let mut shown_any = false;

            let mut folders: BTreeMap<String, Vec<&AssetEntry>> = BTreeMap::new();
            for entry in &asset_cache.entries {
                if !filter.is_empty() && !entry.path.to_lowercase().contains(&filter) {
                    continue;
                }
                let folder = std::path::Path::new(&entry.path)
                    .parent()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                folders.entry(folder).or_default().push(entry);
            }

            for (folder, entries) in folders {
                let header = if folder.is_empty() { "assets".to_string() } else { folder.clone() };
                egui::CollapsingHeader::new(header)
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.add_space(120.0);
                            ui.label("Type");
                        });
                        ui.separator();

                        let row_height = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
                        for entry in entries {
                            let path = std::path::Path::new(&entry.path);
                            let name = path
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&entry.path);

                            shown_any = true;
                            let selected = editor_state
                                .selected_asset
                                .as_ref()
                                .map(|path| path == &entry.path)
                                .unwrap_or(false);

                            let row_rect = ui.available_rect_before_wrap();
                            let row_rect = egui::Rect::from_min_size(
                                row_rect.min,
                                egui::vec2(row_rect.width(), row_height),
                            );
                            let row_id = ui.make_persistent_id(("asset_row", &entry.path));
                            let response = ui.interact(row_rect, row_id, egui::Sense::click_and_drag());
                            if selected {
                                ui.painter().rect_filled(
                                    row_rect,
                                    2.0,
                                    egui::Color32::from_rgb(45, 45, 55),
                                );
                            }
                            ui.painter().text(
                                row_rect.min + egui::vec2(6.0, 3.0),
                                egui::Align2::LEFT_TOP,
                                name,
                                egui::TextStyle::Body.resolve(ui.style()),
                                egui::Color32::from_rgb(220, 220, 220),
                            );
                            ui.painter().text(
                                row_rect.min + egui::vec2(180.0, 3.0),
                                egui::Align2::LEFT_TOP,
                                asset_kind_label(entry.kind),
                                egui::TextStyle::Body.resolve(ui.style()),
                                egui::Color32::from_rgb(160, 160, 160),
                            );

                            response.dnd_set_drag_payload(DragPayload::Asset(entry.path.clone()));
                            if response.clicked() || response.drag_started() {
                                editor_state.selected_asset = Some(entry.path.clone());
                            }
                            if response.double_clicked() {
                                spawn_asset_queue.push(SpawnAssetEvent {
                                    path: entry.path.clone(),
                                    parent: None,
                                });
                            }

                            ui.allocate_space(egui::vec2(0.0, row_height));
                        }
                    });
            }

            if !shown_any {
                if asset_cache.entries.is_empty() {
                    ui.label("No assets found");
                } else {
                    ui.label("No assets match the filter");
                }
            }
        });

        if let Some(selected) = editor_state.selected_asset.as_ref() {
            ui.separator();
            ui.label(format!("Selected: {}", selected));
        }
    });
}

fn color_to_egui(color: Color) -> egui::Color32 {
    let srgba = color.to_srgba();
    let r = (srgba.red.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (srgba.green.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (srgba.blue.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (srgba.alpha.clamp(0.0, 1.0) * 255.0) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn egui_to_color(color: egui::Color32) -> Color {
    let [r, g, b, a] = color.to_array();
    Color::srgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

fn image_handle_label(handle: &Option<Handle<Image>>) -> String {
    handle
        .as_ref()
        .and_then(|handle| handle.path())
        .map(|path| path.to_string())
        .unwrap_or_else(|| "None".to_string())
}

fn material_handle_label(handle: &Handle<StandardMaterial>) -> String {
    handle
        .path()
        .map(|path| path.to_string())
        .unwrap_or_else(|| "Runtime".to_string())
}

fn asset_kind_label(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Image => "[Image]",
        AssetKind::Model => "[Model]",
        AssetKind::Audio => "[Audio]",
        AssetKind::Script => "[Script]",
        AssetKind::Material => "[Material]",
        AssetKind::Other => "[Other]",
    }
}

fn is_image_path(path: &str) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "tga" | "hdr")
}

/// Draw the console panel
pub fn draw_console_panel(
    ui: &mut egui::Ui,
    _editor_settings: &mut EditorSettings,
    editor_output: &mut EditorOutput,
) {
    ui.vertical(|ui| {
        ui.heading("Output");

        ui.separator();

        // Output area
        egui::ScrollArea::vertical().show(ui, |ui| {
            if editor_output.lines.is_empty() {
                ui.label("No output yet");
            } else {
                for line in &editor_output.lines {
                    ui.label(line);
                }
            }
        });

        ui.separator();

        // Output-only console (no input)
    });
}

/// Draw the profiler panel
pub fn draw_profiler_panel(
    ui: &mut egui::Ui,
    _editor_state: &mut EditorState,
    _editor_settings: &mut EditorSettings,
    diagnostics: &bevy::diagnostic::DiagnosticsStore,
) {
    ui.vertical(|ui| {
        ui.heading("Profiler");

        ui.separator();

        let fps = diagnostics
            .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|diag| diag.smoothed());
        let frame_time = diagnostics
            .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|diag| diag.smoothed());

        // Performance metrics
        ui.collapsing("Performance", |ui| {
            ui.label(format!(
                "FPS: {}",
                fps.map(|v| format!("{v:.1}")).unwrap_or_else(|| "--".into())
            ));
            ui.label(format!(
                "Frame Time: {} ms",
                frame_time
                    .map(|v| format!("{:.2}", v))
                    .unwrap_or_else(|| "--".into())
            ));
            ui.label("CPU Usage: 45%");
            ui.label("Memory: 128 MB");
            ui.label("Draw Calls: 150");
        });

        // System timings
        ui.collapsing("Systems", |ui| {
            ui.label("Rendering: 8.3 ms");
            ui.label("Physics: 0.0 ms (disabled)");
            ui.label("UI: 2.1 ms");
            ui.label("Audio: 0.0 ms");
        });

        // Memory usage
        ui.collapsing("Memory", |ui| {
            ui.label("Textures: 64 MB");
            ui.label("Meshes: 8 MB");
            ui.label("Audio: 0 MB");
            ui.label("Scripts: 0 MB");
        });
    });
}

