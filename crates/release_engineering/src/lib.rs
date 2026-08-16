//! Phase 13: Release Engineering
//!
//! Production release management:
//! - Semantic versioning
//! - Changelog generation
//! - Release artifacts
//! - Distribution tracking
//! - Quality gates

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeType {
    Feature,
    Enhancement,
    BugFix,
    Security,
    Performance,
    Documentation,
}

#[derive(Clone, Debug)]
pub struct ChangeLogEntry {
    pub version: Version,
    pub date: String,
    pub change_type: ChangeType,
    pub description: String,
    pub breaking: bool,
}

#[derive(Clone, Debug)]
pub struct ReleaseArtifact {
    pub version: Version,
    pub artifact_type: String,
    pub size_bytes: usize,
    pub checksum: String,
    pub download_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseStatus {
    Development,
    Beta,
    ReleaseCandidate,
    Stable,
    LongTermSupport,
}

pub struct ReleaseManager {
    current_version: Version,
    changelog: Vec<ChangeLogEntry>,
    artifacts: Vec<ReleaseArtifact>,
    status: ReleaseStatus,
    quality_gates: HashMap<String, bool>,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }

    pub fn bump_major(&self) -> Version {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }

    pub fn bump_minor(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
        }
    }

    pub fn bump_patch(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
        }
    }
}

impl ReleaseManager {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        ReleaseManager {
            current_version: Version::new(major, minor, patch),
            changelog: Vec::new(),
            artifacts: Vec::new(),
            status: ReleaseStatus::Development,
            quality_gates: HashMap::new(),
        }
    }

    pub fn get_version(&self) -> &Version {
        &self.current_version
    }

    pub fn set_version(&mut self, version: Version) {
        self.current_version = version;
    }

    pub fn add_changelog_entry(&mut self, entry: ChangeLogEntry) {
        self.changelog.push(entry);
    }

    pub fn register_artifact(&mut self, artifact: ReleaseArtifact) {
        self.artifacts.push(artifact);
    }

    pub fn set_status(&mut self, status: ReleaseStatus) {
        self.status = status;
    }

    pub fn get_status(&self) -> &ReleaseStatus {
        &self.status
    }

    pub fn set_quality_gate(&mut self, gate_name: String, passed: bool) {
        self.quality_gates.insert(gate_name, passed);
    }

    pub fn all_gates_passed(&self) -> bool {
        self.quality_gates.values().all(|&passed| passed)
    }

    pub fn gate_status(&self, gate_name: &str) -> Option<bool> {
        self.quality_gates.get(gate_name).cloned()
    }

    pub fn get_changelog(&self) -> &[ChangeLogEntry] {
        &self.changelog
    }

    pub fn get_artifacts(&self) -> &[ReleaseArtifact] {
        &self.artifacts
    }

    pub fn changelog_summary(&self, version: &Version) -> Vec<ChangeLogEntry> {
        self.changelog
            .iter()
            .filter(|entry| &entry.version == version)
            .cloned()
            .collect()
    }

    pub fn has_breaking_changes(&self) -> bool {
        self.changelog.iter().any(|entry| entry.breaking)
    }

    pub fn total_artifacts(&self) -> usize {
        self.artifacts.len()
    }

    pub fn total_artifact_size(&self) -> usize {
        self.artifacts.iter().map(|a| a.size_bytes).sum()
    }

    pub fn is_release_ready(&self) -> bool {
        self.all_gates_passed() && !self.changelog.is_empty() && !self.artifacts.is_empty()
    }

    pub fn get_feature_count(&self) -> usize {
        self.changelog
            .iter()
            .filter(|e| e.change_type == ChangeType::Feature)
            .count()
    }

    pub fn get_security_fix_count(&self) -> usize {
        self.changelog
            .iter()
            .filter(|e| e.change_type == ChangeType::Security)
            .count()
    }
}

impl Default for ReleaseManager {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_to_string() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_bumping() {
        let version = Version::new(1, 2, 3);

        let major_bump = version.bump_major();
        assert_eq!(major_bump.to_string(), "2.0.0");

        let minor_bump = version.bump_minor();
        assert_eq!(minor_bump.to_string(), "1.3.0");

        let patch_bump = version.bump_patch();
        assert_eq!(patch_bump.to_string(), "1.2.4");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_release_manager_creation() {
        let manager = ReleaseManager::new(1, 0, 0);
        assert_eq!(manager.get_version().to_string(), "1.0.0");
        assert_eq!(*manager.get_status(), ReleaseStatus::Development);
    }

    #[test]
    fn test_changelog_entries() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        let entry = ChangeLogEntry {
            version: Version::new(1, 0, 0),
            date: "2026-08-07".to_string(),
            change_type: ChangeType::Feature,
            description: "Initial release".to_string(),
            breaking: false,
        };

