use crate::net::ScreenInfo;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ConnectedClient {
    pub addr: SocketAddr,
    pub screen_info: ScreenInfo,
}

pub type ConnectedClients = Arc<Mutex<HashMap<SocketAddr, ConnectedClient>>>;

pub fn create_connected_clients() -> ConnectedClients {
    Arc::new(Mutex::new(HashMap::new()))
}
