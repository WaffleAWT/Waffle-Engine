/// Editor UI Module
/// Main UI components and tab management

use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockState, TabViewer};

use super::{
    AssetBrowserCache, EditorOutput, EditorState, EditorSettings, EditorTab, HierarchyReparentEvent,
    HierarchySnapshot, SpawnAssetEvent, SpawnPrimitiveEvent,
};
use super::panels::*;

/// Tab viewer for the dock system
pub struct EditorTabViewer<'a> {
    pub editor_state: &'a mut EditorState,
    pub editor_settings: &'a mut EditorSettings,
    pub editor_output: &'a mut EditorOutput,
    pub hierarchy: &'a HierarchySnapshot,
    pub selected_transform: Option<&'a mut Transform>,
    pub selected_name: Option<&'a mut Name>,
    pub selected_material_handle: Option<Handle<StandardMaterial>>,
    pub selected_overrides: Option<&'a mut crate::rendering::materials::PbrTextureOverrides>,
    pub selected_environment: Option<&'a mut crate::rendering::scene::EnvironmentSettings>,
    pub selected_atmosphere: Option<&'a mut crate::rendering::atmosphere::AtmosphereSettingsComponent>,
    pub material_assets: &'a mut Assets<StandardMaterial>,
    pub asset_server: &'a AssetServer,
    pub selected_asset: Option<&'a str>,
    pub selected_waffle_light: Option<&'a mut crate::rendering::lighting::WaffleLight>,
    pub selected_directional_light: Option<&'a mut DirectionalLight>,
    pub selected_point_light: Option<&'a mut PointLight>,
    pub selected_spot_light: Option<&'a mut SpotLight>,
    pub diagnostics: &'a bevy::diagnostic::DiagnosticsStore,
    pub asset_cache: &'a AssetBrowserCache,
    pub reparent_queue: &'a mut Vec<HierarchyReparentEvent>,
    pub spawn_primitive_queue: &'a mut Vec<SpawnPrimitiveEvent>,
    pub spawn_asset_queue: &'a mut Vec<SpawnAssetEvent>,
    pub viewport_texture_id: Option<egui::TextureId>,
}

impl<'a> TabViewer for EditorTabViewer<'a> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorTab::Viewport => "Viewport".into(),
            EditorTab::Hierarchy => "Hierarchy".into(),
            EditorTab::Inspector => "Inspector".into(),
            EditorTab::Assets => "Assets".into(),
            EditorTab::Console => "Output".into(),
            EditorTab::Profiler => "Profiler".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Viewport => {
                draw_viewport_panel(
                    ui,
                    self.editor_state,
                    self.editor_settings,
                    self.viewport_texture_id,
                );
            }
            EditorTab::Hierarchy => {
                draw_hierarchy_panel(
                    ui,
                    self.editor_state,
                    self.editor_settings,
                    self.hierarchy,
                    self.reparent_queue,
                    self.spawn_primitive_queue,
                    self.spawn_asset_queue,
                );
            }
            EditorTab::Inspector => {
                draw_inspector_panel(
                    ui,
                    self.editor_state,
                    self.editor_settings,
                    self.selected_transform.as_deref_mut(),
                    self.selected_name.as_deref_mut(),
                    self.selected_material_handle.as_ref(),
                    self.selected_overrides.as_deref_mut(),
                    self.selected_environment.as_deref_mut(),
                    self.selected_atmosphere.as_deref_mut(),
                    self.material_assets,
                    self.asset_server,
                    self.selected_asset,
                    self.selected_waffle_light.as_deref_mut(),
                    self.selected_directional_light.as_deref_mut(),
                    self.selected_point_light.as_deref_mut(),
                    self.selected_spot_light.as_deref_mut(),
                );
            }
            EditorTab::Assets => {
                draw_assets_panel(
                    ui,
                    self.editor_state,
                    self.editor_settings,
                    self.asset_cache,
                    self.spawn_asset_queue,
                );
            }
            EditorTab::Console => {
                draw_console_panel(
                    ui,
                    self.editor_settings,
                    self.editor_output,
                );
            }
            EditorTab::Profiler => {
                draw_profiler_panel(
                    ui,
                    self.editor_state,
                    self.editor_settings,
                    self.diagnostics,
                );
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        // Only viewport is not closeable
        match _tab {
            EditorTab::Viewport => false,
            _ => true,
        }
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [true, true]
    }
}

