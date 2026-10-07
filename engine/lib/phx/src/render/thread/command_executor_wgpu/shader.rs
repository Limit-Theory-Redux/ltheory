//! Shader path of the wgpu executor: preprocessed GLSL 330 in, naga modules out.
//!
//! The engine's sources are GLSL 330 with a `#group` layout recorded by the
//! preprocessor. naga's GLSL frontend wants a few more things, and this module
//! adds exactly those, nothing else (no uniform packing, no name rewrites):
//!
//! * `#version 330` becomes `440` (the oldest version naga accepts).
//! * Uniform blocks and samplers get `set`/`binding` from the recorded layout:
//!   `set` is the `#group`, the `k`-th block of a group is binding `k`, the
//!   `i`-th sampler is a `texture` at binding `4 + 2i` and a `sampler` at
//!   `5 + 2i` (see `gpu/layout.rs`). The use sites of a combined sampler
//!   (`texture(src, ...)`) are rewritten to `texture(sampler2D(src_tex,
//!   src_samp), ...)`.
//! * Vertex attributes get the fixed locations GL binds by name, varyings get
//!   locations by their order in the vertex stage (GL links by name, wgpu by
//!   location), and fragment outputs without a location are numbered.
//! * `in mat4` attributes are split into four `vec4` columns.
//! * The vertex stage's `main` is wrapped: the original body runs, then
//!   `gl_Position` is converted from GL clip space (y up in a bottom-up
//!   framebuffer, z in -w..w) to the convention every render target of this
//!   backend uses: y flipped, z in 0..w. See the module docs of
//!   `command_executor_wgpu` for the convention.

use std::collections::HashMap;

use wgpu::naga;

use crate::render::{
    BlockLayout, ShaderLayout, TexDim, blocks_from_naga, wgpu_sampler_binding,
    wgpu_texture_binding,
};

/// Fixed vertex attribute locations (what `glBindAttribLocation` does in the GL
/// executor, and the vertex buffer layouts of the draw commands).
pub(super) const ATTRIBUTE_LOCATIONS: [(&str, u32); 11] = [
    ("vertex_position", 0),
    ("vertex_normal", 1),
    ("vertex_uv", 2),
    ("vertex_color", 3),
    ("instance_matrix_col0", 4),
    ("instance_matrix_col1", 5),
    ("instance_matrix_col2", 6),
    ("instance_matrix_col3", 7),
    ("instance_color", 8),
    ("imm_params", 11),
    ("imm_params2", 12),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    In,
    Out,
}

/// One top-level `in`/`out` declaration line.
struct IoDecl<'a> {
    indent: &'a str,
    /// Interpolation qualifier with its trailing space (`"flat "`), or empty.
    qualifier: &'a str,
    dir: Dir,
    ty: &'a str,
    name: &'a str,
    /// The line already carries `layout(location = ...)`.
    location: Option<u32>,
}

fn parse_io(line: &str) -> Option<IoDecl<'_>> {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let mut rest = trimmed;
    let mut location = None;
    if let Some(after) = rest.strip_prefix("layout") {
        let after = after.trim_start().strip_prefix('(')?;
        let close = after.find(')')?;
        let inner = &after[..close];
        let value = inner
            .split(',')
            .find_map(|part| part.trim().strip_prefix("location"))?;
        let value = value.trim_start().strip_prefix('=')?.trim();
        location = Some(value.parse().ok()?);
        rest = after[close + 1..].trim_start();
    }
    let mut qualifier = "";
    for q in ["flat ", "noperspective ", "smooth ", "centroid "] {
        if let Some(r) = rest.strip_prefix(q) {
            qualifier = &trimmed[trimmed.len() - rest.len()..][..q.len()];
            rest = r.trim_start();
            break;
        }
    }
    let (dir, rest) = if let Some(r) = rest.strip_prefix("in ") {
        (Dir::In, r)
    } else if let Some(r) = rest.strip_prefix("out ") {
        (Dir::Out, r)
    } else {
        return None;
    };
    if rest.contains('(') || rest.contains('{') || !rest.trim_end().ends_with(';') {
        return None;
    }
    let mut tokens = rest.trim_end().trim_end_matches(';').split_whitespace();
    let ty = tokens.next()?;
    let name = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }
    Some(IoDecl {
        indent,
        qualifier,
        dir,
        ty,
        name,
        location,
    })
}

