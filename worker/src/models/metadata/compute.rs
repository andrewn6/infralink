#[derive(Debug, Clone)]
pub struct ComputeMetadata {
    pub num_cores: Option<u64>,
    pub cpus: Option<Vec<Cpu>>,
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub frequency: Option<u64>,
    pub load: Option<f32>,
}

impl ComputeMetadata {
    pub fn new() -> Self {
        Self {
            num_cores: None,
            cpus: None,
        }
    }
}
