use sistema_distribuido::node::Node;
use sistema_distribuido::resource::Resources;
use sistema_distribuido::session::Session;
use sistema_distribuido::process::Process;
use tokio::sync::{Mutex};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::collections::{HashMap, VecDeque};


fn simulate_failure() {
    // Create nodes
    let node1 = Node::new(1);
    let node2 = Node::new(2);
    let node3 = Node::new(3);

    // Initialize session
    let mut session = Session::new(vec![node1, node2, node3], vec![], Resources::new(16 * 1024, 1_000_000, 8));

    // Simulate a node detecting a failure and proposing an action
    let action = session.nodes[0].detect_and_report_failure("Disk failure".to_string());

    session.initiate_voting(session.nodes[0].id, action);

    // The nodes have received the proposal and voted; the action should be executed if consensus is reached
}

#[tokio::main]
async fn main() {
    // Define total shared resources
    let total_resources = Resources {
        ram: 16 * 1024,       // 16 GB in MB
        disk_space: 1_000_000, // 1 TB in MB
        threads: 8,
    };

    // Create nodes
    let node1 = Node::new(1);
    let node2 = Node::new(2);
    let node3 = Node::new(3);

    // Initialize session with shared resources
    let session = Session {
        nodes: vec![node1, node2, node3],
        processes: vec![],
        total_resources: total_resources.clone(),
        available_resources: total_resources,
        pending_votes: HashMap::new(),
        request_queue: VecDeque::new(),
        deferred_replies: HashMap::new(),
        replies_received: HashMap::new(),
    };

    // Wrap session in Arc and Mutex for thread-safe shared ownership
    let session = Arc::new(Mutex::new(session));

    // Create processes
    let process1 = Process {
        id: 1,
        task: "Process A".to_string(),
        needed_resources: Resources {
            ram: 4 * 1024,      // 4 GB
            disk_space: 200_000, // 200 GB
            threads: 2,
        },
    };
    let process2 = Process {
        id: 2,
        task: "Process B".to_string(),
        needed_resources: Resources {
            ram: 8 * 1024,      // 8 GB
            disk_space: 300_000, // 300 GB
            threads: 4,
        },
    };
    let process3 = Process {
        id: 3,
        task: "Process C".to_string(),
        needed_resources: Resources {
            ram: 2 * 1024,      // 2 GB
            disk_space: 100_000, // 100 GB
            threads: 1,
        },
    };

    // Simulate nodes requesting resources concurrently
    let processes = vec![process1, process2, process3];

    // Create a vector to hold the task handles
    let mut handles = vec![];

    for process in processes {
        let session_clone = Arc::clone(&session);
        let process_clone = process.clone();

        let handle = tokio::spawn(async move {
            // Each node will attempt to execute the process
            let node_id = process_clone.id; // Assign node based on process ID

            // Lock the session asynchronously
            let mut session_lock = session_clone.lock().await;

            // Request resource
            session_lock.request_resource(node_id).await;

            // Wait until access is granted
            while !session_lock.can_access_resource(node_id) {
                sleep(Duration::from_millis(100)).await;
            }

            // Access granted; allocate resources
            if session_lock.allocate_resources(&process_clone.needed_resources) {
                println!("Node {} is executing process {}", node_id, process_clone.id);

                // Simulate process execution
                if let Some(node) = session_lock.nodes.iter().find(|n| n.id == node_id) {
                    // Call the async execute_process method
                    node.execute_process(&process_clone).await;
                }

                // Deallocate resources
                session_lock.deallocate_resources(&process_clone.needed_resources);
            } else {
                println!("Node {} failed to allocate resources for process {}", node_id, process_clone.id);
            }

            // Release resource
            session_lock.release_resource(node_id);
        });

        handles.push(handle);
    }

    // Wait for all tasks to finish
    for handle in handles {
        handle.await.unwrap();
    }

    // Print final resource status
    let session_lock = session.lock().await;
    println!("\nFinal available resources: {:?}", session_lock.available_resources);
}
