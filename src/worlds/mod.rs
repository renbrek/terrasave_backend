pub mod routes;

use serde::{Serialize, Deserialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorldFile {
    pub name: String,
    pub local_path: String,
    pub birthtime: i64,
    pub modified: i64,
}

impl WorldFile {
    pub fn new(path: String, name: String, birthtime: i64, modified: i64) -> Self {
        Self { name, local_path: path, birthtime, modified}
    }
}

// Саня лох
