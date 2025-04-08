use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::*;
use crate::math::*;
use crate::rf::Rf;
use crate::system::*;

#[derive(Clone)]
pub struct TexCube {
    shared: Rf<TexCubeShared>,
}

struct TexCubeShared {
    handle: u32,
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

static mut K_FACES: [Face; 6] = [
    Face {
        face: CubeFace_PX,
        look: Vec3::X,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace_NX,
        look: Vec3::NEG_X,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace_PY,
        look: Vec3::Y,
        up: Vec3::NEG_Z,
    },
    Face {
        face: CubeFace_NY,
        look: Vec3::NEG_Y,
        up: Vec3::Z,
    },
    Face {
        face: CubeFace_PZ,
        look: Vec3::Z,
        up: Vec3::Y,
    },
    Face {
        face: CubeFace_NZ,
        look: Vec3::NEG_Z,
        up: Vec3::Y,
    },
];

const K_FACE_EXT: [&str; 6] = ["px", "py", "pz", "nx", "ny", "nz"];

impl Drop for TexCubeShared {
    fn drop(&mut self) {
        if self.handle != 0 {
            glcheck!(gl::DeleteTextures(1, &self.handle));
        }
    }
}

impl TexCubeShared {
    fn init(&self) {
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as i32,
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as i32,
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32,
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32,
        ));
    }
}

