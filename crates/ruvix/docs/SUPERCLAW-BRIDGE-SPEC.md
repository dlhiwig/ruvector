# SuperClaw → RuVix Bridge Specification

> How SuperClaw agents become RuVix kernel Tasks.

**Version:** 0.1.0  
**Date:** 2026-03-15  
**Authors:** SuperClaw Team

---

## 1. Architecture Overview

```
SuperClaw (TypeScript)          RuVix (Bare-metal Kernel)
═══════════════════════         ═════════════════════════
Agent                    →      Task
Moltbook (EventEmitter)  →      Queue (typed ring buffer)
SKYNET governance        →      Proof engine (3-tier)
Lethal Trifecta sandbox  →      Capability derivation tree
CORTEX memory            →      KernelVectorStore (HNSW)
ORACLE learning          →      Coherence scoring
Audit JSON               →      Witness log (append-only)
SONA tuning              →      Coherence-aware scheduler
```

---

## 2. Agent → Task Mapping

### SuperClaw Agent (current)
```typescript
interface Agent {
  id: string;
  role: 'implementer' | 'critic' | 'researcher' | 'simplifier';
  provider: 'claude' | 'gemini' | 'codex' | 'kimi';
  maxTokens: number;
  tools: string[];       // allowed tool names
  systemPrompt: string;
}
```

### RuVix Task (target)
```rust
// Task creation via syscall
let task_handle = kernel.dispatch(Syscall::TaskCreate {
    parent_cap: root_cap,          // capability granting SPAWN right
    priority: TaskPriority::Normal,
    partition_id: partition.id,     // scheduling partition
});

// Task gets capabilities derived from parent
let agent_cap = kernel.dispatch(Syscall::CapGrant {
    source: root_cap,
    rights: CapRights::READ | CapRights::WRITE | CapRights::PROVE,
    badge: agent_role_badge,       // encodes role (implementer=1, critic=2, etc.)
    target_task: task_handle,
});
```

### Mapping Table

| SuperClaw Field | RuVix Equivalent | Mechanism |
|----------------|------------------|-----------|
| `agent.id` | `TaskHandle(id, epoch)` | Kernel-assigned, unforgeable |
| `agent.role` | Capability `badge` field | Badge = role enum value |
| `agent.provider` | RVF component mount | Different model runtimes as RVF packages |
| `agent.maxTokens` | Region size limit | Memory region with bounded allocation |
| `agent.tools` | Capability rights bitmap | Each tool = a right bit |
| `agent.systemPrompt` | Region (immutable) | Read-only region containing prompt bytes |

### Lifecycle

```
SuperClaw                    RuVix Syscall Sequence
─────────                    ──────────────────────
agent.spawn()            →   task_create(parent_cap, priority, partition)
                             region_map(size, Slab)           // agent memory
                             cap_grant(read|write, badge)     // agent capabilities
                             queue_send(task_queue, init_msg)  // start signal

agent.execute(task)      →   queue_recv(work_queue)           // receive task
                             vector_get(key, cap)             // read context
                             vector_put_proved(key, data, proof) // write results
                             queue_send(result_queue, output) // return results

agent.terminate()        →   cap_revoke(agent_cap)            // cascade revocation
                             region_unmap(agent_regions)       // free memory
                             // Task auto-terminates when all caps revoked
```

---

## 3. Moltbook → Queue Mapping

### SuperClaw Moltbook (current)
```typescript
// EventEmitter-based pub/sub
moltbook.publish('agent:result', { agentId, output, confidence });
moltbook.subscribe('swarm:task', (task) => agent.execute(task));
```

### RuVix Queue (target)
```rust
// Typed ring buffer with capability-gated access
// Queue creation
let queue_handle = kernel.dispatch(Syscall::QueueCreate {
    capacity: 256,
    msg_size: 4096,           // max message size in bytes
    cap: root_cap,
});

// Send (requires WRITE right on queue capability)
kernel.dispatch(Syscall::QueueSend {
    queue: queue_handle,
    msg: &SwarmMessage {
        msg_type: MsgType::TaskResult,
        sender_badge: 1,       // implementer
        coherence: 0.95,
        payload: result_bytes,
    },
    cap: agent_queue_cap,
});

// Receive (requires READ right on queue capability)
let msg = kernel.dispatch(Syscall::QueueRecv {
    queue: queue_handle,
    cap: agent_queue_cap,
});
```

