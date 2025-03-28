# Rescue Tool
Apply changes to a reference DB at rest.

# Uses
There are three cases:
1. Twin: replace the validator set from config files
2. Upgrade only: upgrade the framework (maybe the source in reference db is a brick, and can't do the scripts you want).
3. Run script: the framework in reference DB is usable, and we need to execute an admin transaction from a .move source


# Replace validators (Twin testnet)
```
cargo r -- --db-path ~/.libra/data/db \
register-vals \
--operator-yaml ~/.libra/operator.yaml
```
# Node files

You must update the node's files with waypoint information
and genesis transaction
This function is used to update the node's configuration files
after a successful rescue operation.
It updates the safety rules configuration with the new waypoint
and sets the genesis transaction to the latest writeset.

#  Troubleshooting:
The principal issue with a node not being able to start
is that the initial tx (genesis.blob or rescue.blob)
is missing, malformed, or will not bootstrap. At this point
the process, we should have checked that a db would bootstrap.
The next issue is that maybe the node starts by it does not make progress.
You'll likely see one of these known cases:
1. `Unable to initialize safety rules`.
Could possibly mean any kind of malformed data with safety rules, In the context of rescue operations this could mean:
  - If you see `Invalid bitvec from the multi-signature`, it's probably that the "waypoint" and "genesis-waypoint" are not set in the safety rules.
  - Maybe something is off with the NodeConfig's `consensus.safety_rules.initial_safety_rules_config` which should not be "None" in a rescue situation. The code in post_rescue_node_file_updates() should have prevented this.
  - When the node started it failed to create the safety rules in file (or in memory on mainnet).

2. `[RoundManager] SafetyRules Rejected`:
This is similar to the unable to initialize safety rules (you'll also see a `Invalid bitvec from the multi-signature`).
This message is rejecting a proposal from another validator because it does not see it in the validator list.  It likely means that your node is not starting from the right waypoint, reading the latest state.

Full error:
```
{"committed_round":0,"error":"from peer 17bb0cfbbe0a15ea72150245397bc8271e5f54bc74a55e3712ab4138e6196981\n\nCaused by:\n    0: [RoundManager] SafetyRules Rejected [id: fbb2cf85, author: (NIL), epoch: 3, round: 01, parent_id: f9a5f6ab, timestamp: 1743168891732818]\n    1: Invalid EpochChangeProof: Invalid bitvec from the multi-signature","kind":"SafetyRules","pending_votes":"PendingVotes: []","round":1}
```

After checking that the other node is in fact in the validator set, then the issue is most likely that the waypoint is not set
correctly in your node's files. NodeConfig has two places where waypoint is set: `base.waypoint` and `consensus.safety_rules.initial_safety_rules_config.waypoint`. Both of these must be set to the new waypoint.
The code in post_rescue_node_file_updates() should have set the waypoint


3 `safety_rules.rs:434 {"error":"validator_signer is not set, SafetyRules is not initialized"`: If you see this error coupled with the above errors it's a local config problem. If you see this error standalone, your node may not actually be in the validator set.

In this case it's likely that the "waypoint" and "genesis_waypoint" fields in safety rules are not set to the new waypoint (look in secure_data.json in testnet).
