#[derive(Clone)]
pub struct Process {
    pub id: u32,
}

impl Process {
    pub fn nuevo(id: u32) -> Self {
        Process { id }
    }
}
