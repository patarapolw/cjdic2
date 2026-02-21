use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub word: String,
    pub definition: String,
}