/// Per-stage rewrites: version, block and sampler bindings, loose uniforms,
/// `mat4` attributes.
pub(super) fn adapt_stage(code: &str, layout: &ShaderLayout) -> Result<String, String> {
    let mut out_lines: Vec<String> = Vec::with_capacity(code.lines().count() + 16);
    // (sampler name, GLSL dimension suffix, shadow)
    let mut samplers: Vec<(String, String)> = Vec::new();
    for line in code.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];

        if trimmed.starts_with("#version") {
            out_lines.push("#version 440".to_string());
            continue;
        }

        // Blocks: `layout(std140) uniform Name {` -> with set and binding.
        if let Some(rest) = trimmed.strip_prefix("layout(std140) uniform ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if let Some(decl) = layout.block(&name) {
                out_lines.push(format!(
                    "{indent}layout(std140, set={}, binding={}) uniform {rest}",
                    decl.group,
                    crate::render::wgpu_block_binding(decl.index)
                ));
                continue;
            }
            return Err(format!(
                "uniform block '{name}' is not declared under a #group"
            ));
        }

        // Loose uniforms and samplers.
        if trimmed.starts_with("uniform ") && !trimmed.contains('{') {
            // Up to the `;` (a comment may follow it).
            let declaration = trimmed["uniform ".len()..].split(';').next().unwrap_or("");
            let mut tokens = declaration.split_whitespace();
            let ty = tokens.next().unwrap_or("");
            let name = tokens.next().unwrap_or("");
            if ty.is_empty() || name.is_empty() {
                out_lines.push(line.to_string());
                continue;
            }
            let dim = ty
                .strip_prefix("sampler")
                .or_else(|| ty.strip_prefix("isampler"))
                .or_else(|| ty.strip_prefix("usampler"));
            if let Some(dim) = dim {
                // A sampler outside any `#group` has no binding (GL gives all of
                // them unit 0, so such a shader never sampled correctly there
                // either). The declaration goes; a shader that reads it fails
                // to compile with an unknown identifier, one that does not
                // compiles as on GL.
                let Some(decl) = layout.texture(name) else {
                    out_lines.push(format!("{indent}// {declaration}: not declared under a #group"));
                    continue;
                };
                out_lines.push(format!(
                    "{indent}layout(set={}, binding={}) uniform texture{dim} {name}_tex;",
                    decl.group,
                    wgpu_texture_binding(decl.index)
                ));
                out_lines.push(format!(
                    "{indent}layout(set={}, binding={}) uniform sampler{} {name}_samp;",
                    decl.group,
                    wgpu_sampler_binding(decl.index),
                    if ty.ends_with("Shadow") { "Shadow" } else { "" }
                ));
                samplers.push((name.to_string(), dim.to_string()));
            } else {
                // A loose uniform that nothing reads at run time (GL rejects
                // active ones at link time): a zero-valued global keeps the
                // source compiling.
                out_lines.push(format!("{indent}{ty} {name};"));
            }
            continue;
        }

        // `in mat4 name;` -> four vec4 columns and a macro that rebuilds it.
        if let Some(decl) = parse_io(line) {
            if decl.dir == Dir::In && decl.ty == "mat4" {
                let base = decl.location.unwrap_or(106);
                let name = decl.name;
                for c in 0..4 {
                    out_lines.push(format!(
                        "{indent}layout(location = {}) in vec4 {name}_c{c};",
                        base + c
                    ));
                }
                out_lines.push(format!(
                    "{indent}#define {name} mat4({name}_c0, {name}_c1, {name}_c2, {name}_c3)"
                ));
                continue;
            }
        }
        out_lines.push(line.to_string());
    }

    let mut joined = out_lines.join("\n");
    // Sampling functions that take the combined sampler get the constructor;
    // the ones that take the texture alone get the texture.
    const SAMPLING_FNS: [&str; 11] = [
        "textureProjOffset",
        "textureGradOffset",
        "textureLodOffset",
        "textureGatherOffset",
        "textureProjGrad",
        "textureQueryLod",
        "textureGather",
        "textureProj",
        "textureGrad",
        "textureLod",
        "texture",
    ];
    const TEXTURE_ONLY_FNS: [&str; 3] = ["texelFetch", "textureSize", "textureQueryLevels"];
    for (name, dim) in &samplers {
        for prefix in SAMPLING_FNS {
            let pattern = format!("{prefix}({name},");
            let replacement = format!("{prefix}(sampler{dim}({name}_tex, {name}_samp),");
            joined = joined.replace(&pattern, &replacement);
        }
        for prefix in TEXTURE_ONLY_FNS {
            let pattern = format!("{prefix}({name},");
            let replacement = format!("{prefix}({name}_tex,");
            joined = joined.replace(&pattern, &replacement);
        }
    }
    Ok(joined)
}

