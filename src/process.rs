use crate::resource::Resources;

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::Resources;

    #[test]
    fn test_process_creation() {
        let resources = Resources::new(4096, 250_000, 2);
        let process = Process::new(1, "Test Task".to_string(), resources.clone());

        assert_eq!(process.id, 1);
        assert_eq!(process.task, "Test Task");
        assert_eq!(process.needed_resources, resources);
    }

    #[test]
    fn test_process_clone() {
        let resources = Resources::new(8192, 500_000, 4);
        let process1 = Process::new(2, "Clone Task".to_string(), resources.clone());
        let process2 = process1.clone();

        assert_eq!(process1.id, process2.id);
        assert_eq!(process1.task, process2.task);
        assert_eq!(process1.needed_resources, process2.needed_resources);
    }

    #[test]
    fn test_process_debug_format() {
        let resources = Resources::new(1024, 100_000, 1);
        let process = Process::new(3, "Debug Task".to_string(), resources);

        let debug_str = format!("{:?}", process);

        // Ensure that the debug string contains the expected substrings
        assert!(debug_str.contains("Process"));
        assert!(debug_str.contains("id: 3"));
        assert!(debug_str.contains("task: \"Debug Task\""));
        assert!(debug_str.contains("needed_resources: Resources"));
    }
}