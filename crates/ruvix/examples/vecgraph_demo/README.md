# VecGraph Integration Demo

This example demonstrates the RuVix VecGraph subsystem - a kernel-resident vector store with proof-gated mutations, HNSW indexing, coherence tracking, and witness logging.

## What This Demo Shows

### 1. **SIMD Capabilities Detection**
- Runtime detection of AVX2, AVX-512, NEON, and FMA support
- Automatic fallback to scalar implementations
- Lane width reporting for vector operations

### 2. **Distance Functions**
- Cosine similarity (for semantic search)
- Euclidean distance squared (L2 distance)
- L2 norm (vector magnitude)
- SIMD-accelerated when hardware supports it

### 3. **Vector Store Creation**
- 768-dimensional vectors (matching common embedding sizes)
- Capacity for 1000 vectors
- Three separate memory regions:
  - Data region (vector storage)
  - HNSW region (graph index nodes)
  - Witness region (append-only mutation log)

### 4. **Proof-Gated Mutations**
- Every vector insert requires a valid proof token
- Proof engine generates Reflex-tier proofs (<100ns target)
- Proofs include:
  - Mutation hash (ties proof to specific operation)
  - Nonce (prevents replay attacks)
  - Validity window (time-bounded authorization)
- Verification checks:
  - Hash matches mutation
  - Proof not expired
  - Nonce not reused
  - Capability has PROVE rights

### 5. **Coherence Tracking**
- Each vector has a coherence score (0.0-1.0)
- Initial coherence: 1.0 (fully coherent)
- Tracks mutation epochs
- Provides average coherence across the store
- Can be used by scheduler for prioritization

### 6. **Witness Log**
- Append-only log of all mutations
- Each entry contains:
  - Proof attestation
  - Timestamp
  - Chain hash (links to previous entry)
- Enables deterministic replay
- Provides complete audit trail

### 7. **Nearest Neighbor Search**
- Brute-force search over all vectors (demo only)
- Computes cosine distances
- Returns top-k results
- In production: would use HNSW index for sub-linear search

### 8. **HNSW Index (Infrastructure)**
- HNSW nodes allocated for each vector
- Graph structure ready for kernel-side search
- This demo shows the allocation; actual graph search requires kernel runtime

## Architecture Highlights

### Memory Regions
```rust
// Three separate slab-allocated regions
data_backing:    HeapBacking (5MB)  // Vector data
hnsw_backing:    HeapBacking (2MB)  // HNSW graph nodes
witness_backing: HeapBacking (1MB)  // Witness log
```

### Proof Flow
```
1. User computes mutation_hash = hash(key, vector_data)
2. ProofEngine.generate_reflex_proof(hash, current_time)
   └─> ProofToken { hash, tier, nonce, valid_until }
3. VectorStore.vector_put_proved(key, data, proof, cap, time)
   ├─> Recomputes mutation_hash
   ├─> Verifies proof.hash == mutation_hash
   ├─> Checks proof.valid_until > current_time
   ├─> Checks nonce not reused
   ├─> Checks capability has PROVE rights
   └─> If all pass: insert vector, emit witness entry
```

### Coherence Metadata
```rust
pub struct CoherenceMeta {
    coherence_score: u16,           // 0-10000 (0.0-1.0)
    mutation_epoch: u64,            // Monotonic counter
    proof_attestation_hash: [u8; 32],
    last_access_ns: u64,
    access_count: u32,
}
```

## Building

```bash
cd /tmp/ruvector/crates/ruvix
cargo build --manifest-path examples/vecgraph_demo/Cargo.toml
```

## Running

```bash
cargo run --manifest-path examples/vecgraph_demo/Cargo.toml
```

## Sample Output

```
═══════════════════════════════════════════════════════
   RuVix VecGraph Integration Demo
═══════════════════════════════════════════════════════

📊 SIMD Capabilities
────────────────────────────────────────────────────────
  AVX2:      ✗ Not available
  SIMD:      ✗ Scalar fallback

🗄️  Vector Store Demo
────────────────────────────────────────────────────────
  ✓ Inserted 100 vectors
    - Store size: 100
    - Witness entries: 100

  🔍 Nearest Neighbor Search
  Top 5 Nearest Neighbors:
    1. Key 22 - distance: 0.912248
    2. Key 1 - distance: 0.924157
    ...

  📈 Coherence Statistics
  Average coherence:    1.0000
  Entry count:          100
  Current epoch:        100
```

## Key Differences from Traditional Vector DBs

1. **Kernel-Resident**: Vectors live in kernel memory, not userspace
2. **Proof-Gated**: All mutations require cryptographic proof tokens
3. **Capability-Protected**: Access controlled via unforgeable capabilities
4. **Coherence-Aware**: First-class coherence metadata for scheduler hints
5. **Witness-Logged**: Complete, tamper-evident audit trail
6. **No Allocator Overhead**: Slab allocation with O(1) node management

## Integration with RuVix Kernel

This demo runs **host-side** (using HeapBacking) to demonstrate the APIs.

In the actual kernel:
- Regions use physical memory pages
- HNSW search runs in kernel with hardware acceleration
- Scheduler uses coherence scores for task prioritization
- Witness log enables distributed consensus (future)

## Performance Targets (from ADR-087)

- **Reflex proof generation**: <100ns
- **Vector insert (with proof)**: <1μs
- **HNSW search (768-dim, 1M vectors)**: <100μs
- **SIMD distance**: 4-8 floats/cycle (AVX2/NEON)

## Next Steps

For full kernel integration:
1. Replace HeapBacking with physical memory regions
2. Implement kernel-side HNSW search algorithm
3. Add coherence-aware scheduler integration
4. Enable distributed witness log verification
5. Add SIMD auto-tuning for different architectures

## References

- **ADR-047**: Proof-Gated Mutation Protocol
- **ADR-087**: RuVix Cognition Kernel Specification
- **Section 4.3**: Vector/Graph Kernel Objects
- **Section 20**: Reflex Proof Cache
