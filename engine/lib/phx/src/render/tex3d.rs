use glam::IVec3;

use super::{DataFormat, PixelFormat, TexFormat};
use crate::render::{Renderer, ResourceHandle, ResourceId, TexView, ViewDim};
use crate::rf::Rf;
use crate::system::Bytes;

#[derive(Clone)]
pub struct Tex3D {
    shared: Rf<Tex3DShared>,
}

struct Tex3DShared {
    handle: ResourceHandle,
    size: IVec3,
    format: TexFormat,
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

        let mut size = this.size.x * this.size.y * this.size.z;
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
        let byte_len = std::mem::size_of_val(data);
        #[allow(unsafe_code)] // TODO: refactor
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len) };

        r.update_texture_3d_data_by_resource(
            this.handle.id(),
            this.size.x,
            this.size.y,
            this.size.z,
            this.format as i32,
            pf as u32,
            df as u32,
            bytes.to_vec(),
        );
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex3D {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, sx: i32, sy: i32, sz: i32, format: TexFormat) -> Tex3D {
        if TexFormat::is_depth(format) {
            panic!("Cannot create 3D texture with depth format");
        }

        let handle = r.create_resource();
        r.create_texture_3d(handle.id(), sx as u32, sy as u32, sz as u32, format, None);

        Tex3D {
            shared: Rf::new(Tex3DShared {
                handle,
                size: IVec3::new(sx, sy, sz),
                format,
            }),
        }
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
        r.generate_mipmap_by_resource(this.handle.id());
    }

    pub fn get_data_bytes(&mut self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(r, pf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.format
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
