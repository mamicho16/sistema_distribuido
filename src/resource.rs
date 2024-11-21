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

    pub fn allocate(&mut self, requested: Resources) -> bool {
        if self.ram >= requested.ram &&
           self.disk_space >= requested.disk_space &&
           self.threads >= requested.threads {
            self.ram -= requested.ram;
            self.disk_space -= requested.disk_space;
            self.threads -= requested.threads;
            true
        } else {
            false
        }
    }

    pub fn deallocate(&mut self, released: Resources) {
        self.ram += released.ram;
        self.disk_space += released.disk_space;
        self.threads += released.threads;
    }
}