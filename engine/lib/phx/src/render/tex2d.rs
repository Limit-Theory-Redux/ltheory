use crossbeam::channel::bounded;
use glam::{IVec2, Vec3};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::{DataFormat, Draw, PixelFormat, RenderTarget, TexFilter, TexFormat, TexWrapMode};
use crate::render::{RenderCommand, Renderer, ResourceHandle, ResourceId, Viewport, gl};
use crate::rf::Rf;
use crate::system::{Bytes, Resource, ResourceType};

#[derive(Clone, Debug)]
pub struct Tex2D {
    shared: Rf<Tex2DShared>,
}

#[derive(Debug)]
pub struct Tex2DShared {
    handle: ResourceHandle,
    pub size: IVec2,
    pub format: TexFormat,
}

impl Tex2D {
    pub fn resource_id(&self) -> ResourceId {
        self.shared.as_ref().handle.id()
    }

    pub fn get_data<T: Clone + Default>(&self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.size.x * this.size.y;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let (tx, rx) = bounded(1);
        r.submit(RenderCommand::ReadTexture2DData {
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

        r.submit(RenderCommand::UpdateTexture2DDataByResource {
            id: this.handle.id(),
            width: this.size.x,
            height: this.size.y,
            internal_format: this.format as i32,
            pixel_format: pf as u32,
            data_format: df as u32,
            data: bytes.to_vec(),
        });
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex2D {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, sx: i32, sy: i32, format: TexFormat) -> Tex2D {
        let handle = r.create_resource();
        r.submit(RenderCommand::CreateTexture2D {
            id: handle.id(),
            width: sx as u32,
            height: sy as u32,
            format,
            data: None,
        });

        Tex2D {
            shared: Rf::new(Tex2DShared {
                handle,
                size: IVec2::new(sx, sy),
                format,
            }),
        }
    }

    pub fn load(r: &mut Renderer, name: &str) -> Tex2D {
        let path = Resource::get_path(ResourceType::Tex2D, name);

        let reader = ImageReader::open(&path)
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', unable to open file"));
        let img = reader
            .decode()
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', decode failed"));
        let (width, height) = img.dimensions();

        let (pixel_format, buffer) = match img {
            DynamicImage::ImageRgba8(buf) => (gl::RGBA, buf.into_raw()),
            DynamicImage::ImageRgb8(buf) => (gl::RGB, buf.into_raw()),
            _ => panic!("Failed to load image from '{path}', unsupported image format"),
        };

        let size = IVec2::new(width as i32, height as i32);
        let format = TexFormat::RGBA8;
        let handle = r.create_resource();
        // Create empty first, then upload with the source image's own pixel
        // format (RGB or RGBA, whichever it decoded as) - the internal
        // storage format (RGBA8) doesn't have to match the source layout,
        // GL converts on upload.
        r.submit(RenderCommand::CreateTexture2D {
            id: handle.id(),
            width: size.x as u32,
            height: size.y as u32,
            format,
            data: None,
        });
        r.submit(RenderCommand::UpdateTexture2DDataByResource {
            id: handle.id(),
            width: size.x,
            height: size.y,
            internal_format: format as i32,
            pixel_format,
            data_format: gl::UNSIGNED_BYTE,
            data: buffer,
        });

        Tex2D {
            shared: Rf::new(Tex2DShared {
                handle,
                size,
                format,
            }),
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex2D {
        self.clone()
    }

    pub fn screen_capture(r: &mut Renderer) -> Tex2D {
        let size: IVec2 = Viewport::get_size(r);

        let (tx, rx) = bounded(1);
        r.submit(RenderCommand::ReadFramebufferPixels {
            x: 0,
            y: 0,
            width: size.x,
            height: size.y,
            reply_tx: tx,
        });
        let raw = rx.recv().unwrap_or_default();

        // Flip vertically (framebuffer readback is bottom-up).
        let stride = (size.x * 4) as usize;
        let mut buf = vec![0u8; raw.len()];
        for y in 0..size.y as usize {
            if let (Some(src), Some(dst_y)) = (
                raw.get(y * stride..(y + 1) * stride),
                (size.y as usize).checked_sub(1 + y),
            ) {
                buf[dst_y * stride..(dst_y + 1) * stride].copy_from_slice(src);
            }
        }

        let handle = r.create_resource();
        r.submit(RenderCommand::CreateTexture2D {
            id: handle.id(),
            width: size.x as u32,
            height: size.y as u32,
            format: TexFormat::RGBA8,
            data: Some(buf),
        });

        Tex2D {
            shared: Rf::new(Tex2DShared {
                handle,
                size,
                format: TexFormat::RGBA8,
            }),
        }
    }

    pub fn save(&mut self, r: &mut Renderer, path: &str) {
        let size = self.shared.as_ref().size;
        let data: Vec<u8> = self.get_data(r, PixelFormat::RGBA, DataFormat::U8);

        if let Some(buffer) =
            ImageBuffer::<Rgba<u8>, _>::from_raw(size.x as u32, size.y as u32, data)
        {
            let _ = buffer.save(path);
        }
    }

    pub fn pop(&self, r: &mut Renderer) {
        RenderTarget::pop(r);
    }

    pub fn push(&self, r: &mut Renderer) {
        RenderTarget::push_tex2d(r, self);
    }

    pub fn push_level(&mut self, r: &mut Renderer, level: i32) {
        RenderTarget::push_tex2d_level(r, self, level);
    }

    pub fn clear(&mut self, r: &mut Renderer, red: f32, green: f32, blue: f32, alpha: f32) {
        RenderTarget::push_tex2d(r, self);
        Draw::clear(r, red, green, blue, alpha);
        RenderTarget::pop(r);
    }

    pub fn deep_clone(&mut self, r: &mut Renderer) -> Tex2D {
        RenderTarget::push_tex2d(r, self);

        let this = self.shared.as_ref();
        let size = this.size;
        let format = this.format;

        let handle = r.create_resource();
        r.submit(RenderCommand::CreateTexture2D {
            id: handle.id(),
            width: size.x as u32,
            height: size.y as u32,
            format,
            data: None,
        });
        r.submit(RenderCommand::CopyTexture2DFromFramebufferByResource {
            id: handle.id(),
            internal_format: format as i32,
            width: size.x,
            height: size.y,
        });

        RenderTarget::pop(r);

        Tex2D {
            shared: Rf::new(Tex2DShared {
                handle,
                size,
                format,
            }),
        }
    }

    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::GenerateMipmapByResource { id: this.handle.id() });
    }

    pub fn get_data_bytes(&self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(r, pf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.format
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

    pub fn set_anisotropy(&mut self, r: &mut Renderer, factor: f32) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTexture2DAnisotropyByResource {
            id: this.handle.id(),
            factor,
        });
    }

    pub fn set_data_bytes(&mut self, r: &mut Renderer, data: &Bytes, pf: PixelFormat, df: DataFormat) {
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

    /* NOTE : In general, using BASE_LEVEL, MAX_LEVEL, and MIN/MAX_LOD params is
     *        dangerous due to known bugs in old Radeon & Intel drivers. See:
     *        (https://www.opengl.org/discussion_boards/showthread.php/
     *         166266-Using-GL_TEXTURE_BASE_LEVEL-with-a-comple-texture)
     *
     *        However, constraining the mip range to a single level (min_level ==
     *        max_level) seems to be acceptable even on bad drivers. Thus, it is
     *        strongly advised to use this function only to constrain sampling to
     *        a single mip level. */
    pub fn set_mip_range(&mut self, r: &mut Renderer, min_level: i32, max_level: i32) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTexture2DMipRangeByResource {
            id: this.handle.id(),
            min_level,
            max_level,
        });
    }

    pub fn set_texel(&mut self, r: &mut Renderer, x: i32, y: i32, red: f32, green: f32, blue: f32, alpha: f32) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTexel2DByResource {
            id: this.handle.id(),
            x,
            y,
            color: [red, green, blue, alpha],
        });
    }

    pub fn set_wrap_mode(&mut self, r: &mut Renderer, mode: TexWrapMode) {
        let this = self.shared.as_ref();
        r.submit(RenderCommand::SetTextureWrapModeByResource {
            id: this.handle.id(),
            mode,
        });
    }

    /// Sample a single pixel at integer coordinates (x, y)
    /// Coordinates are in OpenGL convention: (0,0) = bottom-left
    /// Returns Vec3f with RGB in [0.0, 1.0] range
    #[bind(name = "Sample")]
    fn sample_pixel(&self, r: &mut Renderer, x: i32, y: i32) -> Vec3 {
        let this = self.shared.as_ref();
        let size = this.size;

        let x = x.clamp(0, size.x - 1);
        let y = y.clamp(0, size.y - 1);

        // Flip Y for OpenGL bottom-left origin
        let gl_y = size.y - 1 - y;

        let (tx, rx) = bounded(1);
        r.submit(RenderCommand::SamplePixel2DByResource {
            id: this.handle.id(),
            x,
            y: gl_y,
            reply_tx: tx,
        });
        let pixel = rx.recv().unwrap_or([0; 4]);

        Vec3::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        )
    }
}
