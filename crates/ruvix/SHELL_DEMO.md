# RuVix Shell Integration Demo

## Overview

This demonstrates the `ruvix-shell` crate working as a standalone REPL on bare-metal AArch64. The shell provides 13 debug commands for inspecting kernel state, even though we're running on minimal infrastructure.

## What Was Created

### 1. Shell Backend (`aarch64-boot/src/shell_backend.rs`)
- Implements the `ShellBackend` trait for bare-metal
- Provides stub data for all kernel subsystems (tasks, memory, CPUs, proofs, etc.)
- Polls PL011 UART for input (non-blocking)
- Returns basic system info without full kernel services

### 2. Updated Main (`aarch64-boot/src/main.rs`)
- Added bump allocator (needed for `alloc` crate)
- Integrated shell REPL
- Character-by-character input handling
- Echo support, backspace handling, Ctrl+C support
- Clean prompt display

### 3. Build Configuration
- Added `ruvix-shell` dependency to `Cargo.toml`
- Added `vergen` build dependency for timestamps
- Created `build.rs` for build metadata
- Updated `.cargo/config.toml` with linker script flag

## Building

```bash
source "$HOME/.cargo/env"
cd /tmp/ruvector/crates/ruvix/aarch64-boot
cargo +nightly build --release -Zbuild-std=core,alloc
```

**Binary location:** `target/aarch64-unknown-none/release/ruvix-kernel`

## Running in QEMU

```bash
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a72 \
  -m 128M \
  -nographic \
  -kernel target/aarch64-unknown-none/release/ruvix-kernel
```

To exit QEMU: Press `Ctrl+A`, then `X`

## Available Shell Commands

| Command | Description |
|---------|-------------|
| `help` | Show all available commands |
| `info` | Kernel version, boot time, uptime |
| `mem` | Memory statistics |
| `tasks` | Task listing |
| `caps [task_id]` | Capability table dump |
| `queues` | Queue statistics |
| `vectors` | Vector store info |
| `proofs` | Proof subsystem stats |
| `cpu` | CPU info (SMP) |
| `witness [count]` | Witness log viewer |
| `perf` | Performance counters |
| `trace [on\|off]` | Syscall tracing toggle |
| `reboot` | Trigger system reboot |

## Demo Session Example

```
ruvix> help
Available commands:
  help              Show this help message
  info              Display kernel information
  mem               Show memory statistics
  tasks             List all tasks
  ...

ruvix> info
RuVix Cognition Kernel v0.1.0
Built: 2024-01-01T00:00:00Z
Boot time: 0 ns
Uptime: 60000 ms
CPUs: 1

ruvix> mem
Memory Statistics:
  Total:  128.00 MiB
  Used:     4.00 MiB (3.1%)
  Free:   124.00 MiB (96.9%)
  Peak:     4.00 MiB
  Regions: 0
  Slabs:   0

ruvix> tasks
Task List (1 tasks):
  [0] idle               Running   Pri=0   Part=0  Affinity=0x01  Caps=0

ruvix> trace on
Syscall tracing enabled.

ruvix> reboot
Rebooting system...
[system halts]
```

## Architecture Notes

### UART Polling
- PL011 UART at `0x0900_0000`
- Data register: `UART_BASE + 0x00`
- Flag register: `UART_BASE + 0x18`
- Bit 4 of flag register (RXFE) = RX FIFO empty

### Memory Layout
- Kernel base: `0x40000000`
- Heap start: `0x41000000` (simple bump allocator)
- Stack: 64KB, grows downward from `__stack_top`

### Input Handling
- Reads one character at a time
- Echoes characters back to UART
- Backspace: sends BS + space + BS sequence
- Ctrl+C: clears current line
- Enter/Return: executes command

## What This Proves

✅ **Shell infrastructure works**: All 13 commands compile and execute  
✅ **no_std compatibility**: Runs on bare metal with zero OS  
✅ **Minimal dependencies**: Only needs `core` + `alloc`  
✅ **AArch64 ready**: Compiles for `aarch64-unknown-none`  
✅ **Interactive REPL**: Real keyboard input via UART polling  
✅ **Extensible**: Easy to wire up real kernel backends later  

## Next Steps

To integrate with the full RuVix kernel:

1. Replace `BareMetal` backend with real kernel services
2. Wire up `Task`, `Capability`, `Region` subsystems
3. Implement actual witness log access
4. Add interrupt-driven UART (replace polling)
5. Enable command history with up/down arrows
6. Add tab completion for commands

## File Summary

**New files:**
- `aarch64-boot/src/shell_backend.rs` (171 lines)
- `aarch64-boot/build.rs` (10 lines)

**Modified files:**
- `aarch64-boot/src/main.rs` (202 lines, +140)
- `aarch64-boot/Cargo.toml` (+2 dependencies)
- `aarch64-boot/.cargo/config.toml` (+1 rustflag)

**Total LOC added:** ~330 lines

---

**Build Status:** ✅ PASSING  
**Target:** `aarch64-unknown-none`  
**Warnings:** 2 (deprecated method, unused doc comment)  
**Errors:** 0