### Message Types

| Moltbook Event | Queue Message Type | Direction |
|---------------|-------------------|-----------|
| `swarm:task` | `MsgType::TaskAssign` | Orchestrator → Agent |
| `agent:result` | `MsgType::TaskResult` | Agent → Orchestrator |
| `agent:error` | `MsgType::TaskError` | Agent → Orchestrator |
| `swarm:critique` | `MsgType::CritiqueRequest` | Orchestrator → Critic |
| `agent:critique` | `MsgType::CritiqueResult` | Critic → Orchestrator |
| `swarm:consensus` | `MsgType::ConsensusVote` | Agent ↔ Agent |
| `skynet:alert` | `MsgType::GovernanceAlert` | Kernel → All |

### Key Difference
- Moltbook: fire-and-forget, no ordering guarantees, no capacity limits
- Queue: bounded ring buffer, FIFO ordering, blocks on full, capability-gated
- **Every message carries a coherence score** — the scheduler uses this

---

## 4. SKYNET → Proof Mapping

### SuperClaw SKYNET (current)
```typescript
// Application-level governance checks
if (thresholds.maxConcurrentAgents > limit) throw new Error('...');
if (action.cost > thresholds.requireApprovalAbove) await requestApproval();
// ... but these are JavaScript checks. An agent CAN bypass them.
```

### RuVix Proof Engine (target)
```rust
// Kernel-enforced. Cannot be bypassed. No proof = no mutation.

// Tier 0: Reflex (<10μs) — vector updates, routine writes
let proof = proof_engine.generate_reflex_proof(&mutation_hash);
kernel.dispatch(Syscall::VectorPutProved {
    key, data, proof, cap
});

// Tier 1: Standard (<100μs) — graph mutations, capability grants
let proof = proof_engine.generate_standard_proof(&mutation_hash, &merkle_witness);
kernel.dispatch(Syscall::GraphApplyProved {
    mutation, proof, cap
});

// Tier 2: Deep (<1ms) — RVF mounts, cross-partition operations
let proof = proof_engine.generate_deep_proof(
    &mutation_hash, &coherence_snapshot, &mincut_analysis
);
kernel.dispatch(Syscall::RvfMount {
    rvf_data, proof, cap
});
```

### Governance Mapping

| SKYNET Component | RuVix Equivalent | Enforcement |
|-----------------|------------------|-------------|
| ThresholdEnforcer | Capability rights + Region limits | Kernel-level |
| Financial gates ($100 approval) | Proof Tier 2 (Deep) required | Cannot bypass |
| Tool permissions (safe/elevated) | Capability READ vs WRITE vs PROVE | Unforgeable |
| Rollback capability | Witness log + deterministic replay | Guaranteed |
| Agent count limit | Task partition scheduling budget | Kernel-enforced |
| Context char limit | Region size (max_size in region_map) | Hard limit |

### The Critical Upgrade
SuperClaw governance is **advisory** — a buggy or malicious agent can ignore JavaScript checks.
RuVix governance is **mandatory** — the kernel rejects unauthorized syscalls. There is no userspace workaround.

---

## 5. CORTEX → VectorStore Mapping

### SuperClaw CORTEX (current)
```typescript
// API call to embedding service + local search
const embedding = await openai.embeddings.create({ input: text, model: 'text-embedding-3-small' });
const results = await vectorDB.search(embedding, topK: 5);
```

### RuVix KernelVectorStore (target)
```rust
// Kernel-resident. No API call. No network. SIMD-accelerated.

// Create store (768 dims for text-embedding-3-small compatibility)
let store = VectorStoreBuilder::new(768, 10000)
    .with_proof_policy(ProofPolicy::standard())
    .with_coherence_config(CoherenceConfig {
        min_coherence_threshold: 5000, // 0.5
        enable_scheduler_hints: true,
        decay_rate: 100,               // slow decay
        initial_coherence: 10000,      // 1.0
        ..Default::default()
    })
    .build(backing)?;

// Write (proof-gated)
let proof = engine.generate_reflex_proof(&hash);
let attestation = store.vector_put_proved(key, &embedding_data, proof, cap)?;
// → attestation logged to witness, coherence updated

// Read (capability-gated, no proof needed)
let (data, coherence_meta) = store.vector_get(key, cap)?;

// Search (SIMD-accelerated cosine similarity)
// HNSW index: O(log n) approximate nearest neighbor
let neighbors = store.hnsw_search(&query_vector, top_k: 5, ef_search: 50)?;
```

