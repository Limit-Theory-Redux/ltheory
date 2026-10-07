use glam::{IVec2, Vec3};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::{DataFormat, PixelFormat, TexFormat};
use crate::render::{
    LoadOp, RenderPassDesc, Renderer, ResourceHandle, ResourceId, TexDesc, TexRegion, TexUsages,
    TexView, ViewDim, convert_slice,
};
use crate::rf::Rf;
use crate::system::{Bytes, Resource, ResourceType};

#[derive(Clone, Debug)]
pub struct Tex2D {
    shared: Rf<Tex2DShared>,
}

#[derive(Debug)]
pub struct Tex2DShared {
    handle: ResourceHandle,
    pub desc: TexDesc,
}

impl Tex2DShared {
    fn size(&self) -> IVec2 {
        IVec2::new(self.desc.size[0] as i32, self.desc.size[1] as i32)
    }
}

impl Tex2D {
    pub fn resource_id(&self) -> ResourceId {
        self.shared.as_ref().handle.id()
    }

    /// A texture of `desc`, optionally with level 0 in the texture's own
    /// format (tightly packed rows).
    pub fn create(r: &mut Renderer, desc: TexDesc, bytes: Option<Vec<u8>>) -> Tex2D {
        let handle = r.create_resource();
        r.create_texture(handle.id(), &desc, bytes);

        Tex2D {
            shared: Rf::new(Tex2DShared { handle, desc }),
        }
    }

