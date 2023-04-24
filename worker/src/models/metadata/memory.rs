#[derive(Debug, Clone)]
pub struct MemoryMetadata {
    pub primary: Option<Memory>,
    pub swaps: Option<Memory>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub total: Option<u64>,
    pub used: Option<u64>,
    pub free: Option<u64>,
}

impl MemoryMetadata {
    pub fn new() -> Self {
        MemoryMetadata {
            primary: None,
            swaps: None,
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            total: None,
            used: None,
            free: None,
        }
    }
}
