use crate::node::{Node};
use crate::process::Process;
use crate::resource::Resources;
use crate::message::{Vote, Action, Request};
use std::collections::{HashMap, VecDeque};

pub struct Session {
    pub nodes: Vec<Node>,
    pub processes: Vec<Process>,
    pub total_resources: Resources,
    pub available_resources: Resources,
    pub pending_votes: HashMap<Action, Vec<(u32, Vote)>>,
    // Mutual exclusion queues
    pub request_queue: VecDeque<Request>,
    pub deferred_replies: HashMap<u32, Vec<Request>>,
    pub replies_received: HashMap<u32, Vec<u32>>,
}

impl Session {
    pub fn new(nodes: Vec<Node>, processes: Vec<Process>, total_resources: Resources) -> Self {
        Session {
            nodes,
            processes,
            total_resources: total_resources.clone(),
            available_resources: total_resources,
            pending_votes: HashMap::new(),
            request_queue: VecDeque::new(),
            deferred_replies: HashMap::new(),
            replies_received: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, node_id: u32) {
        self.nodes.retain(|node| node.id != node_id);
    }

    // Resource access (Ricart-Agrawala algorithm)    

    // Resource management
    pub async fn request_resource(&mut self, node_id: u32) {
        let timestamp = self.generate_timestamp();
        let request = Request {
            from_node_id: node_id,
            timestamp,
        };
    
        println!("Node {} is requesting access to the shared resource", node_id);
    
        // Add the request to the global queue
        self.request_queue.push_back(request.clone());
    
        // Collect the IDs of other nodes
        let other_node_ids: Vec<u32> = self
            .nodes
            .iter()
            .filter(|node| node.id != node_id)
            .map(|node| node.id)
            .collect();
    
        // Initialize replies received for this request
        self.replies_received.insert(node_id, Vec::new());
    
        // Handle the request for each node
        for other_node_id in other_node_ids {
            self.handle_request(other_node_id, request.clone());
        }
    }

    // Generate a logical timestamp
    fn generate_timestamp(&self) -> u64 {
        // Simple logical clock
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        duration.as_millis() as u64
    }

    // Handle an incoming request
    pub fn handle_request(&mut self, to_node_id: u32, request: Request) {
        // Determine if we need to send a reply immediately or defer it
        let should_reply = self.should_reply_immediately(to_node_id, &request);

        if should_reply {
            // Send reply immediately
            self.send_reply(request.from_node_id, to_node_id);
        } else {
            // Defer the reply
            self.deferred_replies
                .entry(to_node_id)
                .or_insert(Vec::new())
                .push(request);
        }
    }

    // Decide whether to reply immediately or defer
    fn should_reply_immediately(&self, node_id: u32, incoming_request: &Request) -> bool {
        // Check if the node has a pending request with higher priority
        if let Some(our_request) = self.request_queue.front() {
            if our_request.from_node_id == node_id {
                // Compare timestamps
                if our_request.timestamp < incoming_request.timestamp {
                    return false; // Defer reply
                } else if our_request.timestamp == incoming_request.timestamp {
                    return node_id < incoming_request.from_node_id; // Tie-breaker
                }
            }
        }
        true // Send reply immediately
    }

    // Send a reply to a node
    pub fn send_reply(&mut self, to_node_id: u32, from_node_id: u32) {
        println!("Node {} sends reply to Node {}", from_node_id, to_node_id);

        // Record that we have replied to this node
        self.replies_received
            .entry(to_node_id)
            .or_insert(Vec::new())
            .push(from_node_id);
    }

    // Check if the node can access the resource
    pub fn can_access_resource(&self, node_id: u32) -> bool {
        if let Some(replies) = self.replies_received.get(&node_id) {
            replies.len() == self.nodes.len() - 1
        } else {
            false
        }
    }

    // Release the resource after usage
    pub fn release_resource(&mut self, node_id: u32) {
        println!("Node {} is releasing the shared resource", node_id);

        // Remove own request from the queue
        self.request_queue
            .retain(|r| r.from_node_id != node_id);

        // Send deferred replies
        if let Some(deferred) = self.deferred_replies.remove(&node_id) {
            for request in deferred {
                self.send_reply(request.from_node_id, node_id);
            }
        }

        // Clear replies received
        self.replies_received.remove(&node_id);
    }

    // Allocate resources for a process
    pub fn allocate_resources(&mut self, needed_resources: &Resources) -> bool {
        if self.available_resources.can_allocate(needed_resources) {
            self.available_resources.allocate(needed_resources);
            println!("Resources allocated: {:?}", needed_resources);
            true
        } else {
            println!("Not enough resources available to allocate: {:?}", needed_resources);
            false
        }
    }

    // Deallocate resources after process completion
    pub fn deallocate_resources(&mut self, used_resources: &Resources) {
        self.available_resources.deallocate(used_resources);
        println!("Resources deallocated: {:?}", used_resources);
    }    

    pub async fn assign_processes(&mut self) {
        let mut waiting_queue: Vec<Process> = vec![];
        for process in self.processes.clone() {
            // Step 1: Find the node with the least active processes
            let node_id = {
                let min_node = self.nodes.iter()
                    .min_by_key(|node| node.active_processes.len());
                if let Some(node) = min_node {
                    node.id
                } else {
                    panic!("No nodes available");
                }
            };

            // Step 2: Allocate resources
            if self.allocate_resources(&process.needed_resources) {
                // Step 3: Assign the process to the node
                if let Some(node) = self.nodes.iter_mut().find(|node| node.id == node_id) {
                    node.active_processes.push(process.clone());
                    println!("Assigned process {} to node {}", process.id, node.id);
                } else {
                    println!("Node with id {} not found", node_id);
                }
            } else {
                println!(
                    "Failed to assign process {} due to insufficient resources.",
                    process.id
                );
                let mut waiting_queue: Vec<Process> = vec![];
            }
            // Re-assign the waiting queue
            self.processes = waiting_queue;
        }
    }

    // Initiate voting on an action proposed by a node
    pub fn initiate_voting(&mut self, proposer_id: u32, action: Action) {
        println!(
            "Session initiating voting on action {:?} proposed by Node {}",
            action, proposer_id
        );

        // Record that the proposer has voted for the action
        self.pending_votes
            .insert(action.clone(), vec![(proposer_id, Vote::Approve)]);

        // Collect votes from other nodes
        let mut collected_votes = vec![];

        for node in self
            .nodes
            .iter_mut()
            .filter(|n| n.id != proposer_id)
        {
            let vote = node.receive_proposal(action.clone());
            collected_votes.push((node.id, vote));
        }

        // Process votes after the loop
        for (node_id, vote) in collected_votes {
            self.cast_vote(node_id, action.clone(), vote);
        }

        // Check if consensus is reached
        self.check_consensus(action);
    }

    // Nodes call this method to cast their vote
    pub fn cast_vote(&mut self, node_id: u32, action: Action, vote: Vote) {
        let votes = self
            .pending_votes
            .entry(action.clone())
            .or_insert(Vec::new());
        votes.push((node_id, vote.clone()));
        println!("Node {} voted {:?} for action {:?}", node_id, vote, action);
    }

    // Check if the action has received enough votes
    fn check_consensus(&mut self, action: Action) {
        if let Some(votes) = self.pending_votes.get(&action) {
            let total_nodes = self.nodes.len();

            let approvals = votes
                .iter()
                .filter(|&(_, ref v)| *v == Vote::Approve)
                .count();

            if approvals > total_nodes / 2 {
                println!("Consensus reached on action {:?}", action);
                self.execute_action(action.clone());
                self.pending_votes.remove(&action);
            } else if votes.len() - approvals > total_nodes / 2 {
                println!("Consensus rejected on action {:?}", action);
                self.pending_votes.remove(&action);
            }
            // Else, keep waiting for more votes
        }
    }

    // Execute the action once consensus is reached
    fn execute_action(&mut self, action: Action) {
        match action {
            Action::ProcessFailure { node_id, reason } => {
                println!("Executing ProcessFailure action for node {}: {}", node_id, reason);
                self.handle_node_failure(node_id, reason);
            },
            Action::RedistributeProcess { process_id } => {
                println!("Executing RedistributeProcess action for process {}", process_id);
                // Implement redistribution logic here
            },
            Action::NodeFailure { node_id, reason } => {
                println!("Executing NodeFailure action for node {}: {}", node_id, reason);
                self.handle_node_failure(node_id, reason);
            },
            // Handle other actions
        }
    }

    fn handle_node_failure(&mut self, node_id: u32, reason: String) {
        println!("Handling failure of node {}: {}", node_id, reason);

        //Finds the node failure
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            // Releases the resources occupied by the node's active processes
            for process in &node.active_processes {
                self.deallocate_resources(&process.needed_resources);
            }
    
            // redistributing active processes
            for process in node.active_processes.clone() {
                self.processes.push(process);
            }
        }

        // Deletes the node from the system
        self.remove_node(node_id);
        println!("Nodo {} deleted. Processes reassigned.", node_id);

        // Opcional: Tries to reinstall the node
        self.try_to_reinstall_node(node_id);
        }

