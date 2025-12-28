/// Editor Windows Module
/// Floating windows and dialogs for the editor

use bevy::prelude::*;
use bevy_egui::egui;

use super::{EditorState, EditorSettings};

/// About dialog window
pub fn show_about_dialog(ctx: &egui::Context, open: &mut bool) {
    let mut is_open = *open;
    let mut should_close = false;
    egui::Window::new("About Waffle Engine")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Waffle Engine");
                ui.label("Version 0.1.0");
                ui.label("A production-ready game engine");
                ui.label("Built with Rust and Bevy");

                ui.separator();

                ui.label("(c) 2025 Waffle Engine Team");
                ui.hyperlink_to("GitHub Repository", "https://github.com/waffle-engine");

                ui.separator();

                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });
        });
    if should_close {
        is_open = false;
    }
    *open = is_open;
}

/// Preferences window
pub fn show_preferences_dialog(
    ctx: &egui::Context,
    open: &mut bool,
    editor_settings: &mut EditorSettings,
) {
    let mut is_open = *open;
    let mut should_close = false;
    egui::Window::new("Preferences")
        .open(&mut is_open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Appearance");

                ui.checkbox(&mut editor_settings.show_fps, "Show FPS Counter");
                ui.checkbox(&mut editor_settings.show_debug_info, "Show Debug Info");
                ui.checkbox(&mut editor_settings.grid_enabled, "Show Grid in Viewport");

                ui.horizontal(|ui| {
                    ui.label("Grid Size:");
                    ui.add(egui::DragValue::new(&mut editor_settings.grid_size).range(0.1..=10.0));
                });

                ui.separator();

                ui.heading("Controls");

                // TODO: Add control settings

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Reset to Defaults").clicked() {
                        *editor_settings = EditorSettings::default();
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            });
        });
    if should_close {
        is_open = false;
    }
    *open = is_open;
}

/// Asset import dialog
pub fn show_asset_import_dialog(ctx: &egui::Context, open: &mut bool) {
    let mut is_open = *open;
    let mut should_close = false;
    egui::Window::new("Import Assets")
        .open(&mut is_open)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Import Assets");

                ui.label("Select files to import:");
                // TODO: File picker integration

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Browse...").clicked() {
                        // TODO: Open file dialog
                    }

                    ui.separator();

                    if ui.button("Import").clicked() {
                        // TODO: Import assets
                        should_close = true;
                    }

                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });
        });
    if should_close {
        is_open = false;
    }
    *open = is_open;
}

/// Save scene dialog
pub fn show_save_scene_dialog(ctx: &egui::Context, open: &mut bool) {
    let mut is_open = *open;
    let mut should_close = false;
    egui::Window::new("Save Scene")
        .open(&mut is_open)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Save Scene");

                ui.label("Scene Name:");
                let mut scene_name = String::from("Untitled Scene");
                ui.text_edit_singleline(&mut scene_name);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        // TODO: Save scene
                        should_close = true;
                    }

                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                });
            });
        });
    if should_close {
        is_open = false;
    }
    *open = is_open;
}

