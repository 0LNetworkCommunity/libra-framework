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

#### A. Distributed Storage Model
- Trust relationships (vouches) are managed by a separate `vouch` module.
- Trust score data is stored at the user level, not centralized.
- Scores are computed on-demand rather than tracking a global state.

#### B. Explicit Revocation Handling
- Revocation is handled by removing vouches through the `vouch` module.
- No separate revocation graph needed, simplifying the model.
- When a user revokes trust, all downstream scores are marked as stale.

**Final Trust Score:**
Score depends on frequency of reaching a node via random walks from roots through the vouch graph.

---

### 3. Implementation Features and Optimizations

#### A. Monte Carlo Trust Score Approximation
- Uses random walks from root nodes to approximate PageRank.
- For each target address, simulates N walks of a configurable depth.
- Counts the number of times the target is reached during these walks.
- Avoids centralized score computation or full graph traversal.

#### B. DAG-Enforced Traversal with Cycle Detection
- Treats the graph as a DAG during traversals to avoid cycles.
- Each walk tracks visited nodes to prevent re-entry.
- This handling optimizes score computation and prevents artificial inflation.

#### C. Lazy Score Invalidation and Caching
- Each node stores a cached score and timestamp.
- Scores are recomputed only when stale or expired.
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
- `vouch`: Manages the actual vouching relationships between addresses

#### C. Score Computation Flow
1. Check if cached score is valid (not stale & within TTL)
2. If valid, return cached score
3. Otherwise, compute fresh score via Monte Carlo walks
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
- No centralized graph storage
- Vouch relationships managed by dedicated `vouch` module
- Minimal metadata required per user

#### B. Computation Efficiency
- On-demand score calculation
- Cached results with staleness tracking
- Cycle detection to prevent redundant walks
- Circuit breaker to ensure bounded computation

#### C. Resilience and Safety Features
- Protection against stack overflows through a circuit breaker
- Cycle detection in both score calculation and staleness propagation
- Self-healing through automatic cache expiration


### 6. Current Implementation

- Trust Flow with Decay:

Trust now flows through the network using a 50% decay model per hop
Each node passes half of its trust value to its neighbors
This creates a natural diminishing effect as distance increases

Direct Trust Recognition:

Root accounts that directly vouch for a user give a full 100 points
This means a user with 10 root vouches will get 1000 points directly, versus 100 points for 1 root vouch

- Path-based Weighting:

The algorithm now correctly considers both path length and number of vouchers
Users further away from roots receive less trust (50% less per hop)
Multiple paths to the same user are additive, reinforcing trust

---
### 7. Future Improvements

#### A. Randomization
- Currently using deterministic neighbor selection
- Could be enhanced with proper randomization for better score distribution

#### B. Parallelization
- Monte Carlo approach naturally supports parallelization
- Could run multiple walks simultaneously for faster score computation

#### C. Prioritized Recalculation
- Could prioritize recalculating scores for high-impact nodes first

This implementation provides an efficient, scalable approach to trust scoring that works well in constrained environments while maintaining accuracy and responsiveness.
