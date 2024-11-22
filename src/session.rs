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
                // Optionally, handle this case (e.g., add to waiting queue)
            }
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
        self.remove_node(node_id);
        // Additional failure handling logic...
    }

    // Method to get total number of nodes
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }    

}