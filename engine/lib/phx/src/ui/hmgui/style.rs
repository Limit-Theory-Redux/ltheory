use std::collections::HashMap;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;

use serde_yaml::Value;
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

/// Contains a map of the property id and property pairs.
#[derive(Clone, Default)]
pub struct HmGuiStyle {
    pub properties: HashMap<HmGuiPropertyId, HmGuiProperty>,
}

impl HmGuiStyle {
    /// Load style from the config file containing property name/value pairs.
    pub fn load<F: FnMut(&str, &str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
        file_path: &Path,
        style_name: &str,
        f: F,
    ) -> Self {
        let file = File::open(file_path).unwrap_or_else(|err| {
            panic!(
                "Cannot load style file: {}. Error: {err}",
                file_path.display()
            )
        });
        let root_value: Value = serde_yaml::from_reader(&file).unwrap_or_else(|err| {
            panic!(
                "Cannot parse style file: {}. Error: {err}",
                file_path.display()
            )
        });

        if root_value.is_null() {
            return Self {
                properties: Default::default(),
            };
        }

        Self::parse_value(style_name, &root_value, f)
            .unwrap_or_else(|err| panic!("{err}. File: {}", file_path.display()))
    }

    /// Parse style from the yaml value. Expecting a map with the property name/value pairs.
    pub fn parse_value<F: FnMut(&str, &str) -> Option<(HmGuiPropertyId, HmGuiPropertyType)>>(
        style_name: &str,
        value: &Value,
        mut f: F,
    ) -> Result<Self, String> {
        let prop_table = value
            .as_mapping()
            .ok_or_else(|| format!("Cannot parse style. Expecting map type but was {value:?}"))?;

        let mut properties = HashMap::new();

        for (name_value, value) in prop_table.iter() {
            let name = parse_string(name_value)?;

            if let Some((id, ty)) = f(style_name, &name) {
                let prop = create_property(ty, &value).map_err(|err| {
                    format!("{err}. Value: {value:?}. Property: {name}/{ty:?}. Error: {err}")
                })?;

                properties.insert(id, prop);
            } else {
                // TODO: panic?
                warn!("Unknown property {name:?}");
            }
        }

        Ok(Self { properties })
    }
}

/// Create property of the provided type from the yaml value.
fn create_property(ty: HmGuiPropertyType, value: &Value) -> Result<HmGuiProperty, String> {
    let prop = match ty {
        HmGuiPropertyType::Bool => HmGuiProperty::Bool(parse_bool(value)?),
        HmGuiPropertyType::I8 => HmGuiProperty::I8(parse_int(value)?),
        HmGuiPropertyType::U8 => HmGuiProperty::U8(parse_int(value)?),
        HmGuiPropertyType::I16 => HmGuiProperty::I16(parse_int(value)?),
        HmGuiPropertyType::U16 => HmGuiProperty::U16(parse_int(value)?),
        HmGuiPropertyType::I32 => HmGuiProperty::I32(parse_int(value)?),
        HmGuiPropertyType::U32 => HmGuiProperty::U32(parse_int(value)?),
        HmGuiPropertyType::I64 => HmGuiProperty::I64(parse_int(value)?),
        HmGuiPropertyType::U64 => HmGuiProperty::U64(parse_int(value)?),
        HmGuiPropertyType::F32 => HmGuiProperty::F32(parse_f32(value)?),
        HmGuiPropertyType::F64 => HmGuiProperty::F64(parse_f64(value)?),
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
        HmGuiPropertyType::Color => HmGuiProperty::Color(parse_f32_vec(value)?),
        HmGuiPropertyType::Box3 => {
            let arr = parse_sequence::<2>(value)?;
            let lower = parse_f32_vec(&arr[0])?;
            let upper = parse_f32_vec(&arr[1])?;

            HmGuiProperty::Box3(Box3::new(lower, upper))
        }
        HmGuiPropertyType::String => HmGuiProperty::String(parse_string(value)?),
        HmGuiPropertyType::Font => {
            let arr = parse_sequence::<2>(value)?;
            let name = parse_string(&arr[0])?;
            let size = parse_int(&arr[1])?;

            HmGuiProperty::Font(Font::load(&name, size))
        }
    };

    Ok(prop)
}

