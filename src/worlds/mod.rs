pub mod routes;
use std::time::SystemTime;

use serde::{Serialize, Deserialize};
use serde_json::json;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorldFile {
    pub name: String,
    pub local_path: String,
    pub birthtime: i64,
    pub modified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorldFileModel {
    pub name: String,
    pub local_path: String,
    pub birthtime: String,
    pub modified: String,
}
impl WorldFile {
    pub fn new(path: String, name: String, birthtime: i64, modified: i64) -> Self {
        Self { name, local_path: path, birthtime, modified}
    }
    
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
}

// struct  ModifiedTime {
//     time: i32
// }
//
// impl From<SystemTime> for ModifiedTime {
//     fn from(value: SystemTime) -> Self {
//         // Self { time: value }
//         let system_time_json = json!(value);
//         todo!()
//     }
// }
// Саня лох
