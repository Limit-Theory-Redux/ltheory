use glam::{Vec2, Vec3};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::{
    CUBE_FACES, CubeFace, DataFormat, PassCmd, PipelineDesc, PixelFormat, Samplers, Tex2D,
    TexFormat, VertexLayout,
};
use crate::math::Rng;
use crate::render::{
    LoadOp, RenderPassDesc, Renderer, ResourceHandle, ResourceId, Shader, TexView, ViewDim, gl,
};
use crate::rf::Rf;
use crate::system::Bytes;

/// See `TexCube::gen_ir_map`.
const CONVOLVE_IRMAP: bool = false;

#[derive(Clone)]
pub struct TexCube {
    shared: Rf<TexCubeShared>,
}

struct TexCubeShared {
    handle: ResourceHandle,
    size: i32,
    format: TexFormat,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Face {
    pub face: CubeFace,
    pub look: Vec3,
    pub up: Vec3,
}

pub(crate) const K_FACES: [Face; 6] = [
    Face {
        face: CubeFace::PX,
        look: Vec3::X,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace::NX,
        look: Vec3::NEG_X,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace::PY,
        look: Vec3::Y,
        up: Vec3::NEG_Z,
    },
    Face {
        face: CubeFace::NY,
        look: Vec3::NEG_Y,
        up: Vec3::Z,
    },
    Face {
        face: CubeFace::PZ,
        look: Vec3::Z,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace::NZ,
        look: Vec3::NEG_Z,
        up: Vec3::Y,
    },
];

const K_FACE_EXT: [&str; 6] = ["px", "py", "pz", "nx", "ny", "nz"];

impl TexCube {
    pub fn resource_id(&self) -> ResourceId {
        self.shared.as_ref().handle.id()
    }

    pub fn get_data<T: Clone + Default>(
        &self,
        r: &mut Renderer,
        face: CubeFace,
        level: i32,
        tf: TexFormat,
        df: DataFormat,
    ) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size * this.size;
        size *= DataFormat::get_size(df);
        size *= TexFormat::components(tf);
        size /= std::mem::size_of::<T>() as i32;

        let bytes = r.read_texture_cube_face_data(
            this.handle.id(),
            face as u32,
            level,
            tf as u32,
            df as u32,
        );

