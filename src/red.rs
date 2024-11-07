use crate::nodo::Nodo;
use crate::proceso::Proceso;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Red {
    pub nodos: HashMap<u32, Arc<Nodo>>,
}

impl Red {
    pub fn nueva() -> Self {
        Red {
            nodos: HashMap::new(),
        }
    }

    pub async fn agregar_nodo(&mut self, id: u32, capacidad: usize) {
        let nodo = Nodo::nuevo(id, capacidad);
        self.nodos.insert(id, Arc::new(nodo));
    }

    pub async fn asignar_proceso(&self, proceso: Proceso) {
        let mut nodo_menos_cargado = None;
        let mut menor_carga = usize::MAX;

        for nodo in self.nodos.values() {
            let estado = *nodo.activo.lock().await;
            let carga_actual = nodo.procesos.lock().await.len();
            if estado && carga_actual < menor_carga {
                menor_carga = carga_actual;
                nodo_menos_cargado = Some(nodo.clone());
            }
        }

        if let Some(nodo) = nodo_menos_cargado {
            nodo.agregar_proceso(proceso).await;
        }
    }

    pub async fn manejar_fallo(&mut self, nodo_id: u32) {
        if let Some(nodo) = self.nodos.get(&nodo_id) {
            nodo.actualizar_estado(false).await;
            let procesos = nodo.procesos.lock().await.clone();
            for proceso in procesos {
                self.asignar_proceso(proceso).await;
            }
        }
    }
    pub async fn estado_nodos(&self) {
        for (id, nodo) in &self.nodos {
            let activo = *nodo.activo.lock().await;
            let cantidad_procesos = nodo.procesos.lock().await.len();
            println!("Nodo {} - Activo: {}, Procesos Actuales: {}/{}", 
                id, 
                activo,
                cantidad_procesos,
                nodo.capacidad
            );
        }
    }
}
