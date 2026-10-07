//! Texture creation and update descriptions, shared by both backends.
//!
//! `TexDesc { dim, format, size, mips, usage }` is everything the executors
//! need to allocate a texture. Pixel updates are format-agnostic: a
//! [`TexRegion`] plus bytes that are already in the texture's own
//! [`TexFormat`] layout (tightly packed rows), so neither backend has to speak
//! GL's `pixel format x data type` pairs. [`convert_texels`] does the
//! conversion from the engine's `(PixelFormat, DataFormat)` source layouts on
//! the main thread.

use std::borrow::Cow;

use crate::render::{CubeFace, DataFormat, PixelFormat, TexDim, TexFormat, gl};

/// Usage bits for `TexUsages` and for the Lua `desc.usage` field
/// (`bit.bor(TexUsage.Sampled, TexUsage.CopySrc)`).
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexUsage {
    /// Sampled by shaders.
    Sampled = 1,
    /// Rendered to (a pass attachment). Not valid for 1D textures.
    Attachment = 2,
    /// Read back or copied from.
    CopySrc = 4,
    /// Uploaded to or copied into.
    CopyDst = 8,
}

/// A set of [`TexUsage`] bits. GL ignores it (every texture can do everything);
/// wgpu maps it to `TextureUsages`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TexUsages(pub u32);

impl TexUsages {
    pub const SAMPLED: u32 = TexUsage::Sampled as u32;
    pub const ATTACHMENT: u32 = TexUsage::Attachment as u32;
    pub const COPY_SRC: u32 = TexUsage::CopySrc as u32;
    pub const COPY_DST: u32 = TexUsage::CopyDst as u32;

    /// What a texture of this kind gets when the caller does not say: sampled,
    /// copied from and to, and rendered to unless it is 1D.
    pub fn default_for(dim: TexDim) -> Self {
        let mut bits = Self::SAMPLED | Self::COPY_SRC | Self::COPY_DST;
        if dim != TexDim::D1 {
            bits |= Self::ATTACHMENT;
        }
        Self(bits)
    }

    pub fn contains(self, bit: u32) -> bool {
        self.0 & bit != 0
    }
}

/// Everything needed to create a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TexDesc {
    pub dim: TexDim,
    pub format: TexFormat,
    /// Width, height, depth. `[n, 1, 1]` for 1D, `[n, n, 1]` for cubes (the
    /// layer count of a cube is always six).
    pub size: [u32; 3],
    /// Mip levels to allocate, at least 1 (see [`TexDesc::with_mips`]).
    pub mips: u32,
    pub usage: TexUsages,
}

/// Number of levels in the full mip chain of a texture of this kind and size.
pub fn full_mip_count(dim: TexDim, size: [u32; 3]) -> u32 {
    let largest = match dim {
        TexDim::D1 => size[0],
        TexDim::D2 | TexDim::Cube => size[0].max(size[1]),
        TexDim::D3 => size[0].max(size[1]).max(size[2]),
    };
    32 - largest.max(1).leading_zeros()
}

impl TexDesc {
    fn new(dim: TexDim, format: TexFormat, size: [u32; 3]) -> Self {
        Self {
            dim,
            format,
            size: size.map(|s| s.max(1)),
            mips: 1,
            usage: TexUsages::default_for(dim),
        }
    }

    pub fn d1(size: u32, format: TexFormat) -> Self {
        Self::new(TexDim::D1, format, [size, 1, 1])
    }

    pub fn d2(width: u32, height: u32, format: TexFormat) -> Self {
        Self::new(TexDim::D2, format, [width, height, 1])
    }

    pub fn d3(width: u32, height: u32, depth: u32, format: TexFormat) -> Self {
        Self::new(TexDim::D3, format, [width, height, depth])
    }

    pub fn cube(size: u32, format: TexFormat) -> Self {
        Self::new(TexDim::Cube, format, [size, size, 1])
    }

    /// `mips` levels, clamped to the full chain; `0` asks for the full chain.
    /// Depth formats never get a chain.
    pub fn with_mips(mut self, mips: u32) -> Self {
        let full = full_mip_count(self.dim, self.size);
        self.mips = if TexFormat::is_depth(self.format) {
            1
        } else if mips == 0 {
            full
        } else {
            mips.min(full)
        };
        self
    }

    pub fn with_usage(mut self, usage: TexUsages) -> Self {
        self.usage = usage;
        self
    }

