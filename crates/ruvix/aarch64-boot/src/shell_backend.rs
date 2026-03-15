//! Shell backend implementation for bare-metal kernel
//!
//! Provides a minimal ShellBackend implementation with stub data
//! to demonstrate the shell infrastructure without full kernel services.

#![allow(dead_code)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;

use ruvix_shell::{
    ShellBackend, KernelInfo, MemoryStats, TaskInfo, TaskState, CpuInfo,
    QueueStats, VectorStats, ProofStats, CapEntry, WitnessEntry, PerfCounters,
};

/// PL011 UART registers
const UART_BASE: usize = 0x0900_0000;
const UART_DR: usize = UART_BASE + 0x00;  // Data register
const UART_FR: usize = UART_BASE + 0x18;  // Flag register
const UART_FR_RXFE: u32 = 1 << 4;         // RX FIFO empty

/// Minimal backend for demonstration
pub struct BareMetal {
    trace_enabled: bool,
    boot_time_ns: u64,
}

impl BareMetal {
    /// Create a new bare-metal backend
    pub fn new() -> Self {
        Self {
            trace_enabled: false,
            boot_time_ns: 0, // Would normally read from timer
        }
    }

    /// Poll UART for input (non-blocking)
    ///
    /// Returns Some(byte) if a character is ready, None otherwise
    pub fn poll_uart(&self) -> Option<u8> {
        let flags = unsafe { core::ptr::read_volatile(UART_FR as *const u32) };
        
        if (flags & UART_FR_RXFE) == 0 {
            // RX FIFO not empty, read the byte
            let data = unsafe { core::ptr::read_volatile(UART_DR as *const u8) };
            Some(data)
        } else {
            None
        }
    }
}

impl Default for BareMetal {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellBackend for BareMetal {
    fn kernel_info(&self) -> KernelInfo {
        KernelInfo {
            version: env!("CARGO_PKG_VERSION"),
            build_time: option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("2024-01-01T00:00:00Z"),
            boot_time_ns: self.boot_time_ns,
            current_time_ns: self.boot_time_ns + 60_000_000_000, // Fake +60s
            cpu_count: 1,
        }
    }

    fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_bytes: 128 * 1024 * 1024, // 128 MiB
            used_bytes: 4 * 1024 * 1024,    // ~4 MiB kernel
            free_bytes: 124 * 1024 * 1024,
            region_count: 0,
            slab_count: 0,
            peak_bytes: 4 * 1024 * 1024,
        }
    }

    fn task_list(&self) -> Vec<TaskInfo> {
        vec![
            TaskInfo {
                id: 0,
                name: *b"idle\0\0\0\0\0\0\0\0\0\0\0\0",
                state: TaskState::Running,
                priority: 0,
                partition: 0,
                cpu_affinity: 0x01,
                cap_count: 0,
            }
        ]
    }

    fn cpu_info(&self) -> Vec<CpuInfo> {
        vec![
            CpuInfo {
                id: 0,
                online: true,
                is_primary: true,
                freq_mhz: 1800,
                load_percent: 5,
            }
        ]
    }

    fn queue_stats(&self) -> QueueStats {
        QueueStats {
            queue_count: 0,
            pending_messages: 0,
            messages_sent: 0,
            messages_received: 0,
            zero_copy_count: 0,
        }
    }

    fn vector_stats(&self) -> VectorStats {
        VectorStats {
            store_count: 0,
            vector_count: 0,
            total_dimensions: 0,
            memory_bytes: 0,
            reads: 0,
            writes: 0,
        }
    }

    fn proof_stats(&self) -> ProofStats {
        ProofStats {
            generated: 0,
            verified: 0,
            rejected: 0,
            cache_entries: 0,
            cache_hits: 0,
            cache_misses: 0,
            tier0_count: 0,
            tier1_count: 0,
            tier2_count: 0,
        }
    }

    fn capability_entries(&self, _task_id: Option<u32>) -> Vec<CapEntry> {
        vec![]
    }

    fn witness_entries(&self, _count: usize) -> Vec<WitnessEntry> {
        vec![]
    }

    fn perf_counters(&self) -> PerfCounters {
        PerfCounters {
            syscalls: 0,
            context_switches: 0,
            interrupts: 0,
            page_faults: 0,
            ipi_sent: 0,
            cpu_cycles: 0,
        }
    }

    fn trace_enabled(&self) -> bool {
        self.trace_enabled
    }

    fn set_trace(&mut self, enabled: bool) {
        self.trace_enabled = enabled;
    }

    fn reboot(&mut self) {
        uart_puts("\nRebooting system...\n");
        // In real implementation, would trigger PSCI reset
        loop {
            unsafe { core::arch::asm!("wfi", options(nostack, nomem)); }
        }
    }
}

/// Write a byte to UART
fn uart_putc(c: u8) {
    const UART_BASE: *mut u8 = 0x0900_0000 as *mut u8;
    unsafe { core::ptr::write_volatile(UART_BASE, c); }
}

/// Write a string to UART
fn uart_puts(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            uart_putc(b'\r');
        }
        uart_putc(b);
    }
}
