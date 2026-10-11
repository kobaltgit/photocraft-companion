use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::ButtonConfig;
use eframe::egui::{self, RichText, Sense, Stroke, Ui, Vec2};

pub struct ToolstripApp {
    pub is_vertical: bool,
    pub current_theme: ThemeMode,
    pub buttons: Vec<ButtonConfig>,
    pub pending_action: Option<String>,
    pub open_manager_requested: bool,
    pub hide_requested: bool,
    pub hovered_tooltip: Option<(String, egui::Rect)>,
}

impl ToolstripApp {
    pub fn new(buttons: Vec<ButtonConfig>, current_theme: ThemeMode) -> Self {
        Self {
            is_vertical: false, // Per spec: default is horizontal
            current_theme,
            buttons,
            pending_action: None,
            open_manager_requested: false,
            hide_requested: false,
            hovered_tooltip: None,
        }
    }

    /// Dynamic sizing:
    /// Fully adaptive calculation based on count N of active plugins.
    /// Supports arbitrary plugin counts by any 3rd party developer without clipping.
    pub fn calculate_size(&self) -> Vec2 {
        let count = self.buttons.len().max(1) as f32;
        if self.is_vertical {
            // Header + tools (count * 36px) + comfortable bottom margin = count * 36.0 + 68.0
            // Width 50px gives comfortable breathing room for 3-button service row (43px) and tool buttons (32px)
            Vec2::new(50.0, count * 36.0 + 68.0)
        } else {
            // Each button stride 38px, margins 16px, min 116px for service controls
            let tools_width = count * 38.0 + 16.0;
            // Height 78px comfortably holds service row (22px) + separator (6px) + buttons (34px) + padding
            Vec2::new(tools_width.max(116.0), 78.0)
        }
    }

    pub fn render(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        self.hovered_tooltip = None;

        // Draw frameless panel background matching PhotoCraft tool palettes with 2px corner radius
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 2.0, palette.panel_fill);
        ui.painter()
            .rect_stroke(rect, 2.0, Stroke::new(1.0, palette.border_stroke), egui::StrokeKind::Inside);

