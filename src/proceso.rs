#[derive(Clone)]
pub struct Proceso {
    pub id: u32,
}

impl Proceso {
    pub fn nuevo(id: u32) -> Self {
        Proceso { id }
    }
}
