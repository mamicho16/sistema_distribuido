#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Vote {
    Approve,
    Reject,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    ProcessFailure { node_id: u32, reason: String },
    NodeFailure { node_id: u32, reason: String },
    RedistributeProcess { process_id: u32 },
}

#[derive(Clone, Debug)]
pub struct Request {
    pub from_node_id: u32,
    pub timestamp: u64,
}