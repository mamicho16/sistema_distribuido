use crate::node::{Node};
use crate::process::Process;
use crate::message::{Vote, Action};
use std::collections::HashMap;

pub struct Session {
    pub nodes: Vec<Node>,
    pub processes: Vec<Process>,
    pub pending_votes: HashMap<Action, Vec<(u32, Vote)>>,
}

impl Session {
    pub fn new(nodes: Vec<Node>, processes: Vec<Process>) -> Self {
        Session {
            nodes,
            processes,
            pending_votes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn remove_node(&mut self, node_id: u32) {
        self.nodes.retain(|node| node.id != node_id);
    }

    pub fn assign_processes(&mut self) {
        for process in &self.processes {
            // Filter nodes that can handle the process's resource requirements
            let suitable_nodes: Vec<&mut Node> = self.nodes.iter_mut()
                .filter(|node| node.available_resources.can_allocate(&process.needed_resources))
                .collect();

            // If there are suitable nodes, pick the one with the least number of active processes
            if let Some(node) = suitable_nodes.into_iter()
                .min_by_key(|node| node.active_processes.len())
            {
                // Allocate resources
                if node.available_resources.allocate(&process.needed_resources) {
                    // Assign the process
                    node.active_processes.push(process.clone());
                    println!("Assigned process {} to node {}", process.id, node.id);
                } else {
                    println!("Failed to allocate resources for process {} on node {}", process.id, node.id);
                }
            } else {
                // No suitable node found
                println!("No available nodes can handle process {} due to resource constraints.", process.id);
                // Optionally, you might want to queue the process or handle this case differently
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
}