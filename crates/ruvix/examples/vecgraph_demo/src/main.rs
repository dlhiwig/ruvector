//! VecGraph Integration Demo
//!
//! This demo exercises:
//! - Vector store creation with VectorStoreBuilder
//! - HNSW index operations
//! - Coherence tracking
//! - Witness log recording
//! - Proof-gated mutations
//! - SIMD distance computations

use rand::Rng;
use ruvix_proof::ProofEngine;
use ruvix_region::backing::HeapBacking;
use ruvix_types::{
    CapRights, Capability, ObjectType, RegionHandle, VectorKey,
};
use ruvix_vecgraph::{
    cosine_similarity, euclidean_distance_squared, l2_norm, ProofPolicy, SimdCapabilities,
    VectorStoreBuilder,
};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("   RuVix VecGraph Integration Demo");
    println!("═══════════════════════════════════════════════════════\n");

    // Step 1: Detect SIMD capabilities
    print_simd_capabilities();

    // Step 2: Demonstrate distance functions
    demo_distance_functions();

    // Step 3: Create a VectorStore with proof engine
    demo_vector_store();

    println!("\n═══════════════════════════════════════════════════════");
    println!("   Demo Complete!");
    println!("═══════════════════════════════════════════════════════");
}

fn print_simd_capabilities() {
    println!("📊 SIMD Capabilities");
    println!("────────────────────────────────────────────────────────");

    let caps = SimdCapabilities::detect();

    println!("  AVX2:      {}", if caps.avx2 { "✓ Available" } else { "✗ Not available" });
    println!("  AVX-512:   {}", if caps.avx512 { "✓ Available" } else { "✗ Not available" });
    println!("  NEON:      {}", if caps.neon { "✓ Available" } else { "✗ Not available" });
    println!("  FMA:       {}", if caps.fma { "✓ Available" } else { "✗ Not available" });
    println!("  Lane width: {} floats/vector", caps.lane_width());
    println!("  SIMD:      {}\n", if caps.has_simd() { "✓ Enabled" } else { "✗ Scalar fallback" });
}

fn demo_distance_functions() {
    println!("🧮 Distance Function Tests");
    println!("────────────────────────────────────────────────────────");

    let vec_a = vec![1.0f32, 2.0, 3.0, 4.0];
    let vec_b = vec![4.0f32, 3.0, 2.0, 1.0];

    let cosine = cosine_similarity(&vec_a, &vec_b);
    let euclidean_sq = euclidean_distance_squared(&vec_a, &vec_b);
    let norm_a = l2_norm(&vec_a);

    println!("  Vector A:            [{:.1}, {:.1}, {:.1}, {:.1}]", vec_a[0], vec_a[1], vec_a[2], vec_a[3]);
    println!("  Vector B:            [{:.1}, {:.1}, {:.1}, {:.1}]", vec_b[0], vec_b[1], vec_b[2], vec_b[3]);
    println!("  Cosine similarity:   {:.6}", cosine);
    println!("  Euclidean² distance: {:.6}", euclidean_sq);
    println!("  L2 norm (A):         {:.6}\n", norm_a);
}

