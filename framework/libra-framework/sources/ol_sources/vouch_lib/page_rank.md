**Title:** Lazy Trust Scoring with Distributed Storage in Constrained Environments

**Context:**
We're working in a constrained compute/storage environment (e.g., a blockchain VM) and need to compute trust scores for user accounts. Each user can vouch for other users. Our goal is to approximate how closely a node is connected to a fixed list of trusted "root" accounts, with minimal storage and on-demand computation.

---

### 1. Problem Definition
- Each account can **vouch for** other accounts to establish trust.
- We need to compute a **trust score** for each account that approximates closeness to the trusted roots.
- Due to constraints, computing the entire graph on every change is infeasible.

---

### 2. Core Design: Distributed Trust Graph with Lazy Evaluation

#### A. Account-Level Distributed Storage Model
- There is **no central graph storage** in the system
- Each user account stores only their own trust score data in their account storage
- Trust relationships (vouches) are stored at the account level by a separate `vouch` module
- Each account only knows about its direct relationships (who it vouches for and who vouches for it)
- The complete graph is never materialized in any single location - it exists as a virtual construct across all accounts
- This distributes both storage and computation across the network

#### B. Explicit Revocation Handling
- Revocation is handled by removing vouches through the `vouch` module.
- No separate revocation graph needed, simplifying the model.
- When a user revokes trust, all downstream scores are marked as stale.

**Final Trust Score:**
Score depends on the paths from root nodes to the target node, with trust value decaying by half at each hop.

---

### 3. Implementation Features and Optimizations

#### A. Exhaustive Graph Traversal with Power Decay
- Uses a deterministic traversal of all paths from root nodes to the target.
- For each path, a trust "power" is passed, which decays by 50% at each hop.
- The final score is the sum of all trust power received via all possible paths.
- This provides a natural weighting where closer nodes to roots receive higher scores.

#### B. Cycle Detection During Traversal
- Treats the graph as a DAG during traversals to avoid cycles.
- Each traversal tracks visited nodes to prevent re-entry.
- This handling optimizes score computation and prevents artificial inflation.

#### C. Lazy Score Invalidation and Caching
- Each node stores a cached score and timestamp.
- Scores are recomputed only when marked as stale.
- When trust relationships change, affected nodes are marked as stale.
- Uses a transitive staleness propagation to ensure downstream accuracy.

#### D. Circuit Breaker for Recursive Operations
- Implemented a hard limit of 1000 processed addresses for any recursive operation.
- Prevents stack overflows in complex trust networks or denial-of-service attacks.
- Ensures predicable resource usage for mark-as-stale operations.

---

### 4. Implementations Details

#### A. Per-User Trust Record Structure
```plaintext
UserTrustRecord {
    cached_score: u64,
    score_computed_at_timestamp: u64,
    is_stale: bool,
}
```

#### B. External Module Dependencies
- `root_of_trust`: Provides the root nodes for trust calculations
- `vouch`: Manages the actual vouching relationships between addresses (with built-in 45-day TTL)

#### C. Score Computation Flow
1. Check if cached score is valid (not stale)
2. If valid, return cached score
3. Otherwise, compute fresh score via exhaustive path traversal
4. Update cache with new score and timestamp

#### D. Handling Trust Changes
When user A changes their vouches (adding/removing):
1. Update A's vouches through the `vouch` module
2. Mark affected nodes as stale
3. Recursively propagate staleness to downstream nodes
4. Apply circuit breaker to prevent excessive computation

---

### 5. Key Benefits of This Implementation

#### A. Storage Efficiency
- No centralized graph storage anywhere in the system
- Each account only stores its own trust record and direct vouch relationships
- The complete graph only exists conceptually as the union of all individual account data
- This distributed approach dramatically reduces storage requirements for any single location

#### B. Computation Efficiency
- On-demand score calculation
- Cached results with staleness tracking
- Cycle detection to prevent redundant traversals
- Circuit breaker to ensure bounded computation

#### C. Resilience and Safety Features
- Protection against stack overflows through a circuit breaker
- Cycle detection in both score calculation and staleness propagation
- Self-healing through staleness propagation

### 6. Current Implementation

- Trust Flow with Decay:
  - Trust flows through the network using a 50% decay model per hop
  - Each node passes half of its trust value to its neighbors
  - This creates a natural diminishing effect as distance increases

- Direct Trust Recognition:
  - Root accounts that directly vouch for a user give a full trust value
  - Multiple direct root vouches accumulate linearly

- Path-based Weighting:
  - The algorithm considers all possible paths from roots to the target
  - Users further away from roots receive less trust (50% less per hop)
  - Multiple paths to the same user are additive, reinforcing trust
  - No artificial depth limitation - all paths are considered regardless of length
  - Paths naturally terminate when trust power becomes too small (< 2)

---
### 7. Future Improvements

#### A. Performance Optimization
- Could implement more efficient graph traversal algorithms
- Potential for batch processing of related nodes

#### B. Parallelization
- Exhaustive traversal approach could support parallelization
- Could run multiple path explorations simultaneously for faster score computation

#### C. Prioritized Recalculation
- Could prioritize recalculating scores for high-impact nodes first

#### D. Monte Carlo Implementation (Roadmap)
- We have already experimented with a Monte Carlo-based approach for trust scoring
- Could implement this algorithm based on field performance evaluation of current approach
- Monte Carlo implementation would:
  - Use random walks from root nodes to estimate trust scores
  - Potentially provide better performance in very large networks
  - Allow for configurable trade-offs between accuracy and compute cost
  - Enable statistical approximation rather than exhaustive traversal
- Decision on implementation will be made after evaluating the current approach's performance in production environments

This implementation provides an efficient, scalable approach to trust scoring that works well in constrained environments while maintaining accuracy and responsiveness.
