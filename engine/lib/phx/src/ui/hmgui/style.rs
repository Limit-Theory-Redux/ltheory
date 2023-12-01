use std::ops::Deref;
use std::{collections::HashMap, path::Path};

use toml::{Table, Value};
use tracing::warn;

use crate::math::Box3;
use crate::render::Font;

use super::{HmGuiProperty, HmGuiPropertyId, HmGuiPropertyType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HmGuiStyleId(usize);

impl Deref for HmGuiStyleId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for HmGuiStyleId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Default)]
pub struct HmGuiStyle {
    pub properties: HashMap<HmGuiPropertyId, HmGuiProperty>,
}

impl HmGuiStyle {
    pub fn load<F: FnMut(&str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
        file_path: &Path,
        mut f: F,
    ) -> Self {
        let s = std::fs::read_to_string(file_path).unwrap_or_else(|err| {
            panic!(
                "Cannot load style file: {}. Error: {err}",
                file_path.display()
            )
        });
        let prop_table: Table = toml::from_str(&s).unwrap_or_else(|err| {
            panic!(
                "Cannot parse style file: {}. Error: {err}",
                file_path.display()
            )
        });

        let mut properties = HashMap::new();

        for (name, value) in prop_table {
            if let Some((id, ty)) = f(&name) {
                let prop = match create_property(ty, &value) {
                    Ok(prop) => prop,
                    Err(err) => panic!(
                        "{err}. Value: {value:?}. Property: {name}/{ty:?}. File: {}",
                        file_path.display()
                    ),
                };

                properties.insert(id, prop);
            } else {
                // TODO: panic?
                warn!("Unknown property {name:?} in {}", file_path.display());
            }
        }

        Self { properties }
    }
}

fn create_property(ty: HmGuiPropertyType, value: &Value) -> Result<HmGuiProperty, String> {
    let prop = match ty {
        HmGuiPropertyType::Bool => {
            HmGuiProperty::Bool(value.as_bool().ok_or("Expected bool value".to_string())?)
        }
        HmGuiPropertyType::I8 => HmGuiProperty::I8(parse_int(value)?),
        HmGuiPropertyType::U8 => HmGuiProperty::U8(parse_int(value)?),
        HmGuiPropertyType::I16 => HmGuiProperty::I16(parse_int(value)?),
        HmGuiPropertyType::U16 => HmGuiProperty::U16(parse_int(value)?),
        HmGuiPropertyType::I32 => HmGuiProperty::I32(parse_int(value)?),
        HmGuiPropertyType::U32 => HmGuiProperty::U32(parse_int(value)?),
        HmGuiPropertyType::I64 => HmGuiProperty::I64(parse_int(value)?),
        HmGuiPropertyType::U64 => HmGuiProperty::U64(parse_int(value)?),
        HmGuiPropertyType::F32 => {
            HmGuiProperty::F32(value.as_float().ok_or("Expected f32 value".to_string())? as _)
        }
        HmGuiPropertyType::F64 => {
            HmGuiProperty::F64(value.as_float().ok_or("Expected f64 value".to_string())?)
        }
        HmGuiPropertyType::Vec2 => HmGuiProperty::Vec2(parse_f32_vec(value)?),
        HmGuiPropertyType::Vec3 => HmGuiProperty::Vec3(parse_f32_vec(value)?),
        HmGuiPropertyType::Vec4 => HmGuiProperty::Vec4(parse_f32_vec(value)?),
        HmGuiPropertyType::IVec2 => HmGuiProperty::IVec2(parse_int_vec(value)?),
        HmGuiPropertyType::IVec3 => HmGuiProperty::IVec3(parse_int_vec(value)?),
        HmGuiPropertyType::IVec4 => HmGuiProperty::IVec4(parse_int_vec(value)?),
        HmGuiPropertyType::UVec2 => HmGuiProperty::UVec2(parse_int_vec(value)?),
        HmGuiPropertyType::UVec3 => HmGuiProperty::UVec3(parse_int_vec(value)?),
        HmGuiPropertyType::UVec4 => HmGuiProperty::UVec4(parse_int_vec(value)?),
        HmGuiPropertyType::DVec2 => HmGuiProperty::DVec2(parse_f64_vec(value)?),
        HmGuiPropertyType::DVec3 => HmGuiProperty::DVec3(parse_f64_vec(value)?),
        HmGuiPropertyType::DVec4 => HmGuiProperty::DVec4(parse_f64_vec(value)?),
        HmGuiPropertyType::Box3 => {
            let arr = value.as_array().ok_or("Expected array value".to_string())?;
            if arr.len() != 2 {
                return Err(format!(
                    "Wring array size. Expected 2 but was {}",
                    arr.len()
                ));
            }

            HmGuiProperty::Box3(Box3::new(parse_f32_vec(&arr[0])?, parse_f32_vec(&arr[1])?))
        }
        HmGuiPropertyType::String => HmGuiProperty::String(value.as_str().expect("").to_string()),
        HmGuiPropertyType::Font => {
            let arr = value.as_array().ok_or("Expected array value".to_string())?;
            if arr.len() != 2 {
                return Err(format!(
                    "Wring array size. Expected 2 but was {}",
                    arr.len()
                ));
            }

            let name = arr[0].as_str().expect("");
            let size = parse_int(&arr[1])?;

            HmGuiProperty::Font(Font::load(name, size))
        }
    };

    Ok(prop)
}

