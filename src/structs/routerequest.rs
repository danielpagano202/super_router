use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")] 
pub struct RouteRequest {
    pub path: String,
    pub body: String,
    pub cookies: String,
    pub method: String,
    pub is_valid: bool,
}