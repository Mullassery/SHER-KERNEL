use serde::{Deserialize, Serialize};
use sher_common::{ObjectId, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxKernelInterface {
    pub driver_id: ObjectId,
    pub supported_apis: Vec<String>,
}

impl LinuxKernelInterface {
    pub fn new(driver_id: ObjectId) -> Self {
        Self {
            driver_id,
            supported_apis: vec![
                "kmalloc".to_string(),
                "kzalloc".to_string(),
                "vmalloc".to_string(),
                "kfree".to_string(),
                "request_irq".to_string(),
                "pci_driver_register".to_string(),
            ],
        }
    }

    pub fn supports_api(&self, api_name: &str) -> bool {
        self.supported_apis.contains(&api_name.to_string())
    }

    pub fn call_api(&self, api_name: &str, _args: Vec<String>) -> Result<String> {
        if self.supports_api(api_name) {
            Ok(format!("Translated: {}", api_name))
        } else {
            Err(sher_common::Error::Driver(format!(
                "Unsupported API: {}",
                api_name
            )))
        }
    }
}
