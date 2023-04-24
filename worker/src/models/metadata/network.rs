#[derive(Debug, Clone)]
pub struct NetworkMetadata {
    pub total_outbound: Option<u64>,
}

impl NetworkMetadata {
    pub fn new() -> Self {
        Self {
            total_outbound: None,
        }
    }
}
