use tokio::sync::Mutex;
use std::sync::Arc;

pub struct Session {
    pub id: u32,
    disponible: Arc<Mutex<bool>>,
}

impl Session {
}
