use crate::config::{ClientConfig, Config, ScreenPosition};
use eframe::egui;
use std::path::PathBuf;

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
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
    adding_position: Option<ScreenPosition>,
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
            adding_position: None,
        }
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AuroraKVM Topology Configuration");
            ui.add_space(20.0);

            // Visual topology editor
            ui.horizontal(|ui| {
                // Left slot
                self.render_slot(ui, ScreenPosition::Left);

                ui.add_space(20.0);

                // Main PC (center)
                ui.vertical(|ui| {
                    ui.add_space(40.0);
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgb(30, 58, 32))
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(76, 175, 80)))
                        .corner_radius(8.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("Main PC").size(20.0).strong());
                                ui.label("Localhost");
                            });
                        });
                });

                ui.add_space(20.0);

                // Right slot
                self.render_slot(ui, ScreenPosition::Right);
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // Save button
            if ui
                .button(egui::RichText::new("💾 Save Configuration").size(16.0))
                .clicked()
            {
                self.save_config();
            }

            // Client add dialog
            if let Some(pos) = self.adding_position.clone() {
                egui::Window::new("Add Client")
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label(format!("Adding client to: {:?}", pos));
                        ui.add_space(10.0);

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
                                    position: pos.clone(),
                                    width: 1920,
                                    height: 1080,
                                });
                                self.new_client_name.clear();
                                self.new_client_ip.clear();
                                self.adding_position = None;
                            }
                            if ui.button("Cancel").clicked() {
                                self.adding_position = None;
                            }
                        });
                    });
            }
        });
    }
}

impl ConfigApp {
    fn render_slot(&mut self, ui: &mut egui::Ui, position: ScreenPosition) {
        ui.vertical(|ui| {
            ui.add_space(40.0);

            let client_idx = self
                .config
                .clients
                .iter()
                .position(|c| c.position == position);

            if let Some(idx) = client_idx {
                let client = &mut self.config.clients[idx];
                // Draggable client
                let id = ui.make_persistent_id(format!("client_{}", idx));
                let item = egui::Frame::default()
                    .fill(egui::Color32::from_rgb(51, 51, 51))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(85, 85, 85)))
                    .corner_radius(8.0)
                    .inner_margin(20.0);

                let response = item
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(150.0, 100.0));
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(&client.name).size(16.0).strong());
                            ui.label(&client.ip);
                            ui.label(format!("{}x{}", client.width, client.height));
                            ui.label(
                                egui::RichText::new(format!("{:?}", position))
                                    .size(12.0)
                                    .color(egui::Color32::GRAY),
                            );
                        });
                    })
                    .response;

                // Drag logic
                let response = response.interact(egui::Sense::drag());
                if response.drag_started() {
                    // Store the index of the client being dragged
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_temp(egui::Id::new("dragged_client_idx"), idx)
                    });
                }

                if response.dragged() {
                    // Show drag preview (optional, egui does some automatically)
                }

                if ui.button("🗑 Remove").clicked() {
                    self.config.clients.remove(idx);
                }
            } else {
                // Empty slot (Drop target)
                let response = egui::Frame::default()
                    .fill(egui::Color32::from_rgb(34, 34, 34))
                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(68, 68, 68)))
                    .corner_radius(8.0)
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(150.0, 100.0));
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{:?}", position))
                                    .color(egui::Color32::GRAY),
                            );
                            if ui.button("➕ Add Client").clicked() {
                                self.adding_position = Some(position.clone());
                            }
                        });
                    })
                    .response;

                // Drop logic
                let is_being_dragged = ui
                    .memory(|mem| {
                        mem.data
                            .get_temp::<usize>(egui::Id::new("dragged_client_idx"))
                    })
                    .is_some();
                if is_being_dragged && response.hovered() {
                    ui.painter().add(egui::Shape::rect_stroke(
                        response.rect,
                        8.0,
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        egui::StrokeKind::Inside,
                    ));

                    if ui.input(|i| i.pointer.any_released()) {
                        if let Some(dragged_idx) = ui.memory(|mem| {
                            mem.data
                                .get_temp::<usize>(egui::Id::new("dragged_client_idx"))
                        }) {
                            // Move client to this position
                            if let Some(client) = self.config.clients.get_mut(dragged_idx) {
                                client.position = position;
                            }
                        }
                    }
                }
            }
        });
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
