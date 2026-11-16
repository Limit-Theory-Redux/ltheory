//! Named shader parameters for the UI shapes (`Shape::Stroke` for now).
//!
//! A UI shader that reads parameters beyond its vertex attributes declares
//! them as a list of names with defaults ([`ShaderParams`]), four to a `vec4`
//! in list order. Lua sets them by name (`glowWidth`), the engine maps the
//! name to its slot, and the batcher writes the resulting blocks into the
//! shape parameter block (`include/shape_params.glsl`): a uniform block of
//! [`SHAPE_PARAM_VEC4S`] `vec4`s, one per run of vertices, bound as the first
//! block of group 2 (the per-draw ring block, like `RenderPass:alloc`). Each
//! vertex carries the offset of its block (in `vec4`s), so primitives with
//! different values still batch into one draw.

/// `vec4`s in the shape parameter block of one run (`shapeParams` in
/// `include/shape_params.glsl`).
pub const SHAPE_PARAM_VEC4S: usize = 64;

/// Bytes of the shape parameter block.
pub const SHAPE_PARAM_BYTES: usize = SHAPE_PARAM_VEC4S * 16;

/// Name of the block in `include/shape_params.glsl`.
pub const SHAPE_PARAM_BLOCK_NAME: &str = "ShapeParams";

/// One parameter a UI shader declares: the name Lua sets it by and the value
/// it has when nothing sets it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderParam {
    pub name: &'static str,
    pub default: f32,
}

impl ShaderParam {
    pub const fn new(name: &'static str, default: f32) -> Self {
        Self { name, default }
    }
}

/// The parameters a UI shader declares, in slot order: slot `i` is component
/// `i % 4` of the block's `vec4` number `i / 4`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderParams {
    pub params: &'static [ShaderParam],
}

impl ShaderParams {
    pub const fn new(params: &'static [ShaderParam]) -> Self {
        Self { params }
    }

    /// The number of parameters.
    pub const fn len(&self) -> usize {
        self.params.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// The `vec4`s a block of these parameters takes.
    pub const fn vec4s(&self) -> usize {
        self.params.len().div_ceil(4)
    }

    /// The slot of the parameter `name`, if declared.
    pub fn slot(&self, name: &str) -> Option<usize> {
        self.params.iter().position(|p| p.name == name)
    }

    /// The declared names, for messages (`core, coreWidth, ...`).
    pub fn names(&self) -> String {
        self.params
            .iter()
            .map(|p| p.name)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The default values, in slot order.
    pub fn defaults<const N: usize>(&self) -> [f32; N] {
        assert_eq!(N, self.len(), "parameter count");
        std::array::from_fn(|i| self.params[i].default)
    }

    /// `values` (in slot order) as the block's `vec4`s, zero padded.
    pub fn pack(&self, values: &[f32]) -> Vec<[f32; 4]> {
        assert_eq!(values.len(), self.len(), "parameter count");
        let mut block = vec![[0.0; 4]; self.vec4s()];
        for (i, value) in values.iter().enumerate() {
            block[i / 4][i % 4] = *value;
        }
        block
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARAMS: ShaderParams = ShaderParams::new(&[
        ShaderParam::new("a", 1.0),
        ShaderParam::new("b", 2.0),
        ShaderParam::new("c", 3.0),
        ShaderParam::new("d", 4.0),
        ShaderParam::new("e", 5.0),
    ]);

    #[test]
    fn parameters_are_found_by_name() {
        assert_eq!(PARAMS.slot("a"), Some(0));
        assert_eq!(PARAMS.slot("e"), Some(4));
        assert_eq!(PARAMS.slot("glow"), None);
        assert_eq!(PARAMS.names(), "a, b, c, d, e");
    }

    #[test]
    fn blocks_take_whole_vec4s() {
        assert_eq!(PARAMS.vec4s(), 2);
        let defaults: [f32; 5] = PARAMS.defaults();
        assert_eq!(defaults, [1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(
            PARAMS.pack(&defaults),
            vec![[1.0, 2.0, 3.0, 4.0], [5.0, 0.0, 0.0, 0.0]]
        );
    }

    #[test]
    fn the_block_in_the_shader_has_the_rust_size() {
        // `include/shape_params.glsl` declares `vec4 shapeParams[N]`.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res/shader/include/shape_params.glsl");
        let src = std::fs::read_to_string(&path).expect("read shape_params.glsl");
        let decl = format!("vec4 shapeParams[{SHAPE_PARAM_VEC4S}];");
        assert!(
            src.contains(&decl),
            "shape_params.glsl must declare `{decl}`"
        );
        assert!(src.contains(&format!("uniform {SHAPE_PARAM_BLOCK_NAME}")));
    }
}
