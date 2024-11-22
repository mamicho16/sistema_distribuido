#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::Duration;
    use sistema_distribuido::{node::Node, session::Session, resource::Resources, process::Process};

    #[tokio::test]
    async fn test_balanceo_de_carga() {
        let node1 = Node::new(1);
        let node2 = Node::new(2);
        let mut session = Session::new(
            vec![node1.clone(), node2.clone()],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        );

        let process1 = Process::new(1, "Cargado".to_string(), Resources::new(8_192, 500_000, 4));
        session.nodes[0].active_processes.push(process1);

        let process2 = Process::new(2, "Nuevo".to_string(), Resources::new(4_096, 200_000, 2));
        session.processes.push(process2);

        session.assign_processes().await;

        assert!(session.nodes[1].active_processes.len() > 0);
    }

    #[tokio::test]
    async fn test_sincronizacion_recursos() {
        let node1 = Node::new(1);
        let node2 = Node::new(2);
        let session = Arc::new(Mutex::new(Session::new(
            vec![node1.clone(), node2.clone()],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        )));

        let session_clone1 = Arc::clone(&session);
        let handle1 = tokio::spawn(async move {
            let mut session = session_clone1.lock().await;
            session.request_resource(1).await;
            assert!(session.can_access_resource(1));
            session.release_resource(1);
        });

        let session_clone2 = Arc::clone(&session);
        let handle2 = tokio::spawn(async move {
            let mut session = session_clone2.lock().await;
            session.request_resource(2).await;
            assert!(session.can_access_resource(2));
            session.release_resource(2);
        });

        handle1.await.unwrap();
        handle2.await.unwrap();
    }

    #[test]
    fn test_manejo_de_fallos() {
        let mut session = Session::new(
            vec![Node::new(1), Node::new(2)],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        );

        let process1 = Process::new(1, "Fallido".to_string(), Resources::new(4_096, 200_000, 2));
        session.nodes[0].active_processes.push(process1);

        session.handle_node_failure(1, "Fallo simulado".to_string());

        assert!(session.nodes.len() == 2);
        assert!(session.processes.len() == 1);
    }

    #[test]
    fn test_escalabilidad() {
        let mut session = Session::new(
            vec![Node::new(1)],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        );
    
        // Agrega un nuevo nodo
        session.add_node(Node::new(2));
    
        assert!(session.total_nodes() == 2);
    }

    #[tokio::test]
    async fn test_redistribucion_automatica() {
        let mut session = Session::new(
            vec![Node::new(1), Node::new(2)],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        );
    
        // Sobrecarga nodo 1
        let process1 = Process::new(1, "Sobrecarga".to_string(), Resources::new(8_192, 500_000, 4));
        session.nodes[0].active_processes.push(process1);
    
        // Agrega procesos adicionales
        let process2 = Process::new(2, "Excedente".to_string(), Resources::new(8_192, 500_000, 4));
        session.processes.push(process2);
    
        session.assign_processes().await;
    
        // Verifica redistribución
        assert!(session.nodes[1].active_processes.len() > 0);
    }
    
}
