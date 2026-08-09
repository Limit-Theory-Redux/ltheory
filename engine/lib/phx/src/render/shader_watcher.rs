//! Shader hot-reloading via file system watching.
//!
//! Watches shader source directories for changes and identifies which
//! cached shaders (by their `vs:fs` key) are affected, including through
//! `#include`d files, so Lua can reload them without a manual trigger.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{debug, error, info, warn};

use crate::render::Renderer;
use crate::system::{Resource, ResourceType};

/// Owned by `RendererData` (see `render/thread/renderer_data.rs`) and reached
/// via `r.data.shader_watcher` - `None` until `ShaderWatcher::Init` runs.
/// No `static`/global: `RendererData` is only ever reached through `&mut
/// Renderer`, so plain field access provides the exclusivity a `Mutex`
/// would otherwise be needed for.
pub struct ShaderWatcherInner {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    /// Map from watched file path -> set of shader keys that depend on it.
    file_to_shaders: HashMap<PathBuf, HashSet<String>>,
    /// Map from shader key -> set of file paths it depends on.
    shader_to_files: HashMap<String, HashSet<PathBuf>>,
    /// Set of shader keys that have changed since last poll.
    changed_shaders: HashSet<String>,
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
        shader_key: &str,
        vs_path: &Path,
        fs_path: &Path,
        include_paths: &[PathBuf],
    ) {
        let key = shader_key.to_string();

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

        // Remove old mappings if this shader was previously registered.
        if let Some(old_paths) = self.shader_to_files.remove(&key) {
            for path in old_paths {
                if let Some(shaders) = self.file_to_shaders.get_mut(&path) {
                    shaders.remove(&key);
                    if shaders.is_empty() {
                        self.file_to_shaders.remove(&path);
                    }
                }
            }
        }

        for path in &all_paths {
            self.file_to_shaders
                .entry(path.clone())
                .or_default()
                .insert(key.clone());
        }
        self.shader_to_files.insert(key, all_paths.clone());

        debug!(
            "ShaderWatcher: Registered shader '{}' watching {} files",
            shader_key,
            all_paths.len()
        );
    }

    fn poll_changes(&mut self) -> Vec<String> {
        while let Ok(event_result) = self.rx.try_recv() {
            match event_result {
                Ok(event) => {
                    if matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    ) {
                        for path in event.paths {
                            let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                            if let Some(shaders) = self.file_to_shaders.get(&canonical) {
                                for shader in shaders {
                                    debug!(
                                        "ShaderWatcher: file changed {:?} -> shader '{}'",
                                        canonical, shader
                                    );
                                    self.changed_shaders.insert(shader.clone());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("ShaderWatcher: Watch error: {:?}", e);
                }
            }
        }

        self.changed_shaders.drain().collect()
    }
}

/// FFI wrapper for shader hot-reload functionality.
/// Use `ShaderWatcher.Init()` to start watching, then poll for changes.
pub struct ShaderWatcher;

#[luajit_ffi_gen::luajit_ffi]
impl ShaderWatcher {
    /// Initializes the shader watcher. Call this once at startup to enable
    /// shader hot-reloading.
    pub fn init(r: &mut Renderer) -> bool {
        if r.data.shader_watcher.is_some() {
            warn!("ShaderWatcher already initialized");
            return true;
        }

        match ShaderWatcherInner::new() {
            Ok(mut watcher) => {
                if let Err(e) = watcher.watch_shader_directories() {
                    error!("Failed to watch shader directories: {}", e);
                    return false;
                }
                r.data.shader_watcher = Some(watcher);
                info!("ShaderWatcher initialized successfully");
                true
            }
            Err(e) => {
                error!("Failed to create ShaderWatcher: {}", e);
                false
            }
        }
    }

    /// Shuts down the shader watcher.
    pub fn shutdown(r: &mut Renderer) {
        if r.data.shader_watcher.take().is_some() {
            info!("ShaderWatcher shutdown");
        }
    }

    /// Checks whether the shader watcher is active.
    pub fn is_active(r: &Renderer) -> bool {
        r.data.shader_watcher.is_some()
    }

    /// Registers a shader for hot-reload tracking.
    ///
    /// * `shader_key` - The shader cache key (format: "vs_name:fs_name")
    /// * `vs_path` - Resolved path to the vertex shader file
    /// * `fs_path` - Resolved path to the fragment shader file
    pub fn register(r: &mut Renderer, shader_key: &str, vs_path: &str, fs_path: &str) {
        if let Some(watcher) = r.data.shader_watcher.as_mut() {
            let vs = PathBuf::from(vs_path);
            let fs = PathBuf::from(fs_path);
            let includes = collect_shader_includes(&vs, &fs);
            watcher.register_shader(shader_key, &vs, &fs, &includes);
        }
    }

    /// Polls for changed shaders and returns the count.
    /// Use `GetChanged` to get the actual shader keys.
    pub fn poll(r: &mut Renderer) -> i32 {
        if let Some(watcher) = r.data.shader_watcher.as_mut() {
            let changed = watcher.poll_changes();
            watcher.changed_shaders = changed.into_iter().collect();
            watcher.changed_shaders.len() as i32
        } else {
            0
        }
    }

    /// Gets a changed shader key by index (0-based). Call `Poll` first.
    pub fn get_changed(r: &Renderer, index: i32) -> Option<String> {
        r.data
            .shader_watcher
            .as_ref()
            .and_then(|w| w.changed_shaders.iter().nth(index as usize).cloned())
    }

    /// Clears the list of changed shaders after processing.
    pub fn clear_changed(r: &mut Renderer) {
        if let Some(watcher) = r.data.shader_watcher.as_mut() {
            watcher.changed_shaders.clear();
        }
    }
}

/// Collects all `#include`d file paths reachable from `vs_path`/`fs_path`,
/// recursively. Kept separate from `GLSLCode::preprocess` in `shader.rs`
/// (which doesn't expose the visited-file list) - a small accepted
/// duplication of the `#include` parsing logic.
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
                    let include_resource = format!("include/{include_name}");
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
