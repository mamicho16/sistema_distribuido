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

pub struct Message {
    pub sender_id: u32,
    pub receiver_id: u32,
    pub payload: String,
    pub timestamp: u64,
}