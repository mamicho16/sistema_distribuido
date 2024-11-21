use sistema_distribuido::node::Node;
use sistema_distribuido::resource::Resources;
use sistema_distribuido::session::Session;
use sistema_distribuido::process::Process;

fn simulate_failure() {
    // Create nodes
    let node1 = Node::new(1, Resources::new(8, 500, 4));
    let node2 = Node::new(2, Resources::new(8, 500, 4));
    let node3 = Node::new(3, Resources::new(8, 500, 4));

    // Initialize session
    let mut session = Session::new(vec![node1, node2, node3], vec![]);

    // Simulate a node detecting a failure and proposing an action
    let action = session.nodes[0].detect_and_report_failure("Disk failure".to_string());

    session.initiate_voting(session.nodes[0].id, action);

    // The nodes have received the proposal and voted; the action should be executed if consensus is reached
}

fn main() {
    // Create resources for nodes
    let node_resources = Resources { ram: 16 * 1024, disk_space: 1_000_000, threads: 8 }; // RAM in MB, Disk in MB

    // Create nodes
    let node1 = Node::new(1, node_resources.clone());
    let node2 = Node::new(2, node_resources.clone());
    let node3 = Node::new(3, node_resources.clone());

    // Create processes with required resources
    let process1 = Process::new(1, "Task A".to_string(), Resources { ram: 4 * 1024, disk_space: 200_000, threads: 2 });
    let process2 = Process::new(2, "Task B".to_string(), Resources { ram: 8 * 1024, disk_space: 300_000, threads: 4 });
    let process3 = Process::new(3, "Task C".to_string(), Resources { ram: 2 * 1024, disk_space: 100_000, threads: 1 });

    // Initialize session
    let mut session = Session::new(vec![node1, node2, node3], vec![process1, process2, process3]);

    // Assign processes
    session.assign_processes();

    // Print node statuses
    for node in &session.nodes {
        node.print_status();
    }

    // Simulate process completion
    session.nodes[0].complete_process(1);

    // Print node statuses after process completion
    for node in &session.nodes {
        node.print_status();
    }
}
