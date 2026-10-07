//! Sampler objects and the `Samplers.*` presets
//! (doc/engine/render-api-v2.md, sections 1.1 and 4).

use std::collections::HashMap;

use super::CompareFn;
use crate::render::{Renderer, TexWrapMode};

/// Index of a created sampler. Presets occupy the first ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerId(pub u16);

#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerFilter {
    Point,
    Linear,
}

#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MipFilter {
    /// Sample level 0 only.
    None,
    Point,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerDesc {
    pub min: SamplerFilter,
    pub mag: SamplerFilter,
    pub mip: MipFilter,
    /// S, T, R.
    pub wrap: [TexWrapMode; 3],
    /// 1 = off.
    pub anisotropy: u8,
    /// Quantized LOD clamps.
    pub lod_min: u8,
    pub lod_max: u8,
    pub compare: Option<CompareFn>,
}

impl SamplerDesc {
    pub const fn new(filter: SamplerFilter, mip: MipFilter, wrap: TexWrapMode) -> Self {
        Self {
            min: filter,
            mag: filter,
            mip,
            wrap: [wrap; 3],
            anisotropy: 1,
            lod_min: 0,
            lod_max: 255,
            compare: None,
        }
    }

    /// `GL_TEXTURE_MIN_FILTER` for this min/mip combination.
    pub fn gl_min_filter(&self) -> u32 {
        use crate::render::gl;
        match (self.min, self.mip) {
            (SamplerFilter::Point, MipFilter::None) => gl::NEAREST,
            (SamplerFilter::Point, MipFilter::Point) => gl::NEAREST_MIPMAP_NEAREST,
            (SamplerFilter::Point, MipFilter::Linear) => gl::NEAREST_MIPMAP_LINEAR,
            (SamplerFilter::Linear, MipFilter::None) => gl::LINEAR,
            (SamplerFilter::Linear, MipFilter::Point) => gl::LINEAR_MIPMAP_NEAREST,
            (SamplerFilter::Linear, MipFilter::Linear) => gl::LINEAR_MIPMAP_LINEAR,
        }
    }