#[inline]
fn parse_int<T: TryFrom<i64>>(value: &Value) -> Result<T, String> {
    let integer = value
        .as_integer()
        .ok_or("Expected integer value".to_string())?;

    match integer.try_into() {
        Ok(v) => Ok(v),
        Err(_) => Err("Cannot parse integer value".into()),
    }
}

#[inline]
fn parse_int_vec<T, const N: usize, V>(value: &Value) -> Result<V, String>
where
    T: Default + Copy + TryFrom<i64>,
    V: From<[T; N]>,
{
    let arr = value.as_array().ok_or("Expected array value".to_string())?;
    if arr.len() != N {
        return Err(format!(
            "Wring array size. Expected {N} but was {}",
            arr.len()
        ));
    }

    let mut vec = [T::default(); N];

    for (i, v) in arr.iter().enumerate() {
        let val_i64 = v.as_integer().ok_or("Expected integer value".to_string())?;

        vec[i] = val_i64
            .try_into()
            .map_err(|_| "Cannot cast i64 value".to_string())?;
    }

    Ok(vec.into())
}

#[inline]
fn parse_f64_vec<const N: usize, V>(value: &Value) -> Result<V, String>
where
    V: From<[f64; N]>,
{
    let arr = value.as_array().ok_or("Expected array value".to_string())?;
    if arr.len() != N {
        return Err(format!(
            "Wring array size. Expected {N} but was {}",
            arr.len()
        ));
    }

    let mut vec = [0.0; N];

    for (i, v) in arr.iter().enumerate() {
        vec[i] = if let Some(val) = v.as_float() {
            val
        } else {
            v.as_integer().ok_or("Expected f64 value".to_string())? as _
        };
    }

    Ok(vec.into())
}

