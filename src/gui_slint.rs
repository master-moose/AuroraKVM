use crate::config::Config;
use crate::connected::ConnectedClients;
use slint::Model;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

slint::include_modules!();

#[derive(Clone)]
struct ScreenData {
    name: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    connected: bool,
}

impl From<ScreenData> for Screen {
    fn from(data: ScreenData) -> Self {
        Screen {
            name: data.name.into(),
            x: data.x,
            y: data.y,
            width: data.width,
            height: data.height,
            connected: data.connected,
        }
    }
}

/// Build the screen model from config and connected clients
fn build_screen_model(
    config: &Config,
    connected_clients: &Option<ConnectedClients>,
    client_positions: &Rc<RefCell<HashMap<String, (f32, f32)>>>,
) -> Vec<ScreenData> {
    let mut screens: Vec<ScreenData> = Vec::new();

    // Add local screens (from config)
    for (i, screen) in config.local_screens.iter().enumerate() {
        screens.push(ScreenData {
            name: format!("Local {}", i + 1),
            x: screen.x as f32,
            y: screen.y as f32,
            width: screen.width as f32,
            height: screen.height as f32,
            connected: false,
        });
    }

    // Add configured clients (gray)
    for client in &config.clients {
        screens.push(ScreenData {
            name: client.name.clone(),
            x: client.x as f32,
            y: client.y as f32,
            width: client.width as f32,
            height: client.height as f32,
            connected: false,
        });
    }

    // Add live connected clients (green)
    if let Some(connected) = connected_clients {
        if let Ok(clients) = connected.lock() {
            for (i, (_, client)) in clients.iter().enumerate() {
                let name = format!("{} (Connected)", client.screen_info.name);

                // Get position from persistence map or calculate default
                let (x, y) = {
                    let mut positions = client_positions.borrow_mut();
                    if let Some(&pos) = positions.get(&name) {
                        pos
                    } else {
                        let offset_x = (i as f32) * 1100.0;
                        let default_x = 1000.0 + offset_x;
                        let default_y = 750.0;
                        positions.insert(name.clone(), (default_x, default_y));
                        (default_x, default_y)
                    }
                };

                screens.push(ScreenData {
                    name: name.clone(),
                    x,
                    y,
                    width: client.screen_info.width as f32,
                    height: client.screen_info.height as f32,
                    connected: true,
                });

                eprintln!(
                    "DEBUG: Added connected client '{}' at ({}, {})",
                    client.screen_info.name, x, y
                );
            }
        }
    }

    eprintln!("DEBUG: Total screens in model: {}", screens.len());

    screens
}

pub fn run_gui_slint(
    connected_clients: Option<ConnectedClients>,
) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    // Load config
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aurora_kvm")
        .join("config.json");

    let mut config: Config = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Config::default()
    };

    // Auto-detect monitors if config has no local screens
    if config.local_screens.is_empty() {
        config.local_screens = crate::monitor::detect_monitors();
        eprintln!(
            "DEBUG: Auto-detected {} local screens",
            config.local_screens.len()
        );
        for (i, screen) in config.local_screens.iter().enumerate() {
            eprintln!(
                "DEBUG:   Local Screen {}: {}x{} at ({}, {})",
                i + 1,
                screen.width,
                screen.height,
                screen.x,
                screen.y
            );
        }
    } else {
        eprintln!(
            "DEBUG: Using {} local screens from config",
            config.local_screens.len()
        );
    }

    // State for persisting client positions during session
    let client_positions = Rc::new(RefCell::new(HashMap::new()));

    // Build initial screen model
    let screens = build_screen_model(&config, &connected_clients, &client_positions);
    let screen_model: Vec<Screen> = screens.into_iter().map(|s| s.into()).collect();
    let model = Rc::new(slint::VecModel::from(screen_model));
    ui.set_screens(model.clone().into());

    // Setup live update timer (500ms interval)
    let ui_weak_timer = ui.as_weak();
    let connected_clients_timer = connected_clients.clone();
    let config_timer = config.clone();
    let model_timer = model.clone();
    let client_positions_timer = client_positions.clone();

    let timer = slint::Timer::default();
    let last_signature = Rc::new(RefCell::new(String::new()));

    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(500),
        move || {
            if let Some(ui) = ui_weak_timer.upgrade() {
                // Calculate current signature to check for changes
                let current_signature = if let Some(connected) = &connected_clients_timer {
                    if let Ok(clients) = connected.lock() {
                        let mut sig = format!("Count:{}", clients.len());
                        for (_, client) in clients.iter() {
                            sig.push_str(&format!("|{}", client.screen_info.name));
                        }
                        sig
                    } else {
                        "Locked".to_string()
                    }
                } else {
                    "None".to_string()
                };

                // Only rebuild if signature changed
                if *last_signature.borrow() != current_signature {
                    eprintln!(
                        "DEBUG: Clients changed, updating model. Sig: {}",
                        current_signature
                    );
                    *last_signature.borrow_mut() = current_signature;

                    // Rebuild screen model
                    let screens = build_screen_model(
                        &config_timer,
                        &connected_clients_timer,
                        &client_positions_timer,
                    );
                    let screen_vec: Vec<Screen> = screens.into_iter().map(|s| s.into()).collect();

                    // Update the model
                    model_timer.set_vec(screen_vec);
                    ui.set_screens(model_timer.clone().into());
                }
            }
        },
    );

    // Setup callbacks
    let ui_weak = ui.as_weak();
    ui.on_save_config(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Config saved!".into());
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_add_client(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text("Add client dialog...".into());
        }
    });

    // Handle screen move (drag) events
    let ui_weak_moved = ui.as_weak();
    let model_moved = model.clone();
    let client_positions_moved = client_positions.clone();
    let connected_clients_moved = connected_clients.clone();

    ui.on_screen_moved(move |index, new_x, new_y| {
        if let Some(_ui) = ui_weak_moved.upgrade() {
            // Update model
            if let Some(mut screen) = model_moved.row_data(index as usize) {
                screen.x = new_x;
                screen.y = new_y;
                model_moved.set_row_data(index as usize, screen.clone());

                let name = screen.name.to_string();

                // Update persistence map so timer doesn't reset position
                client_positions_moved
                    .borrow_mut()
                    .insert(name.clone(), (new_x, new_y));

                // Update shared state for server routing
                if let Some(connected) = &connected_clients_moved {
                    if let Ok(mut clients) = connected.lock() {
                        // The name in the model is "Name (Connected)", but in clients it's just "Name"
                        // We need to match correctly.
                        let clean_name = name.replace(" (Connected)", "");

                        for (_, client) in clients.iter_mut() {
                            if client.screen_info.name == clean_name {
                                client.screen_info.x = new_x as i32;
                                client.screen_info.y = new_y as i32;
                                eprintln!(
                                    "DEBUG: Updated server routing for '{}' to ({}, {})",
                                    clean_name, new_x, new_y
                                );
                            }
                        }
                    }
                }
            }
        }
    });

    // Keep timer alive by moving it into a Box and leaking it
    // This ensures it lives for the duration of the UI
    Box::leak(Box::new(timer));

    ui.run()
}
