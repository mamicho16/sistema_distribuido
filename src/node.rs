use crate::process::Process;
use crate::resource::Resources;
use crate::session::Session;
use crate::message::{Vote, Action};
use rand::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

pub enum NodeStatus {
    Active,
    Halted,
    Recovering,
}

pub struct Node {
    pub id: u32,
    pub active_processes: Vec<Process>,
    pub total_resources: Resources,
    pub available_resources: Resources,
    pub status: NodeStatus,
    pub last_heartbeat: u64,
    pub known_actions: HashMap<Action, bool>,
}

impl Node {
    pub fn new(id: u32, resources: Resources) -> Self {
        Node {
            id,
            active_processes: Vec::new(),
            total_resources: resources.clone(),
            available_resources: resources,
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

    fn validate_action(&self, action: &Action) -> bool {
        // Implement validation logic here
        // For now, randomly approve or reject
        let mut rng = rand::thread_rng();
        rng.gen_bool(0.8) // 80% chance to approve
    }

    pub fn propose_action(&mut self, session: &mut Session, action: Action) {
        println!("Node {} is proposing action {:?}", self.id, action);

        // Start the voting process
        session.initiate_voting(self.id, action);
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

    pub fn complete_process(&mut self, process_id: u32) {
        if let Some(pos) = self.active_processes.iter().position(|p| p.id == process_id) {
            let process = self.active_processes.remove(pos);
            self.resources.deallocate(process.needed_resources);
            println!("Node {} completed process {}", self.id, process_id);
        }
    }

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
}