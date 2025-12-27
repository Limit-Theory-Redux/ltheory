use glam::{IVec2, Vec3};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};

use super::{DataFormat, Draw, PixelFormat, RenderTarget, TexFilter, TexFormat, TexWrapMode};
use crate::logging::warn;
use crate::render::{Viewport, gl, glcheck};
use crate::render::{is_command_mode, submit_command, next_resource_id, GpuHandle, RenderCommand, ResourceId};
use crate::rf::Rf;
use crate::system::{Bytes, Resource, ResourceType};

#[derive(Clone, Debug)]
pub struct Tex2D {
    shared: Rf<Tex2DShared>,
}

#[derive(Debug)]
pub struct Tex2DShared {
    pub handle: u32,
    pub size: IVec2,
    pub format: TexFormat,
    /// ResourceId for textures created in command mode (render thread owns the actual GL handle)
    pub resource_id: Option<ResourceId>,
    /// Cached pixel data for lazy recreation on render thread
    /// This allows textures created before command mode to be recreated when needed
    pub cached_data: Option<CachedTextureData>,
}

/// Cached texture data with format information for correct recreation
#[derive(Debug, Clone)]
pub struct CachedTextureData {
    pub data: Vec<u8>,
    pub pixel_format: PixelFormat,
    pub data_format: DataFormat,
}

impl Drop for Tex2DShared {
    fn drop(&mut self) {
        // If this texture was created in command mode, send destroy command
        if let Some(id) = self.resource_id {
            if is_command_mode() {
                submit_command(RenderCommand::DestroyResource { id });
            }
            // Note: If not in command mode anymore, the resource may already be cleaned up
        } else if self.handle != 0 {
            // Direct GL texture - only delete if we have a valid handle
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

        // NOTE: GPU read-back is inherently synchronous and requires GL context.
        // In command mode, this operation uses direct GL which may cause issues.
        // TODO: Implement proper synchronization for command mode read-back.
        if is_command_mode() {
            warn!("Tex2D::get_data called in command mode - this requires synchronization with render thread");
        }

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

    #[allow(unsafe_code)]
    pub fn set_data<T>(&mut self, data: &[T], pf: PixelFormat, df: DataFormat) {
        let mut this = self.shared.as_mut();

        // Convert data to bytes for caching and command buffer
        // Safety: We're just reinterpreting the slice as bytes for serialization
        let data_bytes: Vec<u8> = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<T>(),
            )
            .to_vec()
        };

        // Cache the data with format info for potential lazy recreation on render thread
        this.cached_data = Some(CachedTextureData {
            data: data_bytes.clone(),
            pixel_format: pf,
            data_format: df,
        });

        if is_command_mode() {
            // In command mode, we need a resource_id to update the texture on the render thread
            if let Some(id) = this.resource_id {
                // Texture already exists on render thread - just update the data
                submit_command(RenderCommand::UpdateTexture2DDataByResource {
                    id,
                    width: this.size.x,
                    height: this.size.y,
                    internal_format: this.format as i32,
                    pixel_format: pf as u32,
                    data_format: df as u32,
                    data: data_bytes,
                });
            } else {
                // Texture was created before command mode - create it on render thread WITH the data
                // This both migrates the texture AND sets its data in one command
                let id = next_resource_id();
                submit_command(RenderCommand::CreateTexture2D {
                    id,
                    width: this.size.x as u32,
                    height: this.size.y as u32,
                    format: this.format,
                    data: Some(data_bytes), // Include the data in creation
                });
                this.resource_id = Some(id);
            }
        } else {
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                this.format as _,
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
}

#[luajit_ffi_gen::luajit_ffi]
impl Tex2D {
    #[bind(name = "Create")]
    pub fn new(sx: i32, sy: i32, format: TexFormat) -> Tex2D {
        if is_command_mode() {
            // Command mode: create texture on render thread
            let resource_id = next_resource_id();

            submit_command(RenderCommand::CreateTexture2D {
                id: resource_id,
                width: sx as u32,
                height: sy as u32,
                format,
                data: None,
            });

            let this = Tex2DShared {
                handle: 0, // No valid GL handle on main thread
                size: IVec2::new(sx, sy),
                format,
                resource_id: Some(resource_id),
                cached_data: None, // No need to cache, already on render thread
            };

            Tex2D {
                shared: Rf::new(this),
            }
        } else {
            // Direct GL mode: create texture immediately
            let mut this = Tex2DShared {
                handle: 0,
                size: IVec2::new(sx, sy),
                format,
                resource_id: None,
                cached_data: None, // Will be populated when set_data is called
            };

            glcheck!(gl::GenTextures(1, &mut this.handle));
            glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as _,
                this.size.x,
                this.size.y,
                0,
                if TexFormat::is_color(format) {
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
    }

    pub fn load(name: &str) -> Tex2D {
        let path = Resource::get_path(ResourceType::Tex2D, name);

        let reader = ImageReader::open(&path)
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', unable to open file"));
        let img = reader
            .decode()
            .unwrap_or_else(|_| panic!("Failed to load image from '{path}', decode failed"));
        let (width, height) = img.dimensions();

        let (pixel_format, buffer) = match img {
            DynamicImage::ImageRgba8(buf) => (PixelFormat::RGBA, buf.into_raw()),
            DynamicImage::ImageRgb8(buf) => (PixelFormat::RGB, buf.into_raw()),
            _ => panic!("Failed to load image from '{path}', unsupported image format"),
        };

        if is_command_mode() {
            // Command mode: create texture on render thread with data
            let resource_id = next_resource_id();

            // Create empty texture first
            submit_command(RenderCommand::CreateTexture2D {
                id: resource_id,
                width,
                height,
                format: TexFormat::RGBA8,
                data: None,
            });

            // Then upload data with correct format
            submit_command(RenderCommand::UpdateTexture2DDataByResource {
                id: resource_id,
                width: width as i32,
                height: height as i32,
                internal_format: TexFormat::RGBA8 as i32,
                pixel_format: pixel_format as u32,
                data_format: DataFormat::U8 as u32,
                data: buffer,
            });

            let this = Tex2DShared {
                handle: 0,
                size: IVec2::new(width as i32, height as i32),
                format: TexFormat::RGBA8,
                resource_id: Some(resource_id),
                cached_data: None, // Already on render thread
            };

            Tex2D {
                shared: Rf::new(this),
            }
        } else {
            // Direct GL mode
            let mut this = Tex2DShared {
                handle: 0,
                size: IVec2::new(width as i32, height as i32),
                format: TexFormat::RGBA8,
                resource_id: None,
                cached_data: Some(CachedTextureData {
                    data: buffer.clone(),
                    pixel_format,
                    data_format: DataFormat::U8,
                }),
            };

            glcheck!(gl::GenTextures(1, &mut this.handle));
            glcheck!(gl::ActiveTexture(gl::TEXTURE0));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                TexFormat::RGBA8 as gl::types::GLint,
                this.size.x,
                this.size.y,
                0,
                pixel_format as gl::types::GLenum,
                gl::UNSIGNED_BYTE,
                buffer.as_ptr() as *const _,
            ));

            this.init();

            glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));

            Tex2D {
                shared: Rf::new(this),
            }
        }
    }

