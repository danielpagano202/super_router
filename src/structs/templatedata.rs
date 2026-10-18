use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")] 
pub struct TemplateData {
    pub template_path: String,
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
}