/// Editor Theme Module
/// Dark theme configuration for the editor UI

use bevy_egui::egui;

/// Editor theme configuration
#[derive(Clone)]
pub struct EditorTheme {
    pub background_color: egui::Color32,
    pub panel_color: egui::Color32,
    pub accent_color: egui::Color32,
    pub text_color: egui::Color32,
    pub border_color: egui::Color32,
    pub hover_color: egui::Color32,
    pub active_color: egui::Color32,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            background_color: egui::Color32::from_rgb(30, 30, 30),
            panel_color: egui::Color32::from_rgb(45, 45, 45),
            accent_color: egui::Color32::from_rgb(70, 130, 180),
            text_color: egui::Color32::from_rgb(220, 220, 220),
            border_color: egui::Color32::from_rgb(60, 60, 60),
            hover_color: egui::Color32::from_rgb(80, 80, 80),
            active_color: egui::Color32::from_rgb(100, 100, 100),
        }
    }
}

impl EditorTheme {
    /// Apply the theme to the EGUI context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        // Window and panel colors
        visuals.window_fill = self.background_color;
        visuals.panel_fill = self.panel_color;
        visuals.faint_bg_color = self.background_color.linear_multiply(0.8);

        // Widget colors
        visuals.widgets.noninteractive.bg_fill = self.panel_color;
        visuals.widgets.noninteractive.fg_stroke.color = self.text_color;
        visuals.widgets.noninteractive.bg_stroke.color = self.border_color;

        visuals.widgets.inactive.bg_fill = self.panel_color;
        visuals.widgets.inactive.fg_stroke.color = self.text_color;
        visuals.widgets.inactive.bg_stroke.color = self.border_color;

        visuals.widgets.hovered.bg_fill = self.hover_color;
        visuals.widgets.hovered.fg_stroke.color = self.text_color;
        visuals.widgets.hovered.bg_stroke.color = self.accent_color;

        visuals.widgets.active.bg_fill = self.active_color;
        visuals.widgets.active.fg_stroke.color = self.text_color;
        visuals.widgets.active.bg_stroke.color = self.accent_color;

        // Selection colors
        visuals.selection.bg_fill = self.accent_color.linear_multiply(0.3);
        visuals.selection.stroke.color = self.accent_color;

        // Hyperlink colors
        visuals.hyperlink_color = self.accent_color;

        // Override text colors
        visuals.override_text_color = Some(self.text_color);

        ctx.set_visuals(visuals);
    }

    /// Get a lighter version of the accent color for highlights
    pub fn accent_light(&self) -> egui::Color32 {
        self.accent_color.linear_multiply(1.2)
    }

    /// Get a darker version of the accent color for pressed states
    pub fn accent_dark(&self) -> egui::Color32 {
        self.accent_color.linear_multiply(0.8)
    }

    /// Get success color (green)
    pub fn success_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(46, 125, 50)
    }

    /// Get warning color (yellow)
    pub fn warning_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(237, 108, 2)
    }

    /// Get error color (red)
    pub fn error_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(211, 47, 47)
    }

    /// Get info color (blue)
    pub fn info_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(25, 118, 210)
    }
}

