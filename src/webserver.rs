use std::{fs, println};
use std::process::Stdio;
use axum::body::to_bytes;
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
use crate::structs::routesettings::RouteSettings;


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
struct RouterResponse {
    response_code: u16,
    data: String,
    response_type: ResponseType,
}



fn interpret_response(response: &str) -> RouterResponse {
    serde_json::from_str::<RouterResponse>(response).unwrap_or_else(|_| {
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

async fn spawn_route_process<'a>(path: &str, processes: &'a mut tokio::sync::MutexGuard<'_, HashMap<String, AsyncProcess>>) -> Option<&'a mut AsyncProcess> {
    
    if let std::collections::hash_map::Entry::Vacant(entry) = processes.entry(path.to_string()) {
        // The output folder
        let root_output_path = std::path::absolute(&*ROOT_PATH.lock().unwrap()).unwrap(); 

        // Gets the directory where the process is in
        let web_path = std::path::Path::new(&path).strip_prefix("/").unwrap_or(std::path::Path::new(&path));
        let process_dir = root_output_path.join(&web_path);

        // Gets the actual process
        let process_path = utils::find_file_by_regex(Path::new(&process_dir), "main*");


        // Gets the route settings
        let route_settings_path = process_dir.join("settings.json");
        let settings_content = fs::read_to_string(&route_settings_path).unwrap_or_else(|_| "{}".to_string());
        let route_settings = serde_json::from_str::<RouteSettings>(&settings_content).unwrap_or(RouteSettings::default());

        println!("Route settings for {}: {:?}", path, route_settings);
        
        if process_path.as_ref().is_none() || process_path.as_ref().expect("Process path is None").exists() == false {
            return None;
        }

        println!("Process path for {}: {:?}", path, process_path);

        let mut final_args: Vec<&str> = Vec::new();

        match route_settings.run_args.as_ref() {
            Some(args) => {
                for arg in args {
                    if arg == "<<file>>" {
                        final_args.push(process_path.as_ref().expect("Process path is None").to_str().unwrap());
                    } else if arg == "<<folder>>" {
                        final_args.push(process_dir.to_str().unwrap());
                    } else {
                        final_args.push(arg);
                    }
                }
            },
            None => {}
        }

        let final_route_command = if route_settings.run_command.contains("<<file>>") {
            println!("Replacing <<file>> in run command with: {:?}", process_path.as_ref().expect("Process path is None").to_str().unwrap());
            //TODO: Make other ones do replace subtext instead of full string
            &route_settings.run_command.replace("<<file>>", process_path.as_ref().expect("Process path is None").to_str().unwrap())
        } else {
            &route_settings.run_command
        };

        println!("Spawning process for path: {} with command: {} and args: {:?}", path, final_route_command, final_args);
        let new_process = AsyncProcess::new(final_route_command, &final_args).unwrap();
        entry.insert(new_process);
    }
    return Some(processes.get_mut(path).unwrap());
}

static ROOT_PATH: LazyLock<Mutex<PathBuf>> = LazyLock::new(|| Mutex::new(PathBuf::new()));

async fn handle_call(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    
    let bytes = to_bytes(req.into_body(), 10 * 1024 * 1024).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();

    let mut processes = ASYNC_GLOBAL_MAP.lock().await;
    
    let current_process = spawn_route_process(&path, &mut processes).await.expect("No process was made!");
    let _ = current_process.write(&(body + "\n")).await;
    let result = current_process.read().await.unwrap();
    let response = interpret_response(&result);

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