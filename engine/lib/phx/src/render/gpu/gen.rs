//! Offscreen generation (doc/engine/render-api-v2.md, section 3e): fullscreen
//! shaders that fill cube maps and volumes, built on render passes.
//!
//! The generating fragment shader declares its group-2 `Params` block with the
//! face (cube) or slice (volume) data as the first three members:
//!
//! | `TexGen.Cube`            | `TexGen.Volume`          |
//! |--------------------------|--------------------------|
//! | `vec4 genLook`  (look)   | `vec4 genOrigin` (slice) |
//! | `vec4 genUp`    (up)     | `vec4 genDu`             |
//! | `vec4 genSize`  (x=size) | `vec4 genDv`             |
//!
//! followed by its own parameters, which `GenDesc:params` fills from the
//! `shader:blockType('Params')` cdata (the face members stay zero there). Its
//! samplers are pass inputs (group 3, up to four, `GenDesc:input`).

use std::sync::Arc;

use glam::Vec3;

use super::{
    BlockLayout, MAX_INPUTS, PassCmd, PipelineDesc, PipelineId, RenderPassDesc, SamplerId, TexView,
    VertexLayout,
};
use crate::render::{
    ClipRect, K_FACES, LoadOp, Renderer, ResourceId, Shader, Tex3D, TexCube, TexFormat,
};
use crate::system::TimeStamp;

const PARAMS_BLOCK: &str = "Params";

/// What a generation pass draws: the shader, its `Params` bytes and its
/// sampler inputs.
#[derive(Clone)]
pub struct GenDesc {
    label: String,
    shader: ResourceId,
    blocks: Arc<Vec<BlockLayout>>,
    params: Vec<u8>,
    inputs: [Option<(TexView, SamplerId)>; MAX_INPUTS],
}

#[luajit_ffi_gen::luajit_ffi]
impl GenDesc {
    /// Generation with `shader` (a `fullscreen_ndc` vertex shader and a
    /// generating fragment shader).
    #[bind(name = "Create")]
    pub fn create(shader: &Shader) -> GenDesc {
        GenDesc {
            label: "TexGen".to_string(),
            shader: shader.resource(),
            blocks: shader.blocks(),
            params: Vec::new(),
            inputs: Default::default(),
        }
    }

    /// Profiler and pass label.
    pub fn label(&mut self, label: &str) {
        self.label = label.to_string();
    }

    /// The bytes of the shader's `Params` block (a `blockType('Params')`
    /// cdata). Copied; the face or slice members are overwritten per draw.
    pub fn params(&mut self, bytes: &[u8]) {
        self.params = bytes.to_vec();
    }

    /// Sampler input `slot` (0..3, group 3).
    pub fn input(&mut self, slot: i32, view: &TexView, sampler: u32) {
        assert!(
            (0..MAX_INPUTS as i32).contains(&slot),
            "GenDesc '{}': input slot {slot} out of range (max {MAX_INPUTS})",
            self.label
        );
        self.inputs[slot as usize] = Some((*view, SamplerId(sampler as u16)));
    }
}

impl GenDesc {
    /// Size of the `Params` block, after checking that its leading members
    /// are the three `gen*` vec4s at offsets 0, 16 and 32.
    fn block_size(&self, members: [&str; 3]) -> u32 {
        let Some(block) = self.blocks.iter().find(|b| b.name == PARAMS_BLOCK) else {
            panic!(
                "TexGen '{}': the shader has no `{PARAMS_BLOCK}` block (declare one in group 2 with {members:?} as its first members)",
                self.label
            );
        };
        for (i, name) in members.iter().enumerate() {
            match block.member(name) {
                Some(m) if m.offset == i as u32 * 16 => {}
                other => panic!(
                    "TexGen '{}': `{PARAMS_BLOCK}` member '{name}' must be at offset {} (found {:?})",
                    self.label,
                    i * 16,
                    other.map(|m| m.offset)
                ),
            }
        }
        assert!(
            self.params.len() as u32 <= block.size,
            "TexGen '{}': {} bytes of params for a {}-byte `{PARAMS_BLOCK}` block",
            self.label,
            self.params.len(),
            block.size
        );
        block.size
    }
}

fn write_vec4(block: &mut [u8], index: usize, v: [f32; 4]) {
    for (i, f) in v.iter().enumerate() {
        let at = index * 16 + i * 4;
        block[at..at + 4].copy_from_slice(&f.to_ne_bytes());
    }
}

fn vec4(v: Vec3, w: f32) -> [f32; 4] {
    [v.x, v.y, v.z, w]
}

impl Renderer {
    /// Open a generation pass and bind its pipeline and inputs.
    fn gen_begin(&mut self, desc: &GenDesc, pass_desc: &RenderPassDesc, pipeline: PipelineId) {
        self.begin_pass_intern(pass_desc);
        self.pass_record("setPipeline", PassCmd::SetPipeline(pipeline));
        for (slot, input) in desc.inputs.iter().enumerate() {
            self.data.encoder.set_input(slot, *input);
        }
    }

