use crate::process::Process;
use crate::session::Session;
use crate::message::{Vote, Action};
use rand::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::{HashMap};
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
pub enum NodeStatus {
    Active,
    Halted,
    Recovering,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: u32,
    // List of active processes
    pub active_processes: Vec<Process>,
    pub timestamp: u64,
    pub status: NodeStatus,
    pub last_heartbeat: u64,
    pub known_actions: HashMap<Action, bool>,
}

impl Node {
    pub fn new(id: u32) -> Self {
        Node {
            id,
            active_processes: Vec::new(),
            timestamp: 0,
            status: NodeStatus::Active,
            last_heartbeat: 0,
            known_actions: HashMap::new(),
        }
    }

    // Node receives a proposal and decides whether to vote
    pub fn receive_proposal(&mut self, action: Action) -> Vote {
        println!("Node {} received proposal for action {:?}", self.id, action);

        // Decide whether to vote for the action
        // In a blockchain, nodes validate the action before voting
        // For simplicity, we'll assume nodes always vote 'Approve'
        let vote = self.vote(action);

        // Cast the vote
        vote
    }    

    pub fn vote(&mut self, action: Action) -> Vote {
        // Record that this node knows about the action
        self.known_actions.insert(action.clone(), true);

        // Implement validation logic here
        // For now, we'll approve 80% of the time
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.8) {
            Vote::Approve
        } else {
            Vote::Reject
        }
    }

    pub fn propose_action(&mut self, session: &mut Session, action: Action) {
        println!("Node {} is proposing action {:?}", self.id, action);

        // Start the voting process
        session.initiate_voting(self.id, action);
    } 

    // Handle process failure and deallocate resources
    pub fn handle_process_failure(&mut self, process_id: u32, reason: String) {
        if let Some(pos) = self.active_processes.iter().position(|p| p.id == process_id) {
            let process = self.active_processes.remove(pos);
            // Deallocate resources
            //self.available_resources.deallocate(&process.needed_resources);
            println!("Process {} failed on node {}: {}", process_id, self.id, reason);
            // Optionally, notify session or propose an action
        } else {
            println!("Process {} not found on node {}", process_id, self.id);
        }
    }        

    pub fn handle_failure(&self, session: &mut Session, reason: String) {
        println!("Node {} detected a failure: {}", self.id, reason);
        session.initiate_voting(self.id, Action::NodeFailure {
            node_id: self.id,
            reason,
        });
    }

    pub fn detect_and_report_failure(&mut self, reason: String) -> Action {
        println!("Node {} detected a failure: {}", self.id, reason);

        // Propose an action to handle the failure
        Action::ProcessFailure {
            node_id: self.id,
            reason,
        }
    }   

    // Simulate process execution
    pub async fn execute_process(&self, process: &Process) {
        println!("Node {} is processing {}", self.id, process.task);
        // Simulate some work
        sleep(Duration::from_secs(1)).await;
        println!("Node {} completed process {}", self.id, process.id);
    }

    // Resource access (Ricart-Agrawala algorithm)    
    
    pub fn heartbeat(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_secs();
        self.last_heartbeat = timestamp;
        println!("Node {} heartbeat at timestamp {}", self.id, self.last_heartbeat);
    }

    pub fn halt(&mut self, session: &mut Session, reason: String) {
        self.status = NodeStatus::Halted;
        self.handle_failure(session, reason);
    }
    
    pub fn print_status(&self, session: &Session) {
        println!(
            "Node {} - Available Resources: {}, Active Processes: {}",
            self.id, session.available_resources, self.active_processes.len()
        );
    }

}