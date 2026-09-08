use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")] 
pub struct RouteSettings{
    pub build_command: String,
    pub build_output_path: String,
    pub run_command: String,
    pub build_args: Option<Vec<String>>,
    pub run_args: Option<Vec<String>>,
}

impl RouteSettings {
    pub fn merge(&self, other: &RouteSettings) -> RouteSettings {
        RouteSettings {
            build_command: if !other.build_command.is_empty() { other.build_command.clone() } else { self.build_command.clone() },
            build_output_path: if !other.build_output_path.is_empty() { other.build_output_path.clone() } else { self.build_output_path.clone() },
            run_command: if !other.run_command.is_empty() { other.run_command.clone() } else { self.run_command.clone() },
            run_args: if let Some(other_args) = &other.run_args {
                Some(other_args.clone())
            } else {
                self.run_args.clone()
            },
            build_args: if let Some(other_args) = &other.build_args {
                Some(other_args.clone())
            } else {
                self.build_args.clone()
            },
        }
    }
    pub fn default() -> RouteSettings {
        RouteSettings {
            build_command: "".to_string(),
            build_output_path: "".to_string(),
            run_command: "".to_string(),
            run_args: None,
            build_args: None,
        }
    }
}