    /// Write `block` (the `Params` bytes with the face or slice data filled
    /// in) as the group-2 block of the draws that follow.
    fn gen_block(&mut self, block: &[u8]) {
        let ptr = self.pass_alloc(block.len() as u32);
        #[allow(unsafe_code)]
        // SAFETY: `pass_alloc` returns `len` writable bytes of ring staging.
        unsafe {
            std::ptr::copy_nonoverlapping(block.as_ptr(), ptr, block.len());
        }
    }

    fn gen_pipeline(&mut self, desc: &GenDesc) -> PipelineId {
        let mut pipeline = PipelineDesc::new(desc.shader);
        pipeline.vertex = VertexLayout::Fullscreen;
        self.get_pipeline(&pipeline)
    }

    /// Fill all six faces of `cube` with `desc`'s shader. Each face is one pass,
    /// drawn as scissored row slices: the slice height adapts to the measured
    /// time per slice (a quarter of a second at most), which bounds the time
    /// one submit can take (TDR).
    pub fn generate_cube(&mut self, desc: &GenDesc, cube: &TexCube) {
        let size = cube.get_size();
        let size_f = size as f32;
        let block_size = desc.block_size(["genLook", "genUp", "genSize"]) as usize;
        let pipeline = self.gen_pipeline(desc);

        let mut block = vec![0u8; block_size];
        block[..desc.params.len()].copy_from_slice(&desc.params);

        for face in K_FACES {
            let pass_desc = RenderPassDesc::with_color(
                &desc.label,
                cube.face_view(face.face),
                LoadOp::Clear,
                [0.0, 0.0, 0.0, 1.0],
            );
            self.gen_begin(desc, &pass_desc, pipeline);
            write_vec4(&mut block, 0, vec4(face.look, 0.0));
            write_vec4(&mut block, 1, vec4(face.up, 0.0));
            write_vec4(&mut block, 2, [size_f, 0.0, 0.0, 0.0]);
            self.gen_block(&block);

            let mut j: i32 = 1;
            let mut job_size: i32 = 1;
            while j <= size {
                let time = TimeStamp::now();

                ClipRect::push(self, 0.0f32, (j - 1) as f32, size_f, job_size as f32);
                self.pass_draw(PassCmd::DrawFullscreen);
                ClipRect::pop(self);

                j += job_size;
                let elapsed = time.get_elapsed();

                job_size = f64::max(
                    1.0,
                    f64::floor(0.25f64 * job_size as f64 / elapsed + 0.5f64) as i32 as f64,
                ) as i32;
                job_size = i32::min(job_size, size - j + 1);
            }

            self.end_pass_intern();
        }
    }

    /// Fill every z-slice of `volume` (a cube spanning [-1,1]^3) with `desc`'s
    /// shader, one pass per slice.
    pub fn generate_volume(&mut self, desc: &GenDesc, volume: &Tex3D, depth: i32) {
        let block_size = desc.block_size(["genOrigin", "genDu", "genDv"]) as usize;
        let pipeline = self.gen_pipeline(desc);

        let mut block = vec![0u8; block_size];
        block[..desc.params.len()].copy_from_slice(&desc.params);
        write_vec4(&mut block, 1, [2.0, 0.0, 0.0, 0.0]);
        write_vec4(&mut block, 2, [0.0, 2.0, 0.0, 0.0]);

        for i in 0..depth {
            let z = 2.0 * (i as f32 / (depth - 1).max(1) as f32) - 1.0;
            // The shader writes every texel, so no load is needed.
            let pass_desc = RenderPassDesc::with_color(
                &desc.label,
                volume.layer_view(i),
                LoadOp::DontCare,
                [0.0; 4],
            );
            self.gen_begin(desc, &pass_desc, pipeline);
            write_vec4(&mut block, 0, [-1.0, -1.0, z, 0.0]);
            self.gen_block(&block);
            self.pass_draw(PassCmd::DrawFullscreen);
            self.end_pass_intern();
        }
    }
}

/// Namespace for `TexGen.Cube` and `TexGen.Volume`. (Not named `Gen`: that is
/// the global of the Legacy generator namespace.)
pub struct TexGen;

#[luajit_ffi_gen::luajit_ffi]
impl TexGen {
    /// A new cube map of `size` and `format` with `mips` levels (0 = the full
    /// chain, the caller then generates it), generated with `desc`.
    pub fn cube(
        r: &mut Renderer,
        desc: &GenDesc,
        size: i32,
        format: TexFormat,
        mips: i32,
    ) -> TexCube {
        let cube = TexCube::new_desc(r, size, format, mips, 0);
        r.generate_cube(desc, &cube);
        cube
    }

    /// Regenerate an existing cube map (ping-pong generation).
    pub fn cube_into(r: &mut Renderer, desc: &GenDesc, cube: &TexCube) {
        r.generate_cube(desc, cube);
    }

    /// A new `size`^3 volume of `format`, generated with `desc`.
    pub fn volume(r: &mut Renderer, desc: &GenDesc, size: i32, format: TexFormat) -> Tex3D {
        let volume = Tex3D::new(r, size, size, size, format);
        r.generate_volume(desc, &volume, size);
        volume
    }
}