        let mut data = vec![T::default(); size as usize];
        let byte_len = (data.len() * std::mem::size_of::<T>()).min(bytes.len());
        #[allow(unsafe_code)] // TODO: refactor
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr() as *mut u8, byte_len);
        }

        data
    }

    pub fn set_data<T>(
        &mut self,
        r: &mut Renderer,
        data: &[T],
        face: CubeFace,
        level: i32,
        tf: TexFormat,
        df: DataFormat,
    ) {
        let this = self.shared.as_ref();
        let byte_len = std::mem::size_of_val(data);
        #[allow(unsafe_code)] // TODO: refactor
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len) };

        r.update_texture_cube_face_data_by_resource(
            this.handle.id(),
            face as u32,
            level,
            this.size,
            this.format as i32,
            tf as u32,
            df as u32,
            bytes.to_vec(),
        );
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TexCube {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, size: i32, format: TexFormat) -> TexCube {
        if TexFormat::is_depth(format) {
            panic!("Cannot create cubemap with depth format");
        }

        let handle = r.create_resource();
        r.create_texture_cube(handle.id(), size as u32, format);

        TexCube {
            shared: Rf::new(TexCubeShared {
                handle,
                size,
                format,
            }),
        }
    }

    pub fn load(r: &mut Renderer, path: &str) -> TexCube {
        let mut size = 0;
        let mut format = TexFormat::RGB8;
        let mut faces: Vec<(gl::types::GLenum, u32, Vec<u8>)> = Vec::with_capacity(6);

        for i in 0..6 {
            let face_path = format!("{}{}.jpg", path, K_FACE_EXT[i as usize]);

            let reader = ImageReader::open(&face_path).unwrap_or_else(|_| {
                panic!("Failed to load cubemap face from '{face_path}', unable to open file")
            });
            let img = reader.decode().unwrap_or_else(|_| {
                panic!("Failed to load cubemap face from '{face_path}', decode failed")
            });
            let (width, height) = img.dimensions();

            let (pixel_format, data_format, buffer) = match img {
                DynamicImage::ImageRgba8(buf) => (gl::RGBA, TexFormat::RGBA8, buf.into_raw()),
                DynamicImage::ImageRgb8(buf) => (gl::RGB, TexFormat::RGB8, buf.into_raw()),
                _ => panic!(
                    "Failed to load cubemap face from '{face_path}', unsupported image format"
                ),
            };

            if width != height {
                panic!("Loaded cubemap face is not square");
            }

            if i != 0 {
                if width != size as u32 || height != size as u32 {
                    panic!("Cubemap face {i} has a different resolution");
                }

                if format != data_format {
                    panic!("Cubemap face {i} has a different number of components");
                }
            } else {
                size = width as i32;
                format = data_format;
            }

            faces.push((
                K_FACES[i as usize].face as gl::types::GLenum,
                pixel_format,
                buffer,
            ));
        }

        let handle = r.create_resource();
        r.create_texture_cube(handle.id(), size as u32, format);
        for (face, pixel_format, buffer) in faces {
            r.update_texture_cube_face_data_by_resource(
                handle.id(),
                face,
                0,
                size,
                format as i32,
                pixel_format,
                gl::UNSIGNED_BYTE,
                buffer,
            );
        }

        TexCube {
            shared: Rf::new(TexCubeShared {
                handle,
                size,
                format,
            }),
        }
    }

    /// View of the whole cube, for sampling.
    pub fn view(&self) -> TexView {
        let size = self.shared.as_ref().size.max(1);
        TexView::full(self.resource_id(), ViewDim::Cube, [size, size])
    }

    /// View of one face at mip level 0, usable as a render attachment.
    pub fn face_view(&self, face: CubeFace) -> TexView {
        self.face_mip_view(face, 0)
    }

    /// View of one face at the given mip level, usable as a render attachment.
    pub fn face_mip_view(&self, face: CubeFace, level: i32) -> TexView {
        let size = (self.shared.as_ref().size >> level).max(1);
        TexView::new(
            self.resource_id(),
            ViewDim::CubeFace(face),
            level,
            [size, size],
        )
    }

    pub fn clear(&mut self, r: &mut Renderer, red: f32, green: f32, blue: f32, alpha: f32) {
        for i in 0..6 {
            let face = K_FACES[i as usize];
            let desc = RenderPassDesc::with_color(
                "TexCube.clear",
                self.face_view(face.face),
                LoadOp::Clear,
                [red, green, blue, alpha],
            );
            r.begin_pass_intern(&desc);
            r.end_pass_intern();
        }
    }

    pub fn save(&mut self, r: &mut Renderer, path: &str) {
        self.save_level(r, path, 0);
    }

    pub fn save_level(&mut self, r: &mut Renderer, path: &str, level: i32) {
        let this = self.shared.as_ref();
        let size = this.size >> level;

        for i in 0..6 {
            let face = K_FACES[i as usize].face;
            let face_path = format!("{}{}.png", path, K_FACE_EXT[i as usize]);

            let data: Vec<u8> = self.get_data(r, face, level, TexFormat::RGBA8, DataFormat::U8);
            if let Some(image_buffer) =
                ImageBuffer::<Rgba<u8>, _>::from_raw(size as u32, size as u32, data)
            {
                let _ = image_buffer.save(face_path);
            }
        }
    }

    pub fn get_data_bytes(
        &mut self,
        r: &mut Renderer,
        face: CubeFace,
        level: i32,
        tf: TexFormat,
        df: DataFormat,
    ) -> Bytes {
        Bytes::from_vec(self.get_data(r, face, level, tf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.format
    }

    pub fn get_size(&self) -> i32 {
        let this = self.shared.as_ref();
        this.size
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.generate_mipmap_by_resource(this.handle.id());
    }

    pub fn set_data_bytes(
        &mut self,
        r: &mut Renderer,
        data: &Bytes,
        face: CubeFace,
        level: i32,
        tf: TexFormat,
        df: DataFormat,
    ) {
        self.set_data(r, data.as_slice(), face, level, tf, df);
    }

    #[bind(name = "GenIRMap")]
    pub fn gen_ir_map(&mut self, r: &mut Renderer, sample_count: i32) -> TexCube {
        let mut size = self.get_size();
        let pf = self.get_format();

        // Level 0 is a straight copy of this cube (a blit per face); the
        // other levels are filtered below.
        let mut result = TexCube::new(r, size, pf);
        for face in CUBE_FACES {
            r.copy_texture(
                self.face_view(face),
                result.face_view(face),
                [size as u32, size as u32, 1],
            );
        }
        result.gen_mipmap(r);

        let shader = r.data.irmap_shader.take().unwrap_or_else(|| {
            Shader::load(r, "vertex/fullscreen_ndc", "fragment/compute/irmap")
        });
        let mut pipeline = PipelineDesc::new(shader.resource());
        pipeline.vertex = VertexLayout::Fullscreen;
        let pipeline = r.get_pipeline(&pipeline);

        // Params { vec4 genLook; vec4 genUp; float angle; int samples; }
        let block_size = {
            let blocks = shader.blocks();
            let block = blocks
                .iter()
                .find(|b| b.name == "Params")
                .expect("irmap shader has no Params block");
            for (name, offset) in [("genLook", 0), ("genUp", 16), ("angle", 32), ("samples", 36)] {
                assert_eq!(
                    block.member(name).map(|m| m.offset),
                    Some(offset),
                    "irmap Params member {name}"
                );
            }
            block.size as usize
        };

        let look = [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
            Vec3::Z,
            Vec3::NEG_Z,
        ];
        let up = [Vec3::Y, Vec3::Y, Vec3::NEG_Z, Vec3::Z, Vec3::Y, Vec3::Y];

        let mut rng = Rng::from_time();
        let mut levels = 0;
        let mut i = size;
        while i > 0 {
            levels += 1;
            i /= 2;
        }

        // The sample directions the filter should convolve with. Before the render
        // API v2 work the shader's `sampleBuffer` was never bound (the uniform was
        // set under another name), so every sample read (pitch, yaw) = (0, 0) and
        // each level was a plain resample of the source. A one texel zero texture
        // reproduces that exactly; `CONVOLVE_IRMAP` switches to the real GGX lobe
        // samples (random, so the lighting of every scene changes slightly).
        let zero_samples = {
            let mut tex = Tex2D::new(r, 1, 1, TexFormat::RG16F);
            tex.set_data(r, &[Vec2::ZERO], PixelFormat::RG, DataFormat::Float);
            tex
        };

        let mut level = 0;
        while size > 1 {
            size /= 2;
            level += 1;

            let mut ggx_width: f64 = level as f64 / levels as f64;
            ggx_width *= ggx_width;
            let sample_tex = if CONVOLVE_IRMAP {
                let mut sample_buffer = vec![Vec2::ZERO; sample_count as usize];
                let mut sample_tex = Tex2D::new(r, sample_count, 1, TexFormat::RG16F);

                for i in 0..sample_count {
                    let e1 = rng.get_uniform();
                    let e2 = rng.get_uniform();
                    let pitch = f64::atan2(ggx_width * f64::sqrt(e1), f64::sqrt(1.0f64 - e1));
                    let yaw = std::f64::consts::TAU * e2;
                    sample_buffer[i as usize] = Vec2::new(pitch as f32, yaw as f32);
                }

                sample_tex.set_data(r, &sample_buffer, PixelFormat::RG, DataFormat::Float);
                sample_tex
            } else {
                zero_samples.clone()
            };
            let mut angle = level as f32 / (levels - 1) as f32;
            angle = angle * angle;

            for i in 0..CUBE_FACES.len() {
                let desc = RenderPassDesc::with_color(
                    "TexCube.genIRMap",
                    result.face_mip_view(CUBE_FACES[i], level),
                    LoadOp::DontCare,
                    [0.0; 4],
                );
                r.begin_pass_intern(&desc);
                r.pass_record("setPipeline", PassCmd::SetPipeline(pipeline));
                r.data.encoder.set_input(
                    0,
                    Some((self.view(), Samplers::LinearMipClamp.id())),
                );
                r.data
                    .encoder
                    .set_input(1, Some((sample_tex.view(), Samplers::Point.id())));

                let mut block = vec![0u8; block_size];
                for (at, v) in [
                    (0, look[i].extend(0.0)),
                    (16, up[i].extend(0.0)),
                ] {
                    for (k, f) in v.to_array().iter().enumerate() {
                        block[at + k * 4..at + k * 4 + 4].copy_from_slice(&f.to_ne_bytes());
                    }
                }
                block[32..36].copy_from_slice(&angle.to_ne_bytes());
                block[36..40].copy_from_slice(&sample_count.to_ne_bytes());
                let ptr = r.pass_alloc(block_size as u32);
                #[allow(unsafe_code)]
                // SAFETY: `pass_alloc` returns `block_size` writable bytes.
                unsafe {
                    std::ptr::copy_nonoverlapping(block.as_ptr(), ptr, block_size);
                }
                r.pass_draw(PassCmd::DrawFullscreen);

                r.end_pass_intern();
            }
        }

        r.data.irmap_shader = Some(shader);

        result
    }
}
