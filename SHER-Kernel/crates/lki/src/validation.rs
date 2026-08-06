// SHER LKI: Validation Layer
// All Linux API calls are validated before translation

use sher_common::Error;
use serde::{Deserialize, Serialize};

// ============================================================================
// VALIDATION FRAMEWORK
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    InvalidPointer,
    InvalidSize,
    InvalidFlags,
    NullPointer,
    AlignmentViolation,
    PermissionDenied,
    ResourceExhausted,
    InvalidIrq,
    InvalidBusId,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidationError::InvalidPointer => write!(f, "Invalid pointer"),
            ValidationError::InvalidSize => write!(f, "Invalid size"),
            ValidationError::InvalidFlags => write!(f, "Invalid flags"),
            ValidationError::NullPointer => write!(f, "Null pointer"),
            ValidationError::AlignmentViolation => write!(f, "Alignment violation"),
            ValidationError::PermissionDenied => write!(f, "Permission denied"),
            ValidationError::ResourceExhausted => write!(f, "Resource exhausted"),
            ValidationError::InvalidIrq => write!(f, "Invalid IRQ number"),
            ValidationError::InvalidBusId => write!(f, "Invalid bus ID"),
        }
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::Driver(err.to_string())
    }
}

pub type ValidationResult = std::result::Result<(), ValidationError>;

// ============================================================================
// VALIDATOR
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct Validator {
    pub total_validations: u64,
    pub failed_validations: u64,
    pub max_allocation_size: u64,
    pub valid_irq_range: (u32, u32),
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            total_validations: 0,
            failed_validations: 0,
            max_allocation_size: 1024 * 1024 * 1024,  // 1GB max
            valid_irq_range: (0, 255),
        }
    }

    /// Validate memory allocation request
    pub fn validate_allocation(&mut self, size: u64, alignment: u32) -> ValidationResult {
        self.total_validations += 1;

        if size == 0 {
            self.failed_validations += 1;
            return Err(ValidationError::InvalidSize);
        }

        if size > self.max_allocation_size {
            self.failed_validations += 1;
            return Err(ValidationError::InvalidSize);
        }

        if alignment > 0 && !alignment.is_power_of_two() {
            self.failed_validations += 1;
            return Err(ValidationError::AlignmentViolation);
        }

        Ok(())
    }

    /// Validate memory deallocation
    pub fn validate_deallocation(&mut self, ptr: u64) -> ValidationResult {
        self.total_validations += 1;

        if ptr == 0 {
            self.failed_validations += 1;
            return Err(ValidationError::NullPointer);
        }

        Ok(())
    }

    /// Validate interrupt request
    pub fn validate_irq(&mut self, irq: u32) -> ValidationResult {
        self.total_validations += 1;

        if irq < self.valid_irq_range.0 || irq > self.valid_irq_range.1 {
            self.failed_validations += 1;
            return Err(ValidationError::InvalidIrq);
        }

        Ok(())
    }

    /// Validate I/O flags
    pub fn validate_flags(&mut self, flags: u32, valid_bits: u32) -> ValidationResult {
        self.total_validations += 1;

        if (flags & !valid_bits) != 0 {
            self.failed_validations += 1;
            return Err(ValidationError::InvalidFlags);
        }

        Ok(())
    }

    /// Validate pointer alignment
    pub fn validate_alignment(&mut self, ptr: u64, alignment: u32) -> ValidationResult {
        self.total_validations += 1;

        if alignment > 0 && (ptr % alignment as u64) != 0 {
            self.failed_validations += 1;
            return Err(ValidationError::AlignmentViolation);
        }

        Ok(())
    }

    /// Get validation success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_validations == 0 {
            100.0
        } else {
            ((self.total_validations - self.failed_validations) as f64 / self.total_validations as f64) * 100.0
        }
    }
}

// ============================================================================
// API SIGNATURE VALIDATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSignature {
    pub name: String,
    pub min_args: u32,
    pub max_args: u32,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SignatureValidator {
    pub signatures: std::collections::HashMap<String, ApiSignature>,
}

impl SignatureValidator {
    pub fn new() -> Self {
        SignatureValidator {
            signatures: std::collections::HashMap::new(),
        }
    }

    pub fn register_api(&mut self, sig: ApiSignature) {
        self.signatures.insert(sig.name.clone(), sig);
    }

    pub fn validate_call(&self, api_name: &str, arg_count: u32, capabilities: &[String]) -> ValidationResult {
        if let Some(sig) = self.signatures.get(api_name) {
            if arg_count < sig.min_args || arg_count > sig.max_args {
                return Err(ValidationError::InvalidFlags);
            }

            for req_cap in &sig.required_capabilities {
                if !capabilities.contains(req_cap) {
                    return Err(ValidationError::PermissionDenied);
                }
            }

            Ok(())
        } else {
            Err(ValidationError::InvalidFlags)
        }
    }
}

// ============================================================================
// VALIDATION STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_calls: u64,
    pub validated_calls: u64,
    pub failed_calls: u64,
    pub error_rate: f64,
}

impl ValidationStats {
    pub fn new() -> Self {
        ValidationStats {
            total_calls: 0,
            validated_calls: 0,
            failed_calls: 0,
            error_rate: 0.0,
        }
    }

    pub fn record_validation(&mut self, passed: bool) {
        self.total_calls += 1;
        self.validated_calls += 1;
        if !passed {
            self.failed_calls += 1;
        }

        if self.validated_calls > 0 {
            self.error_rate = (self.failed_calls as f64 / self.validated_calls as f64) * 100.0;
        }
    }
}