/// Locations of the interface between the two stages of a program. Rewrites
/// the `in`/`out` lines of both sources.
pub(super) fn assign_interface(vs: &str, fs: &str) -> Result<(String, String), String> {
    // Vertex stage: attributes by name, varyings in declaration order.
    let mut varyings: HashMap<String, u32> = HashMap::new();
    let mut vs_out = Vec::new();
    for line in vs.lines() {
        match parse_io(line) {
            Some(decl) if decl.location.is_none() => {
                let location = match decl.dir {
                    Dir::In => ATTRIBUTE_LOCATIONS
                        .iter()
                        .find(|(n, _)| *n == decl.name)
                        .map(|(_, l)| *l)
                        .ok_or_else(|| {
                            format!(
                                "vertex attribute '{}' has no fixed location (add it to ATTRIBUTE_LOCATIONS and the vertex layouts)",
                                decl.name
                            )
                        })?,
                    Dir::Out => {
                        let next = varyings.len() as u32;
                        *varyings.entry(decl.name.to_string()).or_insert(next)
                    }
                };
                vs_out.push(format!(
                    "{}layout(location = {location}) {}{} {} {};",
                    decl.indent,
                    decl.qualifier,
                    if decl.dir == Dir::In { "in" } else { "out" },
                    decl.ty,
                    decl.name
                ));
            }
            Some(decl) if decl.dir == Dir::Out => {
                // An explicit varying location: keep it and remember the name.
                varyings.insert(decl.name.to_string(), decl.location.unwrap_or(0));
                vs_out.push(line.to_string());
            }
            _ => vs_out.push(line.to_string()),
        }
    }

    // Fragment stage: inputs by the vertex stage's names, outputs numbered.
    let mut taken: Vec<u32> = fs
        .lines()
        .filter_map(parse_io)
        .filter(|d| d.dir == Dir::Out)
        .filter_map(|d| d.location)
        .collect();
    let mut fs_out = Vec::new();
    for line in fs.lines() {
        match parse_io(line) {
            Some(decl) if decl.location.is_none() => match decl.dir {
                Dir::In => match varyings.get(decl.name) {
                    Some(location) => fs_out.push(format!(
                        "{}layout(location = {location}) {}in {} {};",
                        decl.indent, decl.qualifier, decl.ty, decl.name
                    )),
                    // The vertex stage never writes it: GL leaves it
                    // undefined (and links when nothing reads it); wgpu
                    // rejects the missing input. A zero global compiles.
                    None => fs_out.push(format!("{}{} {};", decl.indent, decl.ty, decl.name)),
                },
                Dir::Out => {
                    let location = (0u32..).find(|l| !taken.contains(l)).unwrap_or(0);
                    taken.push(location);
                    fs_out.push(format!(
                        "{}layout(location = {location}) out {} {};",
                        decl.indent, decl.ty, decl.name
                    ));
                }
            },
            Some(decl) if decl.dir == Dir::In => {
                // An explicit fragment input location must be the varying's.
                fs_out.push(format!(
                    "{}layout(location = {}) {}in {} {};",
                    decl.indent,
                    varyings.get(decl.name).copied().unwrap_or(decl.location.unwrap_or(0)),
                    decl.qualifier,
                    decl.ty,
                    decl.name
                ));
            }
            _ => fs_out.push(line.to_string()),
        }
    }
    Ok((vs_out.join("\n"), fs_out.join("\n")))
}

