use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BakData {
    pub comp_favs: Option<String>,
}
