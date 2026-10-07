#[cfg(test)]
use std::collections::HashMap;
use std::sync::Arc;

use super::{BlockLayout, DrawBlock, GROUP_COUNT, ShaderLayout, StageLayout, TexDim, ViewBlock};
use crate::logging::{info, warn};
use crate::render::{Renderer, ResourceHandle, ResourceId};
use crate::rf::Rf;
use crate::system::{Resource, ResourceType};

const INCLUDE_PATH: &str = "include/";

#[derive(Clone)]
pub struct Shader {
    shared: Rf<ShaderShared>,
}

struct ShaderShared {
    name: String,
    vs_name: Option<String>,
    fs_name: Option<String>,
    handle: ResourceHandle,
    /// Bind-group layout recorded by the preprocessor (`#group` directives).
    layout: Arc<ShaderLayout>,
    /// Uniform blocks reflected at link time (empty for the error shader).
    blocks: Arc<Vec<BlockLayout>>,
    /// Bumped on every successful reload.
    generation: u32,
}

/// A preprocessed shader stage: `#include`s inlined, `#group` directives
/// stripped, and the `#group` declarations recorded.
#[derive(Clone, Default)]
pub(crate) struct GLSLCode {
    pub(crate) code: String,
    pub(crate) layout: StageLayout,
}

impl GLSLCode {
    fn load(name: &str) -> GLSLCode {
        Self::preprocess(&Resource::load_string(ResourceType::Shader, name))
    }

    fn preprocess(code: &str) -> GLSLCode {
        Self::preprocess_with(code, None, &mut |name| {
            Resource::load_string(ResourceType::Shader, name)
        })
    }

    /// `loader` receives include paths relative to the shader directory
    /// (`include/<name>`). `group` is the group in effect where this source
    /// is included; a `#group` directive inside an include does not leak out
    /// of it.
    pub(crate) fn preprocess_with(
        code: &str,
        group: Option<u8>,
        loader: &mut dyn FnMut(&str) -> String,
    ) -> GLSLCode {
        let mut result = GLSLCode::default();
        let mut group = group;

        for line in code.lines() {
            if let Some(include_val) = line.strip_prefix("#include ") {
                let path = format!("{INCLUDE_PATH}{include_val}");
                let include = Self::preprocess_with(&loader(&path), group, loader);

                result.code += &include.code;
                result.code += "\n";
                result.layout.blocks.extend(include.layout.blocks);
                result.layout.textures.extend(include.layout.textures);
                result.layout.errors.extend(include.layout.errors);
            } else if let Some(value) = line.strip_prefix("#group ") {
                match value.trim().parse::<u8>() {
                    Ok(g) if (g as usize) < GROUP_COUNT => group = Some(g),
                    _ => result
                        .layout
                        .errors
                        .push(format!("bad directive '{line}' (expected #group 0..3)")),
                }
            } else {
                if let Some(g) = group {
                    Self::record_declaration(line, g, &mut result.layout);
                }
                result.code += line;
                result.code += "\n";
            }
        }

        result
    }

    /// Record a uniform block or sampler declaration under `group`.
    fn record_declaration(line: &str, group: u8, layout: &mut StageLayout) {
        let t = line.trim();
        if t.starts_with("//") {
            return;
        }
        let Some(uniform_at) = t.find("uniform ") else {
            return;
        };
        // Only whole-line declarations: `layout(...) uniform ...` or
        // `uniform ...`.
        if uniform_at != 0 && !t.starts_with("layout") {
            return;
        }
        let rest = t[uniform_at + "uniform ".len()..].trim_start();
        if t.contains('{') {
            let name: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '{')
                .collect();
            if !name.is_empty() {
                layout.blocks.push((name, group));
            }
            return;
        }
        let mut tokens = rest.split_whitespace();
        let (Some(ty), Some(name)) = (tokens.next(), tokens.next()) else {
            return;
        };
        if !ty.contains("sampler") {
            return;
        }
        let name = name.trim_end_matches(';');
        match TexDim::from_sampler_type(ty) {
            Some(dim) => layout.textures.push((name.to_string(), group, dim)),
            None => layout.errors.push(format!(
                "unsupported sampler type '{ty}' for '{name}' in a #group"
            )),
        }
    }
}