        let render_button_items = |ui: &mut Ui, buttons: &[ButtonConfig], pending: &mut Option<String>, hovered_tt: &mut Option<(String, egui::Rect)>| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
            for btn in buttons {
                let label_text = match btn.icon.as_deref() {
                    Some("wand") | Some("scissors") => "AI",
                    Some("bw") | Some("film") | Some("adjust") => "BW",
                    Some("copy") => "CP",
                    Some("paste") => "PS",
                    _ => {
                        if btn.label.len() <= 3 {
                            &btn.label
                        } else {
                            &btn.label[..3]
                        }
                    }
                };

                let b = egui::Button::new(
                    RichText::new(label_text)
                        .strong()
                        .size(11.0)
                        .color(palette.text_primary),
                )
                .min_size(Vec2::new(32.0, 32.0))
                .corner_radius(3.0);

                let tooltip = if let Some(ref hk) = btn.hotkey {
                    format!("{} ({})", btn.label, hk)
                } else {
                    btn.label.clone()
                };

                let resp = ui.add(b);
                if resp.hovered() {
                    *hovered_tt = Some((tooltip, resp.rect));
                }
                if resp.clicked() {
                    *pending = Some(btn.action.clone());
                }
            }
        };

        if self.is_vertical {
            // Vertical Layout: Top service bar + column of tools
            ui.vertical(|ui| {
                ui.add_space(3.0);

                // Row 1: Drag handle centered horizontally
                ui.horizontal(|ui| {
                    let avail = ui.available_width();
                    if avail > 16.0 {
                        ui.add_space((avail - 16.0) * 0.5);
                    }
                    let handle_size = Vec2::new(16.0, 7.0);
                    let (handle_rect, response) = ui.allocate_exact_size(handle_size, Sense::drag());
                    if response.drag_started() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    let c = handle_rect.center();
                    for dx in [-4.0, 0.0, 4.0] {
                        ui.painter().circle_filled(c + Vec2::new(dx, -1.8), 1.0, palette.text_muted);
                        ui.painter().circle_filled(c + Vec2::new(dx, 1.8), 1.0, palette.text_muted);
                    }
                });

                ui.add_space(2.0);

                // Row 2: Service controls (Orient, Settings, Minimize)
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                    let total_w = 13.0 * 3.0 + 4.0;
                    let avail = ui.available_width();
                    if avail > total_w {
                        ui.add_space((avail - total_w) * 0.5);
                    }

                    // Orientation toggle (Switch to Horizontal)
                    let orient_btn = egui::Button::new("")
                        .min_size(Vec2::new(13.0, 13.0))
                        .corner_radius(2.0);
                    let resp = ui.add(orient_btn);
                    if resp.hovered() {
                        self.hovered_tooltip = Some(("Switch to Horizontal".to_string(), resp.rect));
                    }
                    let c = resp.rect.center();
                    ui.painter().line_segment([c - Vec2::new(3.0, 0.0), c + Vec2::new(3.0, 0.0)], Stroke::new(1.2, palette.text_muted));
                    ui.painter().line_segment([c - Vec2::new(3.0, 0.0), c - Vec2::new(1.5, 2.0)], Stroke::new(1.0, palette.text_muted));
                    ui.painter().line_segment([c - Vec2::new(3.0, 0.0), c - Vec2::new(1.5, -2.0)], Stroke::new(1.0, palette.text_muted));
                    ui.painter().line_segment([c + Vec2::new(3.0, 0.0), c + Vec2::new(1.5, 2.0)], Stroke::new(1.0, palette.text_muted));
                    ui.painter().line_segment([c + Vec2::new(3.0, 0.0), c + Vec2::new(1.5, -2.0)], Stroke::new(1.0, palette.text_muted));
                    if resp.clicked() {
                        self.is_vertical = false;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(self.calculate_size()));
                    }

                    // Settings gear (vector drawn)
                    let gear_btn = egui::Button::new("")
                        .min_size(Vec2::new(13.0, 13.0))
                        .corner_radius(2.0);
                    let resp_gear = ui.add(gear_btn);
                    if resp_gear.hovered() {
                        self.hovered_tooltip = Some(("Plugins Manager".to_string(), resp_gear.rect));
                    }
                    let cg = resp_gear.rect.center();
                    ui.painter().circle_stroke(cg, 2.3, Stroke::new(1.0, palette.text_muted));
                    ui.painter().line_segment([cg - Vec2::new(0.0, 3.5), cg + Vec2::new(0.0, 3.5)], Stroke::new(1.0, palette.text_muted));
                    ui.painter().line_segment([cg - Vec2::new(3.5, 0.0), cg + Vec2::new(3.5, 0.0)], Stroke::new(1.0, palette.text_muted));
                    if resp_gear.clicked() {
                        self.open_manager_requested = true;
                    }

                    // Minimize to tray
                    let min_btn = egui::Button::new("")
                        .min_size(Vec2::new(13.0, 13.0))
                        .corner_radius(2.0);
                    let resp_min = ui.add(min_btn);
                    if resp_min.hovered() {
                        self.hovered_tooltip = Some(("Minimize to Tray".to_string(), resp_min.rect));
                    }
                    let cm = resp_min.rect.center();
                    ui.painter().line_segment(
                        [cm - Vec2::new(2.8, 0.0), cm + Vec2::new(2.8, 0.0)],
                        Stroke::new(1.2, palette.text_muted),
                    );
                    if resp_min.clicked() {
                        self.hide_requested = true;
                    }
                });

                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                ui.vertical_centered(|ui| {
                    render_button_items(ui, &self.buttons, &mut self.pending_action, &mut self.hovered_tooltip);
                });
                ui.add_space(3.0);
            });
        } else {
            // Horizontal Layout (Two-Tier): Tier 1 service row ON TOP, Tier 2 plugin buttons BELOW
            ui.vertical(|ui| {
                ui.add_space(3.0);

                // Tier 1: Service controls header
                ui.horizontal(|ui| {
                    ui.add_space(3.0);

                    // Drag handle
                    let handle_size = Vec2::new(20.0, 14.0);
                    let (handle_rect, response) = ui.allocate_exact_size(handle_size, Sense::drag());
                    if response.drag_started() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    let c = handle_rect.center();
                    for dx in [-6.0, -2.0, 2.0, 6.0] {
                        ui.painter().circle_filled(c + Vec2::new(dx, -2.5), 1.0, palette.text_muted);
                        ui.painter().circle_filled(c + Vec2::new(dx, 2.5), 1.0, palette.text_muted);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(2.0);

                        // Minimize to tray (clean vector line)
                        let min_btn = egui::Button::new("")
                            .min_size(Vec2::new(14.0, 14.0))
                            .corner_radius(2.0);
                        let resp_min = ui.add(min_btn);
                        if resp_min.hovered() {
                            self.hovered_tooltip = Some(("Minimize to Tray".to_string(), resp_min.rect));
                        }
                        let cm = resp_min.rect.center();
                        ui.painter().line_segment(
                            [cm - Vec2::new(3.0, 0.0), cm + Vec2::new(3.0, 0.0)],
                            Stroke::new(1.2, palette.text_muted),
                        );
                        if resp_min.clicked() {
                            self.hide_requested = true;
                        }

                        // Settings gear (clean vector drawn)
                        let gear_btn = egui::Button::new("")
                            .min_size(Vec2::new(14.0, 14.0))
                            .corner_radius(2.0);
                        let resp_gear = ui.add(gear_btn);
                        if resp_gear.hovered() {
                            self.hovered_tooltip = Some(("Plugins Manager".to_string(), resp_gear.rect));
                        }
                        let cg = resp_gear.rect.center();
                        ui.painter().circle_stroke(cg, 2.6, Stroke::new(1.0, palette.text_muted));
                        ui.painter().line_segment([cg - Vec2::new(0.0, 4.0), cg + Vec2::new(0.0, 4.0)], Stroke::new(1.0, palette.text_muted));
                        ui.painter().line_segment([cg - Vec2::new(4.0, 0.0), cg + Vec2::new(4.0, 0.0)], Stroke::new(1.0, palette.text_muted));
                        if resp_gear.clicked() {
                            self.open_manager_requested = true;
                        }

                        // Toggle orientation (Switch to Vertical)
                        let orient_btn = egui::Button::new("")
                            .min_size(Vec2::new(14.0, 14.0))
                            .corner_radius(2.0);
                        let resp_orient = ui.add(orient_btn);
                        if resp_orient.hovered() {
                            self.hovered_tooltip = Some(("Switch to Vertical".to_string(), resp_orient.rect));
                        }
                        let c = resp_orient.rect.center();
                        ui.painter().line_segment([c - Vec2::new(0.0, 3.5), c + Vec2::new(0.0, 3.5)], Stroke::new(1.2, palette.text_muted));
                        ui.painter().line_segment([c - Vec2::new(0.0, 3.5), c - Vec2::new(2.0, 1.5)], Stroke::new(1.0, palette.text_muted));
                        ui.painter().line_segment([c - Vec2::new(0.0, 3.5), c - Vec2::new(-2.0, 1.5)], Stroke::new(1.0, palette.text_muted));
                        ui.painter().line_segment([c + Vec2::new(0.0, 3.5), c + Vec2::new(2.0, -1.5)], Stroke::new(1.0, palette.text_muted));
                        ui.painter().line_segment([c + Vec2::new(0.0, 3.5), c + Vec2::new(-2.0, -1.5)], Stroke::new(1.0, palette.text_muted));
                        if resp_orient.clicked() {
                            self.is_vertical = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(self.calculate_size()));
                        }
                    });
                });

                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                // Tier 2: Large plugin action buttons strictly below service row
                ui.horizontal_centered(|ui| {
                    ui.add_space(4.0);
                    render_button_items(ui, &self.buttons, &mut self.pending_action, &mut self.hovered_tooltip);
                });
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buttons(count: usize) -> Vec<ButtonConfig> {
        (0..count)
            .map(|i| ButtonConfig {
                label: format!("Tool {}", i + 1),
                action: format!("action_{}", i + 1),
                hotkey: None,
                icon: if i == 0 { Some("wand".to_string()) } else { Some("bw".to_string()) },
            })
            .collect()
    }

    #[test]
    fn test_vertical_toolstrip_geometry_no_clipping() {
        for count in [1, 2, 4] {
            let ctx = egui::Context::default();
            let buttons = create_test_buttons(count);
            let mut app = ToolstripApp::new(buttons, ThemeMode::Dark);
            app.is_vertical = true;

            let expected_size = app.calculate_size();
            assert_eq!(expected_size.x, 50.0);
            assert!(expected_size.y >= (count as f32) * 36.0 + 58.0);

            let mut raw_input = egui::RawInput::default();
            raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size));

            let mut full_output = ctx.run_ui(raw_input, |ctx| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size);
                let mut ui = egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("test_vertical"),
                    egui::UiBuilder::new().max_rect(rect),
                );
                app.render(&mut ui, &ThemePalette::for_mode(ThemeMode::Dark));

                let used_rect = ui.min_rect();
                // Ensure content never overflows the window bounds in vertical mode
                assert!(
                    used_rect.max.y <= expected_size.y,
                    "Vertical mode clipped at bottom for count {}: used y {} > expected y {}",
                    count,
                    used_rect.max.y,
                    expected_size.y
                );
                assert!(
                    used_rect.max.x <= expected_size.x,
                    "Vertical mode clipped at right for count {}: used x {} > expected x {}",
                    count,
                    used_rect.max.x,
                    expected_size.x
                );
            });
            full_output.textures_delta.clear();
        }
    }

    #[test]
    fn test_horizontal_toolstrip_geometry_no_clipping() {
        for count in [1, 2, 4] {
            let ctx = egui::Context::default();
            let buttons = create_test_buttons(count);
            let mut app = ToolstripApp::new(buttons, ThemeMode::Dark);
            app.is_vertical = false;

            let expected_size = app.calculate_size();
            assert_eq!(expected_size.y, 78.0);

            let mut raw_input = egui::RawInput::default();
            raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size));

            let mut full_output = ctx.run_ui(raw_input, |ctx| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size);
                let mut ui = egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("test_horizontal"),
                    egui::UiBuilder::new().max_rect(rect),
                );
                app.render(&mut ui, &ThemePalette::for_mode(ThemeMode::Dark));

                let used_rect = ui.min_rect();
                assert!(
                    used_rect.max.y <= expected_size.y,
                    "Horizontal mode clipped at bottom for count {}: used y {} > expected y {}",
                    count,
                    used_rect.max.y,
                    expected_size.y
                );
                assert!(
                    used_rect.max.x <= expected_size.x,
                    "Horizontal mode clipped at right for count {}: used x {} > expected x {}",
                    count,
                    used_rect.max.x,
                    expected_size.x
                );
            });
            full_output.textures_delta.clear();
        }
    }

    #[test]
    fn test_toolstrip_size_scaling() {
        // 0 buttons defaults to 1 button minimum
        let app_empty_h = ToolstripApp::new(vec![], ThemeMode::Dark);
        assert_eq!(app_empty_h.calculate_size().y, 78.0);
        assert!(app_empty_h.calculate_size().x >= 116.0);

        let mut app_empty_v = ToolstripApp::new(vec![], ThemeMode::Dark);
        app_empty_v.is_vertical = true;
        assert_eq!(app_empty_v.calculate_size().x, 50.0);
        assert_eq!(app_empty_v.calculate_size().y, 1.0 * 36.0 + 68.0);

        // 5 buttons
        let buttons5 = create_test_buttons(5);
        let mut app5 = ToolstripApp::new(buttons5, ThemeMode::Light);
        assert_eq!(app5.calculate_size().y, 78.0);
        assert_eq!(app5.calculate_size().x, (5.0f32 * 38.0 + 16.0).max(116.0));

        app5.is_vertical = true;
        assert_eq!(app5.calculate_size().x, 50.0);
        assert_eq!(app5.calculate_size().y, 5.0 * 36.0 + 68.0);
    }

    #[test]
    fn test_toolstrip_state_defaults() {
        let app = ToolstripApp::new(vec![], ThemeMode::Dark);
        assert!(!app.is_vertical);
        assert!(app.pending_action.is_none());
        assert!(!app.open_manager_requested);
        assert!(!app.hide_requested);
        assert!(app.hovered_tooltip.is_none());
    }
}