    /// Size of one level, `max(1, size >> level)` per axis (the layer count
    /// of a cube stays 1: its faces are separate regions).
    pub fn level_size(&self, level: u32) -> [u32; 3] {
        let shift = |s: u32| (s >> level).max(1);
        match self.dim {
            TexDim::D1 => [shift(self.size[0]), 1, 1],
            TexDim::D2 | TexDim::Cube => [shift(self.size[0]), shift(self.size[1]), 1],
            TexDim::D3 => [
                shift(self.size[0]),
                shift(self.size[1]),
                shift(self.size[2]),
            ],
        }
    }

    /// Levels of the full mip chain of this texture.
    pub fn full_chain(&self) -> u32 {
        full_mip_count(self.dim, self.size)
    }

    /// Bytes of one tightly packed level of one face (or the whole volume).
    pub fn level_bytes(&self, level: u32) -> usize {
        let s = self.level_size(level);
        s[0] as usize * s[1] as usize * s[2] as usize * TexFormat::get_size(self.format) as usize
    }
}

/// The part of a texture an update writes: one mip level, a box of texels.
/// For a cube the layer of `origin[2]` is the face (`CubeFace` order: +X, -X,
/// +Y, -Y, +Z, -Z) and `size[2]` is 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TexRegion {
    pub level: u32,
    pub origin: [u32; 3],
    pub size: [u32; 3],
}

impl TexRegion {
    /// All of `level` of a 1D, 2D or 3D texture.
    pub fn level(desc: &TexDesc, level: u32) -> Self {
        Self {
            level,
            origin: [0; 3],
            size: desc.level_size(level),
        }
    }

    /// All of one face of a cube at `level`.
    pub fn face(desc: &TexDesc, face: CubeFace, level: u32) -> Self {
        let s = desc.level_size(level);
        Self {
            level,
            origin: [0, 0, face_layer(face)],
            size: [s[0], s[1], 1],
        }
    }

    /// A rectangle of mip level 0 of a 2D texture.
    pub fn rect(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            level: 0,
            origin: [x, y, 0],
            size: [width, height, 1],
        }
    }

    pub fn texels(&self) -> usize {
        self.size[0] as usize * self.size[1] as usize * self.size[2] as usize
    }
}

/// Array layer of a cube face (0 to 5).
pub fn face_layer(face: CubeFace) -> u32 {
    face as u32 - gl::TEXTURE_CUBE_MAP_POSITIVE_X
}

// ----------------------------------------------------------------------------
// Texel conversion
// ----------------------------------------------------------------------------

/// How a [`TexFormat`] stores one component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scalar {
    Unorm8,
    Unorm16,
    Half,
    Float,
    /// 24-bit depth in the 32 bits of an unsigned int (`DEPTH_COMPONENT24`
    /// uploaded as `UNSIGNED_INT`).
    Unorm32,
}

fn native_layout(format: TexFormat) -> (usize, Scalar) {
    use TexFormat as F;
    let comps = TexFormat::components(format) as usize;
    let scalar = match format {
        F::R8 | F::RG8 | F::RGBA8 => Scalar::Unorm8,
        F::R16 | F::RG16 | F::RGBA16 | F::Depth16 => Scalar::Unorm16,
        F::R16F | F::RG16F | F::RGBA16F => Scalar::Half,
        F::R32F | F::RG32F | F::RGBA32F | F::Depth32F => Scalar::Float,
        F::Depth24 => Scalar::Unorm32,
    };
    (comps, scalar)
}

/// `f32` to IEEE half bits, round to nearest even.
pub fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = (((bits >> 23) & 0xff) as i32) - 127 + 15;
    let mant = bits & 0x7f_ffff;
    if (bits >> 23) & 0xff == 0xff {
        // Inf or NaN.
        return sign | 0x7c00 | if mant != 0 { 0x200 } else { 0 };
    }
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = mant | 0x80_0000;
        let shift = (14 - exp) as u32;
        let mut v = m >> shift;
        let rem = m & ((1u32 << shift) - 1);
        let halfway = 1u32 << (shift - 1);
        if rem > halfway || (rem == halfway && (v & 1) == 1) {
            v += 1;
        }
        sign | v as u16
    } else if exp >= 31 {
        sign | 0x7c00
    } else {
        let mut v = ((exp as u32) << 10) | (mant >> 13);
        let rem = mant & 0x1fff;
        if rem > 0x1000 || (rem == 0x1000 && (v & 1) == 1) {
            v += 1;
        }
        sign | v as u16
    }
}

