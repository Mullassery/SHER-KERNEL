//! Host hardware profiling.
//!
//! `detect_memory_tier` queries the *actual* memory of the machine this
//! process is running on (via `/proc/meminfo` on Linux, `sysctl hw.memsize`
//! on macOS) and classifies it with [`MemoryTier::from_mb`]. This is real
//! detection, not a hardcoded placeholder — but note it reports total host
//! RAM, not a real embedded/IoT/desktop chassis (there is no such distinct
//! hardware here; this is a userspace process). On platforms where neither
//! detection method is available, it falls back to a documented default
//! tier and logs a warning rather than silently pretending to have
//! detected something.

use crate::budget::ResourceBudget;
use crate::tier::MemoryTier;
use sher_common::Result;
use tracing::warn;

/// Tier used when host memory cannot be determined on this platform.
pub const FALLBACK_TIER: MemoryTier = MemoryTier::Tier2Light;

/// Parse the `MemTotal:` line out of `/proc/meminfo` content, returning
/// total memory in MB. Pure function so it is testable without depending on
/// the actual host's `/proc` filesystem.
pub fn parse_proc_meminfo_mb(contents: &str) -> Option<u32> {
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb_str = rest.split_whitespace().next()?;
            let kb: u64 = kb_str.parse().ok()?;
            return Some((kb / 1024) as u32);
        }
    }
    None
}

/// Parse the byte count `sysctl -n hw.memsize` prints on macOS/BSD into MB.
pub fn parse_sysctl_memsize_mb(output: &str) -> Option<u32> {
    let bytes: u64 = output.trim().parse().ok()?;
    Some((bytes / (1024 * 1024)) as u32)
}

#[cfg(target_os = "linux")]
fn detect_total_memory_mb() -> Option<u32> {
    let contents = std::fs::read_to_string("/proc/meminfo").ok()?;
    parse_proc_meminfo_mb(&contents)
}

#[cfg(target_os = "macos")]
fn detect_total_memory_mb() -> Option<u32> {
    let output = std::process::Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    parse_sysctl_memsize_mb(&text)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn detect_total_memory_mb() -> Option<u32> {
    None
}

/// Number of logical CPUs available to this process, per
/// `std::thread::available_parallelism` (a real, stdlib-provided figure,
/// not a hardcoded constant).
pub fn detect_cpu_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
}

pub fn detect_memory_tier() -> Result<MemoryTier> {
    match detect_total_memory_mb() {
        Some(mb) => Ok(MemoryTier::from_mb(mb)),
        None => {
            warn!(
                "ARO: could not detect host memory on this platform, defaulting to {:?}",
                FALLBACK_TIER
            );
            Ok(FALLBACK_TIER)
        }
    }
}

pub fn calculate_budget(tier: &MemoryTier) -> Result<ResourceBudget> {
    Ok(ResourceBudget::for_tier(tier))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_proc_meminfo_format() {
        let sample = "MemTotal:       16384000 kB\nMemFree:         1000000 kB\n";
        assert_eq!(parse_proc_meminfo_mb(sample), Some(16000));
    }

    #[test]
    fn missing_memtotal_line_returns_none() {
        let sample = "MemFree: 1000 kB\n";
        assert_eq!(parse_proc_meminfo_mb(sample), None);
    }

    #[test]
    fn parses_sysctl_memsize_bytes() {
        // 17179869184 bytes = 16384 MB (16 GiB)
        assert_eq!(parse_sysctl_memsize_mb("17179869184\n"), Some(16384));
    }

    #[test]
    fn malformed_sysctl_output_returns_none() {
        assert_eq!(parse_sysctl_memsize_mb("not-a-number"), None);
    }

    #[test]
    fn detect_memory_tier_always_succeeds() {
        // Whatever platform this runs on, detection either finds a real
        // value or falls back — it must never error.
        assert!(detect_memory_tier().is_ok());
    }

    #[test]
    fn detect_cpu_count_is_at_least_one() {
        assert!(detect_cpu_count() >= 1);
    }
}
