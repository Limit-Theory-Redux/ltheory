pub enum VariantsInfo {
    Simple(Vec<(Vec<String>, String)>),
    Value(Vec<(Vec<String>, String, u64)>),
}

impl VariantsInfo {
    pub fn max_discriminant(&self, start_index: u64) -> u64 {
        match self {
            VariantsInfo::Simple(variants) => start_index + variants.len() as u64,
            VariantsInfo::Value(variants) => variants.iter().map(|v| v.2).max().unwrap_or(0),
        }
    }

    pub fn get_info(&self, start_index: u64) -> Vec<(&[String], &str, u64)> {
        match self {
            VariantsInfo::Simple(variants) => variants
                .iter()
                .enumerate()
                .map(|(i, (docs, name))| (docs.as_slice(), name.as_str(), start_index + i as u64))
                .collect(),
            VariantsInfo::Value(variants) => variants
                .iter()
                .map(|(docs, name, index)| (docs.as_slice(), name.as_str(), *index))
                .collect(),
        }
    }
}
