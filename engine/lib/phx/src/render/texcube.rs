use std::cell::RefCell;
use std::rc::Rc;

use crossbeam::channel::bounded;
use glam::{Vec2, Vec3};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::{
    CUBE_FACES, ClipRect, CubeFace, DataFormat, Draw, PixelFormat, RenderTarget, ShaderState,
    Tex2D, TexFilter, TexFormat,
};
use crate::math::Rng;
use crate::render::{RenderCommand, RenderState, Renderer, ResourceId, Shader, gl};
use crate::rf::Rf;
use crate::system::{Bytes, TimeStamp};

#[derive(Clone)]
pub struct TexCube {
    shared: Rf<TexCubeShared>,
}

struct TexCubeShared {
    id: ResourceId,
    size: i32,
    format: TexFormat,
    destroy_queue: Rc<RefCell<Vec<ResourceId>>>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Face {
    pub face: CubeFace,
    pub look: Vec3,
    pub up: Vec3,
}

const K_FACES: [Face; 6] = [
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

impl Drop for TexCubeShared {
    fn drop(&mut self) {
        self.destroy_queue.borrow_mut().push(self.id);
    }
}

impl TexCube {
    pub fn resource_id(&self) -> ResourceId {
        self.shared.as_ref().id
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

        let (tx, rx) = bounded(1);
        r.submit(RenderCommand::ReadTextureCubeFaceData {
            id: this.id,
            face: face as u32,
            level,
            pixel_format: tf as u32,
            data_format: df as u32,
            reply_tx: tx,
        });
        let bytes = rx.recv().unwrap_or_default();

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

        r.submit(RenderCommand::UpdateTextureCubeFaceDataByResource {
            id: this.id,
            face: face as u32,
            level,
            size: this.size,
            internal_format: this.format as i32,
            pixel_format: tf as u32,
            data_format: df as u32,
            data: bytes.to_vec(),
        });
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TexCube {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, size: i32, format: TexFormat) -> TexCube {
        if TexFormat::is_depth(format) {
            panic!("Cannot create cubemap with depth format");
        }

        let id = r.next_resource_id();
        r.submit(RenderCommand::CreateTextureCube {
            id,
            size: size as u32,
            format,
        });

        TexCube {
            shared: Rf::new(TexCubeShared {
                id,
                size,
                format,
                destroy_queue: r.destroy_queue(),
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

            faces.push((K_FACES[i as usize].face as gl::types::GLenum, pixel_format, buffer));
        }

        let id = r.next_resource_id();
        r.submit(RenderCommand::CreateTextureCube {
            id,
            size: size as u32,
            format,
        });
        for (face, pixel_format, buffer) in faces {
            r.submit(RenderCommand::UpdateTextureCubeFaceDataByResource {
                id,
                face,
                level: 0,
                size,
                internal_format: format as i32,
                pixel_format,
                data_format: gl::UNSIGNED_BYTE,
                data: buffer,
            });
        }

        TexCube {
            shared: Rf::new(TexCubeShared {
                id,
                size,
                format,
                destroy_queue: r.destroy_queue(),
            }),
        }
    }

    pub fn clear(&mut self, r: &mut Renderer, red: f32, green: f32, blue: f32, alpha: f32) {
        let this = self.shared.as_ref();
        let size = this.size;

        for i in 0..6 {
            let face = K_FACES[i as usize];

            RenderTarget::push(r, size, size);
            RenderTarget::bind_tex_cube(r, self, face.face);
            Draw::clear(r, red, green, blue, alpha);
            RenderTarget::pop(r);
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

    pub fn generate(&mut self, r: &mut Renderer, state: &mut ShaderState) {
        let this = self.shared.as_ref();

        RenderState::push_all_defaults(r);

        for i in 0..6 {
            let face = K_FACES[i as usize];
            let size = this.size;
            let size_f = this.size as f32;

            RenderTarget::push(r, size, size);
            RenderTarget::bind_tex_cube(r, self, face.face);
            Draw::clear(r, 0.0, 0.0, 0.0, 1.0);

            state
                .shader()
                .set_float3(r, "cubeLook", face.look.x, face.look.y, face.look.z);
            state
                .shader()
                .set_float3(r, "cubeUp", face.up.x, face.up.y, face.up.z);
            state.shader().set_float(r, "cubeSize", size_f);

            state.start(r);

            let mut j: i32 = 1;
            let mut job_size: i32 = 1;
            while j <= size {
                let time = TimeStamp::now();

                ClipRect::push(r, 0.0f32, (j - 1) as f32, size as f32, job_size as f32);
                Draw::rect(r, 0.0f32, 0.0f32, size_f, size_f);
                Draw::flush(r);
                ClipRect::pop(r);

                j += job_size;
                let elapsed = time.get_elapsed();

                job_size = f64::max(
                    1.0,
                    f64::floor(0.25f64 * job_size as f64 / elapsed + 0.5f64) as i32 as f64,
                ) as i32;
                job_size = i32::min(job_size, size - j + 1);
            }

            state.stop();

            RenderTarget::pop(r);
        }

        RenderState::pop_all(r);
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::GenerateMipmapByResource { id: this.id });
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

    pub fn set_mag_filter(&mut self, r: &mut Renderer, filter: TexFilter) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTextureMagFilterByResource {
            id: this.id,
            filter,
        });
    }

    pub fn set_min_filter(&mut self, r: &mut Renderer, filter: TexFilter) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTextureMinFilterByResource {
            id: this.id,
            filter,
        });
    }

    #[bind(name = "GenIRMap")]
    pub fn gen_ir_map(&mut self, r: &mut Renderer, sample_count: i32) -> TexCube {
        let mut size = self.get_size();
        let pf = self.get_format();

        let mut result = TexCube::new(r, size, pf);
        let df = DataFormat::Float;
        for i in 0..6 {
            let face = CubeFace::get(i);
            // TODO: Reuse buffer for each face.
            let data = self.get_data::<u8>(r, face, 0, pf, df);
            result.set_data(r, &data, face, 0, pf, df);
        }
        result.gen_mipmap(r);

        let mut shader = r
            .irmap_shader
            .take()
            .unwrap_or_else(|| Shader::load("vertex/identity", "fragment/compute/irmap"));

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

        shader.start(r);
        let mut level = 0;
        while size > 1 {
            size /= 2;
            level += 1;

            let mut ggx_width: f64 = level as f64 / levels as f64;
            ggx_width *= ggx_width;
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
            let mut angle = level as f32 / (levels - 1) as f32;
            angle = angle * angle;
            shader.reset_tex_index();
            shader.set_float(r, "angle", angle);
            shader.set_tex_cube(r, "src", self);
            shader.set_tex2d(r, "sample_buffer", &sample_tex);
            shader.set_int(r, "samples", sample_count);
            for i in 0..CUBE_FACES.len() {
                let this_face = CUBE_FACES[i];
                let this_look = look[i];
                let this_up = up[i];

                RenderTarget::push(r, size, size);
                RenderTarget::bind_tex_cube_level(r, &result, this_face, level);

                shader.set_float3(r, "cubeLook", this_look.x, this_look.y, this_look.z);
                shader.set_float3(r, "cubeUp", this_up.x, this_up.y, this_up.z);

                Draw::rect(r, -1.0, -1.0, 2.0, 2.0);

                RenderTarget::pop(r);
            }
        }
        shader.stop();

        r.irmap_shader = Some(shader);

        result.set_mag_filter(r, TexFilter::Linear);
        result.set_min_filter(r, TexFilter::LinearMipLinear);

        result
    }
}