/// Rename the `main` of a stage to `name`; `None` if there is none.
fn rename_main(code: &str, name: &str) -> Option<String> {
    let bytes = code.as_bytes();
    let mut search = 0;
    while let Some(found) = code[search..].find("void") {
        let at = search + found;
        search = at + 4;
        let before_ok =
            at == 0 || !(bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_');
        let tail = code[at + 4..].trim_start();
        if !before_ok || !tail.starts_with("main") {
            continue;
        }
        let after_main = tail["main".len()..].trim_start();
        if !after_main.starts_with('(') {
            continue;
        }
        let name_at = code.len() - tail.len();
        let mut out = String::with_capacity(code.len() + 200);
        out.push_str(&code[..name_at]);
        out.push_str(name);
        out.push_str(&code[name_at + "main".len()..]);
        return Some(out);
    }
    None
}

/// Wrap the vertex stage's `main`: run it, then convert `gl_Position` to this
/// backend's target convention (y flipped, z remapped from -w..w to 0..w).
pub(super) fn wrap_vertex_main(code: &str) -> Result<String, String> {
    let mut out = rename_main(code, "_phx_vs_main").ok_or("vertex shader has no main()")?;
    out.push_str(
        "
void main() {
  _phx_vs_main();
  gl_Position.y = -gl_Position.y;
  gl_Position.z = (gl_Position.z + gl_Position.w) * 0.5;
}
",
    );
    Ok(out)
}

/// A fragment output of the adapted source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FsOutput {
    pub location: u32,
    pub name: String,
    pub ty: String,
}

/// The outputs of an adapted fragment stage (after `assign_interface`).
pub(super) fn fragment_outputs(code: &str) -> Vec<FsOutput> {
    code.lines()
        .filter_map(parse_io)
        .filter(|d| d.dir == Dir::Out)
        .filter_map(|d| {
            Some(FsOutput {
                location: d.location?,
                name: d.name.to_string(),
                ty: d.ty.to_string(),
            })
        })
        .collect()
}

