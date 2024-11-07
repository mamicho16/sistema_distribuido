mod nodo;
mod proceso;
mod recurso;
mod red;

use crate::red::Red;
use crate::proceso::Proceso;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Crear la red
    let mut red = Red::nueva();

    // Agregar nodos a la red
    red.agregar_nodo(1, 3).await;
    red.agregar_nodo(2, 5).await;

    // Crear procesos
    let proceso1 = Proceso::nuevo(1);
    let proceso2 = Proceso::nuevo(2);

    // Asignar procesos a los nodos
    red.asignar_proceso(proceso1).await;
    red.asignar_proceso(proceso2).await;

    // Mostrar estado de los nodos
    println!("Estado inicial de los nodos:");
    red.estado_nodos().await;

    // Simular fallo
    red.manejar_fallo(1).await;

    // Mostrar estado de los nodos después de fallo
    println!("\nEstado de los nodos después del fallo:");
    red.estado_nodos().await;

    Ok(())
}
