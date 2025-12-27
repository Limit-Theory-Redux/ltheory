//! Shader hot-reloading via file system watching.
//!
//! This module provides file watching for shader files, enabling hot-reloading
//! during development. When a shader file changes, affected shaders can be
//! identified and reloaded without restarting the application.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{debug, error, info, warn};

use crate::system::{Resource, ResourceType};

/// Global shader watcher instance
static SHADER_WATCHER: Mutex<Option<ShaderWatcherInner>> = Mutex::new(None);

/// Internal shader watcher state.
struct ShaderWatcherInner {
    /// The file watcher instance
    watcher: RecommendedWatcher,
    /// Receiver for file change events
    rx: Receiver<notify::Result<Event>>,
    /// Map from watched file path -> set of shader names that depend on it
    file_to_shaders: HashMap<PathBuf, HashSet<String>>,
    /// Map from shader name -> set of file paths it depends on
    shader_to_files: HashMap<String, HashSet<PathBuf>>,
    /// Set of shader names that have changed since last poll
    changed_shaders: HashSet<String>,
    /// Directories being watched
    watched_dirs: HashSet<PathBuf>,
}

impl ShaderWatcherInner {
    fn new() -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();

        let watcher = notify::recommended_watcher(move |res| {
            if let Err(e) = tx.send(res) {
                error!("Failed to send file change event: {}", e);
            }
        })?;

