use crate::config::Config;
use crate::connected::ConnectedClients;
use std::path::PathBuf;

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

pub fn run_gui_slint(
    connected_clients: Option<ConnectedClients>,
) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    // Load config
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aurora_kvm")
        .join("config.json");

    let config: Config = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Config::default()
    };

    // Populate screens from config
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
    if let Some(ref connected) = connected_clients {
        if let Ok(clients) = connected.lock() {
            for (_, client) in clients.iter() {
                screens.push(ScreenData {
                    name: format!("{} (Connected)", client.screen_info.name),
                    x: 0.0,
                    y: 0.0,
                    width: client.screen_info.width as f32,
                    height: client.screen_info.height as f32,
                    connected: true,
                });
            }
        }
    }

    // Convert to Slint model
    let screen_model: Vec<Screen> = screens.into_iter().map(|s| s.into()).collect();
    let model = std::rc::Rc::new(slint::VecModel::from(screen_model));
    ui.set_screens(model.into());

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

    ui.run()
}
