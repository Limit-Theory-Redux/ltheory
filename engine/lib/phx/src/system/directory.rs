use std::io::ErrorKind;
use std::{env, fs};

use directories::ProjectDirs;
use internal::ConvertIntoString;
use tracing::error;

#[repr(C)]
pub struct Directory {
    pub iterator: fs::ReadDir,
}

#[luajit_ffi_gen::luajit_ffi]
impl Directory {
    pub fn open(path: &str) -> Option<Self> {
        match fs::read_dir(path) {
            Ok(dir) => Some(Self { iterator: dir }),
            Err(err) => {
                error!("Cannot open directory: {path}. Error: {err}");

                None
            }
        }
    }

    pub fn get_next(&mut self) -> Option<String> {
        match self.iterator.next() {
            Some(Ok(dir)) => dir.file_name().to_str().map(|s| s.to_string()),
            Some(Err(err)) => {
                error!("Cannot get next item in the folder. Error: {err}");
                None
            }
            None => None,
        }
    }

    pub fn change(cwd: &str) -> bool {
        match env::set_current_dir(cwd) {
            Ok(_) => true,
            Err(err) => {
                error!("Cannot change current directory. Error: {err}");
                false
            }
        }
    }

    pub fn create(path: &str) -> bool {
        match fs::create_dir(path) {
            Ok(_) => true,
            Err(err) => match err.kind() {
                ErrorKind::AlreadyExists => true,
                _ => {
                    error!("Failed to create directory. Error: {err}");
                    false
                }
            },
        }
    }

    pub fn get_current() -> Option<String> {
        match env::current_dir() {
            Ok(path) => match path.to_str() {
                Some(path_str) => Some(path_str.into()),
                None => None,
            },
            Err(err) => {
                error!("Cannot get current directory. Error: {err}");
                None
            }
        }
    }

    pub fn get_pref_path(org: &str, app: &str) -> Option<String> {
        if let Some(proj_dirs) = ProjectDirs::from("", org, app) {
            let path = proj_dirs.data_dir();

            if let Err(err) = std::fs::create_dir_all(&path) {
                error!("Cannot create project dir: {path:?}. Error: {err}");
                None
            } else if let Some(path_str) = path.to_str() {
                Some(format!("{path_str}/"))
            } else {
                error!("Path is not valid UTF-8: {path:?}");
                None
            }
        } else {
            error!("Cannot get project directory.");
            None
        }
    }

    pub fn remove(path: &str) -> bool {
        match fs::remove_dir(path) {
            Ok(_) => true,
            Err(err) => {
                error!("Cannot remove directory. Error: {err}");
                false
            }
        }
    }
}
