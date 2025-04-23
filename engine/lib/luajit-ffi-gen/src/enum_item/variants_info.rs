use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub enum VariantValue {
    Literal(u64),
    Expr(TokenStream),
}

impl VariantValue {
    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(..))
    }
}

pub enum VariantsInfo {
    Simple(Vec<(Vec<String>, String)>),
    Value(Vec<(Vec<String>, String, VariantValue)>),
}

impl VariantsInfo {
    pub fn max_discriminant(&self, start_index: u64) -> Option<u64> {
        match self {
            Self::Simple(variants) => Some(start_index + variants.len() as u64),
            Self::Value(variants) => {
                let mut max_value = u64::MIN;
                for (_, _, value) in variants {
                    match value {
                        VariantValue::Literal(v) => max_value = std::cmp::max(max_value, *v),
                        VariantValue::Expr(_) => return None,
                    }
                }
                Some(max_value)
            }
        }
    }

    pub fn get_info(&self, start_index: u64) -> Vec<(&[String], &str, VariantValue)> {
        match self {
            Self::Simple(variants) => variants
                .iter()
                .enumerate()
                .map(|(i, (docs, name))| {
                    (
                        docs.as_slice(),
                        name.as_str(),
                        VariantValue::Literal(start_index + i as u64),
                    )
                })
                .collect(),
            Self::Value(variants) => variants
                .iter()
                .map(|(docs, name, index)| (docs.as_slice(), name.as_str(), index.clone()))
                .collect(),
        }
    }
}
