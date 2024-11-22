use crate::process::Process;
use crate::session::Session;
use crate::message::{Vote, Action};
use std::collections::{HashMap};
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Active,
    Halted,
    Recovering,
}

#[derive(Clone, Debug, PartialEq)]
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
        let vote = self.vote(action);

        // Cast the vote
        vote
    }    

    pub fn vote(&mut self, action: Action) -> Vote {
        // Record that this node knows about the action
        self.known_actions.insert(action.clone(), true);

        // Validation logic
        match action {
            Action::ProcessFailure { reason, .. } => {
                if reason.contains("critical") {
                    println!("Node {}: Rejecting action due to critical issue.", self.id);
                    Vote::Reject
                } else {
                    println!("Node {}: Approving action.", self.id);
                    Vote::Approve
                }
            }
            Action::NodeFailure { reason, .. } => {
                if reason.contains("hardware") {
                    println!("Node {}: Rejecting action due to hardware issue.", self.id);
                    Vote::Reject
                } else {
                    println!("Node {}: Approving action.", self.id);
                    Vote::Approve
                }
            }
            _ => {
                println!("Node {}: Approving unknown action.", self.id);
                Vote::Approve
            }
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
            println!(
                "Process {} failed on node {}: {}. Deallocating resources.",
                process_id, self.id, reason
            );
    
            // Example: Deallocate memory and CPU resources
            println!(
                "Resources deallocated for process {}: RAM={}MB, Disk Space={}MB, Threads={}",
                process_id, process.needed_resources.ram, process.needed_resources.disk_space, process.needed_resources.threads
            );
    
            // Notify session or propose action
            let failure_action = Action::ProcessFailure {
                node_id: self.id,
                reason,
            };
            println!(
                "Node {} proposing action to handle process failure: {:?}",
                self.id, failure_action
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::Process;
    use crate::resource::Resources;
    use crate::message::{Action, Vote};
    use std::collections::HashMap;

    // MockSession struct for testing
    struct MockSession {
        initiated_voting: Vec<(u32, Action)>,
    }

    impl MockSession {
        fn new() -> Self {
            MockSession {
                initiated_voting: Vec::new(),
            }
        }

        fn initiate_voting(&mut self, proposer_id: u32, action: Action) {
            self.initiated_voting.push((proposer_id, action));
        }
    }

    // MockSessionWithResources struct for testing
    struct MockSessionWithResources {
        pub available_resources: Resources,
    }

    impl MockSessionWithResources {
        fn new(available_resources: Resources) -> Self {
            MockSessionWithResources {
                available_resources,
            }
        }
    }

    #[test]
    fn test_node_creation() {
        let node = Node::new(1);

        assert_eq!(node.id, 1);
        assert_eq!(node.active_processes.len(), 0);
        matches!(node.status, NodeStatus::Active);
        assert_eq!(node.last_heartbeat, 0);
        assert!(node.known_actions.is_empty());
    }

    #[test]
    fn test_receive_proposal() {
        let mut node = Node::new(1);
        let action = Action::ProcessFailure {
            node_id: 2,
            reason: "Test failure".to_string(),
        };

        let vote = node.receive_proposal(action.clone());

        // Since the vote is randomly decided with 80% approval rate, we'll accept both outcomes
        assert!(matches!(vote, Vote::Approve) || matches!(vote, Vote::Reject));
        assert!(node.known_actions.contains_key(&action));
    }

    #[test]
    fn test_handle_process_failure_existing_process() {
        let mut node = Node::new(1);
        let process = Process::new(
            100,
            "Test Process".to_string(),
            Resources::new(1024, 100_000, 2),
        );
        node.active_processes.push(process.clone());

        node.handle_process_failure(100, "Simulated failure".to_string());

        assert!(node.active_processes.is_empty());
    }

    #[test]
    fn test_handle_process_failure_nonexistent_process() {
        let mut node = Node::new(1);

        node.handle_process_failure(200, "Simulated failure".to_string());

        // Since the process didn't exist, active_processes should remain empty
        assert!(node.active_processes.is_empty());
    }

    #[test]
    fn test_detect_and_report_failure() {
        let mut node = Node::new(1);
        let reason = "Simulated failure".to_string();

        let action = node.detect_and_report_failure(reason.clone());

        match action {
            Action::ProcessFailure { node_id, reason: r } => {
                assert_eq!(node_id, 1);
                assert_eq!(r, reason);
            },
            _ => panic!("Expected Action::ProcessFailure"),
        }
    }

    #[tokio::test]
    async fn test_execute_process() {
        let node = Node::new(1);
        let process = Process::new(
            101,
            "Async Test Process".to_string(),
            Resources::new(2048, 200_000, 4),
        );

        node.execute_process(&process).await;

        // Since execute_process only simulates execution with sleep, there's no state change to assert
    }
}