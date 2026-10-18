use std::{fs, println};
use axum::body::{to_bytes};
use std::sync::{LazyLock};
use std::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;
use tera::Tera;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{any},
    Router,
    response::{Html, IntoResponse, Json},
};

use std::path::{Path, PathBuf};

use crate::utils::{self, get_folder_by_regex};
use crate::structs::routesettings::{RouteSettings};
use crate::structs::routerequest::{RouteRequest};
use crate::structs::asyncprocess::AsyncProcess;
use crate::structs::routeresponse::{RouteResponse, ResponseType};
use crate::structs::templatedata::{TemplateData};

static ASYNC_GLOBAL_MAP: LazyLock<TokioMutex<HashMap<String, AsyncProcess>>> = LazyLock::new(|| {
    TokioMutex::new(HashMap::new())
});





fn interpret_response(response: &str) -> RouteResponse {
    println!("Interpreting response: {}", response);
    serde_json::from_str::<RouteResponse>(response).unwrap_or_else(|_| {
        println!("Failed to parse response: {}", response);
        // If parsing fails, return a default RouterResponse
        RouteResponse {
            response_code: 500,
            data: "Failed to parse response".to_string(),
            response_type: ResponseType::TEXT,
        }
    })
}

fn convert_router_response(response: &RouteResponse) -> axum::response::Response{

    let response_data = response.data.clone();

    if response.response_code > 399 {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error from process: {}", response.data)).into_response();
    } else if response.response_code > 299 {
        let status = StatusCode::from_u16(response.response_code).unwrap_or(StatusCode::OK);
        return (status, response_data).into_response();
    }

    match response.response_type {
        ResponseType::FILE => return handle_file_response(response),
        ResponseType::HTMLFILE => return handle_html_file_response(response),
        ResponseType::JSON => return (StatusCode::OK, Json(response_data)).into_response(),
        ResponseType::TEXT => return (StatusCode::OK, response_data).into_response(),
        ResponseType::TEMPLATE => return handle_template_response(response),
    }
}

fn handle_file_response(response: &RouteResponse) -> axum::response::Response {
    let contents = fs::read_to_string(&response.data).unwrap_or_else(|_| "".to_string());
    if contents.is_empty() {
        (StatusCode::NOT_FOUND, format!("File not found: {}", response.data)).into_response()
    } else {
        (StatusCode::OK, contents).into_response()
    }
}

fn handle_html_file_response(response: &RouteResponse) -> axum::response::Response {
    let contents = fs::read_to_string(&response.data).unwrap_or_else(|_| "".to_string());
    if contents.is_empty() {
        (StatusCode::NOT_FOUND, format!("File not found: {}", response.data)).into_response()
    } else {
        (StatusCode::OK, Html(contents)).into_response()
    }
}

fn handle_template_response(response: &RouteResponse) -> axum::response::Response {
    let template_data: TemplateData = serde_json::from_str(&response.data).unwrap_or_else(|_| {
        println!("Failed to parse template data: {}", response.data);
        // If parsing fails, return a default TemplateData
        TemplateData {
            template_path: "".to_string(),
            arguments: std::collections::HashMap::new(),
        }
    });

    let contents = fs::read_to_string(&template_data.template_path).unwrap_or_else(|_| "".to_string());
    if contents.is_empty() {
        (StatusCode::NOT_FOUND, format!("Template file not found: {}", &template_data.template_path)).into_response()
    } else {
        let mut tera = Tera::default();
        let title = "base_template";
        tera.add_raw_template(title, &contents).unwrap();
        let mut context = tera::Context::new();
        println!("Template arguments: {:?}", template_data.arguments);
        for (key, value) in &template_data.arguments {
            context.insert(key.clone(), &value);
        }

        let rendered = tera.render(title, &context).unwrap_or_else(|_| "".to_string());
        if rendered.is_empty() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render template".to_string()).into_response();
        }
        (StatusCode::OK, Html(rendered)).into_response()
    }
}

