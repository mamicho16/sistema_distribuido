use sistema_distribuido::node::Node;
use sistema_distribuido::resource::Resources;
use sistema_distribuido::session::Session;

fn main() {

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
