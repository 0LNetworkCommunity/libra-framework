

# Overview
The "framework" which contains all the consensus, account, and econ policies for the network is written in Move. This code is stored in the network database, and effectively executed on demand. This means that framework upgrades can occur without redeploying the Move VM itself, or the supporting system code (network node software). It also means the state machine can upgrade without a coordinated halt.
To do this we require two libra tools `framework` for building the artifacts, and `txs` for proposing, voting, and ultimately deploying the artifacts.

# Procedure
## Build Artifacts

### 1. Build a new head.mrb bundle for the framework into `./framework/releases`.
```
cargo r --release release
```

### 2. Build the upgrade Move transaction scripts

This is a Move package which is custom built for a one-time execution. It contains bytecode which will be allowed to be executed (by anyone), once there is a vote and agreement on the proposal passing. This is guarded by the proposer providing the transaction output hash during the proposal (in advance of the vote). An upgrade script that is tampered with, will yield a different execution hash, and will be prevented from running (it is likely to be blocked by the transaction size limits before entering the mempool).

This will output the newly compiled upgrade Move transaction script, its binary, and the hash.
```
cargo r --release upgrade
```

All the artifacts are now created, the transactions can be submitted.

## Upgrade Ceremony
#### 3. With `txs` anyone can submit the proposal and metadata. You'll need to provide the actual script compiled path, and an optional URL which contains documentation of the proposal (e.g github).

```
# note the actual script dir
txs upgrade propose --script-dir ./framework/release/framework_upgrade --metadata-url <URL>
```
If this transaction succeeds it will produce a VOTE ID, which is a number. Now the proposal is open to voting.

### 4. With `txs` anyone with governance authority (the epoch's validators as of `V7`), can submit a vote in favor (or against it with `--should-fail`).

We assume voting is in favor with:
```
txs vote --proposal-id <number>
```

If voter would like the proposal to be rejected:
```
txs vote --proposal-id <number> --shoul-fail
```

After everyone has voted (to reach the consensus threshold of 66% as of  `V7`), the proposal is in a "Resolvable" state. Anyone can resolve it provided they have the source transaction script for the upgrade (step #2 above).

### 5. Use `txs` to resolve a successfully approved proposal
```
# Note the actual path
txs resolve --proposal-script-dir ./framework/release/framework_upgrade
```

If this transction is successful the new bytecode will be written to the VM, and a new epoch will be triggered.
