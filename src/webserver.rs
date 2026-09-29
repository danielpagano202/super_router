use std::{fs, println};
use std::process::Stdio;
use axum::body::{to_bytes};
use serde::{Deserialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};
use std::sync::{LazyLock};
use std::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{any},
    Router,
    response::{Html, IntoResponse, Json},
};

use std::path::{Path, PathBuf};

use crate::utils;
use crate::structs::routesettings::{RouteSettings};
use crate::structs::routerequest::{RouteRequest};


static ASYNC_GLOBAL_MAP: LazyLock<TokioMutex<HashMap<String, AsyncProcess>>> = LazyLock::new(|| {
    TokioMutex::new(HashMap::new())
});

struct AsyncProcess {
    //child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl AsyncProcess {
    fn new(command: &str, args: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let input = child.stdin.take().ok_or("Failed to open stdin")?;
        let raw_stdout = child.stdout.take().ok_or("Failed to open stdout")?;

        let output = tokio::io::BufReader::new(raw_stdout); 

        Ok(AsyncProcess { input, output })
    }

    async fn write(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.input.write_all(data.as_bytes()).await?;
        self.input.flush().await?; // 2. Added flush so the child process actually receives it
        Ok(())
    }

    async fn read(&mut self) -> Result<String, std::io::Error> {
        let mut response_line = String::new();
        self.output.read_line(&mut response_line).await?;
        Ok(response_line) // 3. Removed trailing semicolon so it correctly returns the value
    }
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ResponseType {
    JSON,
    TEXT,
    FILE,
    HTMLFILE,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouterResponse {
    response_code: u16,
    data: String,
    response_type: ResponseType,
}



fn interpret_response(response: &str) -> RouterResponse {
    println!("Interpreting response: {}", response);
    serde_json::from_str::<RouterResponse>(response).unwrap_or_else(|_| {
        println!("Failed to parse response: {}", response);
        // If parsing fails, return a default RouterResponse
        RouterResponse {
            response_code: 500,
            data: "Failed to parse response".to_string(),
            response_type: ResponseType::TEXT,
        }
    })
}

fn convert_router_response(response: &RouterResponse) -> axum::response::Response{

    let response_data = response.data.clone();

    if response.response_code > 399 {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error from process: {}", response.data)).into_response();
    } else if response.response_code > 299 {
        let status = StatusCode::from_u16(response.response_code).unwrap_or(StatusCode::OK);
        return (status, response_data).into_response();
    }

    if response.response_type == ResponseType::FILE {
        return handle_file_response(&response);
    } else if response.response_type == ResponseType::HTMLFILE  {
        return handle_html_file_response(&response);
    } else if response.response_type == ResponseType::JSON {
        return (StatusCode::OK, Json(response_data)).into_response();
    } else if response.response_type == ResponseType::TEXT {
        return (StatusCode::OK, response_data).into_response();
    }
    (StatusCode::NOT_FOUND, "Route invalid").into_response()
}

fn handle_file_response(response: &RouterResponse) -> axum::response::Response {
    let contents = fs::read_to_string(&response.data).unwrap_or_else(|_| "".to_string());
    if contents.is_empty() {
        (StatusCode::NOT_FOUND, format!("File not found: {}", response.data)).into_response()
    } else {
        (StatusCode::OK, contents).into_response()
    }
}

fn handle_html_file_response(response: &RouterResponse) -> axum::response::Response {
    let contents = fs::read_to_string(&response.data).unwrap_or_else(|_| "".to_string());
    if contents.is_empty() {
        (StatusCode::NOT_FOUND, format!("File not found: {}", response.data)).into_response()
    } else {
        (StatusCode::OK, Html(contents)).into_response()
    }
}

fn get_route_settings(path: &str) -> RouteSettings {
    let root_output_path = std::path::absolute(&*ROOT_PATH.lock().unwrap()).unwrap(); 
    let web_path = std::path::Path::new(&path).strip_prefix("/").unwrap_or(std::path::Path::new(&path));
    let process_dir = root_output_path.join(&web_path);
    let route_settings_path = process_dir.join("settings.json");
    let settings_content = fs::read_to_string(&route_settings_path).unwrap_or_else(|_| "{}".to_string());
    println!("Settings content for path {}: {}", path, settings_content);
    serde_json::from_str::<RouteSettings>(&settings_content).unwrap_or(RouteSettings::default())
}

fn match_json(v_type: &str, value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(_) => v_type.contains("string"),
        serde_json::Value::Number(_) => v_type.contains("number"),
        serde_json::Value::Bool(_) => v_type.contains("bool"),
        serde_json::Value::Null => v_type.contains("?") || v_type == "null",
        _ => false,
    }
}

async fn spawn_route_process<'a>(path: &str, processes: &'a mut tokio::sync::MutexGuard<'_, HashMap<String, AsyncProcess>>) -> Option<&'a mut AsyncProcess> {
    
    if let std::collections::hash_map::Entry::Vacant(entry) = processes.entry(path.to_string()) {
        // Get route settings
        let route_settings = get_route_settings(path);

        println!("Route settings for {}: {:?}", path, route_settings);
       
        // The output folder
        let root_output_path = std::path::absolute(&*ROOT_PATH.lock().unwrap()).unwrap(); 

        // Gets the directory where the process is in
        let web_path = std::path::Path::new(&path).strip_prefix("/").unwrap_or(std::path::Path::new(&path));
        let process_dir = if !route_settings.root_folder.is_empty() {
            PathBuf::from(utils::replace_path(&route_settings.root_folder, &HashMap::from([("<<output_folder>>", root_output_path.join(&web_path).to_str().unwrap())])))
        } else {
            root_output_path.join(&web_path)
        };



        // Gets the actual process
        let pattern = if route_settings.process_path.is_empty() { "main*".to_string() } else { utils::replace_path(&route_settings.process_path, &HashMap::from([("<<folder>>", process_dir.to_str().unwrap())]))  };
        println!("Looking for process matching pattern: {} in directory: {}", pattern, process_dir.to_str().unwrap());
        let process_path = utils::find_file_by_regex(Path::new(&process_dir), &pattern);
        println!("Found process path: {:?}", process_path);
        let mut final_args: Vec<String> = Vec::new();

        if !route_settings.run_args.is_empty() {
            println!("Is not empty");
            for arg in &route_settings.run_args {
                println!("Processing arg: {}", arg);
                let mut formatted_arg = arg.to_string();
                formatted_arg = utils::replace_path(&formatted_arg, &HashMap::from([
                    ("<<file>>", process_path.as_ref().expect("Process path is None").to_str().unwrap()),
                    ("<<folder>>", process_dir.to_str().unwrap()),
                ]));
                final_args.push(formatted_arg);
            }

        }

        let mut final_route_command = route_settings.run_command.clone();
        
        final_route_command = utils::replace_path(&final_route_command, &HashMap::from([
            ("<<file>>", process_path.as_ref().expect("Process path is None").to_str().unwrap()),
            ("<<folder>>", process_dir.to_str().unwrap()),
        ]));

        let final_args_pointers: Vec<&str> = final_args.iter().map(|s| s.as_str()).collect();
        println!("Spawning process for path: {} with command: {} and args: {:?}", path, final_route_command, final_args);
        let new_process = AsyncProcess::new(&final_route_command, &final_args_pointers).unwrap();
        entry.insert(new_process);
    }
    return Some(processes.get_mut(path).unwrap());
}

static ROOT_PATH: LazyLock<Mutex<PathBuf>> = LazyLock::new(|| Mutex::new(PathBuf::new()));

async fn handle_call(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let cookies = match req.headers().get("cookie"){
        Some(cookies) => {
            cookies.to_str().unwrap_or("").to_string()
        },
        None => "".to_string(),
    };

    let bytes = to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();

    let mut processes = ASYNC_GLOBAL_MAP.lock().await;
    
    let current_process = spawn_route_process(&path, &mut processes).await.expect("No process was made!");
    println!("Got the currennt process");
    let route_settings = get_route_settings(&path);
    
    let mut body_types: HashMap<Vec<String>, serde_json::Value> = HashMap::new();
    utils::walk_json(&route_settings.payload.unwrap_or(serde_json::Value::Null), vec![], &mut body_types);

    let body_json: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
    let mut actual_body_types: HashMap<Vec<String>, serde_json::Value> = HashMap::new();
    utils::walk_json(&body_json, vec![], &mut actual_body_types);

    let mut is_valid = true;
    for b_type in body_types.keys() {
        match body_types.get(b_type) {
            Some(expected_value) => {
                match actual_body_types.get(b_type) {
                    Some(actual_value) => {
                        is_valid = match_json(&expected_value.to_string(), actual_value);
                        if !is_valid {
                            break;
                        }
                    },
                    None => {
                        is_valid = false;
                    }
                }
            },
            None => {
                is_valid = false;
            }
        }
        if !actual_body_types.contains_key(b_type) {
            is_valid = false;
        }
    }

    let route_request = RouteRequest {
        path: path.clone(),
        body: body.clone(),
        cookies: cookies.clone(),
        method: method.clone(),
        is_valid: is_valid,
    };
    println!("Route request for path {}: {:?}", path, route_request);
    let _ = current_process.write(&(serde_json::to_string(&route_request).unwrap().to_string() + "\n")).await;

    println!("Wrote data");

    let result = current_process.read().await.unwrap_or("".to_string());

    println!("Read data: {}", result);

    if result.is_empty() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "No response from process").into_response();
    }

    let response = interpret_response(&result);
    println!("Interpreted response");
    return convert_router_response(&response)

}

pub fn start_server(root_path: PathBuf) {

    let source_path = root_path.join("output");

    *ROOT_PATH.lock().unwrap() = source_path;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        
        let app = Router::new()
            .route("/", any(
                handle_call
            ))
            // The asterisk wildcard remains the same
            .route("/{*path}", any(
                handle_call
            ));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
        println!("Server running on http://127.0.0.1:3000");
        axum::serve(listener, app).await.unwrap();
    });
}