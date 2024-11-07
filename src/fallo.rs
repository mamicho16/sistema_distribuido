use crate::nodo::Nodo;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn simular_fallo(nodo: Arc<Nodo>) {
    nodo.actualizar_estado(false).await;
    println!("Fallo simulado en nodo {}", nodo.id);
}
