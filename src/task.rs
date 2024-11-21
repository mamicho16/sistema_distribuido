use crate::resource::Resources;

pub struct Task {
    pub id: u32,
    pub description: String,
    pub resource_requirements: Resources,
}

impl Task {
    pub fn new(id: u32, description: String, resource_requirements: Resources) -> Self {
        Task { id, description, resource_requirements }
    }
}