/// Wrap the fragment stage's `main` so the outputs in `mask` (bit = location)
/// are clamped to 0..1 after it runs. GL clamps the color a fragment writes
/// to a fixed-point (`UNORM`) target before blending; wgpu blends the
/// unclamped value, which changes every blended pixel whose shader writes
/// above 1 (the UI glows do). Float targets are not clamped (in GL neither).
pub(super) fn wrap_fragment_clamp(
    code: &str,
    outputs: &[FsOutput],
    mask: u32,
) -> Result<String, String> {
    let mut out = rename_main(code, "_phx_fs_main").ok_or("fragment shader has no main()")?;
    out.push_str("
void main() {
  _phx_fs_main();
");
    for o in outputs.iter().filter(|o| mask & (1 << o.location) != 0) {
        out.push_str(&format!(
            "  {} = clamp({}, 0.0, 1.0);
",
            o.name, o.name
        ));
    }
    out.push_str("}
");
    Ok(out)
}

/// One stage on its own (the layout-parity and per-shader compile tests): the
/// same adaptation as `compile_pair`, with every fragment input treated as
/// not written by the vertex stage.
#[cfg(test)]
pub(crate) fn adapt_single_stage(
    code: &str,
    stage: naga::ShaderStage,
    layout: &ShaderLayout,
) -> Result<String, String> {
    let adapted = adapt_stage(code, layout)?;
    match stage {
        naga::ShaderStage::Vertex => {
            let (vs, _) = assign_interface(&adapted, "")?;
            wrap_vertex_main(&vs)
        }
        _ => Ok(assign_interface("", &adapted)?.1),
    }
}

/// Parse and validate one adapted stage (tests).
#[cfg(test)]
pub(crate) fn parse_adapted(
    stage: naga::ShaderStage,
    source: &str,
) -> Result<naga::Module, String> {
    parse_stage(stage, source)
}

/// What the wgpu side needs to know about one vertex shader input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VertexInput {
    pub location: u32,
    /// Format used when no vertex buffer supplies the attribute (GL: the
    /// constant default attribute value, `(0, 0, 0, 1)`).
    pub default: wgpu::VertexFormat,
}

fn default_format(inner: &naga::TypeInner) -> Option<wgpu::VertexFormat> {
    use naga::{ScalarKind, TypeInner, VectorSize};
    let (kind, n) = match inner {
        TypeInner::Scalar(s) => (s.kind, 1),
        TypeInner::Vector { size, scalar } => (
            scalar.kind,
            match size {
                VectorSize::Bi => 2,
                VectorSize::Tri => 3,
                VectorSize::Quad => 4,
            },
        ),
        _ => return None,
    };
    use wgpu::VertexFormat as F;
    Some(match (kind, n) {
        (ScalarKind::Float, 1) => F::Float32,
        (ScalarKind::Float, 2) => F::Float32x2,
        (ScalarKind::Float, 3) => F::Float32x3,
        (ScalarKind::Float, 4) => F::Float32x4,
        (ScalarKind::Uint, 1) => F::Uint32,
        (ScalarKind::Uint, 2) => F::Uint32x2,
        (ScalarKind::Uint, 3) => F::Uint32x3,
        (ScalarKind::Uint, 4) => F::Uint32x4,
        (ScalarKind::Sint, 1) => F::Sint32,
        (ScalarKind::Sint, 2) => F::Sint32x2,
        (ScalarKind::Sint, 3) => F::Sint32x3,
        (ScalarKind::Sint, 4) => F::Sint32x4,
        _ => return None,
    })
}

/// Vertex inputs and fragment output locations of a module's entry point.
fn entry_interface(module: &naga::Module) -> (Vec<VertexInput>, Vec<u32>) {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let Some(entry) = module.entry_points.first() else {
        return (inputs, outputs);
    };
    let mut visit = |binding: &Option<naga::Binding>, ty: naga::Handle<naga::Type>, is_input| {
        if let Some(naga::Binding::Location { location, .. }) = binding {
            if is_input {
                if let Some(default) = default_format(&module.types[ty].inner) {
                    inputs.push(VertexInput {
                        location: *location,
                        default,
                    });
                }
            } else {
                outputs.push(*location);
            }
        }
    };
    for arg in &entry.function.arguments {
        match &module.types[arg.ty].inner {
            naga::TypeInner::Struct { members, .. } => {
                for m in members {
                    visit(&m.binding, m.ty, true);
                }
            }
            _ => visit(&arg.binding, arg.ty, true),
        }
    }
    if let Some(result) = &entry.function.result {
        match &module.types[result.ty].inner {
            naga::TypeInner::Struct { members, .. } => {
                for m in members {
                    visit(&m.binding, m.ty, false);
                }
            }
            _ => visit(&result.binding, result.ty, false),
        }
    }
    (inputs, outputs)
}

pub(super) fn parse_stage(stage: naga::ShaderStage, source: &str) -> Result<naga::Module, String> {
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&naga::front::glsl::Options::from(stage), source)
        .map_err(|errors| {
            let mut detail = String::new();
            for e in errors.errors.iter().take(4) {
                let line_no = e
                    .location(source)
                    .map(|l| l.line_number as usize)
                    .unwrap_or(0);
                let line = source
                    .lines()
                    .nth(line_no.saturating_sub(1))
                    .unwrap_or("<eof>")
                    .trim();
                detail.push_str(&format!(" (line {line_no}: {line})"));
            }
            format!("naga GLSL {stage:?} parse failed: {errors}{detail}")
        })?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| format!("naga GLSL {stage:?} validation failed: {}", e.emit_to_string(source)))?;
    Ok(module)
}

/// Both stages of a program as validated naga modules, with what the executor
/// reflects from them.
pub(super) struct CompiledPair {
    pub vs: naga::Module,
    pub fs: naga::Module,
    /// The adapted fragment source and its outputs, kept to compile the
    /// clamped variants (`wrap_fragment_clamp`) lazily.
    pub fs_code: String,
    pub fs_named_outputs: Vec<FsOutput>,
    pub blocks: Vec<BlockLayout>,
    pub vs_inputs: Vec<VertexInput>,
    pub fs_outputs: Vec<u32>,
}

/// Preprocessed GLSL of both stages to naga modules (no device needed).
pub(super) fn compile_pair(
    vertex_src: &str,
    fragment_src: &str,
    layout: &ShaderLayout,
) -> Result<CompiledPair, String> {
    let vs = adapt_stage(vertex_src, layout)?;
    let fs = adapt_stage(fragment_src, layout)?;
    let (vs, fs_code) = assign_interface(&vs, &fs)?;
    let vs = wrap_vertex_main(&vs)?;
    let vs = parse_stage(naga::ShaderStage::Vertex, &vs)?;
    let fs = parse_stage(naga::ShaderStage::Fragment, &fs_code)?;
    let fs_named_outputs = fragment_outputs(&fs_code);

    let mut blocks = blocks_from_naga(&vs);
    for block in blocks_from_naga(&fs) {
        match blocks.iter().find(|b| b.name == block.name) {
            Some(existing) if *existing != block => {
                return Err(format!(
                    "uniform block '{}' differs between the vertex and fragment stage",
                    block.name
                ));
            }
            Some(_) => {}
            None => blocks.push(block),
        }
    }
    let (vs_inputs, _) = entry_interface(&vs);
    let (_, fs_outputs) = entry_interface(&fs);
    Ok(CompiledPair {
        vs,
        fs,
        fs_code,
        fs_named_outputs,
        blocks,
        vs_inputs,
        fs_outputs,
    })
}