fn source_scalar_size(df: DataFormat) -> usize {
    DataFormat::get_size(df) as usize
}

/// A component of the source as a float: integers are normalized the way GL
/// does it (unsigned `c / max`, signed `max(c / max, -1)`).
fn read_component(df: DataFormat, bytes: &[u8]) -> f32 {
    match df {
        DataFormat::U8 => bytes[0] as f32 / 255.0,
        DataFormat::I8 => (bytes[0] as i8 as f32 / 127.0).max(-1.0),
        DataFormat::U16 => u16::from_ne_bytes([bytes[0], bytes[1]]) as f32 / 65535.0,
        DataFormat::I16 => (i16::from_ne_bytes([bytes[0], bytes[1]]) as f32 / 32767.0).max(-1.0),
        DataFormat::U32 => {
            (u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 / 4294967295.0)
                as f32
        }
        DataFormat::I32 => {
            ((i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 / 2147483647.0)
                .max(-1.0)) as f32
        }
        DataFormat::Float => f32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
    }
}

fn write_component(scalar: Scalar, value: f32, out: &mut Vec<u8>) {
    match scalar {
        Scalar::Unorm8 => out.push((value.clamp(0.0, 1.0) * 255.0).round() as u8),
        Scalar::Unorm16 => {
            out.extend_from_slice(&((value.clamp(0.0, 1.0) * 65535.0).round() as u16).to_ne_bytes())
        }
        Scalar::Half => out.extend_from_slice(&f32_to_f16(value).to_ne_bytes()),
        Scalar::Float => out.extend_from_slice(&value.to_ne_bytes()),
        Scalar::Unorm32 => out.extend_from_slice(
            &((value.clamp(0.0, 1.0) as f64 * 4294967295.0).round() as u32).to_ne_bytes(),
        ),
    }
}

/// Convert `bytes`, tightly packed texels laid out as `pixel` x `data`, to the
/// native layout of `dst`. Components the source lacks are 0, except alpha,
/// which is 1 (what GL does for an `RGB` upload into an `RGBA8` texture);
/// components `dst` lacks are dropped. Integer and float sources are converted
/// with GL's rules (round to nearest, clamp for normalized destinations). A
/// source that already is in the native layout is returned as it is.
pub fn convert_texels(
    pixel: PixelFormat,
    data: DataFormat,
    bytes: &[u8],
    dst: TexFormat,
) -> Cow<'_, [u8]> {
    let (dst_comps, dst_scalar) = native_layout(dst);
    let src_comps = PixelFormat::components(pixel) as usize;
    let swizzled = matches!(pixel, PixelFormat::BGR | PixelFormat::BGRA);
    let native_scalar = match (data, dst_scalar) {
        (DataFormat::U8, Scalar::Unorm8)
        | (DataFormat::U16, Scalar::Unorm16)
        | (DataFormat::Float, Scalar::Float) => true,
        _ => false,
    };
    if src_comps == dst_comps && native_scalar && !swizzled {
        return Cow::Borrowed(bytes);
    }

    let scalar_size = source_scalar_size(data);
    let src_stride = src_comps * scalar_size;
    let texels = bytes.len() / src_stride.max(1);
    let mut out = Vec::with_capacity(texels * dst_comps * 4);
    for texel in bytes.chunks_exact(src_stride).take(texels) {
        // Source channels in R, G, B, A order.
        let mut rgba = [0.0f32, 0.0, 0.0, 1.0];
        for c in 0..src_comps {
            let at = c * scalar_size;
            let v = read_component(data, &texel[at..at + scalar_size]);
            let channel = if swizzled && c < 3 { 2 - c } else { c };
            rgba[channel] = v;
        }
        for &v in rgba.iter().take(dst_comps) {
            write_component(dst_scalar, v, &mut out);
        }
    }
    Cow::Owned(out)
}

/// Convert float texels (tightly packed, `comps` floats each) to the native
/// layout of `dst`. Convenience over [`convert_texels`] for engine-side data.
pub fn convert_floats(pixel: PixelFormat, floats: &[f32], dst: TexFormat) -> Vec<u8> {
    #[allow(unsafe_code)]
    // SAFETY: reinterpreting `f32`s as their bytes.
    let bytes = unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const u8, std::mem::size_of_val(floats))
    };
    convert_texels(pixel, DataFormat::Float, bytes, dst).into_owned()
}

