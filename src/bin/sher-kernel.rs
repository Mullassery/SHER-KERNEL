// SHER Kernel - Main Binary Entry Point
//
// IMPORTANT: this is a userspace CLI over a Rust workspace that prototypes
// what a future OS kernel's object model and subsystem APIs might look
// like. It is NOT a bootable kernel: there is no bootloader, no ring-0
// code, and no bare-metal hardware access anywhere in this workspace. It
// runs as an ordinary process under whatever OS you invoke it from. See
// README.md and CLAUDE.md for the full, accurate status.

use std::env;
use std::process;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("SHER Kernel prototype {}", version);
                process::exit(0);
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--status" => {
                print_status();
                process::exit(0);
            }
            "--info" => {
                print_info();
                process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Use --help for usage information");
                process::exit(1);
            }
        }
    } else {
        print_welcome();
    }
}

fn print_welcome() {
    println!("SHER Kernel — userspace architecture prototype (not a bootable kernel)");
    println!();
    println!("Strength. Resilience. Intelligence. Adaptability.");
    println!();
    println!("This process demonstrates object-model, scheduling, memory-bookkeeping,");
    println!("and driver-lifecycle APIs that a future real kernel might expose. It");
    println!("runs as an ordinary program on your existing OS.");
    println!();
    println!("Use --help for more information, --status for what's implemented.");
}

fn print_help() {
    println!("SHER Kernel prototype - Usage");
    println!();
    println!("Usage: sher-kernel [COMMAND]");
    println!();
    println!("Commands:");
    println!("  --version, -v    Show version information");
    println!("  --help, -h       Show this help message");
    println!("  --status         Show what's implemented vs. simulated");
    println!("  --info           Show detailed project information");
    println!();
    println!("Documentation:");
    println!("  README.md         Accurate project status and scope");
    println!("  CLAUDE.md         Architecture and implementation guide");
    println!("  API_REFERENCE.md  Per-crate API reference");
    println!();
    println!("Repository:");
    println!("  https://github.com/Mullassery/SHER-KERNEL");
}

fn print_status() {
    println!("SHER Kernel prototype — Status Report");
    println!("======================================");
    println!();
    println!("This is a userspace Rust workspace, not a bootable kernel.");
    println!("Run `cargo test --workspace` for the current, authoritative test count.");
    println!();
    println!("Real, tested (userspace logic):");
    println!("  - Object model: identity, lifecycle, capabilities, telemetry");
    println!("  - Priority scheduler, memory allocator/slab tiers, timer wheel");
    println!("  - Device registry, driver container lifecycle, hot-plug simulation");
    println!("  - Security: capability grants w/ expiry, sandbox policy, audit log");
    println!("  - Snapshot/rollback store, transactional updater state machine");
    println!();
    println!("Explicitly simulated (no real hardware/kernel privilege access):");
    println!("  - CPU/MMU/interrupt-controller bring-up (would need ring-0)");
    println!("  - GPU/audio/input device I/O (see each crate's module docs)");
    println!();
    println!("See README.md for the full, current breakdown.");
}

fn print_info() {
    println!("SHER Kernel prototype - Detailed Information");
    println!("==============================================");
    println!();
    println!("Project Description:");
    println!("  A userspace Rust workspace prototyping the object model,");
    println!("  scheduling, memory bookkeeping, and driver-lifecycle APIs that a");
    println!("  future OS kernel might expose. It is not a bootable kernel: no");
    println!("  bootloader, no ring-0 code, no bare-metal drivers.");
    println!();
    println!("Design goals being explored:");
    println!("  - Capability-based, time-bounded permissions");
    println!("  - Isolated driver lifecycle with crash recovery");
    println!("  - AI-assisted anomaly detection and adaptive scheduling (simulated)");
    println!("  - A/B immutable-update model with instant rollback");
    println!();
    println!("Documentation:");
    println!("  README.md          Accurate project status and scope");
    println!("  CLAUDE.md           Architecture and implementation guide");
    println!("  API_REFERENCE.md    Per-crate API reference");
    println!();
    println!("Repository:");
    println!("  https://github.com/Mullassery/SHER-KERNEL");
    println!();
    println!("Author:");
    println!("  Georgi Mammen Mullassery");
    println!("  mullassery@gmail.com");
}
