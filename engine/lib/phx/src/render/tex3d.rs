use super::*;
use crate::math::*;
use crate::rf::Rf;
use crate::system::*;

#[derive(Clone)]
pub struct Tex3D {
    shared: Rf<Tex3DShared>,
}

struct Tex3DShared {
    handle: u32,
    size: IVec3,
    format: TexFormat,
}

impl Drop for Tex3DShared {
    fn drop(&mut self) {
        if self.handle != 0 {
            glcheck!(gl::DeleteTextures(1, &self.handle));
        }
    }
}

impl Tex3DShared {
    fn init(&self) {
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_WRAP_S,
            gl::CLAMP_TO_EDGE as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_WRAP_T,
            gl::CLAMP_TO_EDGE as i32
        ));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_WRAP_R,
            gl::CLAMP_TO_EDGE as i32
        ));
    }
}

impl Tex3D {
    pub fn get_data<T: Clone + Default>(&self, pf: PixelFormat, df: DataFormat) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size.x * this.size.y * this.size.z;
        size *= DataFormat_GetSize(df);
        size *= PixelFormat_Components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let mut data = vec![T::default(); size as usize];
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::GetTexImage(
            gl::TEXTURE_3D,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_mut_ptr() as *mut _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));

        data
    }

    pub fn set_data<T>(&mut self, data: &[T], pf: PixelFormat, df: DataFormat) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::TexImage3D(
            gl::TEXTURE_3D,
            0,
            this.format,
            this.size.x,
            this.size.y,
            this.size.z,
            0,
            pf as gl::types::GLenum,
            df as gl::types::GLenum,
            data.as_ptr() as *const _,
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex3D {
    #[bind(name = "Create")]
    pub fn new(sx: i32, sy: i32, sz: i32, format: TexFormat) -> Tex3D {
        if !TexFormat_IsValid(format) {
            panic!("Invalid texture format requested");
        }

        if TexFormat_IsDepth(format) {
            panic!("Cannot create 3D texture with depth format");
        }

        let mut this = Tex3DShared {
            handle: 0,
            size: IVec3::new(sx, sy, sz),
            format,
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::ActiveTexture(gl::TEXTURE0));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::TexImage3D(
            gl::TEXTURE_3D,
            0,
            this.format,
            this.size.x,
            this.size.y,
            this.size.z,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        ));

        this.init();

        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));

        Tex3D {
            shared: Rf::new(this),
        }
    }

    pub fn pop(&self) {
        unsafe {
            RenderTarget_Pop();
        }
    }

    pub fn push(&self, layer: i32) {
        unsafe {
            RenderTarget_PushTex3D(self, layer);
        }
    }

    pub fn push_level(&self, layer: i32, level: i32) {
        unsafe {
            RenderTarget_PushTex3DLevel(self, layer, level);
        }
    }

    pub fn gen_mipmap(&mut self) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::GenerateMipmap(gl::TEXTURE_3D));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
    }

    pub fn get_data_bytes(&mut self, pf: PixelFormat, df: DataFormat) -> Bytes {
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

    pub fn get_size(&self) -> IVec3 {
        let this = self.shared.as_ref();
        this.size
    }

    pub fn get_size_level(&self, level: i32) -> IVec3 {
        let this = self.shared.as_ref();

        let mut out = this.size;
        for _ in 0..level {
            out.x /= 2;
            out.y /= 2;
            out.z /= 2;
        }
        out
    }

    pub fn set_data_bytes(&mut self, data: &mut Bytes, pf: PixelFormat, df: DataFormat) {
        self.set_data(data.as_slice(), pf, df);
    }

    pub fn set_mag_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_MAG_FILTER,
            filter
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
    }

    pub fn set_min_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::TexParameteri(
            gl::TEXTURE_3D,
            gl::TEXTURE_MIN_FILTER,
            filter
        ));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
    }

    pub fn set_wrap_mode(&mut self, mode: TexWrapMode) {
        let this = self.shared.as_ref();

        glcheck!(gl::BindTexture(gl::TEXTURE_3D, this.handle));
        glcheck!(gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_S, mode));
        glcheck!(gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_T, mode));
        glcheck!(gl::TexParameteri(gl::TEXTURE_3D, gl::TEXTURE_WRAP_R, mode));
        glcheck!(gl::BindTexture(gl::TEXTURE_3D, 0));
    }
}