impl TexCube {
    pub fn get_data<T: Clone + Default>(
        &self,
        face: CubeFace,
        level: i32,
        pf: PixelFormat,
        df: DataFormat,
    ) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size * this.size;
        size *= DataFormat_GetSize(df);
        size *= PixelFormat_Components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let mut data = vec![T::default(); size as usize];
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));
        glcheck!(gl::GetTexImage(
            face as gl::types::GLenum,
            level,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_mut_ptr() as *mut _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));

        data
    }

    pub fn set_data<T>(
        &mut self,
        data: &[T],
        face: CubeFace,
        level: i32,
        pf: PixelFormat,
        df: DataFormat,
    ) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));
        glcheck!(gl::TexImage2D(
            face as gl::types::GLenum,
            level,
            this.format,
            this.size,
            this.size,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl TexCube {
    #[bind(name = "Create")]
    pub fn new(size: i32, format: TexFormat) -> TexCube {
        if !TexFormat_IsValid(format) {
            panic!("Invalid texture format requested");
        }
        if TexFormat_IsDepth(format) {
            panic!("Cannot create cubemap with depth format");
        }

        let mut this = TexCubeShared {
            handle: 0,
            size,
            format,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            0,
            format,
            size,
            size,
            0,
            gl::RED,
            gl::BYTE,
            std::ptr::null(),
        ));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
            0,
            format,
            size,
            size,
            0,
            gl::RED,
            gl::BYTE,
            std::ptr::null(),
        ));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
            0,
            format,
            size,
            size,
            0,
            gl::RED,
            gl::BYTE,
            std::ptr::null(),
        ));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
            0,
            format,
            size,
            size,
            0,
            gl::RED,
            gl::BYTE,
            std::ptr::null(),
        ));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            0,
            format,
            size,
            size,
            0,
            gl::RED,
            gl::BYTE,
            std::ptr::null(),
        ));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
            0,
            format,
            size,
            size,
            0,
            gl::RED,
            gl::BYTE,
            std::ptr::null(),
        ));

        this.init();

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));

        TexCube {
            shared: Rf::new(this),
        }
    }

    pub fn load(path: &str) -> TexCube {
        let mut this = TexCubeShared {
            handle: 0,
            size: 0,
            format: 0,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));

        for i in 0..6 {
            let face_path = format!("{}{}.jpg", path, K_FACE_EXT[i as usize]);

            let reader = ImageReader::open(&face_path).unwrap_or_else(|_| {
                panic!("Failed to load cubemap face from '{face_path}', unable to open file")
            });
            let img = reader.decode().unwrap_or_else(|_| {
                panic!("Failed to load cubemap face from '{face_path}', decode failed")
            });
            let (width, height) = img.dimensions();

            let (format, data_format, buffer) = match img {
                DynamicImage::ImageRgba8(buf) => (gl::RGBA, TexFormat_RGBA8, buf.into_raw()),
                DynamicImage::ImageRgb8(buf) => (gl::RGB, TexFormat_RGB8, buf.into_raw()),
                _ => panic!(
                    "Failed to load cubemap face from '{face_path}', unsupported image format"
                ),
            };

            if width != height {
                panic!("Loaded cubemap face is not square");
            }

            if i != 0 {
                if width != this.size as u32 || height != this.size as u32 {
                    panic!("Cubemap face {i} has a different resolution");
                }

                if this.format != data_format {
                    panic!("Cubemap face {i} has a different number of components");
                }
            } else {
                this.size = width as i32;
                this.format = data_format;
            }

            glcheck!(gl::TexImage2D(
                K_FACES[i as usize].face as gl::types::GLenum,
                0,
                this.format as gl::types::GLint,
                this.size,
                this.size,
                0,
                format,
                gl::UNSIGNED_BYTE,
                buffer.as_ptr() as *const _,
            ));
        }

        this.init();

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));

        TexCube {
            shared: Rf::new(this),
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let this = self.shared.as_ref();

        for i in 0..6 {
            #[allow(unsafe_code)] // TODO: remove
            let face = unsafe { K_FACES[i as usize] };

            RenderTarget::push(this.size, this.size);
            RenderTarget::bind_tex_cube(self, face.face);
            Draw::clear(r, g, b, a);
            RenderTarget::pop();
        }
    }

    pub fn save(&mut self, path: &str) {
        self.save_level(path, 0);
    }

    pub fn save_level(&mut self, path: &str, level: i32) {
        let this = self.shared.as_ref();

        let size = this.size >> level;

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));

        let mut image_buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::new(size as u32, size as u32);
        for i in 0..6 {
            #[allow(unsafe_code)] // TODO: remove
            let face = unsafe { K_FACES[i as usize].face };
            let face_path = format!("{}{}.png", path, K_FACE_EXT[i as usize]);

            glcheck!(gl::GetTexImage(
                face as gl::types::GLenum,
                level,
                gl::RGBA8,
                gl::UNSIGNED_BYTE,
                image_buffer.as_mut_ptr() as *mut _,
            ));

            let _ = image_buffer.save(face_path);
        }

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));
    }

    pub fn get_data_bytes(
        &mut self,
        face: CubeFace,
        level: i32,
        pf: PixelFormat,
        df: DataFormat,
    ) -> Bytes {
        Bytes::from_vec(self.get_data(face, level, pf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.format
    }

    pub fn get_handle(&self) -> u32 {
        let this = self.shared.as_ref();
        this.handle
    }

    pub fn get_size(&self) -> i32 {
        let this = self.shared.as_ref();
        this.size
    }

    pub fn generate(&mut self, state: &mut ShaderState) {
        let this = self.shared.as_ref();

        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            RenderState_PushAllDefaults()
        };

        for i in 0..6 {
            #[allow(unsafe_code)] // TODO: remove
            let face: Face = unsafe { K_FACES[i as usize] };
            let size: i32 = this.size;
            let size_f: f32 = this.size as f32;

            RenderTarget::push(size, size);
            RenderTarget::bind_tex_cube(self, face.face);
            Draw::clear(0.0, 0.0, 0.0, 1.0);

            state
                .shader()
                .set_float3("cubeLook", face.look.x, face.look.y, face.look.z);
            state
                .shader()
                .set_float3("cubeUp", face.up.x, face.up.y, face.up.z);
            state.shader().set_float("cubeSize", size_f);

            state.start();

            let mut j: i32 = 1;
            let mut job_size: i32 = 1;
            while j <= size {
                let time: TimeStamp = TimeStamp::now();

                ClipRect::push(0.0f32, (j - 1) as f32, size as f32, job_size as f32);
                Draw::rect(0.0f32, 0.0f32, size_f, size_f);
                Draw::flush();
                ClipRect::pop();

                j += job_size;
                let elapsed = time.get_elapsed();

                job_size = f64::max(
                    1.0,
                    f64::floor(0.25f64 * job_size as f64 / elapsed + 0.5f64) as i32 as f64,
                ) as i32;
                job_size = i32::min(job_size, size - j + 1);
            }

            state.stop();

            RenderTarget::pop();
        }

        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            RenderState_PopAll()
        };
    }

    pub fn gen_mipmap(&mut self) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));
        glcheck!(gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));
    }

    pub fn set_data_bytes(
        &mut self,
        data: &Bytes,
        face: CubeFace,
        level: i32,
        pf: PixelFormat,
        df: DataFormat,
    ) {
        self.set_data(data.as_slice(), face, level, pf, df);
    }

    pub fn set_mag_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            filter
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));
    }

    pub fn set_min_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            filter
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0));
    }

    #[bind(name = "GenIRMap")]
    pub fn gen_ir_map(&mut self, sample_count: i32) -> TexCube {
        let mut size = self.get_size();
        let pf = self.get_format();

        let mut result = TexCube::new(size, pf);
        let df = DataFormat_Float;
        for i in 0..6 {
            #[allow(unsafe_code)] // TODO: remove
            let face = unsafe { CubeFace_Get(i) };
            // TODO: Reuse buffer for each face.
            result.set_data(&self.get_data::<u8>(face, 0, pf, df), face, 0, pf, df);
        }
        result.gen_mipmap();

        // TODO: Store the shader somewhere and use the Box correctly.
        #[allow(unsafe_code)] // TODO: remove
        let shader = unsafe {
            static mut SHADER: *mut Shader = std::ptr::null_mut();
            if SHADER.is_null() {
                SHADER = Box::into_raw(Box::new(Shader::load(
                    "vertex/identity",
                    "fragment/compute/irmap",
                )));
            }
            &mut *SHADER
        };

        let face: [CubeFace; 6] = [
            CubeFace_PX,
            CubeFace_NX,
            CubeFace_PY,
            CubeFace_NY,
            CubeFace_PZ,
            CubeFace_NZ,
        ];
        let look: [Vec3; 6] = [
            Vec3::X,
            Vec3::NEG_X,
            Vec3::Y,
            Vec3::NEG_Y,
            Vec3::Z,
            Vec3::NEG_Z,
        ];
        let up: [Vec3; 6] = [Vec3::Y, Vec3::Y, Vec3::NEG_Z, Vec3::Z, Vec3::Y, Vec3::Y];

        let mut rng = Rng::from_time();
        let mut levels = 0;
        let mut i = size;
        while i > 0 {
            levels += 1;
            i /= 2;
        }

        shader.start();
        let mut level = 0;
        while size > 1 {
            size /= 2;
            level += 1;

            let mut ggx_width: f64 = level as f64 / levels as f64;
            ggx_width *= ggx_width;
            let mut sample_buffer = vec![Vec2::ZERO; sample_count as usize];
            let mut sample_tex = Tex2D::new(sample_count, 1, TexFormat_RG16F);

            for i in 0..sample_count {
                let e1 = rng.get_uniform();
                let e2 = rng.get_uniform();
                let pitch = f64::atan2(ggx_width * f64::sqrt(e1), f64::sqrt(1.0f64 - e1));
                let yaw = std::f64::consts::TAU * e2;
                sample_buffer[i as usize] = Vec2::new(pitch as f32, yaw as f32);
            }

            sample_tex.set_data(&sample_buffer, PixelFormat_RG, DataFormat_Float);
            let mut angle = level as f32 / (levels - 1) as f32;
            angle = angle * angle;
            shader.reset_tex_index();
            shader.set_float("angle", angle);
            shader.set_tex_cube("src", self);
            shader.set_tex2d("sample_buffer", &sample_tex);
            shader.set_int("samples", sample_count);
            for i in 0..6 {
                let this_face = face[i];
                let this_look = look[i];
                let this_up = up[i];

                RenderTarget::push(size, size);
                RenderTarget::bind_tex_cube_level(&result, this_face, level);

                shader.set_float3("cubeLook", this_look.x, this_look.y, this_look.z);
                shader.set_float3("cubeUp", this_up.x, this_up.y, this_up.z);

                Draw::rect(-1.0, -1.0, 2.0, 2.0);

                RenderTarget::pop();
            }
        }
        shader.stop();

        result.set_mag_filter(TexFilter_Linear);
        result.set_min_filter(TexFilter_LinearMipLinear);

        result
    }
}