        manager.add_changelog_entry(entry);
        assert_eq!(manager.get_changelog().len(), 1);
    }

    #[test]
    fn test_release_artifacts() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        let artifact = ReleaseArtifact {
            version: Version::new(1, 0, 0),
            artifact_type: "binary".to_string(),
            size_bytes: 1024 * 1024,
            checksum: "abc123def456".to_string(),
            download_url: "https://releases.example.com/sher-1.0.0".to_string(),
        };

        manager.register_artifact(artifact);
        assert_eq!(manager.total_artifacts(), 1);
        assert_eq!(manager.total_artifact_size(), 1024 * 1024);
    }

    #[test]
    fn test_quality_gates() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        manager.set_quality_gate("tests_passing".to_string(), true);
        manager.set_quality_gate("security_audit".to_string(), true);

        assert!(manager.gate_status("tests_passing").unwrap());
        assert!(manager.all_gates_passed());
    }

    #[test]
    fn test_quality_gate_failure() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        manager.set_quality_gate("tests_passing".to_string(), true);
        manager.set_quality_gate("security_audit".to_string(), false);

        assert!(!manager.all_gates_passed());
    }

    #[test]
    fn test_release_readiness() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        assert!(!manager.is_release_ready());

        manager.set_quality_gate("tests".to_string(), true);

        let entry = ChangeLogEntry {
            version: Version::new(1, 0, 0),
            date: "2026-08-07".to_string(),
            change_type: ChangeType::Feature,
            description: "Feature".to_string(),
            breaking: false,
        };
        manager.add_changelog_entry(entry);

        let artifact = ReleaseArtifact {
            version: Version::new(1, 0, 0),
            artifact_type: "binary".to_string(),
            size_bytes: 1024,
            checksum: "abc123".to_string(),
            download_url: "https://example.com/release".to_string(),
        };
        manager.register_artifact(artifact);

        assert!(manager.is_release_ready());
    }

    #[test]
    fn test_breaking_changes() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        let entry = ChangeLogEntry {
            version: Version::new(1, 0, 0),
            date: "2026-08-07".to_string(),
            change_type: ChangeType::Feature,
            description: "Breaking change".to_string(),
            breaking: true,
        };

        manager.add_changelog_entry(entry);
        assert!(manager.has_breaking_changes());
    }

    #[test]
    fn test_feature_and_security_counts() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        for i in 0..3 {
            let entry = ChangeLogEntry {
                version: Version::new(1, 0, 0),
                date: "2026-08-07".to_string(),
                change_type: ChangeType::Feature,
                description: format!("Feature {}", i),
                breaking: false,
            };
            manager.add_changelog_entry(entry);
        }

        for i in 0..2 {
            let entry = ChangeLogEntry {
                version: Version::new(1, 0, 0),
                date: "2026-08-07".to_string(),
                change_type: ChangeType::Security,
                description: format!("Security fix {}", i),
                breaking: false,
            };
            manager.add_changelog_entry(entry);
        }

        assert_eq!(manager.get_feature_count(), 3);
        assert_eq!(manager.get_security_fix_count(), 2);
    }

    #[test]
    fn test_changelog_for_version() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        let entry1 = ChangeLogEntry {
            version: Version::new(1, 0, 0),
            date: "2026-08-07".to_string(),
            change_type: ChangeType::Feature,
            description: "Feature 1".to_string(),
            breaking: false,
        };

        let entry2 = ChangeLogEntry {
            version: Version::new(1, 1, 0),
            date: "2026-08-08".to_string(),
            change_type: ChangeType::Feature,
            description: "Feature 2".to_string(),
            breaking: false,
        };

        manager.add_changelog_entry(entry1);
        manager.add_changelog_entry(entry2);

        let v1_changes = manager.changelog_summary(&Version::new(1, 0, 0));
        assert_eq!(v1_changes.len(), 1);
    }

    #[test]
    fn test_release_status_transitions() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        assert_eq!(*manager.get_status(), ReleaseStatus::Development);

        manager.set_status(ReleaseStatus::Beta);
        assert_eq!(*manager.get_status(), ReleaseStatus::Beta);

        manager.set_status(ReleaseStatus::Stable);
        assert_eq!(*manager.get_status(), ReleaseStatus::Stable);
    }

    #[test]
    fn test_multiple_artifacts() {
        let mut manager = ReleaseManager::new(1, 0, 0);

        for i in 0..5 {
            let artifact = ReleaseArtifact {
                version: Version::new(1, 0, 0),
                artifact_type: format!("artifact_{}", i),
                size_bytes: (i + 1) * 1024,
                checksum: format!("checksum_{}", i),
                download_url: format!("https://example.com/artifact_{}", i),
            };
            manager.register_artifact(artifact);
        }

        assert_eq!(manager.total_artifacts(), 5);
        assert_eq!(manager.total_artifact_size(), 15 * 1024);
    }
}
