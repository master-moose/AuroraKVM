use crate::config::{ClientConfig, Config};
use eframe::egui;
use std::path::PathBuf;

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("AuroraKVM Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "AuroraKVM",
        options,
        Box::new(|_cc| Ok(Box::new(ConfigApp::default()))),
    )
}

struct ConfigApp {
    config: Config,
    config_path: PathBuf,
    new_client_name: String,
    new_client_ip: String,
    show_add_dialog: bool,
    scale_factor: f32,
}

impl Default for ConfigApp {
    fn default() -> Self {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aurora_kvm")
            .join("config.json");

        let config = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        };

        Self {
            config,
            config_path,
            new_client_name: String::new(),
            new_client_ip: String::new(),
            show_add_dialog: false,
            scale_factor: 0.1, // 10% scale for visualization
        }
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AuroraKVM Topology Configuration");
            ui.label("Drag clients to arrange them. The center screen is your local machine.");
            ui.add_space(10.0);

            // Toolbar
            ui.horizontal(|ui| {
                if ui.button("➕ Add Client").clicked() {
                    self.show_add_dialog = true;
                }
                if ui.button("💾 Save Configuration").clicked() {
                    self.save_config();
                }
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.scale_factor, 0.05..=0.5).text("Scale"));
            });
            ui.separator();

            // Canvas
            let canvas_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::drag());

            // Center of the canvas is (0,0) in virtual coordinates
            let center_offset = response.rect.center().to_vec2();

            // Draw Local Machine Screens
            for (i, screen) in self.config.local_screens.iter().enumerate() {
                let local_rect = egui::Rect::from_min_size(
                    egui::pos2(screen.x as f32, screen.y as f32),
                    egui::vec2(screen.width as f32, screen.height as f32),
                );
                let local_screen_rect =
                    Self::to_screen_rect(local_rect, center_offset, self.scale_factor);

                painter.rect_filled(local_screen_rect, 5.0, egui::Color32::from_rgb(30, 58, 32));
                painter.rect_stroke(
                    local_screen_rect,
                    5.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(76, 175, 80)),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    local_screen_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Local {}", i + 1),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );
            }

            // Draw Clients
            let mut remove_idx = None;
            for (idx, client) in self.config.clients.iter_mut().enumerate() {
                let client_rect = egui::Rect::from_min_size(
                    egui::pos2(client.x as f32, client.y as f32),
                    egui::vec2(client.width as f32, client.height as f32),
                );
                let screen_rect =
                    Self::to_screen_rect(client_rect, center_offset, self.scale_factor);

                // Interaction
                let client_id = response.id.with(idx);
                let client_response = ui.interact(screen_rect, client_id, egui::Sense::drag());

                if client_response.dragged() {
                    let delta = client_response.drag_delta() / self.scale_factor;
                    client.x += delta.x as i32;
                    client.y += delta.y as i32;
                }

                // Visuals
                let color = if client_response.hovered() || client_response.dragged() {
                    egui::Color32::from_rgb(70, 70, 70)
                } else {
                    egui::Color32::from_rgb(51, 51, 51)
                };

                painter.rect_filled(screen_rect, 5.0, color);
                painter.rect_stroke(
                    screen_rect,
                    5.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(85, 85, 85)),
                    egui::StrokeKind::Inside,
                );

                // Text
                painter.text(
                    screen_rect.center() - egui::vec2(0.0, 10.0),
                    egui::Align2::CENTER_CENTER,
                    &client.name,
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );
                painter.text(
                    screen_rect.center() + egui::vec2(0.0, 10.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{}x{}", client.width, client.height),
                    egui::FontId::monospace(10.0),
                    egui::Color32::GRAY,
                );

                // Context menu to remove
                client_response.context_menu(|ui| {
                    if ui.button("Remove Client").clicked() {
                        remove_idx = Some(idx);
                        ui.close();
                    }
                });
            }

            if let Some(idx) = remove_idx {
                self.config.clients.remove(idx);
            }

            // Add Client Dialog
            if self.show_add_dialog {
                egui::Window::new("Add Client")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_client_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("IP:");
                            ui.text_edit_singleline(&mut self.new_client_ip);
                        });
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                self.config.clients.push(ClientConfig {
                                    name: self.new_client_name.clone(),
                                    ip: self.new_client_ip.clone(),
                                    x: 2000, // Default to right
                                    y: 0,
                                    width: 1920,
                                    height: 1080,
                                });
                                self.new_client_name.clear();
                                self.new_client_ip.clear();
                                self.show_add_dialog = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_add_dialog = false;
                            }
                        });
                    });
            }
        });
    }
}

impl ConfigApp {
    fn to_screen_rect(
        rect: egui::Rect,
        center_offset: egui::Vec2,
        scale_factor: f32,
    ) -> egui::Rect {
        let min = rect.min.to_vec2() * scale_factor + center_offset;
        let max = rect.max.to_vec2() * scale_factor + center_offset;
        egui::Rect::from_min_max(min.to_pos2(), max.to_pos2())
    }

    fn save_config(&self) {
        if let Some(parent) = self.config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            if std::fs::write(&self.config_path, json).is_ok() {
                println!("Configuration saved to {:?}", self.config_path);
            }
        }
    }
}