        Ok(Self {
            watcher,
            rx,
            file_to_shaders: HashMap::new(),
            shader_to_files: HashMap::new(),
            changed_shaders: HashSet::new(),
            watched_dirs: HashSet::new(),
        })
    }

    fn watch_shader_directories(&mut self) -> Result<(), notify::Error> {
        for folder in Resource::get_folders(ResourceType::Shader) {
            if folder.exists() && !self.watched_dirs.contains(&folder) {
                info!("ShaderWatcher: Watching directory {:?}", folder);
                self.watcher.watch(&folder, RecursiveMode::Recursive)?;
                self.watched_dirs.insert(folder);
            }
        }
        Ok(())
    }

    fn register_shader(
        &mut self,
        shader_name: &str,
        vs_path: &Path,
        fs_path: &Path,
        include_paths: &[PathBuf],
    ) {
        let name = shader_name.to_string();

        // Collect all file paths (only valid ones)
        let mut all_paths: HashSet<PathBuf> = HashSet::new();
        if let Ok(p) = vs_path.canonicalize() {
            all_paths.insert(p);
        }
        if let Ok(p) = fs_path.canonicalize() {
            all_paths.insert(p);
        }
        for p in include_paths {
            if let Ok(canonical) = p.canonicalize() {
                all_paths.insert(canonical);
            }
        }

        // Remove old mappings if shader was previously registered
        if let Some(old_paths) = self.shader_to_files.remove(&name) {
            for path in old_paths {
                if let Some(shaders) = self.file_to_shaders.get_mut(&path) {
                    shaders.remove(&name);
                    if shaders.is_empty() {
                        self.file_to_shaders.remove(&path);
                    }
                }
            }
        }

        // Add new mappings
        for path in &all_paths {
            self.file_to_shaders
                .entry(path.clone())
                .or_default()
                .insert(name.clone());
        }
        self.shader_to_files.insert(name.clone(), all_paths.clone());

        debug!(
            "ShaderWatcher: Registered shader '{}' watching {} files",
            shader_name,
            all_paths.len()
        );
    }

    fn poll_changes(&mut self) -> Vec<String> {
        // Process all pending events
        while let Ok(event_result) = self.rx.try_recv() {
            match event_result {
                Ok(event) => {
                    info!("ShaderWatcher: File event {:?} for {:?}", event.kind, event.paths);
                    if matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    ) {
                        for path in event.paths {
                            let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                            info!("ShaderWatcher: Checking path {:?} (canonical: {:?})", path, canonical);
                            if let Some(shaders) = self.file_to_shaders.get(&canonical) {
                                for shader in shaders {
                                    info!(
                                        "ShaderWatcher: >>> FILE CHANGED {:?} -> shader '{}'",
                                        canonical, shader
                                    );
                                    self.changed_shaders.insert(shader.clone());
                                }
                            } else {
                                info!("ShaderWatcher: Path {:?} not in tracked files", canonical);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("ShaderWatcher: Watch error: {:?}", e);
                }
            }
        }

        let changed: Vec<String> = self.changed_shaders.drain().collect();
        if !changed.is_empty() {
            info!("ShaderWatcher: Returning {} changed shader(s): {:?}", changed.len(), changed);
        }
        changed
    }
}

/// FFI wrapper for shader hot-reload functionality.
/// Use ShaderWatcher::Init() to start watching, then poll for changes.
pub struct ShaderWatcher;

#[luajit_ffi_gen::luajit_ffi]
impl ShaderWatcher {
    /// Initializes the global shader watcher.
    /// Call this once at startup to enable shader hot-reloading.
    #[bind(name = "Init")]
    pub fn init() -> bool {
        let mut guard = SHADER_WATCHER.lock().unwrap();
        if guard.is_some() {
            warn!("ShaderWatcher already initialized");
            return true;
        }

        match ShaderWatcherInner::new() {
            Ok(mut watcher) => {
                if let Err(e) = watcher.watch_shader_directories() {
                    error!("Failed to watch shader directories: {}", e);
                    return false;
                }
                *guard = Some(watcher);
                info!("ShaderWatcher initialized successfully");
                true
            }
            Err(e) => {
                error!("Failed to create ShaderWatcher: {}", e);
                false
            }
        }
    }

    /// Shuts down the global shader watcher.
    #[bind(name = "Shutdown")]
    pub fn shutdown() {
        let mut guard = SHADER_WATCHER.lock().unwrap();
        if guard.take().is_some() {
            info!("ShaderWatcher shutdown");
        }
    }

    /// Checks if shader watcher is active.
    #[bind(name = "IsActive")]
    pub fn is_active() -> bool {
        SHADER_WATCHER.lock().unwrap().is_some()
    }

    /// Registers a shader for hot-reload tracking.
    ///
    /// # Arguments
    /// * `shader_key` - The shader cache key (format: "vs_name:fs_name")
    /// * `vs_path` - Resolved path to vertex shader file
    /// * `fs_path` - Resolved path to fragment shader file
    #[bind(name = "Register")]
    pub fn register(shader_key: &str, vs_path: &str, fs_path: &str) {
        let mut guard = SHADER_WATCHER.lock().unwrap();
        if let Some(watcher) = guard.as_mut() {
            let vs = PathBuf::from(vs_path);
            let fs = PathBuf::from(fs_path);

            let includes = collect_shader_includes(&vs, &fs);
            watcher.register_shader(shader_key, &vs, &fs, &includes);
        }
    }

    /// Polls for changed shaders and returns count.
    /// Use GetChanged() to get the actual shader keys.
    #[bind(name = "Poll")]
    pub fn poll() -> i32 {
        let mut guard = SHADER_WATCHER.lock().unwrap();
        if let Some(watcher) = guard.as_mut() {
            let changed = watcher.poll_changes();
            watcher.changed_shaders = changed.into_iter().collect();
            watcher.changed_shaders.len() as i32
        } else {
            0
        }
    }

    /// Gets a changed shader key by index (0-based).
    /// Call Poll() first to get the count.
    #[bind(name = "GetChanged")]
    pub fn get_changed(index: i32) -> Option<String> {
        let guard = SHADER_WATCHER.lock().unwrap();
        if let Some(watcher) = guard.as_ref() {
            watcher.changed_shaders.iter().nth(index as usize).cloned()
        } else {
            None
        }
    }

    /// Clears the list of changed shaders after processing.
    #[bind(name = "ClearChanged")]
    pub fn clear_changed() {
        let mut guard = SHADER_WATCHER.lock().unwrap();
        if let Some(watcher) = guard.as_mut() {
            watcher.changed_shaders.clear();
        }
    }
}

// Public functions for internal Rust use

/// Initializes the shader watcher (Rust API).
pub fn shader_watcher_init() -> bool {
    ShaderWatcher::init()
}

/// Checks if shader watcher is active (Rust API).
pub fn shader_watcher_is_active() -> bool {
    ShaderWatcher::is_active()
}

/// Registers a shader for hot-reload tracking (Rust API).
pub fn shader_watcher_register(shader_key: &str, vs_path: &str, fs_path: &str) {
    ShaderWatcher::register(shader_key, vs_path, fs_path);
}

/// Collects all #include paths from shader source files.
fn collect_shader_includes(vs_path: &Path, fs_path: &Path) -> Vec<PathBuf> {
    let mut includes = HashSet::new();

    fn collect_from_file(
        path: &Path,
        includes: &mut HashSet<PathBuf>,
        visited: &mut HashSet<PathBuf>,
    ) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if visited.contains(&canonical) {
            return;
        }
        visited.insert(canonical.clone());

        if let Ok(content) = std::fs::read_to_string(&canonical) {
            for line in content.lines() {
                if let Some(include_val) = line.strip_prefix("#include ") {
                    let include_name = include_val.trim();
                    // Resolve using Resource system
                    let include_resource = format!("include/{}", include_name);
                    if Resource::exists(ResourceType::Shader, &include_resource) {
                        let include_path_str =
                            Resource::get_path(ResourceType::Shader, &include_resource);
                        let include_path = PathBuf::from(&include_path_str);

                        if let Ok(canonical_include) = include_path.canonicalize() {
                            includes.insert(canonical_include.clone());
                            collect_from_file(&canonical_include, includes, visited);
                        } else {
                            includes.insert(include_path.clone());
                            collect_from_file(&include_path, includes, visited);
                        }
                    }
                }
            }
        }
    }

    let mut visited = HashSet::new();
    collect_from_file(vs_path, &mut includes, &mut visited);
    collect_from_file(fs_path, &mut includes, &mut visited);

    includes.into_iter().collect()
}
