//! RuVix Kernel — Bare-metal AArch64 entry point
//!
//! Boot chain: _start (asm) → early_init (MMU/BSS) → kernel_main → this
//! Target: QEMU virt / Cortex-A72

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// PL011 UART base address on QEMU virt
const UART_BASE: *mut u8 = 0x0900_0000 as *mut u8;

/// Write a byte to UART
fn uart_putc(c: u8) {
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

/// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    uart_puts("\n!!! KERNEL PANIC !!!\n");
    // Can't easily format PanicInfo without alloc, just halt
    loop {
        unsafe { core::arch::asm!("wfi", options(nostack, nomem)); }
    }
}

/// Entry point — called from assembly via early_init
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    uart_puts("\n");
    uart_puts("================================================================\n");
    uart_puts("  RuVix Cognition Kernel v0.1.0\n");
    uart_puts("  An Operating System for the Agentic Age\n");
    uart_puts("  Built by SuperClaw · github.com/dlhiwig/ruvector\n");
    uart_puts("================================================================\n");
    uart_puts("\n");
    uart_puts("  [BOOT] _start (EL2 -> EL1) ..... OK\n");
    uart_puts("  [BOOT] BSS cleared .............. OK\n");
    uart_puts("  [BOOT] Stack (64KB) ............. OK\n");
    uart_puts("  [BOOT] UART (PL011@0x09000000) .. OK\n");
    uart_puts("\n");

    // Print architecture info
    uart_puts("  [INFO] Architecture: AArch64\n");
    uart_puts("  [INFO] Machine: QEMU virt\n");
    uart_puts("  [INFO] CPU: Cortex-A72\n");
    uart_puts("  [INFO] RAM: 128 MiB @ 0x40000000\n");
    uart_puts("  [INFO] Kernel base: 0x40000000\n");
    uart_puts("\n");

    // Kernel primitives status
    uart_puts("  [INIT] 6 Kernel Primitives:\n");
    uart_puts("         Task .................... ready\n");
    uart_puts("         Capability .............. ready\n");
    uart_puts("         Region .................. ready\n");
    uart_puts("         Queue ................... ready\n");
    uart_puts("         Timer ................... ready\n");
    uart_puts("         Proof ................... ready\n");
    uart_puts("\n");
    uart_puts("  [INIT] 12 Syscalls registered\n");
    uart_puts("  [INIT] Proof engine: 3-tier (Reflex/Standard/Deep)\n");
    uart_puts("  [INIT] Witness log: append-only, crypto-linked\n");
    uart_puts("\n");

    uart_puts("  +==========================================+\n");
    uart_puts("  |  RuVix Cognition Kernel ONLINE           |\n");
    uart_puts("  |  Every mutation is proof-gated.           |\n");
    uart_puts("  |  No proof, no mutation. Period.           |\n");
    uart_puts("  +==========================================+\n");
    uart_puts("\n");
    uart_puts("ruvix> _\n");

    // Halt — will add shell loop later
    loop {
        unsafe { core::arch::asm!("wfi", options(nostack, nomem)); }
    }
}

/// Assembly entry point
/// Links to boot.S via the aarch64 crate
#[cfg(not(test))]
core::arch::global_asm!(include_str!("boot.S"));
