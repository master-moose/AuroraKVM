use crate::config::{Config, ScreenPosition};

pub struct Topology {
    config: Config,
    current_focus: Focus,
    screen_width: f64,
    screen_height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Local,
    Client(String), // Client Name
}

impl Topology {
    pub fn new(config: Config) -> Self {
        // TODO: Get actual screen size. For now default to 1920x1080
        Self {
            config,
            current_focus: Focus::Local,
            screen_width: 1920.0,
            screen_height: 1080.0,
        }
    }

    pub fn update_screen_size(&mut self, width: f64, height: f64) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn check_edge(&mut self, x: f64, y: f64) -> Option<Focus> {
        if self.current_focus != Focus::Local {
            return None; // Only switch from Local for now (simplified)
        }

        // Simple edge detection with 1px threshold
        if x >= self.screen_width - 1.0 {
            self.find_client(ScreenPosition::Right)
        } else if x <= 1.0 {
            self.find_client(ScreenPosition::Left)
        } else if y <= 1.0 {
            self.find_client(ScreenPosition::Above)
        } else if y >= self.screen_height - 1.0 {
            self.find_client(ScreenPosition::Below)
        } else {
            None
        }
    }

    fn find_client(&self, pos: ScreenPosition) -> Option<Focus> {
        self.config
            .clients
            .iter()
            .find(|c| c.position == pos)
            .map(|c| Focus::Client(c.name.clone()))
    }

    pub fn set_focus(&mut self, focus: Focus) {
        self.current_focus = focus;
    }

    pub fn get_focus(&self) -> &Focus {
        &self.current_focus
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }
}
