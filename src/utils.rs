use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use regex::Regex;
use walkdir::WalkDir;

use serde_json::Value;

pub fn find_file_by_regex(dir: &Path, pattern: &str) -> Option<std::path::PathBuf> {
    let re: Regex = match Regex::new(pattern) {
        // 1. Unwraps the successfully compiled Regex
        Ok(code) => code, 
        
        // 2. Handles the error and falls back to a default valid Regex
        Err(error) => {
            println!("Failed to compile regex pattern: {}, falling back to default.", error);
            Regex::new("main.*").unwrap() 
        }
    };
    let entries = fs::read_dir(dir).expect("Error opening directory");
    for entry in entries.flatten() {
        println!("Found entry: {:?}", entry.path());
        if entry.file_type().unwrap().is_file() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            println!("Checking file: {}", file_name);
            if re.is_match(&file_name) {
                return Some(entry.path());
            }
        }
    }

    return None;
}

/*
pub fn find_files_by_regex(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    let re = Regex::new(pattern).unwrap();
    let mut files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(dir).ok();

    for entry in entries.into_iter().flatten() {
        let entry_unwrap = entry.unwrap_or_else(|e| panic!("Failed to read directory entry: {}", e));
        if *(&entry_unwrap.file_type().unwrap().is_file()) {
            let file_name = &entry_unwrap.file_name().to_string_lossy().into_owned();
            if re.is_match(&file_name) {
                files.push(PathBuf::from(&entry_unwrap.path()))
            }
        }
    }

    return files;
}*/

pub fn find_files_by_regex_recursive(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    let re = Regex::new(pattern).unwrap();
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if re.is_match(&file_name) {
                files.push(PathBuf::from(entry.path()))
            }
        }
    }

    return files;
}

pub fn find_files_by_regex_inverse(dir: &Path, pattern: &str) -> Vec<PathBuf> {
    let re = Regex::new(pattern).unwrap();
    let mut files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(dir).ok();

    for entry in entries.into_iter().flatten() {
        let entry_unwrap = entry.unwrap_or_else(|e| panic!("Failed to read directory entry: {}", e));
        if *(&entry_unwrap.file_type().unwrap().is_file()) {
            let file_name = &entry_unwrap.file_name().to_string_lossy().into_owned();
            if !re.is_match(&file_name) {
                files.push(PathBuf::from(&entry_unwrap.path()))
            }
        }
    }

    return files;
}

pub fn walk_json(
    value: &Value,
    previous: Vec<String>,
    result: &mut HashMap<Vec<String>, Value>,
) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let mut current = previous.clone();
                current.push(key.clone());

                walk_json(value, current, result);
            }
        }

        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                let mut current = previous.clone();
                current.push(index.to_string());

                walk_json(value, current, result);
            }
        }

        // We've reached an actual value
        Value::String(_) |
        Value::Number(_) |
        Value::Bool(_) |
        Value::Null => {
            result.insert(previous, value.clone());
        }
    }
}

pub fn replace_path(path: &str, replacements: &HashMap<&str, &str>) -> String {
    let mut result = path.to_string();
    for (key, value) in replacements {
        result = result.replace(key, value);
    }
    result
}