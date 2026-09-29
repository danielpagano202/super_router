use serde::Serialize;
use std::env;
use std::io::{self, BufRead, Write};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseData {
    response_type: &'static str,
    response_code: u16,
    data: String,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Loop through each line coming from stdin
    for line in stdin.lock().lines() {
        // Suppress errors during line reading to mimic Python's try-except block
        if let Ok(_input_line) = line {
            match process_response() {
                Ok(json_response) => {
                    // Print the JSON line and force flush it immediately
                    if writeln!(handle, "{}", json_response).is_ok() {
                        let _ = handle.flush();
                    }
                }
                Err(e) => {
                    // If anything breaks, print to stdout so the reader can see the error
                    if writeln!(handle, "ERROR: {}", e).is_ok() {
                        let _ = handle.flush();
                    }
                }
            }
        }
    }
}

fn process_response() -> Result<String, Box<dyn std::error::Error>> {
    // 1. Get the directory where this executable lives and join "index.html"
    let mut exe_dir = env::current_exe()?;
    exe_dir.pop(); // Removes the binary name to get the directory
    let html_path = exe_dir.join("index.html");

    // Convert path to a lossy string representation
    let html_path_str = html_path.to_string_lossy().into_owned();

    // 2. Build the data structure
    let data = ResponseData {
        response_type: "htmlfile",
        response_code: 200,
        data: html_path_str,
    };

    // 3. Convert structure to a clean single-line JSON string
    let json_response = serde_json::to_string(&data)?;
    Ok(json_response)
}
