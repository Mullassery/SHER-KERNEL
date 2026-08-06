GitHub Guide: SHER Kernel

Welcome to the SHER Kernel repository. This guide helps you navigate the project and understand its structure.

## What is SHER Kernel?

SHER Kernel is a ground-up reimagining of operating system architecture for the AI era. It's not a Linux fork—it's a completely new kernel design built for artificial intelligence workloads, with strong security guarantees and capability-based access control.

**Key Innovation**: SHER proves that you can run existing Linux drivers on a fundamentally different kernel architecture without inheriting Linux's internal design constraints.

## Quick Navigation

### For the Impatient (5 minutes)
- Start with the README overview
- Check QUICK_START.md for running tests
- Look at PERFORMANCE_METRICS.md for actual numbers

### For Developers (30 minutes)
- Read CLAUDE.md for architecture specifications
- Review ARCHITECTURE.md for system design
- Check .github/CONTRIBUTING.md for development guidelines
- Explore crates/ directory structure

### For System Designers (1 hour)
- Study ENGINEERING_CHARTER.md for design philosophy
- Review ROADMAP.md for long-term vision
- Examine sher_memory_architecture.md and linux_memory_analysis.md
- Check test coverage and performance benchmarks

### For Researchers (2+ hours)
- Deep dive into ARCHITECTURE.md and CLAUDE.md
- Analyze AI services implementation in crates/ai/
- Review capability-based security system in crates/security/
- Study Linux Kernel Interface translation layer in crates/lki/

## Repository Structure

```
SHER-Kernel/
├── README.md                    # Start here: Project overview
├── QUICK_START.md              # Get running in 5 minutes
├── PERFORMANCE_METRICS.md      # Actual benchmark results
├── ARCHITECTURE.md             # System design deep-dive
├── CLAUDE.md                   # Complete architecture spec
├── LICENSE                     # Attribution-based license
├── .github/
│   ├── CONTRIBUTING.md         # How to contribute
│   ├── CODE_OF_CONDUCT.md      # Community standards
│   └── GITHUB_GUIDE.md         # This file
└── crates/
    ├── common/                 # Shared types and utilities
    ├── objectmodel/            # Core kernel object model
    ├── security/               # Capability-based security system
    ├── memory/                 # Lock-free memory allocator (750 LOC, 50+ tests)
    ├── device_manager/         # Hardware discovery (1,800 LOC, 65+ tests)
    ├── driver_runtime/         # Isolated driver execution (2,600 LOC, 81 tests)
    ├── lki/                    # Linux Kernel Interface (2,727 LOC, 72 tests)
    ├── ai/                     # AI-native services (3,018 LOC, 48 tests)
    └── kernel/                 # Main kernel entry point
```

## Key Features

### Anomaly Detection
- MemoryLeakDetector: 50MB/s threshold
- InterruptStormDetector: 10k+/sec threshold
- DmaAbuseDetector: 100+ concurrent ops detection

### Predictive Allocation
- 1-second ahead predictions with confidence scoring
- Exponential moving average learning
- CPU affinity and NUMA optimization

### Adaptive Scheduling
- Real-time strategy selection (4 strategies)
- SLO tracking and enforcement
- Automatic strategy switching

### Continuous Learning
- Behavior model with peak/average tracking
- Correlation analysis (CPU-memory, CPU-latency, IO-latency)
- Trend prediction

### Inference Engine
- 8-dimensional feature vector
- Four decision types
- <1ms inference latency

### Reinforcement Learning
- 7 reward signal types
- Per-driver policy learning
- Global policy aggregation

## Test Coverage

- 335+ comprehensive tests
- 100% pass rate
- All subsystems tested independently
- Run with: `cargo test --lib`

## Performance

- Memory allocation: <0.2μs (vs Linux ~0.25μs)
- Device management: -40% to <35% overhead (better than Linux in many cases)
- Lock-free allocation fast-path: <50ns target
- Overall overhead: <25% vs Linux for comparable features

## Development

### Building
```bash
cargo build --release
```

### Running Tests
```bash
cargo test --lib
```

### Checking Code Quality
```bash
cargo check
cargo clippy --all-targets
```

### Reading Documentation
- Inline code documentation in all modules
- Architecture overview in ARCHITECTURE.md
- Design decisions in CLAUDE.md
- Performance analysis in PERFORMANCE_METRICS.md

## Project Status

**Phase 6 (Week 3) Complete**: AI Services Foundation
- Anomaly detection engines
- Predictive resource allocation
- Adaptive scheduling
- Continuous learning
- Inference engine
- Reinforcement learning

**Total**: 14,095 LOC, 335+ tests, 100% pass rate

## Philosophy

SHER Kernel embodies four design principles:

1. **AI-Native**: Artificial intelligence is OS infrastructure, not an application
2. **Compatibility Without Dependency**: Linux driver ecosystem support without Linux inheritance
3. **Modular by Design**: Every subsystem is independently replaceable and testable
4. **Security by Architecture**: Capability-based permissions from first principles

## Next Steps

1. **Star the Repository** if you find this interesting
2. **Read the README** for comprehensive overview
3. **Run the Tests** to see it in action
4. **Review CONTRIBUTING.md** if you want to participate
5. **Open Issues** for bugs or feature requests

## Important Links

- **GitHub**: https://github.com/Mullassery/SHER-KERNEL
- **Author Email**: mullassery@gmail.com
- **License**: Attribution-based (see LICENSE file)

## Community

- Code of Conduct: See .github/CODE_OF_CONDUCT.md
- Contributing Guidelines: See .github/CONTRIBUTING.md
- Discussion: Use GitHub issues for discussions

---

SHER Kernel: Where AI meets systems architecture. Not evolution. Revolution.
