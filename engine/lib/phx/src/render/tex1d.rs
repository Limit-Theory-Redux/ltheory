use super::*;
use crate::rf::Rf;
use crate::system::*;

#[derive(Clone)]
pub struct Tex1D {
    shared: Rf<Tex1DShared>,
}

struct Tex1DShared {
    handle: u32,
    size: i32,
    format: TexFormat,
}

impl Drop for Tex1DShared {
    fn drop(&mut self) {
        if self.handle != 0 {
            glcheck!(gl::DeleteTextures(1, &self.handle));
        }
    }
}

impl Tex1DShared {
    fn init(&self) {
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_1D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_1D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_1D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32
        ));
    }
}

impl Tex1D {
    pub fn get_data<T: Clone + Default>(&self, pf: PixelFormat, df: DataFormat) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let mut data = vec![T::default(); size as usize];
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::GetTexImage(
            gl::TEXTURE_1D,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_mut_ptr() as *mut _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));

        data
    }

    pub fn set_data<T>(&mut self, data: &[T], pf: PixelFormat, df: DataFormat) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::TexImage1D(
            gl::TEXTURE_1D,
            0,
            this.format,
            this.size,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex1D {
    #[bind(name = "Create")]
    pub fn new(size: i32, format: TexFormat) -> Tex1D {
        if !TexFormat_IsValid(format) {
            panic!("Invalid texture format requested");
        }

        let mut this = Tex1DShared {
            handle: 0,
            size,
            format,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::TexImage1D(
            gl::TEXTURE_1D,
            0,
            format,
            size,
            0,
            (if TexFormat_IsColor(format) as i32 != 0 {
                gl::RED
            } else {
                gl::DEPTH_COMPONENT
            }) as gl::types::GLenum,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        ));

        this.init();

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));

        Tex1D {
            shared: Rf::new(this),
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex1D {
        self.clone()
    }

    pub fn gen_mipmap(&mut self) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::GenerateMipmap(gl::TEXTURE_1D));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
    }

    pub fn get_format(&mut self) -> TexFormat {
        let this = self.shared.as_ref();

        this.format
    }

    pub fn get_data_bytes(&mut self, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(pf, df))
    }

    pub fn get_handle(&self) -> u32 {
        let this = self.shared.as_ref();
        this.handle
    }

    pub fn get_size(&self) -> u32 {
        let this = self.shared.as_ref();
        this.size as u32
    }

    pub fn set_data_bytes(&mut self, data: &Bytes, pf: PixelFormat, df: DataFormat) {
        self.set_data(data.as_slice(), pf, df);
    }

    pub fn set_mag_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_1D,
            gl::TEXTURE_MAG_FILTER,
            filter
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
    }

    pub fn set_min_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_1D,
            gl::TEXTURE_MIN_FILTER,
            filter
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
    }

    pub fn set_texel(&mut self, x: i32, r: f32, g: f32, b: f32, a: f32) {
        let this = self.shared.as_ref();

        let mut rgba: [f32; 4] = [r, g, b, a];

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::TexSubImage1D(
            gl::TEXTURE_1D,
            0,
            x,
            1,
            gl::RGBA,
            gl::FLOAT,
            rgba.as_mut_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
    }

    pub fn set_wrap_mode(&mut self, mode: TexWrapMode) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_1D, this.handle));
        glcheck!(gl::TexParameteri(gl::TEXTURE_1D, gl::TEXTURE_WRAP_S, mode));
        glcheck!(gl::BindTexture(gl::TEXTURE_1D, 0));
    }
}