/// Convert a slice of engine data (`Vec3`s, `Vec4`s, floats, ...) laid out as
/// `pixel` x `data` to the native layout of `dst`.
pub fn convert_slice<T>(data: &[T], pixel: PixelFormat, df: DataFormat, dst: TexFormat) -> Vec<u8> {
    #[allow(unsafe_code)]
    // SAFETY: a byte view of plain data the caller owns for the call.
    let bytes = unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
    };
    convert_texels(pixel, df, bytes, dst).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mip_counts() {
        assert_eq!(full_mip_count(TexDim::D2, [1, 1, 1]), 1);
        assert_eq!(full_mip_count(TexDim::D2, [256, 128, 1]), 9);
        assert_eq!(full_mip_count(TexDim::D2, [1280, 720, 1]), 11);
        assert_eq!(full_mip_count(TexDim::D1, [256, 1, 1]), 9);
        assert_eq!(full_mip_count(TexDim::D3, [8, 16, 4]), 5);
    }

    #[test]
    fn desc_mips_are_clamped() {
        let d = TexDesc::d2(64, 64, TexFormat::RGBA8);
        assert_eq!(d.mips, 1);
        assert_eq!(d.with_mips(0).mips, 7);
        assert_eq!(d.with_mips(3).mips, 3);
        assert_eq!(d.with_mips(99).mips, 7);
        assert_eq!(
            TexDesc::d2(64, 64, TexFormat::Depth32F).with_mips(0).mips,
            1
        );
    }

    #[test]
    fn level_sizes_halve_and_stop_at_one() {
        let d = TexDesc::d3(8, 4, 2, TexFormat::R8);
        assert_eq!(d.level_size(0), [8, 4, 2]);
        assert_eq!(d.level_size(1), [4, 2, 1]);
        assert_eq!(d.level_size(3), [1, 1, 1]);
        assert_eq!(d.level_bytes(1), 8);
    }

    #[test]
    fn default_usage_excludes_attachment_for_1d() {
        assert!(!TexUsages::default_for(TexDim::D1).contains(TexUsages::ATTACHMENT));
        assert!(TexUsages::default_for(TexDim::D2).contains(TexUsages::ATTACHMENT));
    }

    #[test]
    fn native_layouts_pass_through() {
        let bytes = [1u8, 2, 3, 4];
        let out = convert_texels(PixelFormat::RGBA, DataFormat::U8, &bytes, TexFormat::RGBA8);
        assert!(matches!(out, Cow::Borrowed(_)));
        let floats = [0.5f32, 0.25];
        let out = convert_floats(PixelFormat::RG, &floats, TexFormat::RG32F);
        assert_eq!(out.len(), 8);
    }

    #[test]
    fn rgb_float_to_rgba8_rounds_and_sets_alpha() {
        let out = convert_floats(
            PixelFormat::RGB,
            &[0.0, 0.5, 1.0, 0.2, 0.4, 0.6],
            TexFormat::RGBA8,
        );
        assert_eq!(out, vec![0, 128, 255, 255, 51, 102, 153, 255]);
    }

    #[test]
    fn float_to_half() {
        let out = convert_floats(PixelFormat::RG, &[0.0, 1.0], TexFormat::RG16F);
        assert_eq!(out, [0u16.to_ne_bytes(), 0x3c00u16.to_ne_bytes()].concat());
        assert_eq!(f32_to_f16(65504.0), 0x7bff);
        assert_eq!(f32_to_f16(1e-8), 0);
        assert_eq!(f32_to_f16(f32::INFINITY), 0x7c00);
    }

    #[test]
    fn bgr_is_swizzled() {
        let out = convert_texels(
            PixelFormat::BGR,
            DataFormat::U8,
            &[10, 20, 30],
            TexFormat::RGBA8,
        );
        assert_eq!(out.as_ref(), &[30, 20, 10, 255]);
    }

    #[test]
    fn face_layers_follow_cube_face_order() {
        assert_eq!(face_layer(CubeFace::PX), 0);
        assert_eq!(face_layer(CubeFace::NX), 1);
        assert_eq!(face_layer(CubeFace::NZ), 5);
    }
}