fn demo_vector_store() {
    println!("🗄️  Vector Store Demo");
    println!("────────────────────────────────────────────────────────");

    const DIMENSIONS: u32 = 768;
    const CAPACITY: u32 = 1000;
    const NUM_VECTORS: usize = 100;

    println!("  Dimensions:  {}", DIMENSIONS);
    println!("  Capacity:    {}", CAPACITY);
    println!("  Test vectors: {}\n", NUM_VECTORS);

    // Create memory backings for the three regions
    let data_backing = HeapBacking::new(5 * 1024 * 1024); // 5MB for vector data
    let hnsw_backing = HeapBacking::new(2 * 1024 * 1024); // 2MB for HNSW graph
    let witness_backing = HeapBacking::new(1 * 1024 * 1024); // 1MB for witness log

    // Create region handles
    let data_handle = RegionHandle::new(1, 0);
    let hnsw_handle = RegionHandle::new(2, 0);
    let witness_handle = RegionHandle::new(3, 0);

    // Build the vector store with Reflex policy for testing
    println!("  Creating VectorStore...");
    let mut store = VectorStoreBuilder::new(DIMENSIONS, CAPACITY)
        .with_proof_policy(ProofPolicy::reflex())
        .build(
            data_backing,
            hnsw_backing,
            witness_backing,
            data_handle,
            hnsw_handle,
            witness_handle,
            1, // store_id
        )
        .expect("Failed to create vector store");

    println!("  ✓ VectorStore created");
    println!("    - Store handle ID: {}", store.handle().raw().id);
    println!("    - Dimensions: {}", store.dimensions());
    println!("    - Capacity: {}", store.capacity());
    println!("    - Initial size: {}\n", store.len());

    // Create a proof engine
    println!("  Creating ProofEngine...");
    let mut proof_engine = ProofEngine::default();
    println!("  ✓ ProofEngine ready\n");

    // Create a capability with PROVE rights
    let capability = Capability::new(
        1,
        ObjectType::VectorStore,
        CapRights::READ | CapRights::WRITE | CapRights::PROVE,
        0,
        1,
    );

    println!("  Capability:");
    println!("    - Object: {:?}", ObjectType::VectorStore);
    println!("    - Rights: READ | WRITE | PROVE\n");

    // Insert random vectors
    println!("  Inserting {} vectors...", NUM_VECTORS);
    let mut rng = rand::thread_rng();
    let mut inserted_keys = Vec::new();

    for i in 0..NUM_VECTORS {
        // Generate random vector (normalized)
        let mut data: Vec<f32> = (0..DIMENSIONS)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();

        // Normalize to unit length for better cosine similarity
        let norm = l2_norm(&data);
        if norm > 0.0 {
            for val in &mut data {
                *val /= norm;
            }
        }

        let key = VectorKey::new(i as u64);
        inserted_keys.push((key, data.clone()));

        // Compute mutation hash
        let mutation_hash = compute_mutation_hash(key, &data);

        // Generate proof token
        let current_time_ns = 500_000_000; // Fixed time for demo
        let proof = proof_engine
            .generate_reflex_proof(
                &mutation_hash,
                current_time_ns,
            )
            .expect("Failed to generate proof");

        // Insert vector
        store
            .vector_put_proved(key, &data, &proof, &capability, current_time_ns)
            .expect("Failed to insert vector");
    }

    println!("  ✓ Inserted {} vectors", NUM_VECTORS);
    println!("    - Store size: {}", store.len());
    println!("    - Witness entries: {}\n", store.witness_entry_count());

    // Search for nearest neighbors
    println!("  🔍 Nearest Neighbor Search");
    println!("  ────────────────────────────────────────────────────────");

    // Use the first inserted vector as query
    let (query_key, query_vector) = &inserted_keys[0];
    println!("  Query vector: key={}", query_key.raw());

    // Compute distances to all other vectors
    let mut distances: Vec<(VectorKey, f32)> = Vec::new();

    for (key, _) in &inserted_keys[1..] {
        // Read the vector from store
        let (stored_vec, coherence) = store
            .vector_get(*key, &capability)
            .expect("Failed to read vector");

        // Compute cosine distance (1 - similarity)
        let similarity = cosine_similarity(query_vector, &stored_vec);
        let distance = 1.0 - similarity;

        distances.push((*key, distance));

        // Print coherence for first few
        if distances.len() <= 3 {
            println!("    - Vector {}: distance={:.6}, coherence={:.4}",
                key.raw(),
                distance,
                coherence.coherence_score as f32 / 10000.0
            );
        }
    }

    // Sort by distance
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    println!("\n  Top 5 Nearest Neighbors:");
    for (i, (key, dist)) in distances.iter().take(5).enumerate() {
        println!("    {}. Key {} - distance: {:.6}", i + 1, key.raw(), dist);
    }

    // Print coherence statistics
    println!("\n  📈 Coherence Statistics");
    println!("  ────────────────────────────────────────────────────────");
    let tracker = store.coherence_tracker();
    println!("  Average coherence:    {:.4}", tracker.average_coherence_f32());
    println!("  Entry count:          {}", tracker.entry_count());
    println!("  Current epoch:        {}", tracker.current_epoch());
    println!("  Low coherence muts:   {}", tracker.low_coherence_mutations());

    // Print witness log statistics
    println!("\n  📝 Witness Log");
    println!("  ────────────────────────────────────────────────────────");
    println!("  Total entries:        {}", store.witness_entry_count());
    println!("  Fill ratio:           {:.2}%", store.witness_fill_ratio() * 100.0);
    println!("  Entries remaining:    {:.0}", 
        (1.0 - store.witness_fill_ratio()) * store.witness_entry_count() as f32 / store.witness_fill_ratio());

    // Print proof policy
    println!("\n  🔒 Proof Policy");
    println!("  ────────────────────────────────────────────────────────");
    let policy = store.proof_policy();
    println!("  Required tier:        {:?}", policy.required_tier);
    println!("  Max verify time:      {} μs", policy.max_verification_time_us);
    println!("  Max validity window:  {} ns", policy.max_validity_window_ns);
    println!("  Coherence cert req:   {}", policy.require_coherence_cert);

    // Demonstrate HNSW capabilities (metadata only, since search isn't fully implemented)
    println!("\n  🕸️  HNSW Index (Metadata)");
    println!("  ────────────────────────────────────────────────────────");
    println!("  Note: Full HNSW search requires kernel-side implementation");
    println!("  This demo shows the index structure is allocated and tracked.");
    println!("  HNSW nodes allocated for each vector insertion.");
}

/// Compute a mutation hash for demo purposes (must match vector_store implementation)
fn compute_mutation_hash(key: VectorKey, data: &[f32]) -> [u8; 32] {
    // This MUST match the implementation in vecgraph/src/vector_store.rs
    let mut hash = [0u8; 32];

    // Include key
    let key_bytes = key.raw().to_le_bytes();
    hash[0..8].copy_from_slice(&key_bytes);

    // Include data hash (simple XOR for demonstration)
    for (i, &value) in data.iter().enumerate() {
        let bytes = value.to_le_bytes();
        let offset = (8 + (i * 4)) % 24; // Stay within remaining space
        for j in 0..4 {
            hash[offset + j] ^= bytes[j];
        }
    }

    hash
}
