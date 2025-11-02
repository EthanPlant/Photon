# Photon Roadmap
## Milestone 1 - Core Foundation
**Goal**: Build the minimal, auditable kernel infrastructure to support modular, safe, and async-first operations
- Async Runtime
    - Event driven scheduler
    - Support for thousands of in-flight tasks
    - Cooperative scheduler for low-latency I/O
- Memory Manager
    - Safe allocation for kernel and userspace
    - Capability Scoped memory regions
    - Minimal unsafe code confined to hardware primitives
- Capability Manager:
    - Issue, track, and revoke unforgable tokens
    - Enforce least privilege for kernel and userspace
- Namespace Manager
    - Define logical scopes of resources
    - Support nested or hierarchical namespaces
    - Map processes and kernel modules to namespace-specific capabilities
- Hardware abstraction layer
    - Minimal drivers for early boot and basic I/O
    - Setup for async I/O integration
## Phase 2 - Kernel Modules
**Goal** : Introduce modularity and high-performance drivers
- Device Drivers: NVMe, network, DMA, USB
- Filesystem Modules: Ext4, FAT
- Newtwork Stack: TCP/IP, routing
- Module Infrastructure
    - Dynamic loading/unloading
    - Capability enforcement for each module
    - Namespace-aware resource access
- Async Integration
    - Kernel modules submit and complete async requests via unified queues
    - Event-driven completion notifications for userspace tasks
## Phase 3 - Userspace and Async Services
**Goal**: Enable secure, multi-tenant services using the kernel primitives
- Rust Services
    - Direct access to capabilities
    - Async-first APIs for storage, network, and IPC
- Sanboxed Wasm Modules:
    - Multi-tenant or untrusted extensions
    - Restricted capabilities and namespace scoping
- Namespace-Aware Async IPC
    - Scoped communication between processes and kernel modules
    - Isolation accross tenants or service groups
- Auditing and Logging
    - Track capability creation, delegation, and revocation
    - Namespace-aware logs for debugging and security audits
## Phase 4 – Ecosystem Expansion
**Goal**: Grow the OS into a fully functional platform for real workloads.
- Extended Hardware Support
    - Additional NICs, storage devices, GPUs, accelerators
- High-Throughput Services
    - Storage servers, networking proxies, compute services
    - Optimized async pipelines
- Developer Tooling
    - Debuggers, performance analyzers, module creation frameworks
- Documentation & Tutorials
    - For kernel development, module creation, async programming, and namespace usage
- Community Contributions:
    - Open-source ecosystem for modules, drivers, and services
## Phase 5 – Long-Term Goals
- Formal Verification / Auditable Kernel Proofs
    - Reduce unsafe surfaces further
    - Verify core subsystems (scheduler, memory, capability, namespace managers)
- Multi-Tenant Cloud-Oriented Features
    - Per-tenant resource quotas
    - Secure delegation of capabilities between tenants
- Performance Optimizations
    - Async batching and queue optimizations
    - Kernel thread minimization and zero-copy I/O pipelines