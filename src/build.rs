use std::{collections::HashMap, path::{Path, PathBuf}};
use std::process::Command;
use std::fs;
use crate::utils;
use crate::structs::routesettings::RouteSettings;




pub fn build_code(root_path: PathBuf) {
    let route_settings: HashMap<&str, RouteSettings> = HashMap::from([
        ("py", RouteSettings{
            build_command: "".to_string(),
            build_output_path: "main.py".to_string(),
            run_command: "py".to_string(),
            build_args: vec![],
            run_args: vec!["-u".to_string(), "<<file>>".to_string()],
            payload: None,
            ignore: false,
            root_folder: "".to_string(),
            ignore_files: vec![],
            process_path: "".to_string(),
        }),
        ("js", RouteSettings{
            build_command: "".to_string(),
            build_output_path: "main.js".to_string(),
            run_command: "node".to_string(),
            build_args: vec![],
            run_args: vec!["<<file>>".to_string()],
            payload: None,
            ignore: false,
            root_folder: "".to_string(),
            ignore_files: vec![],
            process_path: "".to_string(),
        }),
        ("java", RouteSettings{
            build_command: "javac".to_string(),
            build_output_path: "".to_string(),
            run_command: "java".to_string(),
            build_args: vec!["-d".to_string(), "<<output_folder>>".to_string(), "<<source_file>>".to_string()],
            run_args: vec!["-cp".to_string(), "<<folder>>".to_string(), "main".to_string()],
            payload: None,
            ignore: false,
            root_folder: "".to_string(),
            ignore_files: vec![],
            process_path: "".to_string(),
        }),
    ]);


    let output_path = root_path.join("output");
    let source_path = root_path.join("src");
    
    if output_path.exists() {
        println!("Deleting output folder: {:?}", std::path::absolute(&output_path).unwrap());
        let _ = fs::remove_dir_all(&output_path);
    }

    let files = utils::find_files_by_regex_recursive(&source_path, r"^main\.");

    for file in files {
        println!("Found file: {:?}", &file);

        // The relative path of the file to build
        let relative_path = file.strip_prefix(&source_path).unwrap();
        let file_extension = relative_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        // The folder the file is in
        let file_folder = file.parent().unwrap_or_else(|| Path::new(""));

        // Gets the route settings provided by the user
        let route_settings_path = file_folder.join("settings.json");
        let new_route_settings = if route_settings_path.exists() {
            let settings_content = fs::read_to_string(&route_settings_path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str::<RouteSettings>(&settings_content).unwrap_or(RouteSettings::default())
        } else {
            RouteSettings::default()
        };

        // Merges the default route settings with the user-provided settings
        let route_settings = if route_settings.contains_key(file_extension) {
            route_settings.get(file_extension).unwrap().merge(&new_route_settings)
        } else {
            new_route_settings
        };

        if route_settings.ignore {
            println!("Ignoring file: {:?} due to ignore flag in settings.", &file);
            continue;
        }

        println!("Route settings for file {:?}: {:?}", &file, &route_settings);
        // Skips if no route settings are found for the file
        if route_settings == RouteSettings::default() || route_settings.run_command.is_empty() {
            println!("No route settings found for file: {:?}, skipping build.", &file);
            continue;
        }

        // Gets the file path of the build output file
        let build_output_file_name = &route_settings.build_output_path;
        let build_output_file_path = std::path::absolute(file_folder.join(&build_output_file_name)).unwrap();

        // Gets the folder the build output will go to and the file path of the build output file in the output folder
        let router_output_folder_path = std::path::absolute(output_path.join(relative_path).parent().unwrap()).unwrap();
        let router_output_file_path = std::path::absolute(router_output_folder_path.join(&build_output_file_name)).unwrap();

        

        // Creates the required output folder if it doesn't exist
        let _ = fs::create_dir_all(&router_output_folder_path);

        // Builds the file using the specified build command
        let build_command = &route_settings.build_command;

        if !build_command.is_empty() {
            let mut final_args: Vec<String> = Vec::new();
            for arg in &route_settings.build_args {
                let mut formatted_arg = arg.to_string();
                formatted_arg = utils::replace_path(&formatted_arg, &HashMap::from([
                    ("<<source_file>>", std::path::absolute(&file).unwrap().to_str().unwrap()),
                    ("<<source_folder>>", file_folder.to_str().unwrap()),
                    ("<<output_file>>", build_output_file_path.to_str().unwrap()),
                    ("<<output_folder>>", router_output_folder_path.to_str().unwrap()),
                ]));
                
                final_args.push(formatted_arg);
            }
            println!("Building file: {:?} with command: {:?} in parent {:?} with arguments {:?}", &file, &build_command, &file_folder.to_str().unwrap(), final_args);
            let mut build_command = Command::new(build_command)
            .args(final_args)
            .spawn()
            .expect("Failed to execute build command");

            build_command.wait().expect("Failed to wait on build process");
        }
        
        

        if !build_output_file_name.is_empty() {
            println!("Copying build output from {:?} to {:?}", &build_output_file_path, &router_output_file_path);

            // Copies the build output file to the output folder
            match fs::copy(&build_output_file_path, &router_output_file_path) {
                Ok(_) => println!("Successfully copied build output to {:?}", &router_output_file_path),
                Err(e) => eprintln!("Failed to copy build output: {}", e),
            }
        }

        // Copies any other files in the folder that aren't the main file to the output folder
        let other_files = utils::find_files_by_regex_inverse(&file_folder, r"^main\.");
        for other in other_files{
            // TODO: Ensure this works
            if route_settings.ignore_files.contains(&other.file_name().unwrap().to_str().unwrap().to_string()) || other.file_name().unwrap().to_str().unwrap() == "settings.json" {
                continue;
            }

            let other_file_name = other.file_name().unwrap();

            let settings_file_path = if route_settings.root_folder.is_empty() { "<<output_folder>>" } else { &route_settings.root_folder };
            let other_file_path_str = utils::replace_path(settings_file_path, &HashMap::from([
                ("<<output_folder>>", router_output_folder_path.to_str().unwrap()),
            ]));
            let other_file_path = std::path::PathBuf::from(other_file_path_str);
            let router_other_file_path = std::path::absolute(other_file_path.join(other_file_name)).unwrap();

            println!("Copying extra file from {:?} to {:?}", &other, &router_other_file_path);
            match fs::copy(&other, &router_other_file_path) {
                Ok(_) => println!("Successfully copied extra file to {:?}", &router_output_file_path),
                Err(e) => eprintln!("Failed to copy extra file: {}", e),
            }
        }


        // Writes the route settings to the output folder
        let json_string = serde_json::to_string_pretty(&route_settings);
        fs::write(router_output_folder_path.join("settings.json"), json_string.unwrap()).expect("Failed to write settings.json");

        println!("\n--------------------\n");
        // A list of commands to grab the build data and copy it into the output folder and a command to run the output file
    }
}