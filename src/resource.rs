use std::fmt;

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resources = Resources::new(8192, 500_000, 4);

        assert_eq!(resources.ram, 8192);
        assert_eq!(resources.disk_space, 500_000);
        assert_eq!(resources.threads, 4);
    }

    #[test]
    fn test_can_allocate_true() {
        let available = Resources::new(16000, 1_000_000, 8);
        let requested = Resources::new(8000, 500_000, 4);

        assert!(available.can_allocate(&requested));
    }

    #[test]
    fn test_can_allocate_false_ram() {
        let available = Resources::new(4000, 1_000_000, 8);
        let requested = Resources::new(8000, 500_000, 4);

        assert!(!available.can_allocate(&requested));
    }

    #[test]
    fn test_can_allocate_false_disk_space() {
        let available = Resources::new(16000, 200_000, 8);
        let requested = Resources::new(8000, 500_000, 4);

        assert!(!available.can_allocate(&requested));
    }

    #[test]
    fn test_can_allocate_false_threads() {
        let available = Resources::new(16000, 1_000_000, 2);
        let requested = Resources::new(8000, 500_000, 4);

        assert!(!available.can_allocate(&requested));
    }

    #[test]
    fn test_can_allocate_exact_match() {
        let available = Resources::new(8000, 500_000, 4);
        let requested = Resources::new(8000, 500_000, 4);

        assert!(available.can_allocate(&requested));
    }

    #[test]
    fn test_allocate_success() {
        let mut available = Resources::new(16000, 1_000_000, 8);
        let requested = Resources::new(8000, 500_000, 4);

        let result = available.allocate(&requested);

        assert!(result);
        assert_eq!(available.ram, 8000);
        assert_eq!(available.disk_space, 500_000);
        assert_eq!(available.threads, 4);
    }

    #[test]
    fn test_allocate_failure() {
        let mut available = Resources::new(4000, 1_000_000, 8);
        let requested = Resources::new(8000, 500_000, 4);

        let result = available.allocate(&requested);

        assert!(!result);
        // Resources should remain unchanged
        assert_eq!(available.ram, 4000);
        assert_eq!(available.disk_space, 1_000_000);
        assert_eq!(available.threads, 8);
    }

    #[test]
    fn test_deallocate() {
        let mut available = Resources::new(8000, 500_000, 4);
        let released = Resources::new(4000, 200_000, 2);

        available.deallocate(&released);

        assert_eq!(available.ram, 12_000);          // 8000 + 4000
        assert_eq!(available.disk_space, 700_000);  // 500,000 + 200,000
        assert_eq!(available.threads, 6);           // 4 + 2
    }

    #[test]
    fn test_display() {
        let resources = Resources::new(8192, 500_000, 4);
        let display_str = format!("{}", resources);

        assert_eq!(
            display_str,
            "Resources [RAM: 8192 MB, Disk: 500000 MB, Threads: 4]"
        );
    }
}