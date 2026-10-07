use super::{DataFormat, PixelFormat, TexFormat};
use crate::render::{
    Renderer, ResourceHandle, ResourceId, TexDesc, TexRegion, TexUsages, TexView, ViewDim,
    convert_slice,
};
use crate::rf::Rf;
use crate::system::Bytes;

#[derive(Clone)]
pub struct Tex1D {
    shared: Rf<Tex1DShared>,
}

struct Tex1DShared {
    handle: ResourceHandle,
    desc: TexDesc,
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

        let mut size = this.desc.size[0] as i32;
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
        let bytes = convert_slice(data, pf, df, this.desc.format);
        r.update_texture(this.handle.id(), TexRegion::level(&this.desc, 0), bytes);
    }

    fn create(r: &mut Renderer, desc: TexDesc) -> Tex1D {
        let handle = r.create_resource();
        r.create_texture(handle.id(), &desc, None);

        Tex1D {
            shared: Rf::new(Tex1DShared { handle, desc }),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex1D {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, size: i32, format: TexFormat) -> Tex1D {
        Self::create(r, TexDesc::d1(size as u32, format))
    }

    /// A texture with `mips` levels (0 = the full chain) and the `TexUsage`
    /// bits in `usage` (0 = the default for the kind).
    #[bind(name = "CreateDesc")]
    pub fn new_desc(
        r: &mut Renderer,
        size: i32,
        format: TexFormat,
        mips: i32,
        usage: u32,
    ) -> Tex1D {
        let mut desc = TexDesc::d1(size as u32, format).with_mips(mips.max(0) as u32);
        if usage != 0 {
            desc = desc.with_usage(TexUsages(usage));
        }
        Self::create(r, desc)
    }

    /// View of the whole texture, for sampling.
    pub fn view(&self) -> TexView {
        TexView::full(
            self.resource_id(),
            ViewDim::D1,
            [self.shared.as_ref().desc.size[0] as i32, 1],
        )
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex1D {
        self.clone()
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.generate_mips(this.handle.id());
    }

    pub fn get_format(&mut self) -> TexFormat {
        let this = self.shared.as_ref();
        this.desc.format
    }

    pub fn get_data_bytes(&mut self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(r, pf, df))
    }

    pub fn get_size(&self) -> u32 {
        let this = self.shared.as_ref();
        this.desc.size[0]
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
}
