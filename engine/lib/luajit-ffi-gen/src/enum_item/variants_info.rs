pub enum VariantsInfo {
    Simple(Vec<String>),
    Value(Vec<(String, u64)>),
}

impl VariantsInfo {
    pub fn max_discriminant(&self, start_index: u64) -> u64 {
        match self {
            VariantsInfo::Simple(variants) => start_index + variants.len() as u64,
            VariantsInfo::Value(variants) => variants.iter().map(|v| v.1).max().unwrap_or(0),
        }
    }

    pub fn get_names(&self) -> Vec<&str> {
        match self {
            VariantsInfo::Simple(variants) => variants.iter().map(|v| v.as_str()).collect(),
            VariantsInfo::Value(variants) => variants.iter().map(|(v, _)| v.as_str()).collect(),
        }
    }

    pub fn get_pairs(&self, start_index: u64) -> Vec<(String, u64)> {
        match self {
            VariantsInfo::Simple(variants) => variants
                .iter()
                .enumerate()
                .map(|(i, v)| (v.clone(), start_index + i as u64))
                .collect(),
            VariantsInfo::Value(variants) => variants.clone(),
        }
    }
}
