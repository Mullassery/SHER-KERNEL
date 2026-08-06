use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMapping {
    pub linux_api: String,
    pub sher_primitive: String,
}

#[derive(Debug, Clone, Default)]
pub struct TranslationEngine {
    pub mappings: Vec<TranslationMapping>,
}

impl TranslationEngine {
    pub fn add_mapping(&mut self, linux_api: impl Into<String>, sher_primitive: impl Into<String>) {
        self.mappings.push(TranslationMapping {
            linux_api: linux_api.into(),
            sher_primitive: sher_primitive.into(),
        });
    }

    pub fn translate(&self, linux_api: &str) -> Option<&str> {
        self.mappings
            .iter()
            .find(|m| m.linux_api == linux_api)
            .map(|m| m.sher_primitive.as_str())
    }
}