impl Shader {
    fn from_preprocessed(
        r: &mut Renderer,
        name: String,
        vs_code: GLSLCode,
        fs_code: GLSLCode,
        vs_name: Option<String>,
        fs_name: Option<String>,
    ) -> Shader {
        let (layout, layout_errors) = ShaderLayout::merge(&[&vs_code.layout, &fs_code.layout]);
        let layout = Arc::new(layout);

        let compiled = if layout_errors.is_empty() {
            create_shader_blocking(r, &vs_code.code, &fs_code.code, &layout)
        } else {
            Err(format!(
                "invalid #group layout: {}",
                layout_errors.join("; ")
            ))
        };

        let (handle, layout, blocks) = match compiled {
            Ok((handle, blocks)) => {
                check_fixed_blocks(&name, &blocks);
                (handle, layout, blocks)
            }
            Err(e) => {
                r.data.shader_errors.push(
                    &shader_error_key(&vs_name, &fs_name, &name),
                    "compile",
                    &e,
                );
                warn!("Shader '{name}' failed to compile, using error shader: {e}");
                // The error shader declares nothing.
                (
                    create_error_shader_handle(r),
                    Arc::new(ShaderLayout::default()),
                    Vec::new(),
                )
            }
        };

        Shader {
            shared: Rf::new(ShaderShared {
                name,
                vs_name,
                fs_name,
                handle,
                layout,
                blocks: Arc::new(blocks),
                generation: 0,
            }),
        }
    }

    /// The bind-group layout recorded from the shader's `#group` directives.
    pub fn layout(&self) -> Arc<ShaderLayout> {
        self.shared.as_ref().layout.clone()
    }

    /// The uniform blocks reflected when the program was linked.
    pub fn blocks(&self) -> Arc<Vec<BlockLayout>> {
        self.shared.as_ref().blocks.clone()
    }