    pub fn get_data<T: Clone + Default>(
        &self,
        r: &mut Renderer,
        pf: PixelFormat,
        df: DataFormat,
    ) -> Vec<T> {
        let this = self.shared.as_ref();

        let mut size = this.desc.size[0] as i32 * this.desc.size[1] as i32;
        size *= DataFormat::get_size(df);
        size *= PixelFormat::components(pf);
        size /= std::mem::size_of::<T>() as i32;

        let bytes = r.read_texture_2d_data(this.handle.id(), pf as u32, df as u32);

        let mut data = vec![T::default(); size as usize];
        let byte_len = (data.len() * std::mem::size_of::<T>()).min(bytes.len());
        #[allow(unsafe_code)] // TODO: refactor
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.as_mut_ptr() as *mut u8, byte_len);
        }

        data
    }

    /// A texture of `format` created with `bytes` (tightly packed rows in the
    /// texture's own format).
    pub fn new_with_bytes(
        r: &mut Renderer,
        sx: i32,
        sy: i32,
        format: TexFormat,
        bytes: Vec<u8>,
    ) -> Tex2D {
        Self::create(r, TexDesc::d2(sx as u32, sy as u32, format), Some(bytes))
    }

    /// Replace a rectangle of a one-byte-per-texel texture (tightly packed rows).
    pub fn update_rect_bytes(
        &self,
        r: &mut Renderer,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        bytes: Vec<u8>,
    ) {
        let this = self.shared.as_ref();
        debug_assert_eq!(this.desc.format, TexFormat::R8);
        r.update_texture(
            this.handle.id(),
            TexRegion::rect(x as u32, y as u32, width as u32, height as u32),
            bytes,
        );
    }

    pub fn set_data<T>(&mut self, r: &mut Renderer, data: &[T], pf: PixelFormat, df: DataFormat) {
        let this = self.shared.as_ref();
        let bytes = convert_slice(data, pf, df, this.desc.format);
        r.update_texture(this.handle.id(), TexRegion::level(&this.desc, 0), bytes);
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex2D {
    #[bind(name = "Create")]
    pub fn new(r: &mut Renderer, sx: i32, sy: i32, format: TexFormat) -> Tex2D {
        Self::create(r, TexDesc::d2(sx as u32, sy as u32, format), None)
    }

    /// A texture with `mips` levels (0 = the full chain) and the `TexUsage`
    /// bits in `usage` (0 = the default for the kind).
    #[bind(name = "CreateDesc")]
    pub fn new_desc(
        r: &mut Renderer,
        sx: i32,
        sy: i32,
        format: TexFormat,
        mips: i32,
        usage: u32,
    ) -> Tex2D {
        let mut desc = TexDesc::d2(sx as u32, sy as u32, format).with_mips(mips.max(0) as u32);
        if usage != 0 {
            desc = desc.with_usage(TexUsages(usage));
        }
        Self::create(r, desc, None)
    }

    pub fn load(r: &mut Renderer, name: &str) -> Tex2D {
        let path = Resource::get_path(ResourceType::Tex2D, name);

        let reader = ImageReader::open(&path)
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', unable to open file"));
        let img = reader
            .decode()
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', decode failed"));
        let (width, height) = img.dimensions();

        // Textures are RGBA8: an RGB image gets an opaque alpha, which is what
        // the GL upload of RGB data into an RGBA8 texture did.
        let buffer = match img {
            DynamicImage::ImageRgba8(buf) => buf.into_raw(),
            DynamicImage::ImageRgb8(buf) => DynamicImage::ImageRgb8(buf).into_rgba8().into_raw(),
            _ => panic!("Failed to load image from '{path}', unsupported image format"),
        };

        Self::create(
            r,
            TexDesc::d2(width, height, TexFormat::RGBA8),
            Some(buffer),
        )
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex2D {
        self.clone()
    }

    pub fn screen_capture(r: &mut Renderer) -> Tex2D {
        let size: IVec2 = r.target_size();

        let raw = r.read_framebuffer_pixels(0, 0, size.x, size.y);

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

        Self::create(
            r,
            TexDesc::d2(size.x as u32, size.y as u32, TexFormat::RGBA8),
            Some(buf),
        )
    }

    pub fn save(&mut self, r: &mut Renderer, path: &str) {
        let size = self.shared.as_ref().size();
        let data: Vec<u8> = self.get_data(r, PixelFormat::RGBA, DataFormat::U8);

        if let Some(buffer) =
            ImageBuffer::<Rgba<u8>, _>::from_raw(size.x as u32, size.y as u32, data)
        {
            let _ = buffer.save(path);
        }
    }

    /// View of the whole texture: mip level 0 as a render attachment, every
    /// level when sampled (`TexView:mips` narrows it).
    pub fn view(&self) -> TexView {
        let size = self.get_size_level(0);
        TexView::full(self.resource_id(), ViewDim::D2, [size.x, size.y])
    }

    /// View of one mip level, usable as a render attachment.
    pub fn mip_view(&self, level: i32) -> TexView {
        let size = self.get_size_level(level);
        TexView::new(self.resource_id(), ViewDim::D2, level, [size.x, size.y])
    }

    pub fn clear(&mut self, r: &mut Renderer, red: f32, green: f32, blue: f32, alpha: f32) {
        let desc = RenderPassDesc::with_color(
            "Tex2D.clear",
            self.view(),
            LoadOp::Clear,
            [red, green, blue, alpha],
        );
        r.begin_pass_intern(&desc);
        r.end_pass_intern();
    }

    pub fn deep_clone(&mut self, r: &mut Renderer) -> Tex2D {
        let desc =
            RenderPassDesc::with_color("Tex2D.deepClone", self.view(), LoadOp::Load, [0.0; 4]);
        r.begin_pass_intern(&desc);

        let this = self.shared.as_ref();
        let size = this.size();
        let format = this.desc.format;

        let result = Self::create(r, TexDesc::d2(size.x as u32, size.y as u32, format), None);
        r.copy_texture_2d_from_framebuffer_by_resource(
            result.resource_id(),
            format,
            size.x,
            size.y,
        );

        r.end_pass_intern();

        result
    }

    /// Fill the mip levels below 0 from level 0.
    pub fn gen_mipmap(&mut self, r: &mut Renderer) {
        let this = self.shared.as_ref();
        r.generate_mips(this.handle.id());
    }

    pub fn get_data_bytes(&self, r: &mut Renderer, pf: PixelFormat, df: DataFormat) -> Bytes {
        Bytes::from_vec(self.get_data(r, pf, df))
    }

    pub fn get_format(&self) -> TexFormat {
        let this = self.shared.as_ref();
        this.desc.format
    }

    pub fn get_size(&self) -> IVec2 {
        let this = self.shared.as_ref();
        this.size()
    }

    pub fn get_size_level(&self, level: i32) -> IVec2 {
        let this = self.shared.as_ref();

        let mut out = this.size();
        for _ in 0..level {
            out.x /= 2;
            out.y /= 2;
        }
        out
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
        y: i32,
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    ) {
        let this = self.shared.as_ref();
        r.set_texel_2d_by_resource(this.handle.id(), x, y, [red, green, blue, alpha]);
    }

    /// Sample a single pixel at integer coordinates (x, y)
    /// Coordinates are in OpenGL convention: (0,0) = bottom-left
    /// Returns Vec3f with RGB in [0.0, 1.0] range
    #[bind(name = "Sample")]
    fn sample_pixel(&self, r: &mut Renderer, x: i32, y: i32) -> Vec3 {
        let this = self.shared.as_ref();
        let size = this.size();

        let x = x.clamp(0, size.x - 1);
        let y = y.clamp(0, size.y - 1);

        // Flip Y for OpenGL bottom-left origin
        let gl_y = size.y - 1 - y;

        let pixel = r.sample_pixel_2d_by_resource(this.handle.id(), x, gl_y);

        Vec3::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        )
    }
}
