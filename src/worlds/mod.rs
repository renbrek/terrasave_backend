pub mod routes;
use std::time::SystemTime;

use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFile {
    name: String,
    local_path: String,
    birthtime: i64,
    modified: i64,
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