#[inline]
fn parse_f32_vec<const N: usize, V>(value: &Value) -> Result<V, String>
where
    V: From<[f32; N]>,
{
    let arr = value.as_array().ok_or("Expected array value".to_string())?;
    if arr.len() != N {
        return Err(format!(
            "Wring array size. Expected {N} but was {}",
            arr.len()
        ));
    }

    let mut vec = [0.0; N];

    for (i, v) in arr.iter().enumerate() {
        vec[i] = if let Some(val) = v.as_float() {
            val as _
        } else {
            v.as_integer().ok_or("Expected f32 value".to_string())? as _
        };
    }

    Ok(vec.into())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use glam::*;

    use crate::{
        math::Box3,
        ui::hmgui::{HmGuiProperty, HmGuiPropertyType, HmGuiStyle},
    };

    #[rustfmt::skip]
    const TEST_DATA1: &[(&str, HmGuiPropertyType, HmGuiProperty)] = &[
        ("prop.bool", HmGuiPropertyType::Bool, HmGuiProperty::Bool(true)),
        ("prop.i8", HmGuiPropertyType::I8, HmGuiProperty::I8(-10)),
        ("prop.u8", HmGuiPropertyType::U8, HmGuiProperty::U8(63)),
        ("prop.i16", HmGuiPropertyType::I16, HmGuiProperty::I16(-400)),
        ("prop.u16", HmGuiPropertyType::U16, HmGuiProperty::U16(1000)),
        ("prop.i32", HmGuiPropertyType::I32, HmGuiProperty::I32(-100000)),
        ("prop.u32", HmGuiPropertyType::U32, HmGuiProperty::U32(630000)),
        ("prop.i64", HmGuiPropertyType::I64, HmGuiProperty::I64(-10)),
        ("prop.u64", HmGuiPropertyType::U64, HmGuiProperty::U64(63)),
        ("prop.f32", HmGuiPropertyType::F32, HmGuiProperty::F32(-10.69)),
        ("prop.f64", HmGuiPropertyType::F64, HmGuiProperty::F64(63.132)),
        ("prop.vec2", HmGuiPropertyType::Vec2, HmGuiProperty::Vec2(Vec2::new(-10.2, 4.729))),
        ("prop.vec3", HmGuiPropertyType::Vec3, HmGuiProperty::Vec3(Vec3::new(-10.2, 4.729, 0.0))),
        ("prop.vec4", HmGuiPropertyType::Vec4, HmGuiProperty::Vec4(Vec4::new(-10.2, 4.729, 740.0, 44.6))),
        ("prop.ivec2", HmGuiPropertyType::IVec2, HmGuiProperty::IVec2(IVec2::new(-10, 4))),
        ("prop.ivec3", HmGuiPropertyType::IVec3, HmGuiProperty::IVec3(IVec3::new(-10, 4, 0))),
        ("prop.ivec4", HmGuiPropertyType::IVec4, HmGuiProperty::IVec4(IVec4::new(-10, 4, 740, 44))),
        ("prop.uvec2", HmGuiPropertyType::UVec2, HmGuiProperty::UVec2(UVec2::new(10, 4))),
        ("prop.uvec3", HmGuiPropertyType::UVec3, HmGuiProperty::UVec3(UVec3::new(10, 4, 0))),
        ("prop.uvec4", HmGuiPropertyType::UVec4, HmGuiProperty::UVec4(UVec4::new(10, 4, 740, 44))),
        ("prop.dvec2", HmGuiPropertyType::DVec2, HmGuiProperty::DVec2(DVec2::new(-10.2, 4.729))),
        ("prop.dvec3", HmGuiPropertyType::DVec3, HmGuiProperty::DVec3(DVec3::new(-10.2, 4.729, 0.0))),
        ("prop.dvec4", HmGuiPropertyType::DVec4, HmGuiProperty::DVec4(DVec4::new(-10.2, 4.729, 740.0, 44.6))),
        ("prop.box3", HmGuiPropertyType::Box3, HmGuiProperty::Box3(Box3::new(Vec3::new(10.2, 4.729, 1.0), Vec3::new(740.0, 44.6, -1.0)))),
        // ("prop.string", HmGuiPropertyType::String, HmGuiProperty::String("Test".into())),
    ];

    #[test]
    fn test_hmgui_load_style() {
        let file_path = PathBuf::from("test_data/style1.toml");
        let style = HmGuiStyle::load(&file_path, |name| {
            TEST_DATA1
                .iter()
                .enumerate()
                .find(|(_, (n, _, _))| *n == name)
                .map(|(id, (_, ty, _))| (id.into(), *ty))
        });

        assert_eq!(style.properties.len(), 24);

        for (id, (name, ty, expected)) in TEST_DATA1.iter().enumerate() {
            let actual = style.properties.get(&id.into()).expect(&format!(
                "Cannot find property. {id}/{name}/{ty:?}/{}",
                expected.name()
            ));

            if expected != actual {
                panic!(
                    "Mismatched property: {id}/{name}/{ty:?}/{} - {}",
                    expected.name(),
                    actual.name()
                );
            }
        }
    }

    #[test]
    fn test_hmgui_load_style_str() {
        let file_path = PathBuf::from("test_data/style2.toml");
        let style = HmGuiStyle::load(&file_path, |name| match name {
            "prop.string" => Some((0.into(), HmGuiPropertyType::String)),
            _ => None,
        });

        assert_eq!(style.properties.len(), 1);

        let actual = style.properties.get(&0.into()).expect(&format!(
            "Cannot find string property. 0/prop.string/String/String"
        ));

        if *actual != HmGuiProperty::String("Test".into()) {
            panic!("Mismatched property: 0/prop.string/String/String - String");
        }
    }
}
