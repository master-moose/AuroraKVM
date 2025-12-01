use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub port: u16,
    pub secret: Option<String>,
    pub input_grab_hotkey: Option<String>,
    #[serde(default = "default_local_screens")]
    pub local_screens: Vec<LocalScreen>,
    pub clients: Vec<ClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalScreen {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn default_local_screens() -> Vec<LocalScreen> {
    vec![LocalScreen {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    }]
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ClientConfig {
    pub name: String,
    pub ip: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
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