fn get_route_settings(path: &str) -> RouteSettings {
    let root_output_path = std::path::absolute(&*ROOT_PATH.lock().unwrap()).unwrap(); 
    let web_path = std::path::Path::new(&path).strip_prefix("/").unwrap_or(std::path::Path::new(&path));
    let process_dir = get_folder_path(web_path.to_str().unwrap(), &root_output_path).unwrap_or(root_output_path.join(&web_path));
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
 
fn get_folder_path(web_path: &str, root_path: &PathBuf) -> Option<PathBuf> {
    let mut accumulated_path = root_path.to_str().unwrap_or("").to_string();
    for segment in web_path.split("/").filter(|s| !s.is_empty()) {
        let accumulated_as_path = Path::new(&accumulated_path);
        let regular_path = get_folder_by_regex(accumulated_as_path, segment);
        match regular_path {
            Some(path) => {
                accumulated_path = path.to_str().unwrap_or("").to_string();
            },
            None => {
                let id_path = get_folder_by_regex(accumulated_as_path, r"\[.*\]");
                match id_path {
                    Some(path) => {
                        accumulated_path = path.to_str().unwrap_or("").to_string();
                    },
                    None => {
                        return None;
                    }
                }
            }
        }
    }
    return Some(PathBuf::from(accumulated_path));
}

fn get_supplied_values(original_path: &PathBuf, actual_path: &PathBuf) -> HashMap<String, String> {
    let mut supplied_values = HashMap::new();
    let original_segments: Vec<&str> = original_path.components()
        .map(|comp| comp.as_os_str().to_str().unwrap_or(""))
        .rev()
        .collect();
    let actual_segments: Vec<&str> = actual_path.components()
        .map(|comp| comp.as_os_str().to_str().unwrap_or(""))
        .rev()
        .collect();


    for (original_segment, actual_segment) in original_segments.iter().zip(actual_segments.iter()) {
        println!("Comparing original segment: {} with actual segment: {}", original_segment, actual_segment);
        if actual_segment.starts_with("[") && actual_segment.ends_with("]") {
            let key = actual_segment.trim_start_matches("[").trim_end_matches("]").to_string();
            supplied_values.insert(key, original_segment.to_string());
        }
    }

    supplied_values
}

fn get_process_path(path: &str, process_dir: &PathBuf) -> Option<PathBuf> {
    let route_settings = get_route_settings(path);
    let pattern = if route_settings.process_path.is_empty() { "main*".to_string() } else { utils::replace_path(&route_settings.process_path, &HashMap::from([("<<folder>>", process_dir.to_str().unwrap())]))  };
    println!("Looking for process matching pattern: {} in directory: {}", pattern, process_dir.to_str().unwrap());
    utils::find_file_by_regex(Path::new(&process_dir), &pattern)
}

async fn spawn_route_process<'a>(path: &str, process_path: &PathBuf, process_dir: &PathBuf, processes: &'a mut tokio::sync::MutexGuard<'_, HashMap<String, AsyncProcess>>) -> Option<&'a mut AsyncProcess> {
    
    if let std::collections::hash_map::Entry::Vacant(entry) = processes.entry(path.to_string()) {
        // Get route settings
        let route_settings = get_route_settings(path);

        let mut final_args: Vec<String> = Vec::new();

        if !route_settings.run_args.is_empty() {
            for arg in &route_settings.run_args {
                let mut formatted_arg = arg.to_string();
                formatted_arg = utils::replace_path(&formatted_arg, &HashMap::from([
                    ("<<file>>", process_path.to_str().unwrap()),
                    ("<<folder>>", process_dir.to_str().unwrap()),
                ]));
                final_args.push(formatted_arg);
            }

        }

        let mut final_route_command = route_settings.run_command.clone();
        
        final_route_command = utils::replace_path(&final_route_command, &HashMap::from([
            ("<<file>>", process_path.to_str().unwrap()),
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

fn check_json_validity(body: &str, route_settings: &RouteSettings) -> bool {
    let mut body_types: HashMap<Vec<String>, serde_json::Value> = HashMap::new();
    utils::walk_json(&route_settings.payload.as_ref().unwrap_or(&serde_json::Value::Null), vec![], &mut body_types);

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
    return is_valid;
}

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
    
    let process_folder = get_folder_path(&path, &ROOT_PATH.lock().unwrap()).unwrap_or(PathBuf::from(&path));
    let process_path = get_process_path(&path, &process_folder).expect("No process found for this route!");

    let current_process = spawn_route_process(&path, &process_path, &process_folder, &mut processes).await.expect("No process was made!");
    println!("Got the currennt process");
    let route_settings = get_route_settings(&path);
    
    let is_valid = check_json_validity(&body, &route_settings);

    let route_request = RouteRequest {
        path: path.clone(),
        body: body.clone(),
        cookies: cookies.clone(),
        method: method.clone(),
        is_valid: is_valid,
        params: get_supplied_values(&PathBuf::from(&path), &process_folder),
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