    /// The shader's GPU resource id.
    pub fn resource(&self) -> ResourceId {
        self.shared.as_ref().handle.id()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Shader {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, vs: &str, fs: &str) -> Shader {
        Self::from_preprocessed(
            r,
            "[anonymous shader]".into(),
            GLSLCode::preprocess(vs),
            GLSLCode::preprocess(fs),
            None,
            None,
        )
    }

    pub fn load(r: &mut Renderer, vs_name: &str, fs_name: &str) -> Shader {
        Self::from_preprocessed(
            r,
            format!("[vs: {vs_name}, fs: {fs_name}]"),
            GLSLCode::load(vs_name),
            GLSLCode::load(fs_name),
            Some(vs_name.to_string()),
            Some(fs_name.to_string()),
        )
    }

    /// Reload shader from disk. Returns true on success.
    /// On compile/link failure, keeps the old shader and returns false.
    pub fn reload(&mut self, r: &mut Renderer) -> bool {
        let s = self.shared.as_ref();
        let (Some(vs_name), Some(fs_name)) = (s.vs_name.clone(), s.fs_name.clone()) else {
            warn!("Cannot reload shader {} — no source paths stored", s.name);
            return false;
        };
        let name = s.name.clone();
        drop(s);

        // Reload and preprocess from disk
        let vs_code = GLSLCode::load(&vs_name);
        let fs_code = GLSLCode::load(&fs_name);
        let (layout, layout_errors) = ShaderLayout::merge(&[&vs_code.layout, &fs_code.layout]);
        let layout = Arc::new(layout);

        // Try compile (non-panicking)
        let compiled = if layout_errors.is_empty() {
            create_shader_blocking(r, &vs_code.code, &fs_code.code, &layout)
        } else {
            Err(format!(
                "invalid #group layout: {}",
                layout_errors.join("; ")
            ))
        };
        let (new_handle, blocks) = match compiled {
            Ok(result) => result,
            Err(e) => {
                r.data
                    .shader_errors
                    .push(&format!("{vs_name}:{fs_name}"), "compile", &e);
                warn!("Shader '{name}' reload failed: {e}");
                return false;
            }
        };
        check_fixed_blocks(&name, &blocks);

        // Success - swap the resource handle in-place (all Rf clones see the
        // update); the old handle drops here, enqueuing its own destroy.
        {
            let s = &mut *self.shared.as_mut();
            s.handle = new_handle;
            s.layout = layout;
            s.blocks = Arc::new(blocks);
            s.generation += 1;
        }

        info!("Reloaded shader {}", name);
        true
    }

    pub fn name(&self) -> String {
        self.shared.as_ref().name.clone()
    }

    /// A LuaJIT `ffi.typeof` struct declaration with the byte layout of the
    /// shader's uniform block `name` (empty if the shader has no such block).
    /// The Lua side wraps it as `shader:blockType(name)`.
    pub fn block_decl(&self, name: &str) -> String {
        self.shared
            .as_ref()
            .blocks
            .iter()
            .find(|b| b.name == name)
            .map(|b| b.lua_struct())
            .unwrap_or_default()
    }

    /// Size in bytes of the uniform block `name` (0 if absent).
    pub fn block_size(&self, name: &str) -> u32 {
        self.shared
            .as_ref()
            .blocks
            .iter()
            .find(|b| b.name == name)
            .map_or(0, |b| b.size)
    }

    /// Bumped each time hot reload relinks the shader, so cached block types
    /// can be regenerated.
    pub fn generation(&self) -> u32 {
        self.shared.as_ref().generation
    }

    /// The shader's GPU resource id (as a plain scalar: `ResourceId` itself
    /// is not an FFI type), e.g. for caches keyed by the shader's program
    /// (`Render.Pipelines`; a hot reload gives the shader a new resource).
    /// Unlike `Mesh::resource_id`, this is a plain getter - `ShaderShared::handle` is always created eagerly in
    /// `new`/`from_preprocessed`, never lazily.
    pub fn resource_id(&self) -> u64 {
        self.shared.as_ref().handle.id().0
    }

    #[bind(name = "Clone")]
    pub fn acquire(&self) -> Shader {
        self.clone()
    }
}

/// Key used to attribute a compile/reload error to a shader in the error
/// queue, matching the canonical `vs:fs` cache key used everywhere else
/// (`Cache.lua`, `ShaderWatcher`). Falls back to the shader's display name
/// for anonymous shaders (`Shader.Create`), which have no source paths.
fn shader_error_key(vs_name: &Option<String>, fs_name: &Option<String>, name: &str) -> String {
    match (vs_name, fs_name) {
        (Some(vs), Some(fs)) => format!("{vs}:{fs}"),
        _ => name.to_string(),
    }
}

/// Minimal magenta placeholder shader used when a shader fails to compile on
/// first load, so a broken shader renders visibly wrong instead of crashing
/// the whole application. Deliberately has no uniforms - its only
/// job is to compile and be unmistakably obvious on screen.
fn create_error_shader_handle(r: &mut Renderer) -> ResourceHandle {
    const ERROR_VS: &str = "#version 330\n\
        in vec3 vertex_position;\n\
        void main() {\n\
        \x20   gl_Position = vec4(vertex_position, 1.0);\n\
        }\n";
    const ERROR_FS: &str = "#version 330\n\
        out vec4 fragColor;\n\
        void main() {\n\
        \x20   fragColor = vec4(1.0, 0.0, 1.0, 1.0);\n\
        }\n";

    create_shader_blocking(r, ERROR_VS, ERROR_FS, &Arc::new(ShaderLayout::default()))
        .expect("Failed to compile fallback error shader")
        .0
}

/// Compile+link a shader on the render thread and block for the result,
/// which carries the uniform blocks reflected from the linked program.
/// Mints a fresh `ResourceHandle` up front - on error it's simply dropped,
/// which harmlessly enqueues a destroy for a resource that was never created
/// (the executor's `DestroyResource` handler already no-ops on a missing id).
fn create_shader_blocking(
    r: &mut Renderer,
    vertex_src: &str,
    fragment_src: &str,
    layout: &Arc<ShaderLayout>,
) -> Result<(ResourceHandle, Vec<BlockLayout>), String> {
    let handle = r.create_resource();
    let blocks = r.create_shader(
        handle.id(),
        vertex_src.to_string(),
        fragment_src.to_string(),
        layout.clone(),
    )?;
    Ok((handle, blocks))
}

/// Startup assert: the fixed Rust block structs must match what the linked
/// shader declares (size and member offsets), or every pass would feed the
/// GPU garbage.
fn check_fixed_blocks(shader_name: &str, blocks: &[BlockLayout]) {
    for block in blocks {
        if block.name == ViewBlock::NAME {
            if let Err(e) = ViewBlock::check_layout(block) {
                panic!("Shader '{shader_name}': {e}");
            }
        } else if block.name == DrawBlock::NAME {
            if let Err(e) = DrawBlock::check_layout(block) {
                panic!("Shader '{shader_name}': {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pre(code: &str, includes: &[(&str, &str)]) -> GLSLCode {
        let includes: HashMap<String, String> = includes
            .iter()
            .map(|(n, c)| (format!("include/{n}"), c.to_string()))
            .collect();
        GLSLCode::preprocess_with(code, None, &mut |name| {
            includes
                .get(name)
                .unwrap_or_else(|| panic!("no include {name}"))
                .clone()
        })
    }

    #[test]
    fn group_directive_is_stripped_and_declarations_recorded() {
        let out = pre(
            "#version 330\n#group 1\nlayout(std140) uniform Mat {\n  vec4 c;\n};\nuniform sampler2D albedo;\n#group 3\nuniform samplerCube env;\nuniform float loose;\n",
            &[],
        );
        assert!(!out.code.contains("#group"));
        assert!(out.layout.errors.is_empty(), "{:?}", out.layout.errors);
        assert_eq!(out.layout.blocks, vec![("Mat".to_string(), 1)]);
        assert_eq!(
            out.layout.textures,
            vec![
                ("albedo".to_string(), 1, TexDim::D2),
                ("env".to_string(), 3, TexDim::Cube)
            ]
        );
    }

    #[test]
    fn declarations_without_a_group_are_not_recorded() {
        let out = pre(
            "uniform sampler2D a;\nlayout(std140) uniform B { float x; };\n",
            &[],
        );
        assert!(out.layout.blocks.is_empty() && out.layout.textures.is_empty());
    }

    #[test]
    fn includes_inherit_the_group_but_do_not_leak_theirs() {
        let out = pre(
            "#group 2\n#include inc\nuniform sampler2D after;\n",
            &[(
                "inc",
                "uniform sampler2D inherited;\n#group 0\nuniform sampler2D inner;\n",
            )],
        );
        assert_eq!(
            out.layout.textures,
            vec![
                ("inherited".to_string(), 2, TexDim::D2),
                ("inner".to_string(), 0, TexDim::D2),
                // the include's `#group 0` did not change the includer's group
                ("after".to_string(), 2, TexDim::D2),
            ]
        );
    }

    #[test]
    fn comments_and_non_declarations_are_ignored() {
        let out = pre(
            "#group 0\n// uniform sampler2D commented;\nfloat uniformity = 1.0;\nuniform sampler2D real; // trailing\n",
            &[],
        );
        assert_eq!(
            out.layout.textures,
            vec![("real".to_string(), 0, TexDim::D2)]
        );
    }

    #[test]
    fn bad_directives_and_sampler_types_are_errors() {
        let out = pre(
            "#group 9\n#group x\n#group 0\nuniform sampler2DShadow s;\n",
            &[],
        );
        assert_eq!(out.layout.errors.len(), 3, "{:?}", out.layout.errors);
    }

    #[test]
    fn merged_stage_layouts_number_by_first_appearance() {
        let vs = pre("#group 0\nlayout(std140) uniform V { vec4 a; };\n", &[]);
        let fs = pre(
            "#group 0\nlayout(std140) uniform V { vec4 a; };\nuniform samplerCube e;\nuniform samplerCube i;\n",
            &[],
        );
        let (layout, errors) = ShaderLayout::merge(&[&vs.layout, &fs.layout]);
        assert!(errors.is_empty());
        assert_eq!(layout.block("V").unwrap().binding(), 0);
        assert_eq!(layout.texture("e").unwrap().unit(), 0);
        assert_eq!(layout.texture("i").unwrap().unit(), 1);
    }
}
