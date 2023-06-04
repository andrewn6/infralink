// Used in builder
use chrono::{DateTime, Utc};
use uuid::uuid;

#[derive(Debug)]
pub struct Deployment {
    pub id: uuid,
    pub build_id: uuid,
    pub status: String, 
    pub created_at: DateTime<Utc>,
}

impl Deployment {
    pub fn new(build_id: uuid, status: String) -> Self {
        Deployment {
            id: uuid::new_v4(),
            build_id,
            status,
            created_at: Utc::now(),
        }
    }
}