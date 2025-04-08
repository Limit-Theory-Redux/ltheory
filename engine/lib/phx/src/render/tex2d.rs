use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::*;
use crate::logging::warn;
use crate::math::*;
use crate::rf::Rf;
use crate::system::*;

#[derive(Clone, Debug)]
pub struct Tex2D {
    shared: Rf<Tex2DShared>,
}

#[derive(Debug)]
pub struct Tex2DShared {
    pub handle: u32,
    pub size: IVec2,
    pub format: TexFormat,
}

impl Drop for Tex2DShared {
    fn drop(&mut self) {
        if self.handle != 0 {
            glcheck!(gl::DeleteTextures(1, &self.handle));
        }
    }
}

impl Tex2DShared {
    fn init(&self) {
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32
        ));
    }
}

impl Tex2D {
    pub fn get_data<T: Clone + Default>(&self, pf: PixelFormat, df: DataFormat) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size.x * this.size.y;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let mut data = vec![T::default(); size as usize];
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_mut_ptr() as *mut _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

        data
    }

    pub fn set_data<T>(&mut self, data: &[T], pf: PixelFormat, df: DataFormat) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            this.format,
            this.size.x,
            this.size.y,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex2D {
    #[bind(name = "Create")]
    pub fn new(sx: i32, sy: i32, format: TexFormat) -> Tex2D {
        if !TexFormat_IsValid(format) {
            panic!("Invalid texture format requested");
        }

        let mut this = Tex2DShared {
            handle: 0,
            size: IVec2::new(sx, sy),
            format,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format,
            this.size.x,
            this.size.y,
            0,
            if TexFormat_IsColor(format) {
                gl::RED
            } else {
                gl::DEPTH_COMPONENT
            },
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        ));

        this.init();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

        Tex2D {
            shared: Rf::new(this),
        }
    }

    pub fn load(name: &str) -> Tex2D {
        let path = Resource::get_path(ResourceType::Tex2D, name);

        let reader = ImageReader::open(&path)
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', unable to open file"));
        let img = reader
            .decode()
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', decode failed"));
        let (width, height) = img.dimensions();

        let (format, buffer) = match img {
            DynamicImage::ImageRgba8(buf) => (gl::RGBA, buf.into_raw()),
            DynamicImage::ImageRgb8(buf) => (gl::RGB, buf.into_raw()),
            _ => panic!("Failed to load image from '{path}', unsupported image format"),
        };

        let mut this = Tex2DShared {
            handle: 0,
            size: IVec2::new(width as i32, height as i32),
            format: TexFormat_RGBA8,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            TexFormat_RGBA8 as gl::types::GLint,
            this.size.x,
            this.size.y,
            0,
            format,
            gl::UNSIGNED_BYTE,
            buffer.as_ptr() as *const _,
        ));

        this.init();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

        Tex2D {
            shared: Rf::new(this),
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex2D {
        self.clone()
    }

    pub fn screen_capture() -> Tex2D {
        let mut size: IVec2 = IVec2::ZERO;
        #[allow(unsafe_code)] // TODO: remove
        unsafe {
            Viewport_GetSize(&mut size);
        }

        let mut buf = vec![0u32; (size.x * size.y) as usize];
        glcheck!(gl::ReadPixels(
            0,
            0,
            size.x,
            size.y,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            buf.as_mut_ptr() as *mut _,
        ));

        for y in 0..(size.y / 2) {
            for x in 0..size.x {
                buf.swap(
                    (size.x * y + x) as usize,
                    (size.x * (size.y - y - 1) + x) as usize,
                );
            }
        }

        let mut this = Tex2DShared {
            handle: 0,
            size,
            format: TexFormat_RGBA8,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            TexFormat_RGBA8,
            size.x,
            size.y,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            buf.as_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

        Tex2D {
            shared: Rf::new(this),
        }
    }

    pub fn save(&mut self, path: &str) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));

        let mut buffer: ImageBuffer<Rgba<u8>, _> =
            ImageBuffer::new(this.size.x as u32, this.size.y as u32);
        glcheck!(gl::GetTexImage(
            gl::TEXTURE_2D,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            buffer.as_mut_ptr() as *mut _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

        let _ = buffer.save(path);
    }

    pub fn pop(&self) {
        RenderTarget::pop();
    }

    pub fn push(&self) {
        RenderTarget::push_tex2d(self);
    }

    pub fn push_level(&mut self, level: i32) {
        RenderTarget::push_tex2d_level(self, level);
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        RenderTarget::push_tex2d(self);
        Draw::clear(r, g, b, a);
        RenderTarget::pop();
    }

    pub fn deep_clone(&mut self) -> Tex2D {
        RenderTarget::push_tex2d(self);

        let this = self.shared.as_ref();

        let mut clone = Tex2DShared {
            handle: 0,
            size: this.size,
            format: this.format,
        };

        glcheck!(gl::GenTextures(1, &mut clone.handle));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, clone.handle));
        glcheck!(gl::CopyTexImage2D(
            gl::TEXTURE_2D,
            0,
            this.format as gl::types::GLenum,
            0,
            0,
            this.size.x,
            this.size.y,
            0,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

        RenderTarget::pop();

        Tex2D {
            shared: Rf::new(clone),
        }
    }

    pub fn gen_mipmap(&mut self) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::GenerateMipmap(gl::TEXTURE_2D));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }

    pub fn get_data_bytes(&self, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(pf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.format
    }

    pub fn get_handle(&self) -> u32 {
        let this = self.shared.as_ref();
        this.handle
    }

    pub fn get_size(&self) -> IVec2 {
        let this = self.shared.as_ref();
        this.size
    }

    pub fn get_size_level(&self, level: i32) -> IVec2 {
        let this = self.shared.as_ref();

        let mut out = this.size;
        for _ in 0..level {
            out.x /= 2;
            out.y /= 2;
        }
        out
    }

    pub fn set_anisotropy(&mut self, factor: f32) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexParameterf(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAX_ANISOTROPY_EXT,
            factor
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }

    pub fn set_data_bytes(&mut self, data: &Bytes, pf: PixelFormat, df: DataFormat) {
        self.set_data(data.as_slice(), pf, df);
    }

    pub fn set_mag_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            filter as _
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }

    pub fn set_min_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            filter as _
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }

    /* NOTE : In general, using BASE_LEVEL, MAX_LEVEL, and MIN/MAX_LOD params is
     *        dangerous due to known bugs in old Radeon & Intel drivers. See:
     *        (https://www.opengl.org/discussion_boards/showthread.php/
     *         166266-Using-GL_TEXTURE_BASE_LEVEL-with-a-comple-texture)
     *
     *        However, constraining the mip range to a single level (min_level ==
     *        max_level) seems to be acceptable even on bad drivers. Thus, it is
     *        strongly advised to use this function only to constrain sampling to
     *        a single mip level. */
    pub fn set_mip_range(&mut self, min_level: i32, max_level: i32) {
        let this = self.shared.as_ref();

        if min_level != max_level {
            warn!("Tex2D_SetMipRange: Setting mip range with min != max; this may fail on old drivers with mip-handling bugs.");
        }
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_BASE_LEVEL,
            min_level
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAX_LEVEL,
            max_level
        ));
    }

    pub fn set_texel(&mut self, x: i32, y: i32, r: f32, g: f32, b: f32, a: f32) {
        let this = self.shared.as_ref();

        let mut rgba: [f32; 4] = [r, g, b, a];

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            x,
            y,
            1,
            1,
            gl::RGBA,
            gl::FLOAT,
            rgba.as_mut_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }

    pub fn set_wrap_mode(&mut self, mode: TexWrapMode) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_S,
            mode as _
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_WRAP_T,
            mode as _
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }
}