    // This simply forwards calls from Lua to the Clone trait.
    #[bind(name = "Clone")]
    fn clone_impl(&self) -> Tex2D {
        self.clone()
    }

    pub fn screen_capture() -> Tex2D {
        // Note: screen_capture requires GL context access for ReadPixels
        // In command mode, this would need synchronization - for now warn and return empty
        if is_command_mode() {
            warn!("Tex2D::screen_capture called in command mode - not supported");
            // Return empty texture
            return Tex2D::new(1, 1, TexFormat::RGBA8);
        }

        let size: IVec2 = Viewport::get_size();
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

        // Convert u32 RGBA to u8 bytes for caching
        let cached_bytes: Vec<u8> = buf.iter()
            .flat_map(|&rgba| rgba.to_le_bytes())
            .collect();

        let mut this = Tex2DShared {
            handle: 0,
            size,
            format: TexFormat::RGBA8,
            resource_id: None,
            cached_data: Some(CachedTextureData {
                data: cached_bytes,
                pixel_format: PixelFormat::RGBA,
                data_format: DataFormat::U8,
            }),
        };

        glcheck!(gl::GenTextures(1, &mut this.handle));
        glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
        glcheck!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            TexFormat::RGBA8 as _,
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
        // Note: deep_clone requires GL context access for CopyTexImage2D
        // In command mode, this would need special handling
        if is_command_mode() {
            warn!("Tex2D::deep_clone called in command mode - not fully supported");
        }

        RenderTarget::push_tex2d(self);

        let this = self.shared.as_ref();

        let mut clone = Tex2DShared {
            handle: 0,
            size: this.size,
            format: this.format,
            resource_id: None,
            cached_data: None, // deep_clone doesn't have access to pixel data
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

        if is_command_mode() {
            submit_command(RenderCommand::GenerateMipmap2D {
                handle: GpuHandle(this.handle),
            });
        } else {
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::GenerateMipmap(gl::TEXTURE_2D));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
        }
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

    /// Get the ResourceId if this texture was created in command mode
    /// Internal use only - not exposed to Lua FFI
    #[bind(lua_ffi = false)]
    pub fn get_resource_id(&self) -> Option<ResourceId> {
        let this = self.shared.as_ref();
        this.resource_id
    }

    /// Check if this texture was created in command mode
    /// Internal use only - not exposed to Lua FFI
    #[bind(lua_ffi = false)]
    pub fn is_command_mode_texture(&self) -> bool {
        self.shared.as_ref().resource_id.is_some()
    }

    /// Ensure this texture has a ResourceId for use in command mode.
    /// If the texture was created in direct mode (before render thread),
    /// this will create it on the render thread using the cached data and format info.
    /// For textures without cached data (FBO/render targets), creates empty.
    /// Returns the ResourceId.
    /// Internal use only - not exposed to Lua FFI
    #[bind(lua_ffi = false)]
    pub fn ensure_resource_id(&mut self) -> Option<ResourceId> {
        let mut this = self.shared.as_mut();

        // Already has a resource_id
        if let Some(id) = this.resource_id {
            return Some(id);
        }

        // Create on render thread
        let resource_id = next_resource_id();

        // First create the texture (empty)
        submit_command(RenderCommand::CreateTexture2D {
            id: resource_id,
            width: this.size.x as u32,
            height: this.size.y as u32,
            format: this.format,
            data: None,
        });

        // If we have cached data, upload it with the correct format info
        // This is critical: cached_data stores the original pixel/data format
        // which may differ from what TexFormat.to_gl_formats() would return
        if let Some(ref cached) = this.cached_data {
            submit_command(RenderCommand::UpdateTexture2DDataByResource {
                id: resource_id,
                width: this.size.x,
                height: this.size.y,
                internal_format: this.format as i32,
                pixel_format: cached.pixel_format as u32,
                data_format: cached.data_format as u32,
                data: cached.data.clone(),
            });
        }

        this.resource_id = Some(resource_id);
        Some(resource_id)
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

        if is_command_mode() {
            submit_command(RenderCommand::SetTexture2DAnisotropy {
                handle: GpuHandle(this.handle),
                factor,
            });
        } else {
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::TexParameterf(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAX_ANISOTROPY_EXT,
                factor
            ));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
        }
    }

