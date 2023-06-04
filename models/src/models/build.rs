// Used in builder
use chrono::{DateTime, Utc};
use uuid::uuid;

#[derive(Debug)]
pub struct Build {
    pub id: uuid,
    pub image: String,
    pub created_at: DateTime<Utc>,
}

impl Build {
    pub fn new(image: String) -> Self {
        Build {
            id: uuid::new_v4(),
            image,
            created_at: Utc::now(),
        }
    }
}