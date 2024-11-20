use tokio::sync::Mutex;
use std::sync::Arc;

pub struct Resource {
    pub id: u32,
    disponible: Arc<Mutex<bool>>,
}

impl Resource {
    pub fn nuevo(id: u32) -> Self {
        Resource {
            id,
            disponible: Arc::new(Mutex::new(true)),
        }
    }

    pub async fn solicitar(&self) -> bool {
        let mut disponible = self.disponible.lock().await;
        if *disponible {
            *disponible = false;
            true
        } else {
            false
        }
    }

    pub async fn liberar(&self) {
        let mut disponible = self.disponible.lock().await;
        *disponible = true;
    }
}
