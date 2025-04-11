# Distance to Root Algorithm

## Overview

The Distance to Root algorithm computes the shortest path distance from any node in the trust graph to a trusted root node, which helps establish a foundation for determining trust scores and relationships within the network.

## Purpose

- **Trust Metric**: Provides a simple, deterministic measure of how "close" an account is to trusted root validators
- **Complement to PageRank**: While PageRank provides a probabilistic trust score, distance-to-root provides a clear, discrete metric
- **Path Discovery**: Identifies the shortest path between any account and root validators
- **Security Foundation**: Acts as a fundamental building block for trust-based security decisions

## Algorithm Description

The Distance to Root algorithm works as follows:

1. **Initialization**:
   - Root nodes have a distance of 0
   - All other nodes initially have an infinite (or maximum) distance

2. **Graph Traversal**:
   - Starting from root nodes, perform a breadth-first search (BFS)
   - For each node visited, update its neighbors' distances if the path through the current node is shorter
   - The algorithm handles cycles in the vouch graph and ensures termination

3. **Caching**:
   - Computed distances are cached to prevent redundant calculations
   - Cached distances are marked as stale when vouch relationships change

## Implementation Details

The algorithm is implemented in the `distance_to_root.move` module with the following key components:

```rust
// Key Data Structures
struct NodeDistance has key {
    distance: u64,
    computed_at: u64,
    is_stale: bool,
    parent: address, // Next hop on the shortest path to root
}

// Main Functions
public fun get_distance_to_root(addr: address): u64
public fun compute_distance_to_root(addr: address): (u64, address)
public fun mark_distance_stale(addr: address)
```

### Computation Method

The distance computation uses a breadth-first search from the roots to efficiently find the shortest path:

1. Start with all root nodes in an initial queue (distance 0)
2. For each node in the queue:
   - Examine all accounts that have received a vouch from this node
   - If we find a shorter path to any account, update its distance
   - Add that account to the queue for further exploration
3. Continue until the queue is empty or we've found the target account

### Staleness and Recalculation

When vouch relationships change, affected distances need recalculation:

- Adding a vouch may create shorter paths
- Removing a vouch may invalidate existing paths
- The algorithm propagates staleness to potentially affected nodes
- Stale distances trigger recalculation upon next access

## Comparison with PageRank

| Feature | Distance to Root | PageRank |
|---------|-----------------|----------|
| Metric Type | Deterministic | Probabilistic |
| Calculation | BFS | Random walks |
| Value Range | 0 to N (discrete) | 0 to unbounded (continuous) |
| Sensitivity | Path existence | Path count and popularity |
| Purpose | Basic reachability | Weighted influence |

## Integration with Trust Framework

The Distance to Root algorithm works in conjunction with:

- **Vouch System**: Uses vouching relationships for edge traversal
- **Root of Trust**: Sources trusted root nodes from the registry
- **PageRank**: Complements the PageRank algorithm's probabilistic approach

## Future Improvements

Potential enhancements to the distance algorithm include:

- Weighted paths based on vouch age or strength
- Multi-path redundancy metrics
- Time-based decay of path value
- Integration with PageRank for combined scoring

## Usage Examples

```rust
// Get distance from account to root
let distance = distance_to_root::get_distance_to_root(account_addr);

// Check if account is within a certain trust radius
let is_trusted = distance_to_root::get_distance_to_root(account_addr) <= TRUST_THRESHOLD;

// Get both distance and next hop toward root
let (distance, next_hop) = distance_to_root::compute_distance_to_root(account_addr);
```

## Limitations

- Only considers the shortest path, not path redundancy
- Binary trust model (vouched or not vouched)
- Doesn't consider the age or strength of vouches
- Graph changes require recalculation of affected paths

## Conclusion

The Distance to Root algorithm provides a simple yet powerful metric for trust assessment in the Libra network. By identifying how many hops away an account is from trusted root validators, it creates a foundation for more complex trust evaluations and security mechanisms.
