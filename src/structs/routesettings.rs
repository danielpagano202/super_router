use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")] 
pub struct RouteSettings{
    pub build_command: String,
    #[serde(default)]
    pub build_output_path: String,
    pub run_command: String,
    #[serde(default)]
    pub build_args: Vec<String>,
    #[serde(default)]
    pub run_args: Vec<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub code_file: String,
    #[serde(default)]
    pub ignore: bool,
}

impl RouteSettings {
    pub fn merge(&self, other: &RouteSettings) -> RouteSettings {
        RouteSettings {
            build_command: if !other.build_command.is_empty() { other.build_command.clone() } else { self.build_command.clone() },
            build_output_path: if !other.build_output_path.is_empty() { other.build_output_path.clone() } else { self.build_output_path.clone() },
            run_command: if !other.run_command.is_empty() { other.run_command.clone() } else { self.run_command.clone() },
            build_args: if !other.build_args.is_empty() { other.build_args.clone() } else { self.build_args.clone() },
            run_args: if !other.run_args.is_empty() { other.run_args.clone() } else { self.run_args.clone() },
            payload: other.payload.clone().or_else(|| self.payload.clone()),
            code_file: if !other.code_file.is_empty() { other.code_file.clone() } else { self.code_file.clone() },
            ignore: other.ignore || self.ignore,
        }
    }
    pub fn default() -> RouteSettings {
        RouteSettings {
            build_command: "".to_string(),
            build_output_path: "".to_string(),
            run_command: "".to_string(),
            run_args: Vec::new(),
            build_args: Vec::new(),
            payload: None,
            code_file: "".to_string(), //TODO: Actually implement this
            ignore: false,
        }
    }
}