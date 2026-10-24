# Security Policy

## Supported versions

SHER Kernel is an experimental research prototype. There is no supported
release, security-maintenance window, or SLA.

## Reporting a vulnerability

Please do not open a public issue for a security vulnerability. Use GitHub's
[private vulnerability reporting](https://github.com/Mullassery/SHER-KERNEL/security/advisories/new)
for this repository. If that is unavailable, open an issue titled
"Security contact request" with no vulnerability details.

## Security model

**This project is not a security boundary.** It is a userspace prototype of
kernel object-model, scheduling, memory, and driver-lifecycle concepts. It does
not boot, does not isolate real hardware, and must not be used to protect
untrusted workloads.

The zero-trust and driver-isolation ideas in the README describe design intent
and tested object-model behavior, not a verified kernel security guarantee.
Treat every capability claim as experimental until an independent review
exists.

## Scope

All code in this repository is in scope for correctness and memory-safety
reports. Claims about production isolation, hardware enforcement, or Linux
compatibility are explicitly out of scope until implemented and verified.