        // Reinstalls a failure node (if possible)
        fn try_to_reinstall_node(&mut self, node_id: u32) {
            println!("Trying to reinstall node {}", node_id);
            let new_node = Node::new(node_id);
            self.add_node(new_node);
            println!("Node {} reinstalled.", node_id);
        }

    // Method to get total number of nodes
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }    

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::process::Process;
    use crate::resource::Resources;
    use crate::message::{Action, Vote};
    use crate::message::Request;

    #[test]
    fn test_session_new() {
        let nodes = vec![Node::new(1), Node::new(2)];
        let processes = vec![
            Process::new(1, "Process 1".to_string(), Resources::new(1024, 100_000, 2)),
            Process::new(2, "Process 2".to_string(), Resources::new(2048, 200_000, 4)),
        ];
        let total_resources = Resources::new(16_384, 1_000_000, 8);
    
        let session = Session::new(nodes.clone(), processes.clone(), total_resources.clone());
    
        assert_eq!(session.nodes, nodes);
        assert_eq!(session.processes, processes);
        assert_eq!(session.total_resources, total_resources);
        assert_eq!(session.available_resources, total_resources);
        assert!(session.pending_votes.is_empty());
        assert!(session.request_queue.is_empty());
        assert!(session.deferred_replies.is_empty());
        assert!(session.replies_received.is_empty());
    }

    #[test]
    fn test_add_and_remove_node() {
        let mut session = Session::new(vec![], vec![], Resources::new(0, 0, 0));
        let node1 = Node::new(1);
        let node2 = Node::new(2);
    
        session.add_node(node1.clone());
        assert_eq!(session.nodes.len(), 1);
        assert_eq!(session.nodes[0], node1);
    
        session.add_node(node2.clone());
        assert_eq!(session.nodes.len(), 2);
        assert_eq!(session.nodes[1], node2);
    
        session.remove_node(1);
        assert_eq!(session.nodes.len(), 1);
        assert_eq!(session.nodes[0], node2);
    }

    #[test]
    fn test_total_nodes() {
        let mut session = Session::new(vec![], vec![], Resources::new(0, 0, 0));
        assert_eq!(session.total_nodes(), 0);
    
        session.add_node(Node::new(1));
        session.add_node(Node::new(2));
        assert_eq!(session.total_nodes(), 2);
    
        session.remove_node(1);
        assert_eq!(session.total_nodes(), 1);
    }

    #[test]
    fn test_allocate_resources_success() {
        let total_resources = Resources::new(16_384, 1_000_000, 8);
        let mut session = Session::new(vec![], vec![], total_resources.clone());
    
        let needed_resources = Resources::new(4_096, 200_000, 2);
    
        let result = session.allocate_resources(&needed_resources);
        assert!(result);
        assert_eq!(
            session.available_resources,
            Resources::new(12_288, 800_000, 6)
        );
    }
    
    #[test]
    fn test_allocate_resources_failure() {
        let total_resources = Resources::new(16_384, 1_000_000, 8);
        let mut session = Session::new(vec![], vec![], total_resources.clone());
    
        let needed_resources = Resources::new(32_768, 2_000_000, 16);
    
        let result = session.allocate_resources(&needed_resources);
        assert!(!result);
        assert_eq!(session.available_resources, total_resources);
    }

    #[test]
    fn test_deallocate_resources() {
        let total_resources = Resources::new(16_384, 1_000_000, 8);
        let mut session = Session::new(vec![], vec![], total_resources.clone());
    
        let needed_resources = Resources::new(4_096, 200_000, 2);
        session.allocate_resources(&needed_resources);
    
        session.deallocate_resources(&needed_resources);
        assert_eq!(session.available_resources, total_resources);
    }

    #[tokio::test]
    async fn test_assign_processes() {
        let node1 = Node::new(1);
        let node2 = Node::new(2);
        let mut session = Session::new(
            vec![node1.clone(), node2.clone()],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        );
    
        let process1 = Process::new(1, "Process 1".to_string(), Resources::new(4_096, 200_000, 2));
        let process2 = Process::new(2, "Process 2".to_string(), Resources::new(4_096, 200_000, 2));
        session.processes.push(process1.clone());
        session.processes.push(process2.clone());
    
        session.assign_processes().await;
    
        // Check that processes have been assigned
        let node1_processes = session.nodes.iter().find(|n| n.id == 1).unwrap().active_processes.clone();
        let node2_processes = session.nodes.iter().find(|n| n.id == 2).unwrap().active_processes.clone();
    
        assert_eq!(node1_processes.len() + node2_processes.len(), 2);
    
        // Check that resources have been allocated
        assert_eq!(
            session.available_resources,
            Resources::new(8_192, 600_000, 4)
        );
    }

    #[tokio::test]
    async fn test_request_resource_and_can_access_resource() {
        let node1 = Node::new(1);
        let node2 = Node::new(2);
        let node3 = Node::new(3);
        let mut session = Session::new(
            vec![node1.clone(), node2.clone(), node3.clone()],
            vec![],
            Resources::new(16_384, 1_000_000, 8),
        );
    
        // Node 1 requests resource
        session.request_resource(1).await;
        // Since no other nodes have requested, Node 1 should have access
        assert!(session.can_access_resource(1));
    
        // Node 2 requests resource
        session.request_resource(2).await;
        // Node 2 should have access
        assert!(session.can_access_resource(2));
    
        // Node 1 releases resource
        session.release_resource(1);
    
        // Now Node 2 should have access
        assert!(session.can_access_resource(2));
    }

    #[test]
    fn test_handle_request_and_should_reply_immediately() {
        let node1 = Node::new(1);
        let node2 = Node::new(2);
        let mut session = Session::new(vec![node1.clone(), node2.clone()], vec![], Resources::new(0, 0, 0));
    
        // Simulate Node 1's request
        let request1 = Request {
            from_node_id: 1,
            timestamp: 100,
        };
        session.request_queue.push_back(request1.clone());
    
        // Node 2 receives Node 1's request
        session.handle_request(2, request1.clone());
    
        // Since Node 2 hasn't requested, it should send a reply immediately
        assert_eq!(session.replies_received.get(&1).unwrap().len(), 1);
        assert_eq!(session.replies_received.get(&1).unwrap()[0], 2);
    
        // Now, Node 2 requests the resource with a higher timestamp
        let request2 = Request {
            from_node_id: 2,
            timestamp: 200,
        };
        session.request_queue.push_back(request2.clone());
    
        // Node 1 handles Node 2's request
        session.handle_request(1, request2.clone());
    
        // Node 1 should defer the reply since its request has a lower timestamp
        assert!(session.deferred_replies.get(&1).unwrap().len() == 1);
    }

    #[test]
    fn test_release_resource_and_deferred_replies() {
        let node1 = Node::new(1);
        let node2 = Node::new(2);
        let mut session = Session::new(vec![node1.clone(), node2.clone()], vec![], Resources::new(0, 0, 0));
    
        // Node 1 requests resource
        let request1 = Request {
            from_node_id: 1,
            timestamp: 100,
        };
        session.request_queue.push_back(request1.clone());
    
        // Node 2 handles Node 1's request and replies immediately
        session.handle_request(2, request1.clone());
    
        // Node 2 requests resource
        let request2 = Request {
            from_node_id: 2,
            timestamp: 200,
        };
        session.request_queue.push_back(request2.clone());
    
        // Node 1 handles Node 2's request and defers reply
        session.handle_request(1, request2.clone());
    
        // Node 1 releases resource
        session.release_resource(1);
    
        // Node 1 should send the deferred reply to Node 2
        assert_eq!(session.replies_received.get(&2).unwrap().len(), 1);
        assert_eq!(session.replies_received.get(&2).unwrap()[0], 1);
    }

    // TODO: Add tests for voting and consensus
    // #[test]
    // fn test_voting_and_consensus() {
    //     // Create nodes with overridden receive_proposal methods
    //     let mut node1 = Node::new(1);
    //     let mut node2 = Node::new(2);
    //     let mut node3 = Node::new(3);
    
    //     // Override receive_proposal to return specific votes
    //     node1.receive_proposal = Box::new(|_| Vote::Approve);
    //     node2.receive_proposal = Box::new(|_| Vote::Approve);
    //     node3.receive_proposal = Box::new(|_| Vote::Reject);
    
    //     let mut session = Session::new(vec![node1, node2, node3], vec![], Resources::new(0, 0, 0));
    
    //     let action = Action::NodeFailure {
    //         node_id: 2,
    //         reason: "Test failure".to_string(),
    //     };
    
    //     session.initiate_voting(1, action.clone());
    
    //     // Since two out of three nodes approve, consensus should be reached
    //     // Check that the pending_votes for the action have been removed
    //     assert!(!session.pending_votes.contains_key(&action));
    // }
}