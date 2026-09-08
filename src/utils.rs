use std::{fs, path::{Path, PathBuf}};

use regex::Regex;
use walkdir::WalkDir;

pub fn find_file_by_regex(dir: &Path, pattern: &str) -> Option<std::path::PathBuf> {
    let re = Regex::new(pattern).unwrap();
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        if entry.file_type().unwrap().is_file() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if re.is_match(&file_name) {
                return Some(entry.path());
            }
        }
    }

    return None;
}

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
}

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