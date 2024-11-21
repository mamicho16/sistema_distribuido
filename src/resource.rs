use std::fmt;

#[derive(Clone, Debug)]
pub struct Resources {
    pub ram: u64,
    pub disk_space: u64,
    pub threads: u32,
}

impl Resources {

    pub fn new(ram: u64, disk_space: u64, threads: u32) -> Self {
        Resources { ram, disk_space, threads }
    }

    // Check if the requested resources can be allocated
    pub fn can_allocate(&self, requested: &Resources) -> bool {
        self.ram >= requested.ram
            && self.disk_space >= requested.disk_space
            && self.threads >= requested.threads
    }

    // Allocate resources
    pub fn allocate(&mut self, requested: &Resources) -> bool {
        if self.can_allocate(requested) {
            self.ram -= requested.ram;
            self.disk_space -= requested.disk_space;
            self.threads -= requested.threads;
            true
        } else {
            false
        }
    }

    // Deallocate resources
    pub fn deallocate(&mut self, released: &Resources) {
        self.ram += released.ram;
        self.disk_space += released.disk_space;
        self.threads += released.threads;
    }
}

impl fmt::Display for Resources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Resources [RAM: {} MB, Disk: {} MB, Threads: {}]",
            self.ram, self.disk_space, self.threads
        )
    }    
}