use glam::IVec3;

use super::{DataFormat, PixelFormat, TexFormat};
use crate::render::{
    Renderer, ResourceHandle, ResourceId, TexDesc, TexRegion, TexUsages, TexView, ViewDim,
    convert_slice,
};
use crate::rf::Rf;
use crate::system::Bytes;

#[derive(Clone)]
pub struct Tex3D {
    shared: Rf<Tex3DShared>,
}

struct Tex3DShared {
    handle: ResourceHandle,
    desc: TexDesc,
}

impl Tex3D {
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

        let mut size = (this.desc.size[0] * this.desc.size[1] * this.desc.size[2]) as i32;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let bytes = r.read_texture_3d_data(this.handle.id(), pf as u32, df as u32);

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

    fn create(r: &mut Renderer, desc: TexDesc) -> Tex3D {
        if TexFormat::is_depth(desc.format) {
            panic!("Cannot create 3D texture with depth format");
        }

        let handle = r.create_resource();
        r.create_texture(handle.id(), &desc, None);

        Tex3D {
            shared: Rf::new(Tex3DShared { handle, desc }),
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex3D {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, sx: i32, sy: i32, sz: i32, format: TexFormat) -> Tex3D {
        Self::create(r, TexDesc::d3(sx as u32, sy as u32, sz as u32, format))
    }

    /// A texture with `mips` levels (0 = the full chain) and the `TexUsage`
    /// bits in `usage` (0 = the default for the kind).
    #[bind(name = "CreateDesc")]
    pub fn new_desc(
        r: &mut Renderer,
        sx: i32,
        sy: i32,
        sz: i32,
        format: TexFormat,
        mips: i32,
        usage: u32,
    ) -> Tex3D {
        let mut desc =
            TexDesc::d3(sx as u32, sy as u32, sz as u32, format).with_mips(mips.max(0) as u32);
        if usage != 0 {
            desc = desc.with_usage(TexUsages(usage));
        }
        Self::create(r, desc)
    }

    /// View of the whole volume, for sampling.
    pub fn view(&self) -> TexView {
        let size = self.get_size_level(0);
        TexView::full(self.resource_id(), ViewDim::D3, [size.x, size.y])
    }

    /// View of one z-slice at mip level 0, usable as a render attachment.
    pub fn layer_view(&self, layer: i32) -> TexView {
        self.layer_mip_view(layer, 0)
    }

    /// View of one z-slice at the given mip level, usable as a render attachment.
    pub fn layer_mip_view(&self, layer: i32, level: i32) -> TexView {
        let size = self.get_size_level(level);
        TexView::new(
            self.resource_id(),
            ViewDim::D2Layer(layer as u16),
            level,
            [size.x, size.y],
        )
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.generate_mips(this.handle.id());
    }

    pub fn get_data_bytes(&mut self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(r, pf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.desc.format
    }

    pub fn get_size(&self) -> IVec3 {
        let this = self.shared.as_ref();
        IVec3::from_array(this.desc.size.map(|s| s as i32))
    }

    pub fn get_size_level(&self, level: i32) -> IVec3 {
        let mut out = self.get_size();
        for _ in 0..level {
            out.x /= 2;
            out.y /= 2;
            out.z /= 2;
        }
        out
    }

    pub fn set_data_bytes(
        &mut self,
        r: &mut Renderer,
        data: &mut Bytes,
        pf: PixelFormat,
        df: DataFormat,
    ) {
        self.set_data(r, data.as_slice(), pf, df);
    }
}
