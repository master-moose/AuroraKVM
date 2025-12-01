use crate::config::Config;

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
        // Calculate total bounding box of local screens for initial "screen size"
        // This is a simplification; we'll rely on the config for actual geometry.
        let mut max_x = 0.0;
        let mut max_y = 0.0;
        for screen in &config.local_screens {
            let right = (screen.x + screen.width as i32) as f64;
            let bottom = (screen.y + screen.height as i32) as f64;
            if right > max_x {
                max_x = right;
            }
            if bottom > max_y {
                max_y = bottom;
            }
        }

        Self {
            config,
            current_focus: Focus::Local,
            screen_width: max_x,
            screen_height: max_y,
        }
    }

    pub fn update_screen_size(&mut self, width: f64, height: f64) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn check_edge(&mut self, x: f64, y: f64) -> Option<Focus> {
        if self.current_focus != Focus::Local {
            return None;
        }

        // Check if point is inside ANY local screen
        let mut inside_local = false;
        for screen in &self.config.local_screens {
            let sx = screen.x as f64;
            let sy = screen.y as f64;
            let sw = screen.width as f64;
            let sh = screen.height as f64;

            if x >= sx && x < sx + sw && y >= sy && y < sy + sh {
                inside_local = true;
                break;
            }
        }

        if !inside_local {
            // If we are not inside any local screen, we might be crossing into a client
            return self.find_client_at(x as i32, y as i32);
        }
        None
    }

    fn find_client_at(&self, x: i32, y: i32) -> Option<Focus> {
        // Simple bounding box check against all clients
        // We treat the local screen as (0,0) to (width, height)
        // Clients are positioned relative to this.

        for client in &self.config.clients {
            let cx = client.x;
            let cy = client.y;
            let cw = client.width as i32;
            let ch = client.height as i32;

            if x >= cx && x < cx + cw && y >= cy && y < cy + ch {
                return Some(Focus::Client(client.name.clone()));
            }
        }
        None
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
