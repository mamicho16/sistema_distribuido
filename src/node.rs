use crate::proceso::Proceso;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Node {
    pub id: u32,
    pub capacidad: usize,
    pub procesos: Arc<Mutex<Vec<Proceso>>>,
    pub activo: Arc<Mutex<bool>>,
}

impl Node {
    pub fn nuevo(id: u32, capacidad: usize) -> Self {
        Node {
            id,
            capacidad,
            procesos: Arc::new(Mutex::new(Vec::new())),
            activo: Arc::new(Mutex::new(true)),
        }
    }

    pub async fn agregar_proceso(&self, proceso: Proceso) -> bool {
        let mut procesos = self.procesos.lock().await;
        if procesos.len() < self.capacidad {
            procesos.push(proceso);
            true
        } else {
            false
        }
    }

    pub async fn actualizar_estado(&self, activo: bool) {
        let mut estado = self.activo.lock().await;
        *estado = activo;
    }
}
