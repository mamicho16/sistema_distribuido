use crate::resource::Resources;

#[derive(Clone, Debug)]
pub struct Process {
    pub id: u32,
    pub task: String,
    pub needed_resources: Resources,
}

impl Process {
    pub fn new(id: u32, task: String, needed_resources: Resources) -> Self {
        Process { id, task, needed_resources }
    }
}