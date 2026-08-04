use crossbeam::channel::bounded;
use glam::IVec3;

use super::{DataFormat, PixelFormat, RenderTarget, TexFilter, TexFormat, TexWrapMode};
use crate::render::{RenderCommand, Renderer, ResourceHandle, ResourceId};
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

    pub fn get_data<T: Clone + Default>(&self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size.x * this.size.y * this.size.z;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let (tx, rx) = bounded(1);
        r.submit(RenderCommand::ReadTexture3DData {
            id: this.handle.id(),
            pixel_format: pf as u32,
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

    pub fn set_data<T>(&mut self, r: &mut Renderer, data: &[T], pf: PixelFormat, df: DataFormat) {
        let this = self.shared.as_ref();
        let byte_len = std::mem::size_of_val(data);
        #[allow(unsafe_code)] // TODO: refactor
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len) };

        r.submit(RenderCommand::UpdateTexture3DDataByResource {
            id: this.handle.id(),
            width: this.size.x,
            height: this.size.y,
            depth: this.size.z,
            internal_format: this.format as i32,
            pixel_format: pf as u32,
            data_format: df as u32,
            data: bytes.to_vec(),
        });
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
        r.submit(RenderCommand::CreateTexture3D {
            id: handle.id(),
            width: sx as u32,
            height: sy as u32,
            depth: sz as u32,
            format,
            data: None,
        });

        Tex3D {
            shared: Rf::new(Tex3DShared {
                handle,
                size: IVec3::new(sx, sy, sz),
                format,
            }),
        }
    }

    pub fn pop(&self, r: &mut Renderer) {
        RenderTarget::pop(r);
    }

    pub fn push(&self, r: &mut Renderer, layer: i32) {
        RenderTarget::push_tex3d(r, self, layer);
    }

    pub fn push_level(&self, r: &mut Renderer, layer: i32, level: i32) {
        RenderTarget::push_tex3d_level(r, self, layer, level);
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::GenerateMipmapByResource { id: this.handle.id() });
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

    pub fn set_data_bytes(&mut self, r: &mut Renderer, data: &mut Bytes, pf: PixelFormat, df: DataFormat) {
        self.set_data(r, data.as_slice(), pf, df);
    }

    pub fn set_mag_filter(&mut self, r: &mut Renderer, filter: TexFilter) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTextureMagFilterByResource {
            id: this.handle.id(),
            filter,
        });
    }

    pub fn set_min_filter(&mut self, r: &mut Renderer, filter: TexFilter) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTextureMinFilterByResource {
            id: this.handle.id(),
            filter,
        });
    }

    pub fn set_wrap_mode(&mut self, r: &mut Renderer, mode: TexWrapMode) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTextureWrapModeByResource {
            id: this.handle.id(),
            mode,
        });
    }
}