    pub fn set_data_bytes(&mut self, data: &Bytes, pf: PixelFormat, df: DataFormat) {
        self.set_data(data.as_slice(), pf, df);
    }

    pub fn set_mag_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        if is_command_mode() {
            submit_command(RenderCommand::SetTexture2DMagFilter {
                handle: GpuHandle(this.handle),
                filter,
            });
        } else {
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                filter as _
            ));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
        }
    }

    pub fn set_min_filter(&mut self, filter: TexFilter) {
        let this = self.shared.as_ref();

        if is_command_mode() {
            submit_command(RenderCommand::SetTexture2DMinFilter {
                handle: GpuHandle(this.handle),
                filter,
            });
        } else {
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, this.handle));
            glcheck!(gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                filter as _
            ));
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
        }
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
            warn!(
                "Tex2D_SetMipRange: Setting mip range with min != max; this may fail on old drivers with mip-handling bugs."
            );
        }

        if is_command_mode() {
            submit_command(RenderCommand::SetTexture2DMipRange {
                handle: GpuHandle(this.handle),
                min_level,
                max_level,
            });
        } else {
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
            glcheck!(gl::BindTexture(gl::TEXTURE_2D, 0));
        }
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

        if is_command_mode() {
            submit_command(RenderCommand::SetTexture2DWrapMode {
                handle: GpuHandle(this.handle),
                mode,
            });
        } else {
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

    /// Sample a single pixel at integer coordinates (x, y)
    /// Coordinates are in OpenGL convention: (0,0) = bottom-left
    /// Returns Vec3f with RGB in [0.0, 1.0] range
    #[bind(name = "Sample")]
    fn sample_pixel(&self, x: i32, y: i32) -> Vec3 {
        let this = self.shared.as_ref();
        let size = this.size;

        // Clamp coordinates
        let x = x.clamp(0, size.x - 1);
        let y = y.clamp(0, size.y - 1);

        // Flip Y for OpenGL bottom-left origin
        let gl_y = size.y - 1 - y;

        // Temporary FBO
        let mut fbo: u32 = 0;
        glcheck!(gl::GenFramebuffers(1, &mut fbo));
        glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));
        glcheck!(gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            this.handle,
            0
        ));

        if glcheck!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER)) != gl::FRAMEBUFFER_COMPLETE {
            glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
            glcheck!(gl::DeleteFramebuffers(1, &fbo));
            warn!("Sample: Incomplete framebuffer");
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let mut pixel: [u8; 4] = [0; 4];
        glcheck!(gl::ReadPixels(
            x,
            gl_y,
            1,
            1,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixel.as_mut_ptr() as *mut _
        ));

        glcheck!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        glcheck!(gl::DeleteFramebuffers(1, &fbo));

        Vec3::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        )
    }
}
