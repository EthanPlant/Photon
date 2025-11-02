# Photon - High Level Design
## Vision
Photon is a Rust-first, secure-by-construction operating system for modern hardware. It combines the performance of a monolithic kernel, modular architecture, capability-based security, async-first I/O, and namespaces for resource isolation.

## 1. Guiding Principles

1. Safety as Default
    - Memory safety enforced via Rust across the kernel and modules
    - Unsafe code isolated to small, auditable low-level primitives (device registers, DMA, interrupts)
    - Modular interfaces reduce kernel complexity and increase reviewability
2. Capabilities Over Global State
    - Access to all kernel resources—devices, memory regions, IPC channels—is controlled via unforgeable tokens
    - Delegation is explicit and auditable, even for kernel modules
    - Modules cannot escalate privileges outside their granted capabilities
3. Async-First I/O
    - Kernel and modules expose asynchronous interfaces for storage, networking, GPU, and DMA
    - Thousands of in-flight operations share event-driven queues, minimizing thread overhead
    - Userspace services directly leverage kernel async primitives for high-performance workflows
4. Modularity within the Kernel
    - Core kernel remains small: scheduler, memory manager, async runtime, capability manager
    - Device drivers, filesystem handlers, and network protocols are loadable modules
    - Modules interact through well-defined async-capability interfaces
5. Namespaces for Isolation
    - Namespaces define logical scopes of resources for processes and modules
    - Each namespace contains its own set of capabilities
    - Namespaces enable multi-tenant isolation and composable permissions without requiring a microkernel.

## 2. Kernel Architecture
```
+-------------------------------------------------+
|                   Userspace                     |
|  +----------------+   +----------------------+  |
|  | Rust Services  |   | Sandboxed Wasm Apps  |  |
|  +----------------+   +----------------------+  |
|        Async I/O & Namespaced Capabilities      |
+-------------------------------------------------+
|                    Kernel                       |
|  +--------------------------------------------+ |
|  | Core Kernel:                               | |
|  | - Scheduler                                | |
|  | - Memory Manager                           | |
|  | - Async Runtime                            | |
|  | - Capability Manager                       | |
|  | - Namespace Manager                        | |
|  +--------------------------------------------+ |
|  | Loadable Modules:                          | |
|  | - Device Drivers                           | |
|  | - Filesystem                               | |
|  | - Network Stack                            | |
|  | - Optional Services                        | |
|  +--------------------------------------------+ |
|                 Hardware Abstraction            |
+-------------------------------------------------+
|                    Hardware                     |
+-------------------------------------------------+
```

### 2.1 Core Kernel Subsystems
1. Scheduler and Async Runtime
    - Drives async tasks across kernel and userspace
    - Cooperative scheduling minimizes context switches and thread overhead
    - Prioritizes I/O-bound tasks for low-latency response
2. Memory Manager
    - Manages kernel and userspace memory safely
    - Memory regions require capability tokens for access
    - Supports allocations for kernel and userspace tasks
3. Capability Manager
    - Issues unforgable tokens for hardware, IPC, and memory
    - Tracks delegation, revocation, and usage audit trails
4. Namespace Manager
    - Represents logical resource scopes (filesystems, devices, network interfaces, IPC services)
    - Maps processes/modules to namespaces
    - Ensures only allowed capabilities are visible per namespace
    - Supports nested or hierarchical namespaces for multi-tenant isolation

### 2.2 Loadable Kernel Modules
- Device Drivers: NVMe, network, GPU, DMA, USB
- Filesystem modules: Ext4, FAT
- Networking Modules: TCP/IP, routing
- Optional Services: Logging, metrics, auditing, or security extensions
#### Characteristics
- Modules request capabilities scoped to their namespace
- Async interfaces connect modules to kernel queues and userspace
- Modules remain small and auditable, easing reasoning about correctness

## 3. Userspace Model
- Rust ServiceS: Native performance, type-safe, namespace-aware capability access
- Sandboxed Wasm modules: Isolated components restricted to namespace-assigned capabilities
- Async Service Composition: Userspace services communicate via namespace-aware async-capability APIs

## 4. Security Model
- Least Privilege: Capabilities are required for every resource; namespace scoping prevents overreach
- Explicit Delegation: Capabilities can only be delegated within allowed namespaces
- Auditability: Capability creation, usage, and revocation logged per namespace
- Memory Safety: Rust ownership prevents dangling references; unsafe code is isolated
- Sandboxing: Userspace and optional kernel modules can be sandboxed to prevent cross-namespace interference

## 5. I/O Model
- Unified async interface for access to hardware
- Async submission/completion queues are namespace-aware: only tasks within a namespace see their events
- Kernel modules and userspace tasks share coherent async primitives
- Optimized for high-throughput, low-latency workloads

## 6. Extensibility
- Dynamic Module Loading: Drivers and subsystems can be added/removed at runtime
- Third-party extensions: Run in restricted namespaces or sandboxed Wasm
- Composability: Async-first design allows building multi-tenant services safely

## 7. Performance Considerations
- Minimize kernel threads; leverage async tasks for concurrency
- Namespace-aware queues reduce cross-task interference
- Modular design keeps code auditable while maintaining low-latency access