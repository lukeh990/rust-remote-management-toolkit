use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Bug {
    pub machine: String,
    pub body: String
}