    pub fn gl_mag_filter(&self) -> u32 {
        use crate::render::gl;
        match self.mag {
            SamplerFilter::Point => gl::NEAREST,
            SamplerFilter::Linear => gl::LINEAR,
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl SamplerDesc {
    /// Linear filtering, no mips, clamp: the common starting point.
    #[bind(name = "Create")]
    pub fn create() -> SamplerDesc {
        SamplerDesc::new(SamplerFilter::Linear, MipFilter::None, TexWrapMode::Clamp)
    }

    pub fn min(&mut self, filter: SamplerFilter) {
        self.min = filter;
    }

    pub fn mag(&mut self, filter: SamplerFilter) {
        self.mag = filter;
    }

    pub fn mip(&mut self, filter: MipFilter) {
        self.mip = filter;
    }

    /// Wrap mode on all three axes.
    pub fn wrap(&mut self, mode: TexWrapMode) {
        self.wrap = [mode; 3];
    }

    pub fn wrap_axes(&mut self, s: TexWrapMode, t: TexWrapMode, r: TexWrapMode) {
        self.wrap = [s, t, r];
    }

    pub fn anisotropy(&mut self, max: i32) {
        self.anisotropy = max.clamp(1, 16) as u8;
    }

    pub fn lod_range(&mut self, min: i32, max: i32) {
        self.lod_min = min.clamp(0, 255) as u8;
        self.lod_max = max.clamp(0, 255) as u8;
    }

    /// Depth-comparison sampler (shadow lookups).
    pub fn compare(&mut self, func: CompareFn) {
        self.compare = Some(func);
    }
}

/// Sampler presets. The discriminant is the `SamplerId`, so
/// `Samplers.LinearClamp` can be passed wherever a sampler is expected.
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Samplers {
    Point,
    PointRepeat,
    LinearClamp,
    LinearRepeat,
    LinearMipClamp,
    LinearMipRepeat,
}

impl Samplers {
    pub const ALL: [Samplers; 6] = [
        Samplers::Point,
        Samplers::PointRepeat,
        Samplers::LinearClamp,
        Samplers::LinearRepeat,
        Samplers::LinearMipClamp,
        Samplers::LinearMipRepeat,
    ];

    pub fn desc(self) -> SamplerDesc {
        use MipFilter as M;
        use SamplerFilter as F;
        use TexWrapMode as W;
        match self {
            Samplers::Point => SamplerDesc::new(F::Point, M::None, W::Clamp),
            Samplers::PointRepeat => SamplerDesc::new(F::Point, M::None, W::Repeat),
            Samplers::LinearClamp => SamplerDesc::new(F::Linear, M::None, W::Clamp),
            Samplers::LinearRepeat => SamplerDesc::new(F::Linear, M::None, W::Repeat),
            Samplers::LinearMipClamp => SamplerDesc::new(F::Linear, M::Linear, W::Clamp),
            Samplers::LinearMipRepeat => SamplerDesc::new(F::Linear, M::Linear, W::Repeat),
        }
    }

    pub fn id(self) -> SamplerId {
        SamplerId(self as u16)
    }
}

/// Main-thread hash cache of samplers. The presets are pre-registered, so
/// their ids are `Samplers as u16`.
pub struct SamplerCache {
    map: HashMap<SamplerDesc, SamplerId>,
    next: u16,
}

impl SamplerCache {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for preset in Samplers::ALL {
            map.entry(preset.desc()).or_insert(preset.id());
        }
        Self {
            map,
            next: Samplers::ALL.len() as u16,
        }
    }

    /// The `CreateSampler` commands that must be sent once at startup.
    pub fn presets() -> impl Iterator<Item = (SamplerId, SamplerDesc)> {
        Samplers::ALL.into_iter().map(|p| (p.id(), p.desc()))
    }

    /// The id of `desc`, and `true` if it was just allocated.
    pub fn get_or_insert(&mut self, desc: &SamplerDesc) -> (SamplerId, bool) {
        if let Some(&id) = self.map.get(desc) {
            return (id, false);
        }
        let id = SamplerId(self.next);
        self.next = self.next.checked_add(1).expect("sampler ids exhausted");
        self.map.insert(*desc, id);
        (id, true)
    }
}

impl Default for SamplerCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Namespace for `Sampler.Get`.
pub struct Sampler;

#[luajit_ffi_gen::luajit_ffi]
impl Sampler {
    /// The `SamplerId` for `desc`, creating the sampler on first use.
    pub fn get(r: &mut Renderer, desc: &SamplerDesc) -> u32 {
        r.get_sampler(desc).0 as u32
    }
}

impl Renderer {
    pub fn get_sampler(&mut self, desc: &SamplerDesc) -> SamplerId {
        let (id, created) = self.data.samplers.get_or_insert(desc);
        if created {
            self.create_sampler(id, *desc);
        }
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_stable_distinct_ids() {
        let mut cache = SamplerCache::new();
        for preset in Samplers::ALL {
            let (id, created) = cache.get_or_insert(&preset.desc());
            assert!(!created, "{preset:?} is pre-registered");
            assert_eq!(id, preset.id());
        }
        let mut custom = Samplers::LinearClamp.desc();
        custom.anisotropy = 8;
        let (id, created) = cache.get_or_insert(&custom);
        assert!(created);
        assert_eq!(id.0 as usize, Samplers::ALL.len());
        assert_eq!(cache.get_or_insert(&custom), (id, false));
    }

    #[test]
    fn gl_filters_follow_min_and_mip() {
        use crate::render::gl;
        let d = Samplers::LinearMipClamp.desc();
        assert_eq!(d.gl_min_filter(), gl::LINEAR_MIPMAP_LINEAR);
        assert_eq!(d.gl_mag_filter(), gl::LINEAR);
        assert_eq!(Samplers::Point.desc().gl_min_filter(), gl::NEAREST);
    }
}
