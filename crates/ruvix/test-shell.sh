#!/bin/bash
# Test script for RuVix shell integration

set -e

cd "$(dirname "$0")/aarch64-boot"

echo "========================================="
echo "RuVix Shell Integration Test"
echo "========================================="
echo

# Check if binary exists
if [ ! -f "target/aarch64-unknown-none/release/ruvix-kernel" ]; then
    echo "ERROR: Kernel binary not found. Run build first:"
    echo "  cargo +nightly build --release -Zbuild-std=core,alloc"
    exit 1
fi

echo "✓ Binary exists"
echo

# Test 1: Boot and show prompt
echo "Test 1: Boot to shell prompt"
echo "----------------------------"
timeout 2 qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 128M \
    -nographic \
    -kernel target/aarch64-unknown-none/release/ruvix-kernel \
    2>&1 | grep -q "ruvix>" && echo "✓ Shell prompt detected" || echo "✗ FAILED"
echo

# Test 2: Send 'help' command
echo "Test 2: Execute 'help' command"
echo "-------------------------------"
(sleep 1; echo "help"; sleep 1) | timeout 3 qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 128M \
    -nographic \
    -kernel target/aarch64-unknown-none/release/ruvix-kernel \
    2>&1 | grep -q "Available commands" && echo "✓ Help command works" || echo "✗ FAILED"
echo

# Test 3: Send 'info' command
echo "Test 3: Execute 'info' command"
echo "-------------------------------"
(sleep 1; echo "info"; sleep 1) | timeout 3 qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 128M \
    -nographic \
    -kernel target/aarch64-unknown-none/release/ruvix-kernel \
    2>&1 | grep -q "Kernel" && echo "✓ Info command works" || echo "✗ FAILED"
echo

# Test 4: Send 'mem' command
echo "Test 4: Execute 'mem' command"
echo "------------------------------"
(sleep 1; echo "mem"; sleep 1) | timeout 3 qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 128M \
    -nographic \
    -kernel target/aarch64-unknown-none/release/ruvix-kernel \
    2>&1 | grep -q "Memory" && echo "✓ Mem command works" || echo "✗ FAILED"
echo

echo "========================================="
echo "All shell integration tests complete!"
echo "========================================="
echo
echo "To run interactively:"
echo "  qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 128M -nographic -kernel target/aarch64-unknown-none/release/ruvix-kernel"
echo
echo "To exit QEMU: Ctrl+A, then X"
