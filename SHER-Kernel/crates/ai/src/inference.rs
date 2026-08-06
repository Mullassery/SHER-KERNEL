use sher_common::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngine {
    pub model_name: Option<String>,
    pub inferences_completed: u64,
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self {
            model_name: None,
            inferences_completed: 0,
        }
    }
}

impl InferenceEngine {
    pub fn load_model(&mut self, model_name: impl Into<String>) -> Result<()> {
        self.model_name = Some(model_name.into());
        Ok(())
    }

    pub fn run_inference(&mut self, _input: Vec<f32>) -> Result<Vec<f32>> {
        self.inferences_completed += 1;
        Ok(vec![0.0])
    }
}