### Performance Comparison

| Operation | CORTEX (API) | RuVix (Kernel) | Improvement |
|-----------|-------------|----------------|-------------|
| Embedding lookup | ~200ms (network) | ~0.1ms (SIMD) | **2000×** |
| Vector write | ~50ms (DB write) | ~10μs (slab + proof) | **5000×** |
| Nearest search | ~100ms (DB query) | ~1ms (HNSW) | **100×** |
| Cold start | ~2s (connection) | 0 (kernel-resident) | **∞** |

---

## 6. ORACLE → Coherence Mapping

### SuperClaw ORACLE (current)
```typescript
// JSON-based pattern learning
oracle.recordOutcome({ taskId, agentRole, success, latency, cost });
oracle.getRecommendation(taskType); // returns preferred agent/model
```

### RuVix Coherence System (target)
```rust
// Every vector has coherence metadata (0.0 - 1.0)
pub struct CoherenceMeta {
    pub coherence_score: u16,           // 0-10000 (fixed point)
    pub last_mutation_epoch: u64,
    pub proof_attestation_hash: [u8; 32],
}

// The scheduler uses coherence to:
// 1. BOOST tasks processing novel information (high vector distance from recent)
// 2. DEPRIORITIZE tasks that would lower coherence
// 3. FAST-PATH mutations within coherent partitions

// Combined priority = deadline_pressure + novelty_signal - structural_risk
```

### How ORACLE Patterns Become Coherence

| ORACLE Pattern | Coherence Signal |
|---------------|-----------------|
| Agent produces good results | High coherence on output vectors |
| Agent produces garbage | Low coherence → scheduler deprioritizes |
| Task type matches agent strength | Novelty signal high (new useful info) |
| Repeated failures | Coherence decay over epochs |
| Cost optimization | Partition scheduling budget (time slices) |

---

## 7. Sandbox → Capability Mapping

### SuperClaw Lethal Trifecta (current)
```typescript
// PrivateDataSandbox: string-based classification
sandbox.classify(data, 'SENSITIVE' | 'INTERNAL' | 'PUBLIC');
// ToolPermissionBoundary: safe tools default, elevated requires approval
if (tool.elevated) await requestApproval();
// RollbackCapability: checkpoint before risky ops
const checkpoint = await sandbox.checkpoint();
```

### RuVix Capability Tree (target)
```rust
// Root capability: full rights
let root = cap_mgr.create_root_capability(obj_id, ObjectType::VectorStore, 0, kernel_task)?;

// Derived capability: read-only (for safe agents)
let readonly = cap_mgr.grant(root, CapRights::READ, badge, kernel_task, agent_task)?;

// Derived capability: read+write but no GRANT (can't delegate further)
let readwrite = cap_mgr.grant(root, CapRights::READ | CapRights::WRITE, badge, kernel_task, agent_task)?;

// GRANT_ONCE: agent can delegate exactly once, non-transitively
let one_shot = cap_mgr.grant(root, CapRights::READ | CapRights::GRANT_ONCE, badge, kernel_task, agent_task)?;

// Revocation: kill root → ALL derived capabilities die instantly
cap_mgr.revoke(root)?;
// → readonly: DEAD
// → readwrite: DEAD
// → one_shot: DEAD
// → anything one_shot granted: DEAD
```

### Mapping

| Lethal Trifecta | Capability Model |
|----------------|-----------------|
| Data classification | Separate Region per classification level |
| Safe tools | READ-only capabilities |
| Elevated tools | WRITE + PROVE capabilities (requires proof) |
| Rollback | Witness log replay to checkpoint epoch |
| Max delegation depth | `config.max_delegation_depth = 8` |

---

## 8. RVF Package Format

An RVF (RuVector Format) package bundles an agent for deployment:

```
agent-implementer.rvf
├── manifest.json          # Agent metadata
│   {
│     "name": "implementer",
│     "version": "1.0.0",
│     "role": "implementer",
│     "badge": 1,
│     "required_rights": ["READ", "WRITE", "PROVE"],
│     "partition_budget_us": 10000,
│     "memory_limit_bytes": 2097152,
│     "queue_config": {
│       "capacity": 64,
│       "msg_size": 4096
│     },
│     "vector_config": {
│       "dimensions": 768,
│       "capacity": 1000
│     }
│   }
├── prompt.bin             # System prompt (immutable region)
├── model.bin              # Model weights or runtime config
├── signature.sig          # ML-DSA-65 signature
└── checksum.sha256
```

### Deployment Syscall Sequence
```rust
// 1. Verify signature (Tier 2 proof required)
let proof = engine.generate_deep_proof(&rvf_hash, &coherence, &mincut);
kernel.dispatch(Syscall::RvfMount { rvf_data, proof, root_cap })?;

// 2. Kernel creates:
//    - Task with partition budget
//    - Immutable Region for prompt
//    - Slab Region for working memory
//    - Queue for communication
//    - VectorStore for agent memory
//    - Capabilities (derived from deployer's caps)

// 3. Task starts executing
```

---

## 9. Migration Path

### Phase 1: Bridge Mode (Now → Month 1)
```
[SuperClaw TS] ←→ [Unix Socket] ←→ [ruvix-bridge binary]
                                          ↕
                                    [RuVix Kernel (QEMU)]
```
- SuperClaw agents remain TypeScript
- ruvix-bridge translates API calls to kernel syscalls
- Governance checks happen at kernel level
- Vector operations use kernel VectorStore
- **Benefit:** Kernel-enforced governance immediately

### Phase 2: Hybrid Mode (Month 2-3)
```
[SuperClaw Orchestrator] ←→ [Queue Bridge]
         ↕                        ↕
    [TS Agents]           [Rust RVF Agents]
                                  ↕
                           [RuVix Kernel]
```
- New agents written in Rust as RVF packages
- Existing TS agents bridged via Queue translation
- Orchestrator runs on host, agents run in kernel
- **Benefit:** Native performance for new agents

### Phase 3: Full Native (Month 4+)
```
[RuVix Kernel]
    ↕
[RVF Orchestrator Task]
    ↕
[RVF Agent Tasks] ←→ [Queues] ←→ [RVF Agent Tasks]
    ↕                                    ↕
[VectorStores]                    [VectorStores]
```
- Everything runs as kernel Tasks
- No TypeScript, no Node.js, no Linux
- Boot → agents running in <1 second
- **Benefit:** Full bare-metal performance, complete governance

---

## 10. Hardware Deployment Targets

| Target | Hardware | Use Case | Timeline |
|--------|----------|----------|----------|
| QEMU virt | Emulated Cortex-A72 | Development & testing | **Now** |
| Raspberry Pi 4 | BCM2711, Cortex-A72 | BIC bank node | Month 1 |
| Jetson Orin Nano | Cortex-A78AE + Ampere GPU | SWAI Baseline product | Month 2-3 |
| Multi-node mesh | N × RPi4 via Ethernet | Distributed swarm | Month 3+ |

---

## Appendix: Syscall Reference

| # | Syscall | Category | Proof Required |
|---|---------|----------|---------------|
| 1 | `region_map` | Memory | No |
| 2 | `region_unmap` | Memory | No |
| 3 | `cap_grant` | Security | No (but GRANT right required) |
| 4 | `cap_revoke` | Security | No (revocation always allowed) |
| 5 | `queue_send` | IPC | No (WRITE right required) |
| 6 | `queue_recv` | IPC | No (READ right required) |
| 7 | `task_create` | Scheduling | No (SPAWN right required) |
| 8 | `timer_set` | Scheduling | No |
| 9 | `vector_get` | Cognition | No (READ right required) |
| 10 | `vector_put_proved` | Cognition | **Yes** (Reflex/Standard) |
| 11 | `graph_apply_proved` | Cognition | **Yes** (Standard/Deep) |
| 12 | `rvf_mount` | Deployment | **Yes** (Deep only) |
