use super::{DataFormat, PixelFormat, TexFilter, TexFormat, TexWrapMode};
use crate::render::{Renderer, ResourceHandle, ResourceId};
use crate::rf::Rf;
use crate::system::Bytes;

#[derive(Clone)]
pub struct Tex1D {
    shared: Rf<Tex1DShared>,
}

struct Tex1DShared {
    handle: ResourceHandle,
    size: i32,
    format: TexFormat,
}

impl Tex1D {
    pub fn resource_id(&self) -> ResourceId {
        self.shared.as_ref().handle.id()
    }

    pub fn get_data<T: Clone + Default>(
        &self,
        r: &mut Renderer,
        pf: PixelFormat,
        df: DataFormat,
    ) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let bytes = r.read_texture_1d_data(this.handle.id(), pf as u32, df as u32);

        let mut data = vec![T::default(); size as usize];
        let byte_len = (data.len() * std::mem::size_of::<T>()).min(bytes.len());
        #[allow(unsafe_code)] // TODO: refactor
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr() as *mut u8, byte_len);
        }

        data
    }

    pub fn set_data<T>(&mut self, r: &mut Renderer, data: &[T], pf: PixelFormat, df: DataFormat) {
        let this = self.shared.as_ref();
        let byte_len = std::mem::size_of_val(data);
        #[allow(unsafe_code)] // TODO: refactor
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len) };

        r.update_texture_1d_data_by_resource(
            this.handle.id(),
            this.size,
            this.format as i32,
            pf as u32,
            df as u32,
            bytes.to_vec(),
        );
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex1D {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, size: i32, format: TexFormat) -> Tex1D {
        let handle = r.create_resource();
        r.create_texture_1d(handle.id(), size as u32, format, None);

        Tex1D {
            shared: Rf::new(Tex1DShared {
                handle,
                size,
                format,
            }),
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex1D {
        self.clone()
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.generate_mipmap_by_resource(this.handle.id());
    }

    pub fn get_format(&mut self) -> TexFormat {
        let this = self.shared.as_ref();
        this.format
    }

    pub fn get_data_bytes(&mut self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(r, pf, df))
    }

    pub fn get_size(&self) -> u32 {
        let this = self.shared.as_ref();
        this.size as u32
    }

    pub fn set_data_bytes(
        &mut self,
        r: &mut Renderer,
        data: &Bytes,
        pf: PixelFormat,
        df: DataFormat,
    ) {
        self.set_data(r, data.as_slice(), pf, df);
    }

    pub fn set_mag_filter(&mut self, r: &mut Renderer, filter: TexFilter) {
        let this = self.shared.as_ref();
        r.set_texture_mag_filter_by_resource(this.handle.id(), filter);
    }

    pub fn set_min_filter(&mut self, r: &mut Renderer, filter: TexFilter) {
        let this = self.shared.as_ref();
        r.set_texture_min_filter_by_resource(this.handle.id(), filter);
    }

    pub fn set_texel(
        &mut self,
        r: &mut Renderer,
        x: i32,
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    ) {
        let this = self.shared.as_ref();
        r.set_texel_1d_by_resource(this.handle.id(), x, [red, green, blue, alpha]);
    }

    pub fn set_wrap_mode(&mut self, r: &mut Renderer, mode: TexWrapMode) {
        let this = self.shared.as_ref();
        r.set_texture_wrap_mode_by_resource(this.handle.id(), mode);
    }
}
