use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub port: u16,
    pub secret: Option<String>,
    pub input_grab_hotkey: Option<String>,
    pub clients: Vec<ClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ClientConfig {
    pub name: String,
    pub ip: String,
    pub position: ScreenPosition,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_width() -> u32 {
    1920
}
fn default_height() -> u32 {
    1080
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum ScreenPosition {
    #[default]
    Right,
    Left,
    Above,
    Below,
}
