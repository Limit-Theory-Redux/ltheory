//! Layout parity (doc/engine/render-api-v2.md, section 2, rule 3).
//!
//! The preprocessor records every `#group` declaration of a shader; the GL
//! executor applies that record at link time and reflects the real block
//! layout from the linked program. This test covers the half that needs no GL
//! context: it runs every shader in `res/shader` through the preprocessor and
//! through naga's GLSL frontend (the wgpu path, with the recorded block
//! bindings injected) and asserts the two agree on what the shader declares
//! (blocks, their bindings and member layout; sampler names and dimensions).
//!
//! What it cannot cover: the GL side of the comparison (`glGetActiveUniform*`
//! reflection of the same programs) needs a context. The GL reflection is
//! checked when each shader links (`ViewBlock` is asserted against the
//! reflected block there), and the validation scenes exercise it end to end.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::command_executor_wgpu::WgpuCommandExecutor;
use crate::render::{
    BlockLayout, GLSLCode, GlslType, ShaderLayout, TexDim, ViewBlock, blocks_from_naga,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

/// Every vertex and fragment shader tracked by git (directory listings can be
/// partial on network shares; `git ls-files` is deterministic).
fn shader_files(root: &Path) -> Vec<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "res/shader/vertex", "res/shader/fragment"])
        .current_dir(root)
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    let mut files: Vec<PathBuf> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.ends_with(".glsl"))
        .map(|l| root.join(l))
        .collect();
    // Documented dead shaders (referenced by no script; both also fail to
    // compile on GL): `uv_metal` has an unbalanced paren at line 30, and
    // `ptracer` needs an engine-provided `SCENE_DESC` define.
    let excluded = [
        "res/shader/fragment/material/uv_metal.glsl",
        "res/shader/fragment/ptracer.glsl",
    ];
    files.retain(|f| {
        let rel = f
            .strip_prefix(root)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        !excluded.contains(&rel.as_str())
    });
    files.sort();
    files
}