#[inline]
fn parse_sequence<const N: usize>(value: &Value) -> Result<Vec<&Value>, String> {
    let val = value
        .as_sequence()
        .ok_or_else(|| format!("Expected sequence value but was {value:?}"))?;
    if val.len() != N {
        return Err(format!(
            "Wrong sequence size. Expected {N} but was {}",
            val.len()
        ));
    }

    Ok(val.iter().map(|v| v).collect())
}

#[inline]
fn parse_bool(value: &Value) -> Result<bool, String> {
    let val = value
        .as_bool()
        .ok_or_else(|| format!("Expected bool value but was {value:?}"))?;
    Ok(val)
}

#[inline]
fn parse_int<T: TryFrom<i64>>(value: &Value) -> Result<T, String> {
    let integer = value
        .as_i64()
        .ok_or_else(|| format!("Expected integer value but was {value:?}"))?;

    match integer.try_into() {
        Ok(v) => Ok(v),
        Err(_) => Err(format!("Cannot parse integer value {value:?}")),
    }
}

#[inline]
fn parse_f32(value: &Value) -> Result<f32, String> {
    let val = value
        .as_f64()
        .ok_or_else(|| format!("Expected f32 value but was {value:?}"))?;
    Ok(val as _)
}

#[inline]
fn parse_f64(value: &Value) -> Result<f64, String> {
    let val = value
        .as_f64()
        .ok_or_else(|| format!("Expected f64 value but was {value:?}"))?;
    Ok(val)
}

#[inline]
pub fn parse_string(value: &Value) -> Result<String, String> {
    let val = value
        .as_str()
        .ok_or_else(|| format!("Expected string value but was {value:?}"))?;
    Ok(val.into())
}

#[inline]
fn parse_int_vec<T, const N: usize, V>(value: &Value) -> Result<V, String>
where
    T: Default + Copy + TryFrom<i64>,
    V: From<[T; N]>,
{
    let arr = parse_sequence::<N>(value)?;
    let mut vec = [T::default(); N];

    for (i, v) in arr.iter().enumerate() {
        let val_i64: i64 = parse_int(v)?;

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
    let arr = parse_sequence::<N>(value)?;
    let mut vec = [0.0; N];

    for (i, v) in arr.iter().enumerate() {
        vec[i] = if let Some(val) = v.as_f64() {
            val
        } else {
            parse_int::<i64>(v)? as _
        };
    }

    Ok(vec.into())
}

#[inline]
fn parse_f32_vec<const N: usize, V>(value: &Value) -> Result<V, String>
where
    V: From<[f32; N]>,
{
    let arr = parse_sequence::<N>(value)?;
    let mut vec = [0.0; N];

    for (i, v) in arr.iter().enumerate() {
        vec[i] = if let Some(val) = v.as_f64() {
            val as _
        } else {
            parse_int::<i64>(v)? as _
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
    ];

    fn test_style(style: &HmGuiStyle, expected: &[(&str, HmGuiPropertyType, HmGuiProperty)]) {
        assert_eq!(style.properties.len(), expected.len());

        for (id, (name, ty, expected)) in expected.iter().enumerate() {
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
    fn test_hmgui_load_style() {
        let file_path = PathBuf::from("test_data/style1.yaml");
        let style = HmGuiStyle::load(&file_path, "style1", |_, name| {
            TEST_DATA1
                .iter()
                .enumerate()
                .find(|(_, (n, _, _))| *n == name)
                .map(|(id, (_, ty, _))| (id.into(), *ty))
        });

        test_style(&style, TEST_DATA1);
    }

    #[test]
    fn test_hmgui_load_style_str() {
        let file_path = PathBuf::from("test_data/style2.yaml");
        let style = HmGuiStyle::load(&file_path, "style2", |_, name| match name {
            "prop.string" => Some((0.into(), HmGuiPropertyType::String)),
            _ => None,
        });

        assert_eq!(style.properties.len(), 1);

        let actual = style.properties.get(&0.into()).expect(&format!(
            "Cannot find string property. 0/prop.string/String/String"
        ));

        let HmGuiProperty::String(val) = actual else {
            panic!(
                "Wrong property type. Expected string but was {:?}",
                actual.get_type()
            );
        };
        assert_eq!(val, "Test");
    }
}