/// Dimension of a declared sampler as wgpu names it.
pub(super) fn view_dimension(dim: TexDim) -> wgpu::TextureViewDimension {
    match dim {
        TexDim::D1 => wgpu::TextureViewDimension::D1,
        TexDim::D2 => wgpu::TextureViewDimension::D2,
        TexDim::D3 => wgpu::TextureViewDimension::D3,
        TexDim::Cube => wgpu::TextureViewDimension::Cube,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::GLSLCode;

    fn layout_of(code: &str) -> (String, ShaderLayout) {
        let pre = GLSLCode::preprocess_with(code, None, &mut |_| String::new());
        let (layout, errors) = ShaderLayout::merge(&[&pre.layout]);
        assert!(errors.is_empty(), "{errors:?}");
        (pre.code, layout)
    }

    #[test]
    fn io_lines_parse() {
        let d = parse_io("  flat out vec4 imm_color;").unwrap();
        assert_eq!((d.qualifier, d.dir, d.ty, d.name), ("flat ", Dir::Out, "vec4", "imm_color"));
        let d = parse_io("layout(location = 10) in uint instanceIndex;").unwrap();
        assert_eq!((d.location, d.dir, d.name), (Some(10), Dir::In, "instanceIndex"));
        let d = parse_io("layout (location = 0) out vec4 outColor;").unwrap();
        assert_eq!(d.location, Some(0));
        assert!(parse_io("void main() {").is_none());
        assert!(parse_io("  in_range = 3;").is_none());
    }

    #[test]
    fn wrapper_renames_main() {
        let wrapped = wrap_vertex_main("void  main (void) {\n gl_Position = vec4(0.0);\n}\n").unwrap();
        assert!(wrapped.contains("void  _phx_vs_main (void)"));
        assert!(wrapped.contains("void main() {\n  _phx_vs_main();"));
        assert!(wrap_vertex_main("int x;").is_err());
    }

    #[test]
    fn a_small_program_compiles_with_matching_interface() {
        let vs_src = "#version 330\n\
            #group 2\n\
            layout(std140) uniform Params { vec4 tint; };\n\
            in vec3 vertex_position;\n\
            in vec3 vertex_color;\n\
            out vec2 uv;\n\
            flat out vec4 imm_color;\n\
            void main() { uv = vertex_position.xy; imm_color = tint; gl_Position = vec4(vertex_position, 1.0); }\n";
        let fs_src = "#version 330\n\
            #group 2\n\
            layout(std140) uniform Params { vec4 tint; };\n\
            #group 3\n\
            uniform sampler2D src;\n\
            in vec2 uv;\n\
            flat in vec4 imm_color;\n\
            in vec3 unusedVarying;\n\
            uniform vec3 starColor;\n\
            layout(location = 0) out vec4 outColor;\n\
            void main() { outColor = texture(src, uv) * imm_color * tint; }\n";
        let (vs_code, vs_layout) = layout_of(vs_src);
        let (fs_code, fs_layout) = layout_of(fs_src);
        let (layout, errors) = ShaderLayout::merge(&[
            &GLSLCode::preprocess_with(vs_src, None, &mut |_| String::new()).layout,
            &GLSLCode::preprocess_with(fs_src, None, &mut |_| String::new()).layout,
        ]);
        assert!(errors.is_empty(), "{errors:?}");
        let _ = (vs_layout, fs_layout);
        let pair = compile_pair(&vs_code, &fs_code, &layout).expect("compile");
        assert_eq!(pair.blocks.len(), 1);
        assert_eq!(pair.fs_outputs, vec![0]);
        let locations: Vec<u32> = pair.vs_inputs.iter().map(|i| i.location).collect();
        assert!(locations.contains(&0) && locations.contains(&3));
    }
}