fn preprocess(root: &Path, file: &Path) -> GLSLCode {
    let res_shader = root.join("res").join("shader");
    let mut loader = |name: &str| -> String {
        let path = res_shader.join(format!("{name}.glsl"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot load include {path:?}: {e}"))
    };
    let src = std::fs::read_to_string(file).expect("read shader");
    GLSLCode::preprocess_with(&src, None, &mut loader)
}

struct NagaView {
    blocks: Vec<BlockLayout>,
    /// `(block name, binding)` of every uniform-space global with a binding.
    block_bindings: Vec<(String, u32)>,
    /// `(sampler name, dimension)` from the `<name>_tex` image globals.
    textures: Vec<(String, TexDim)>,
}

/// Parse `code` the way the wgpu executor would (adapted, with `layout`'s
/// block bindings) and collect what naga sees.
fn naga_view(
    code: &str,
    stage: wgpu::naga::ShaderStage,
    layout: &ShaderLayout,
) -> Result<NagaView, String> {
    use wgpu::naga::{AddressSpace, ImageDimension, TypeInner};

    let adapted = WgpuCommandExecutor::adapt_glsl_for_naga_with_layout(code, 0, layout);
    let mut frontend = wgpu::naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&wgpu::naga::front::glsl::Options::from(stage), &adapted)
        .map_err(|e| format!("parse: {e}"))?;
    wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| format!("validate: {e}"))?;

    let mut block_bindings = Vec::new();
    let mut textures = Vec::new();
    for (_, var) in module.global_variables.iter() {
        let ty = &module.types[var.ty];
        match var.space {
            AddressSpace::Uniform => {
                let name = ty
                    .name
                    .clone()
                    .or_else(|| var.name.clone())
                    .unwrap_or_default();
                if let Some(binding) = var.binding {
                    block_bindings.push((name, binding.binding));
                }
            }
            AddressSpace::Handle => {
                if let (TypeInner::Image { dim, .. }, Some(name)) = (&ty.inner, &var.name) {
                    if let Some(base) = name.strip_suffix("_tex") {
                        let dim = match dim {
                            ImageDimension::D1 => TexDim::D1,
                            ImageDimension::D2 => TexDim::D2,
                            ImageDimension::D3 => TexDim::D3,
                            ImageDimension::Cube => TexDim::Cube,
                        };
                        textures.push((base.to_string(), dim));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(NagaView {
        blocks: blocks_from_naga(&module),
        block_bindings,
        textures,
    })
}

#[test]
fn recorded_layout_matches_naga_for_every_shader() {
    let root = repo_root();
    let files = shader_files(&root);
    assert!(
        files.len() >= 130,
        "expected 130+ shaders, got {}",
        files.len()
    );

    let mut checked_blocks = 0;
    let mut checked_textures = 0;
    let mut grouped_shaders = 0;
    let mut problems: Vec<String> = Vec::new();

    for file in &files {
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let stage = if rel.contains("/vertex/") {
            wgpu::naga::ShaderStage::Vertex
        } else {
            wgpu::naga::ShaderStage::Fragment
        };
        let pre = preprocess(&root, file);
        let (layout, errors) = ShaderLayout::merge(&[&pre.layout]);
        if !errors.is_empty() {
            problems.push(format!("{rel}: layout errors {errors:?}"));
            continue;
        }
        if layout.is_empty() {
            continue; // nothing recorded, nothing to compare
        }
        grouped_shaders += 1;

        let view = match naga_view(&pre.code, stage, &layout) {
            Ok(view) => view,
            Err(e) => {
                problems.push(format!("{rel}: naga {e}"));
                continue;
            }
        };

        // Every recorded block exists in naga with the recorded binding.
        for decl in &layout.blocks {
            checked_blocks += 1;
            match view.block_bindings.iter().find(|(n, _)| n == &decl.name) {
                Some((_, binding)) if *binding == decl.binding() => {}
                Some((_, binding)) => problems.push(format!(
                    "{rel}: block {} recorded at binding {} but naga sees {binding}",
                    decl.name,
                    decl.binding()
                )),
                None => problems.push(format!(
                    "{rel}: block {} recorded but naga found no such uniform block",
                    decl.name
                )),
            }
        }
        // Every real uniform block naga sees was recorded.
        for (name, _) in &view.block_bindings {
            let known = layout.block(name).is_some()
                || name.starts_with("_phx")
                || matches!(name.as_str(), "LightUBO" | "MaterialUBO");
            if !known {
                problems.push(format!(
                    "{rel}: naga sees block {name} the preprocessor did not record"
                ));
            }
        }
        // Samplers: recorded name and dimension match naga's image.
        for decl in &layout.textures {
            checked_textures += 1;
            match view.textures.iter().find(|(n, _)| n == &decl.name) {
                Some((_, dim)) if *dim == decl.dim => {}
                Some((_, dim)) => problems.push(format!(
                    "{rel}: sampler {} recorded as {:?} but naga sees {dim:?}",
                    decl.name, decl.dim
                )),
                None => problems.push(format!(
                    "{rel}: sampler {} recorded but naga found no such texture",
                    decl.name
                )),
            }
        }
    }

    println!(
        "layout parity: {grouped_shaders} grouped shader stages, {checked_blocks} blocks, {checked_textures} samplers"
    );
    assert!(
        problems.is_empty(),
        "layout parity failures:\n  {}",
        problems.join("\n  ")
    );
    assert!(
        grouped_shaders >= 20,
        "too few grouped shaders: {grouped_shaders}"
    );
    assert!(checked_blocks > 0 && checked_textures > 0);
}

#[test]
fn view_block_matches_the_rust_struct() {
    let root = repo_root();
    for rel in [
        "res/shader/vertex/wvp.glsl",
        "res/shader/fragment/simple_color.glsl",
    ] {
        let file = root.join(rel);
        let pre = preprocess(&root, &file);
        let (layout, errors) = ShaderLayout::merge(&[&pre.layout]);
        assert!(errors.is_empty(), "{rel}: {errors:?}");
        let stage = if rel.contains("/vertex/") {
            wgpu::naga::ShaderStage::Vertex
        } else {
            wgpu::naga::ShaderStage::Fragment
        };
        let view = naga_view(&pre.code, stage, &layout).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let block = view
            .blocks
            .iter()
            .find(|b| b.name == ViewBlock::NAME)
            .unwrap_or_else(|| panic!("{rel}: no ViewBlock"));
        ViewBlock::check_layout(block).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert_eq!(layout.block(ViewBlock::NAME).unwrap().binding(), 0);
    }
}

#[test]
fn naga_reflection_reports_std140_offsets() {
    // A synthetic shader: vec3 + float packing, a vec2 on the next 16-byte
    // boundary, an array of vec4, trailing padding to 16 bytes.
    let src = "#version 330\n\
        #group 2\n\
        layout(std140) uniform Params {\n\
          vec3 a;\n\
          float b;\n\
          vec2 c;\n\
          vec4 d[2];\n\
        };\n\
        out vec4 outColor;\n\
        void main() { outColor = vec4(a, b) + vec4(c, 0.0, 0.0) + d[1]; }\n";
    let pre = GLSLCode::preprocess_with(src, None, &mut |_| String::new());
    let (layout, errors) = ShaderLayout::merge(&[&pre.layout]);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(layout.block("Params").unwrap().binding(), 8);
    let view = naga_view(&pre.code, wgpu::naga::ShaderStage::Fragment, &layout).expect("naga");
    let params = view
        .blocks
        .iter()
        .find(|b| b.name == "Params")
        .expect("block");
    let by_name = |n: &str| params.member(n).unwrap_or_else(|| panic!("member {n}"));
    assert_eq!((by_name("a").offset, by_name("a").ty), (0, GlslType::Vec3));
    assert_eq!(
        (by_name("b").offset, by_name("b").ty),
        (12, GlslType::Float)
    );
    assert_eq!((by_name("c").offset, by_name("c").ty), (16, GlslType::Vec2));
    assert_eq!(by_name("d").offset, 32);
    assert_eq!((by_name("d").count, by_name("d").array_stride), (2, 16));
    assert_eq!(params.size, 64);
    let used: BTreeSet<&str> = params.members.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(used.len(), 4);
}
